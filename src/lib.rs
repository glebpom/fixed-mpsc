extern crate fixed_vec_deque;

use core::mem;

pub use fixed_mpsc::{channel, Receiver, Sender};

mod fixed_mpsc;

/// Trait which defines the element of `FixedVecDeque` inside the `fixed-mpsc` channel
///
/// We need a container with defined `Default` trait in order to store the data in `FixedVecDeque`.
/// This trait should be implemented on the type, which is passed to an array storage
pub trait Container<I>: Default + Sized {
    fn extract(&mut self) -> I;
    fn wrap(item: I) -> Self;
}

impl<I> Container<I> for Option<I> {
    fn extract(&mut self) -> I {
        self.take().unwrap()
    }
    fn wrap(item: I) -> Self {
        Some(item)
    }
}

impl<I> Container<I> for I
where
    I: Default,
{
    fn extract(&mut self) -> I {
        mem::replace(self, Default::default())
    }
    fn wrap(item: I) -> Self {
        item
    }
}
