#[macro_use]
extern crate criterion;
extern crate futures;
extern crate tokio;
extern crate tokio_current_thread;

use std::num::NonZeroU32;

use criterion::black_box;
use criterion::Criterion;
use futures::lazy;
use futures::unsync::mpsc as futures_mpsc;
use futures::{Future, Sink, Stream};
use tokio::prelude::*;
use tokio::runtime::current_thread::Runtime;

pub const BUF_SIZE: usize = 256;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("futures unsync", move |b| {
        b.iter(|| {
            let mut rt = Runtime::new().unwrap();

            let (tx, rx) = futures_mpsc::channel(BUF_SIZE);
            rt.spawn(
                stream::iter_ok::<_, ()>(1u32..10000)
                    .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                    .forward(tx.clone().sink_map_err(|_| ()))
                    .then(|_| Ok::<(), ()>(())),
            );
            rt.spawn(
                stream::iter_ok::<_, ()>(1u32..10000)
                    .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                    .forward(tx.clone().sink_map_err(|_| ()))
                    .then(|_| Ok::<(), ()>(())),
            );
            rt.spawn(
                stream::iter_ok::<_, ()>(1u32..10000)
                    .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                    .forward(tx.sink_map_err(|_| ()))
                    .then(|_| Ok::<(), ()>(())),
            );
            rt.block_on(
                rx.for_each(|r| {
                    let _ = black_box(r);
                    Ok::<(), ()>(())
                })
                .then(|_| Ok::<(), ()>(())),
            )
            .unwrap();
        })
    });
    c.bench_function("futures sync", move |b| {
        b.iter(|| {
            let mut rt = Runtime::new().unwrap();

            let (tx, rx) = futures::sync::mpsc::channel(BUF_SIZE);
            rt.spawn(
                stream::iter_ok::<_, ()>(1u32..10000)
                    .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                    .forward(tx.clone().sink_map_err(|_| ()))
                    .then(|_| Ok::<(), ()>(())),
            );
            rt.spawn(
                stream::iter_ok::<_, ()>(1u32..10000)
                    .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                    .forward(tx.clone().sink_map_err(|_| ()))
                    .then(|_| Ok::<(), ()>(())),
            );
            rt.spawn(
                stream::iter_ok::<_, ()>(1u32..10000)
                    .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                    .forward(tx.sink_map_err(|_| ()))
                    .then(|_| Ok::<(), ()>(())),
            );
            rt.block_on(
                rx.for_each(|r| {
                    let _ = black_box(r);
                    Ok::<(), ()>(())
                })
                .then(|_| Ok::<(), ()>(())),
            )
            .unwrap();
        })
    });
    c.bench_function("fixed", move |b| {
        b.iter(|| {
            let mut rt = Runtime::new().unwrap();

            let (tx, rx) = fixed_mpsc::channel::<[Option<NonZeroU32>; BUF_SIZE], NonZeroU32>();
            rt.spawn(
                stream::iter_ok::<_, ()>(1u32..10000)
                    .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                    .forward(tx.clone().sink_map_err(|_| ()))
                    .then(|_| Ok::<(), ()>(())),
            );
            rt.spawn(
                stream::iter_ok::<_, ()>(1u32..10000)
                    .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                    .forward(tx.clone().sink_map_err(|_| ()))
                    .then(|_| Ok::<(), ()>(())),
            );
            rt.spawn(
                stream::iter_ok::<_, ()>(1u32..10000)
                    .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                    .forward(tx.sink_map_err(|_| ()))
                    .then(|_| Ok::<(), ()>(())),
            );
            rt.block_on(
                rx.for_each(|r| {
                    let _ = black_box(r);
                    Ok::<(), ()>(())
                })
                .then(|_| Ok::<(), ()>(())),
            )
            .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
