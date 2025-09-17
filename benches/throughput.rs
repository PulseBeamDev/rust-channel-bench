use bytes::Bytes;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::{Sink, SinkExt, Stream, StreamExt};
use std::hint::black_box;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::task::JoinSet;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamMap;
use tokio_util::sync::PollSender;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const MESSAGE_SIZE: usize = 1200;

// Configuration for benchmarks
#[derive(Clone, Debug)]
struct BenchConfig {
    n_tasks: usize,
    n_msgs: usize,
    capacity: usize,
}

impl BenchConfig {
    fn total_messages(&self) -> usize {
        self.n_tasks * self.n_msgs
    }

    fn capacity_per_task(&self) -> usize {
        self.capacity / self.n_tasks
    }
}

// Benchmark configurations to test
fn get_benchmark_configs() -> Vec<BenchConfig> {
    let base_msgs = 64 * 1024; // Smaller than original for reasonable benchmark time
    let base_cap = 512 * 64;

    vec![
        BenchConfig {
            n_tasks: 2,
            n_msgs: base_msgs / 2,
            capacity: base_cap,
        },
        BenchConfig {
            n_tasks: 4,
            n_msgs: base_msgs / 4,
            capacity: base_cap,
        },
        BenchConfig {
            n_tasks: 8,
            n_msgs: base_msgs / 8,
            capacity: base_cap,
        },
        BenchConfig {
            n_tasks: 16,
            n_msgs: base_msgs / 16,
            capacity: base_cap,
        },
        BenchConfig {
            n_tasks: 64,
            n_msgs: base_msgs / 64,
            capacity: base_cap,
        },
        BenchConfig {
            n_tasks: 128,
            n_msgs: base_msgs / 128,
            capacity: base_cap,
        },
    ]
}

fn bench_tokio_cloned_sender(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokio_cloned_sender");

    for config in get_benchmark_configs() {
        group.throughput(Throughput::Elements(config.total_messages() as u64));
        group.bench_with_input(
            BenchmarkId::new("tasks", config.n_tasks),
            &config,
            |b, config| {
                let rt = Runtime::new().unwrap();
                b.to_async(&rt)
                    .iter(|| async { black_box(tokio_cloned_sender_impl(config.clone()).await) });
            },
        );
    }
    group.finish();
}

fn bench_tokio_merged_receiver(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokio_merged_receiver");

    for config in get_benchmark_configs() {
        group.throughput(Throughput::Elements(config.total_messages() as u64));
        group.bench_with_input(
            BenchmarkId::new("tasks", config.n_tasks),
            &config,
            |b, config| {
                let rt = Runtime::new().unwrap();
                b.to_async(&rt)
                    .iter(|| async { black_box(tokio_merged_receiver_impl(config.clone()).await) });
            },
        );
    }
    group.finish();
}

fn bench_flume_cloned_sender(c: &mut Criterion) {
    let mut group = c.benchmark_group("flume_cloned_sender");

    for config in get_benchmark_configs() {
        group.throughput(Throughput::Elements(config.total_messages() as u64));
        group.bench_with_input(
            BenchmarkId::new("tasks", config.n_tasks),
            &config,
            |b, config| {
                let rt = Runtime::new().unwrap();
                b.to_async(&rt)
                    .iter(|| async { black_box(flume_cloned_sender_impl(config.clone()).await) });
            },
        );
    }
    group.finish();
}

fn bench_flume_merged_receiver(c: &mut Criterion) {
    let mut group = c.benchmark_group("flume_merged_receiver");

    for config in get_benchmark_configs() {
        group.throughput(Throughput::Elements(config.total_messages() as u64));
        group.bench_with_input(
            BenchmarkId::new("tasks", config.n_tasks),
            &config,
            |b, config| {
                let rt = Runtime::new().unwrap();
                b.to_async(&rt)
                    .iter(|| async { black_box(flume_merged_receiver_impl(config.clone()).await) });
            },
        );
    }
    group.finish();
}

fn bench_async_channel_cloned_sender(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_channel_cloned_sender");

    for config in get_benchmark_configs() {
        group.throughput(Throughput::Elements(config.total_messages() as u64));
        group.bench_with_input(
            BenchmarkId::new("tasks", config.n_tasks),
            &config,
            |b, config| {
                let rt = Runtime::new().unwrap();
                b.to_async(&rt).iter(|| async {
                    black_box(async_channel_cloned_sender_impl(config.clone()).await)
                });
            },
        );
    }
    group.finish();
}

fn bench_async_channel_merged_receiver(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_channel_merged_receiver");

    for config in get_benchmark_configs() {
        group.throughput(Throughput::Elements(config.total_messages() as u64));
        group.bench_with_input(
            BenchmarkId::new("tasks", config.n_tasks),
            &config,
            |b, config| {
                let rt = Runtime::new().unwrap();
                b.to_async(&rt).iter(|| async {
                    black_box(async_channel_merged_receiver_impl(config.clone()).await)
                });
            },
        );
    }
    group.finish();
}

