#[macro_use]
extern crate criterion;
extern crate futures;
extern crate tokio;
extern crate tokio_current_thread;

use tokio::prelude::*;
use criterion::black_box;
use criterion::Criterion;
use futures::{Future, Sink, Stream};
use futures::unsync::mpsc as futures_mpsc;
use tokio::runtime::current_thread::Runtime;
use tokio::prelude::*;
use std::num::NonZeroU32;
use futures::lazy;

pub const BUF_SIZE: usize = 64;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("futures", move |b| b.iter(|| {
        let mut rt = Runtime::new().unwrap();

        let (tx, rx) = futures_mpsc::channel(BUF_SIZE);
        rt.spawn(stream::iter_ok::<_, ()>(1u32..10000).and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap()))).forward(tx.clone().sink_map_err(|_| ())).then(|_| Ok::<(),()>(())));
        rt.spawn(stream::iter_ok::<_, ()>(1u32..10000).and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap()))).forward(tx.clone().sink_map_err(|_| ())).then(|_| Ok::<(),()>(())));
        rt.spawn(stream::iter_ok::<_, ()>(1u32..10000).and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap()))).forward(tx.sink_map_err(|_| ())).then(|_| Ok::<(),()>(())));
        rt.block_on(
            rx
                .for_each(|r| {
                    let _ = black_box(r);
                    Ok::<(),()>(())
                })
                .then(|_| Ok::<(), ()>(()))
        );
    }));
    c.bench_function("fixed", move |b| b.iter(|| {
        let mut rt = Runtime::new().unwrap();

        let (tx, rx) = fixed_mpsc::channel::<[Option<NonZeroU32>; BUF_SIZE], NonZeroU32>();
        rt.spawn(stream::iter_ok::<_, ()>(1u32..10000).and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap()))).forward(tx.clone().sink_map_err(|_| ())).then(|_| Ok::<(), ()>(())));
        rt.spawn(stream::iter_ok::<_, ()>(1u32..10000).and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap()))).forward(tx.clone().sink_map_err(|_| ())).then(|_| Ok::<(), ()>(())));
        rt.spawn(stream::iter_ok::<_, ()>(1u32..10000).and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap()))).forward(tx.sink_map_err(|_| ())).then(|_| Ok::<(), ()>(())));
        rt.block_on(
            rx
                .for_each(|r| {
                    let _ = black_box(r);
                    Ok::<(),()>(())
                })
                .then(|_| Ok::<(), ()>(()))
        );
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);