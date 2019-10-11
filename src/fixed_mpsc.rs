//! A multi-producer, single-consumer, futures-aware, FIFO queue backed
//! by fixed size array with back pressure, for use communicating between
//! tasks on the same thread.
//!
//! These queues are the same as those in `futures::sync`, except they're not
//! intended to be sent across threads.

use std::any::Any;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::mem;
use std::rc::{Rc, Weak};

use fixed_vec_deque::{Array, FixedVecDeque};
use futures::{Async, AsyncSink, Future, Poll, Sink, StartSend, Stream};
use futures::future::Executor;
use futures::sink::SendAll;
use futures::task::{self, Task};
use futures::unsync::oneshot;

use crate::resultstream::{self, Results};

/// Creates a bounded in-memory channel with buffered storage.
///
/// This method creates concrete implementations of the `Stream` and `Sink`
/// traits which can be used to communicate a stream of values between tasks
/// with backpressure. The channel capacity is exactly `buffer`. On average,
/// sending a message through this channel performs no dynamic allocation.
pub fn channel<T, I>() -> (Sender<T, I>, Receiver<T, I>) where T: Array<Item=Option<I>> {
    let shared = Rc::new(RefCell::new(Shared {
        buffer: FixedVecDeque::new(),
        blocked_senders: VecDeque::new(),
        blocked_recv: None,
    }));
    let sender = Sender { shared: Rc::downgrade(&shared) };
    let receiver = Receiver { state: State::Open(shared) };
    (sender, receiver)
}

#[derive(Debug)]
struct Shared<T, I> where T: Array<Item=Option<I>> {
    buffer: FixedVecDeque<T>,
    blocked_senders: VecDeque<Task>,
    blocked_recv: Option<Task>,
}

/// The transmission end of a channel.
///
/// This is created by the `channel` function.
#[derive(Debug)]
pub struct Sender<T, I> where T: Array<Item=Option<I>> {
    shared: Weak<RefCell<Shared<T, I>>>,
}

impl<T, I> Sender<T, I> where T: Array<Item=Option<I>> {
    fn do_send(&self, msg: I) -> StartSend<I, SendError<I>> {
        let shared = match self.shared.upgrade() {
            Some(shared) => shared,
            None => return Err(SendError(msg)), // receiver was dropped
        };
        let mut shared = shared.borrow_mut();

        if shared.buffer.is_full() {
            shared.blocked_senders.push_back(task::current());
            Ok(AsyncSink::NotReady(msg))
        } else {
            *shared.buffer.push_back() = Some(msg);
            if let Some(task) = shared.blocked_recv.take() {
                task.notify();
            }
            Ok(AsyncSink::Ready)
        }
    }
}

impl<T, I> Clone for Sender<T, I> where T: Array<Item=Option<I>> {
    fn clone(&self) -> Self {
        Sender { shared: self.shared.clone() }
    }
}

impl<T, I> Sink for Sender<T, I> where T: Array<Item=Option<I>> {
    type SinkItem = I;
    type SinkError = SendError<I>;

    fn start_send(&mut self, msg: I) -> StartSend<I, SendError<I>> {
        self.do_send(msg)
    }

    fn poll_complete(&mut self) -> Poll<(), SendError<I>> {
        Ok(Async::Ready(()))
    }

    fn close(&mut self) -> Poll<(), SendError<I>> {
        Ok(Async::Ready(()))
    }
}

impl<T, I> Drop for Sender<T, I> where T: Array<Item=Option<I>> {
    fn drop(&mut self) {
        let shared = match self.shared.upgrade() {
            Some(shared) => shared,
            None => return,
        };
        // The number of existing `Weak` indicates if we are possibly the last
        // `Sender`. If we are the last, we possibly must notify a blocked
        // `Receiver`. `self.shared` is always one of the `Weak` to this shared
        // data. Therefore the smallest possible Rc::weak_count(&shared) is 1.
        if Rc::weak_count(&shared) == 1 {
            if let Some(task) = shared.borrow_mut().blocked_recv.take() {
                // Wake up receiver as its stream has ended
                task.notify();
            }
        }
    }
}

/// The receiving end of a channel which implements the `Stream` trait.
///
/// This is created by the `channel` function.
#[derive(Debug)]
pub struct Receiver<T, I> where T: Array<Item=Option<I>> {
    state: State<T, I>,
}

/// Possible states of a receiver. We're either Open (can receive more messages)
/// or we're closed with a list of messages we have left to receive.
#[derive(Debug)]
enum State<T, I> where T: Array<Item=Option<I>> {
    Open(Rc<RefCell<Shared<T, I>>>),
    Closed(FixedVecDeque<T>),
}

impl<T, I> Receiver<T, I> where T: Array<Item=Option<I>> {
    /// Closes the receiving half
    ///
    /// This prevents any further messages from being sent on the channel while
    /// still enabling the receiver to drain messages that are buffered.
    pub fn close(&mut self) {
        let (blockers, items) = match self.state {
            State::Open(ref state) => {
                let mut state = state.borrow_mut();
                let items = mem::replace(&mut state.buffer, FixedVecDeque::new());
                let blockers = mem::replace(&mut state.blocked_senders, VecDeque::new());
                (blockers, items)
            }
            State::Closed(_) => return,
        };
        self.state = State::Closed(items);
        for task in blockers {
            task.notify();
        }
    }
}

impl<T, I> Stream for Receiver<T, I> where T: Array<Item=Option<I>> {
    type Item = I;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let me = match self.state {
            State::Open(ref mut me) => me,
            State::Closed(ref mut items) => {
                return Ok(Async::Ready(items.pop_front().map(|r| r.take().unwrap())));
            }
        };

        if let Some(shared) = Rc::get_mut(me) {
            // All senders have been dropped, so drain the buffer and end the
            // stream.
            return Ok(Async::Ready(shared.borrow_mut().buffer.pop_front().map(|r| r.take().unwrap())));
        }

        let mut shared = me.borrow_mut();
        if let Some(msg) = shared.buffer.pop_front().map(|r| r.take().unwrap()) {
            if let Some(task) = shared.blocked_senders.pop_front() {
                drop(shared);
                task.notify();
            }
            Ok(Async::Ready(Some(msg)))
        } else {
            shared.blocked_recv = Some(task::current());
            Ok(Async::NotReady)
        }
    }
}

impl<T, I> Drop for Receiver<T, I> where T: Array<Item=Option<I>> {
    fn drop(&mut self) {
        self.close();
    }
}

/// Error type for sending, used when the receiving end of a channel is
/// dropped
pub struct SendError<I>(I);

impl<I> fmt::Debug for SendError<I> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("SendError")
            .field(&"...")
            .finish()
    }
}

impl<I> fmt::Display for SendError<I> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "send failed because receiver is gone")
    }
}

impl<I: Any> Error for SendError<I> {
    fn description(&self) -> &str {
        "send failed because receiver is gone"
    }
}

impl<I> SendError<I> {
    /// Returns the message that was attempted to be sent but failed.
    pub fn into_inner(self) -> I {
        self.0
    }
}