// Implementation functions (converted from original benchmarks)
async fn tokio_cloned_sender_impl(config: BenchConfig) {
    let (tx, rx) = tokio::sync::mpsc::channel::<Arc<Bytes>>(config.capacity);
    let mut join_set = JoinSet::new();

    for i in 0..config.n_tasks {
        let payload = Arc::new(Bytes::from(vec![(i % 256) as u8; MESSAGE_SIZE]));
        join_set.spawn(send_all(
            PollSender::new(tx.clone()),
            config.n_msgs,
            move |_| payload.clone(),
        ));
    }
    drop(tx);

    let rx = ReceiverStream::new(rx);
    join_set.spawn(recv_count(rx, config.total_messages()));
    join_all(join_set).await;
}

async fn tokio_merged_receiver_impl(config: BenchConfig) {
    let mut join_set = JoinSet::new();
    let mut rx_all = vec![];

    for i in 0..config.n_tasks {
        let (tx, rx) = tokio::sync::mpsc::channel::<Arc<Bytes>>(config.capacity_per_task());
        let payload = Arc::new(Bytes::from(vec![(i % 256) as u8; MESSAGE_SIZE]));
        join_set.spawn(send_all(PollSender::new(tx), config.n_msgs, move |_| {
            payload.clone()
        }));
        rx_all.push((i, ReceiverStream::new(rx)));
    }

    let rx = StreamMap::from_iter(rx_all);
    join_set.spawn(recv_count(rx, config.total_messages()));
    join_all(join_set).await;
}

async fn flume_cloned_sender_impl(config: BenchConfig) {
    let (tx, rx) = flume::bounded::<Arc<Bytes>>(config.capacity);
    let mut join_set = JoinSet::new();

    for i in 0..config.n_tasks {
        let tx = tx.clone();
        let payload = Arc::new(Bytes::from(vec![(i % 256) as u8; MESSAGE_SIZE]));
        join_set.spawn(send_all(tx.into_sink(), config.n_msgs, move |_| {
            payload.clone()
        }));
    }
    drop(tx);

    join_set.spawn(recv_count(rx.into_stream(), config.total_messages()));
    join_all(join_set).await;
}

async fn flume_merged_receiver_impl(config: BenchConfig) {
    let mut join_set = JoinSet::new();
    let mut rx_all = vec![];

    for i in 0..config.n_tasks {
        let (tx, rx) = flume::bounded::<Arc<Bytes>>(config.capacity_per_task());
        let payload = Arc::new(Bytes::from(vec![(i % 256) as u8; MESSAGE_SIZE]));
        join_set.spawn(send_all(tx.into_sink(), config.n_msgs, move |_| {
            payload.clone()
        }));
        rx_all.push((i, rx.into_stream()));
    }

    let rx = StreamMap::from_iter(rx_all);
    join_set.spawn(recv_count(rx, config.total_messages()));
    join_all(join_set).await;
}

async fn async_channel_cloned_sender_impl(config: BenchConfig) {
    let (tx, rx) = async_channel::bounded::<Arc<Bytes>>(config.capacity);
    let mut join_set = JoinSet::new();

    for i in 0..config.n_tasks {
        let tx = tx.clone();
        let payload = Arc::new(Bytes::from(vec![(i % 256) as u8; MESSAGE_SIZE]));
        join_set.spawn(async move {
            for _ in 0..config.n_msgs {
                tx.send(payload.clone()).await.map_err(|_| ()).unwrap();
            }
        });
    }
    drop(tx);

    join_set.spawn(recv_count(rx, config.total_messages()));
    join_all(join_set).await;
}

async fn async_channel_merged_receiver_impl(config: BenchConfig) {
    let mut join_set = JoinSet::new();
    let mut rx_all = vec![];

    for i in 0..config.n_tasks {
        let (tx, rx) = async_channel::bounded::<Arc<Bytes>>(config.capacity_per_task());
        let payload = Arc::new(Bytes::from(vec![(i % 256) as u8; MESSAGE_SIZE]));
        join_set.spawn(async move {
            for _ in 0..config.n_msgs {
                tx.send(payload.clone()).await.map_err(|_| ()).unwrap();
            }
        });
        rx_all.push((i, Box::pin(rx)));
    }

    let rx = StreamMap::from_iter(rx_all);
    join_set.spawn(recv_count(rx, config.total_messages()));
    join_all(join_set).await;
}

// Helper functions
async fn send_all<T: Send>(
    mut sink: impl Sink<T> + Unpin,
    count: usize,
    create_item: impl Fn(usize) -> T,
) {
    for i in 0..count {
        let value = create_item(i);
        sink.send(value).await.map_err(|_| ()).unwrap();
    }
}

async fn recv_count<T>(rx: impl Stream<Item = T>, expected: usize) {
    tokio::pin!(rx);
    let count = rx.count().await;
    assert_eq!(count, expected);
}

async fn join_all<T: 'static>(mut join_set: JoinSet<T>) {
    while let Some(r) = join_set.join_next().await {
        r.unwrap();
    }
}

criterion_group!(
    benches,
    bench_tokio_cloned_sender,
    bench_tokio_merged_receiver,
    bench_flume_cloned_sender,
    bench_flume_merged_receiver,
    bench_async_channel_cloned_sender,
    bench_async_channel_merged_receiver
);

criterion_main!(benches);
