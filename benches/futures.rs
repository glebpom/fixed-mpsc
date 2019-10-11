#[macro_use]
extern crate criterion;
extern crate fixed_vec_deque;
extern crate futures;
extern crate tokio;
extern crate tokio_current_thread;

use std::mem;
use std::num::NonZeroU32;

use criterion::black_box;
use criterion::BenchmarkId;
use criterion::Criterion;
use fixed_vec_deque::FixedVecDeque;
use futures::lazy;
use futures::unsync::mpsc as futures_mpsc;
use futures::{Future, Sink, Stream};
use std::collections::HashMap;
use tokio::prelude::*;
use tokio::runtime::current_thread::Runtime;

#[derive(Debug)]
struct LargeStruct {
    a: u128,
    b: [u8; 32],
    c: String,
    d: HashMap<u32, u128>,
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("channels");

    for size in [16, 64, 256, 1024].iter() {
        group.bench_with_input(
            BenchmarkId::new("futures u32", size),
            size,
            move |b, &size| {
                b.iter(|| {
                    let mut rt = Runtime::new().unwrap();

                    let (tx, rx) = futures_mpsc::channel(size);
                    for _ in 0..4 {
                        rt.spawn(
                            stream::iter_ok::<_, ()>(1u32..10000)
                                .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                                .forward(tx.clone().sink_map_err(|_| ()))
                                .then(|_| Ok::<(), ()>(())),
                        );
                    }
                    mem::drop(tx);
                    rt.block_on(
                        rx.for_each(|r| {
                            let _ = black_box(r);
                            Ok::<(), ()>(())
                        })
                        .then(|_| Ok::<(), ()>(())),
                    )
                    .unwrap();
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("futures LargeStruct", size),
            size,
            move |b, &size| {
                b.iter(|| {
                    let mut rt = Runtime::new().unwrap();

                    let (tx, rx) = futures_mpsc::channel(size);
                    for _ in 0..4 {
                        rt.spawn(
                            stream::iter_ok::<_, ()>(1u32..10000)
                                .and_then(|r| {
                                    lazy(move || {
                                        Ok(black_box(LargeStruct {
                                            a: 1235245,
                                            b: [50; 32],
                                            c: "12321525".to_string(),
                                            d: Default::default(),
                                        }))
                                    })
                                })
                                .forward(tx.clone().sink_map_err(|_| ()))
                                .then(|_| Ok::<(), ()>(())),
                        );
                    }
                    mem::drop(tx);
                    rt.block_on(
                        rx.for_each(|r| {
                            let _ = black_box(r);
                            Ok::<(), ()>(())
                        })
                        .then(|_| Ok::<(), ()>(())),
                    )
                    .unwrap();
                })
            },
        );
    }

    group.bench_function(BenchmarkId::new("fixed u32", 16), move |b| {
        b.iter(|| {
            let mut rt = Runtime::new().unwrap();

            let (tx, rx) = fixed_mpsc::channel::<[Option<NonZeroU32>; 16], NonZeroU32>();
            for _ in 0..4 {
                rt.spawn(
                    stream::iter_ok::<_, ()>(1u32..10000)
                        .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                        .forward(tx.clone().sink_map_err(|_| ()))
                        .then(|_| Ok::<(), ()>(())),
                );
            }
            mem::drop(tx);
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

    group.bench_function(BenchmarkId::new("fixed u32", 64), move |b| {
        b.iter(|| {
            let mut rt = Runtime::new().unwrap();

            let (tx, rx) = fixed_mpsc::channel::<[Option<NonZeroU32>; 64], NonZeroU32>();
            for _ in 0..4 {
                rt.spawn(
                    stream::iter_ok::<_, ()>(1u32..10000)
                        .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                        .forward(tx.clone().sink_map_err(|_| ()))
                        .then(|_| Ok::<(), ()>(())),
                );
            }
            mem::drop(tx);
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

    group.bench_function(BenchmarkId::new("fixed u32", 256), move |b| {
        b.iter(|| {
            let mut rt = Runtime::new().unwrap();

            let (tx, rx) = fixed_mpsc::channel::<[Option<NonZeroU32>; 256], NonZeroU32>();
            for _ in 0..4 {
                rt.spawn(
                    stream::iter_ok::<_, ()>(1u32..10000)
                        .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                        .forward(tx.clone().sink_map_err(|_| ()))
                        .then(|_| Ok::<(), ()>(())),
                );
            }
            mem::drop(tx);
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

    group.bench_function(BenchmarkId::new("fixed u32", 1024), move |b| {
        b.iter(|| {
            let mut rt = Runtime::new().unwrap();

            let (tx, rx) = fixed_mpsc::channel::<[Option<NonZeroU32>; 1024], NonZeroU32>();
            for _ in 0..4 {
                rt.spawn(
                    stream::iter_ok::<_, ()>(1u32..10000)
                        .and_then(|r| lazy(move || Ok(NonZeroU32::new(r).unwrap())))
                        .forward(tx.clone().sink_map_err(|_| ()))
                        .then(|_| Ok::<(), ()>(())),
                );
            }
            mem::drop(tx);
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

    group.bench_function(BenchmarkId::new("fixed LargeStruct", 16), move |b| {
        b.iter(|| {
            let mut rt = Runtime::new().unwrap();

            let (tx, rx) = fixed_mpsc::channel::<[Option<LargeStruct>; 16], LargeStruct>();
            for _ in 0..4 {
                rt.spawn(
                    stream::iter_ok::<_, ()>(1u32..10000)
                        .and_then(|r| {
                            lazy(move || {
                                Ok(black_box(LargeStruct {
                                    a: 1235245,
                                    b: [50; 32],
                                    c: "12321525".to_string(),
                                    d: Default::default(),
                                }))
                            })
                        })
                        .forward(tx.clone().sink_map_err(|_| ()))
                        .then(|_| Ok::<(), ()>(())),
                );
            }
            mem::drop(tx);
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

    group.bench_function(BenchmarkId::new("fixed LargeStruct", 64), move |b| {
        b.iter(|| {
            let mut rt = Runtime::new().unwrap();

            let (tx, rx) = fixed_mpsc::channel::<[Option<LargeStruct>; 64], LargeStruct>();
            for _ in 0..4 {
                rt.spawn(
                    stream::iter_ok::<_, ()>(1u32..10000)
                        .and_then(|r| {
                            lazy(move || {
                                Ok(black_box(LargeStruct {
                                    a: 1235245,
                                    b: [50; 32],
                                    c: "12321525".to_string(),
                                    d: Default::default(),
                                }))
                            })
                        })
                        .forward(tx.clone().sink_map_err(|_| ()))
                        .then(|_| Ok::<(), ()>(())),
                );
            }
            mem::drop(tx);
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

    group.bench_function(BenchmarkId::new("fixed LargeStruct", 256), move |b| {
        b.iter(|| {
            let mut rt = Runtime::new().unwrap();

            let (tx, rx) = fixed_mpsc::channel::<[Option<LargeStruct>; 256], LargeStruct>();
            for _ in 0..4 {
                rt.spawn(
                    stream::iter_ok::<_, ()>(1u32..10000)
                        .and_then(|r| {
                            lazy(move || {
                                Ok(black_box(LargeStruct {
                                    a: 1235245,
                                    b: [50; 32],
                                    c: "12321525".to_string(),
                                    d: Default::default(),
                                }))
                            })
                        })
                        .forward(tx.clone().sink_map_err(|_| ()))
                        .then(|_| Ok::<(), ()>(())),
                );
            }
            mem::drop(tx);
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

    group.bench_function(BenchmarkId::new("fixed LargeStruct", 1024), move |b| {
        b.iter(|| {
            let mut rt = Runtime::new().unwrap();

            let (tx, rx) = fixed_mpsc::channel::<[Option<LargeStruct>; 1024], LargeStruct>();
            for _ in 0..4 {
                rt.spawn(
                    stream::iter_ok::<_, ()>(1u32..10000)
                        .and_then(|r| {
                            lazy(move || {
                                Ok(black_box(LargeStruct {
                                    a: 1235245,
                                    b: [50; 32],
                                    c: "12321525".to_string(),
                                    d: Default::default(),
                                }))
                            })
                        })
                        .forward(tx.clone().sink_map_err(|_| ()))
                        .then(|_| Ok::<(), ()>(())),
                );
            }
            mem::drop(tx);
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

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
