use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use concurrent_queue::ConcurrentQueue;
use futures_util::stream::Stream;
use futures_util::task::AtomicWaker;

/// A sending half of the channel. Each `Sender` owns its own SPSC queue.
pub struct Sender<T> {
    id: usize,
    queue: Arc<ConcurrentQueue<T>>,
    shared: Arc<Shared<T>>,
}

/// The receiving half of the channel.
pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    round_robin: bool,
    last_index: usize,
}

struct Shared<T> {
    queues: parking_lot::Mutex<Vec<Option<Arc<ConcurrentQueue<T>>>>>,
    ready: ConcurrentQueue<usize>, // sender ids with messages
    ready_flags: Vec<AtomicBool>,  // prevent duplicate enqueue
    recv_waker: AtomicWaker,
    sender_count: AtomicUsize,
    closed: AtomicBool,
}

/// Create a bounded multi-SPSC channel.
pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared {
        queues: parking_lot::Mutex::new(Vec::new()),
        ready: ConcurrentQueue::bounded(capacity * 64),
        ready_flags: Vec::new(),
        recv_waker: AtomicWaker::new(),
        sender_count: AtomicUsize::new(0),
        closed: AtomicBool::new(false),
    });

    // create first sender
    let (id, queue) = new_queue(&shared, capacity);
    {
        let mut queues = shared.queues.lock();
        queues.push(Some(queue.clone()));
    }
    shared.ready_flags.push(AtomicBool::new(false));
    shared.sender_count.store(1, Ordering::SeqCst);

    let sender = Sender {
        id,
        queue,
        shared: shared.clone(),
    };
    let receiver = Receiver {
        shared,
        round_robin: false,
        last_index: 0,
    };
    (sender, receiver)
}

fn new_queue<T>(shared: &Arc<Shared<T>>, capacity: usize) -> (usize, Arc<ConcurrentQueue<T>>) {
    let mut queues = shared.queues.lock();
    let id = queues.len();
    (id, Arc::new(ConcurrentQueue::bounded(capacity)))
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let cap = self.queue.capacity().unwrap_or(1024);
        let (id, queue) = new_queue(&self.shared, cap);
        {
            let mut queues = self.shared.queues.lock();
            queues.push(Some(queue.clone()));
        }
        self.shared.ready_flags.push(AtomicBool::new(false));
        self.shared.sender_count.fetch_add(1, Ordering::SeqCst);

        Sender {
            id,
            queue,
            shared: self.shared.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if self.shared.sender_count.fetch_sub(1, Ordering::SeqCst) == 1 {
            // last sender dropped
            self.shared.closed.store(true, Ordering::SeqCst);
            self.shared.recv_waker.wake();
        }
    }
}

impl<T> Sender<T> {
    pub fn try_send(&self, val: T) -> Result<(), T> {
        match self.queue.push(val) {
            Ok(()) => {
                let flag = &self.shared.ready_flags[self.id];
                if !flag.swap(true, Ordering::SeqCst) {
                    let _ = self.shared.ready.push(self.id);
                }
                self.shared.recv_waker.wake();
                Ok(())
            }
            Err(e) => Err(e.into_inner()),
        }
    }

    pub async fn send(&self, val: T) -> Result<(), T> {
        SendFuture {
            sender: self,
            value: Some(val),
        }
        .await
    }
}

struct SendFuture<'a, T> {
    sender: &'a Sender<T>,
    value: Option<T>,
}

impl<'a, T> Future for SendFuture<'a, T> {
    type Output = Result<(), T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let val = self.value.take().unwrap();
        match self.sender.queue.push(val) {
            Ok(()) => {
                let flag = &self.sender.shared.ready_flags[self.sender.id];
                if !flag.swap(true, Ordering::SeqCst) {
                    let _ = self.sender.shared.ready.push(self.sender.id);
                }
                self.sender.shared.recv_waker.wake();
                Poll::Ready(Ok(()))
            }
            Err(err) => {
                self.value = Some(err.into_inner());
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

impl<T> Receiver<T> {
    pub fn enable_round_robin(&mut self, enable: bool) {
        self.round_robin = enable;
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        if self.shared.closed.load(Ordering::SeqCst)
            && self.shared.sender_count.load(Ordering::SeqCst) == 0
        {
            return Err(TryRecvError::Closed);
        }

        let id = match self.shared.ready.pop() {
            Ok(id) => {
                self.shared.ready_flags[id].store(false, Ordering::SeqCst);
                id
            }
            Err(concurrent_queue::PopError::Empty) => {
                if self.shared.closed.load(Ordering::SeqCst)
                    && self.shared.sender_count.load(Ordering::SeqCst) == 0
                {
                    return Err(TryRecvError::Closed);
                } else {
                    return Err(TryRecvError::Empty);
                }
            }
            Err(_) => return Err(TryRecvError::Closed),
        };

        let queues = self.shared.queues.lock();
        if let Some(Some(q)) = queues.get(id) {
            match q.pop() {
                Ok(val) => Ok(val),
                Err(_) => Err(TryRecvError::Empty),
            }
        } else {
            Err(TryRecvError::Closed)
        }
    }

    pub async fn recv(&self) -> Option<T> {
        RecvFuture { receiver: self }.await
    }
}

pub enum TryRecvError {
    Empty,
    Closed,
}

struct RecvFuture<'a, T> {
    receiver: &'a Receiver<T>,
}

impl<'a, T> Future for RecvFuture<'a, T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.receiver.try_recv() {
            Ok(val) => Poll::Ready(Some(val)),
            Err(TryRecvError::Closed) => Poll::Ready(None),
            Err(TryRecvError::Empty) => {
                self.receiver.shared.recv_waker.register(cx.waker());
                Poll::Pending
            }
        }
    }
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.try_recv() {
            Ok(val) => Poll::Ready(Some(val)),
            Err(TryRecvError::Closed) => Poll::Ready(None),
            Err(TryRecvError::Empty) => {
                self.shared.recv_waker.register(cx.waker());
                Poll::Pending
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use tokio::task;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn basic_send_recv() {
        let (tx, mut rx) = channel::<i32>(4);
        tx.send(42).await.unwrap();
        assert_eq!(rx.recv().await, Some(42));
    }

    #[tokio::test]
    async fn multiple_senders() {
        let (tx, mut rx) = channel::<i32>(8);
        let tx2 = tx.clone();
        tx.send(1).await.unwrap();
        tx2.send(2).await.unwrap();
        let mut got = vec![];
        got.push(rx.recv().await.unwrap());
        got.push(rx.recv().await.unwrap());
        got.sort();
        assert_eq!(got, vec![1, 2]);
    }

    #[tokio::test]
    async fn recv_none_after_all_senders_dropped() {
        let (tx, mut rx) = channel::<i32>(2);
        tx.send(7).await.unwrap();
        drop(tx);
        assert_eq!(rx.recv().await, Some(7));
        assert_eq!(rx.recv().await, None);
    }

    #[tokio::test]
    async fn backpressure_works() {
        let (tx, _rx) = channel::<i32>(1);
        assert!(tx.try_send(1).is_ok());
        assert!(tx.try_send(2).is_err());
    }

    #[tokio::test]
    async fn fairness_round_robin() {
        let (tx, mut rx) = channel::<i32>(2);
        let tx2 = tx.clone();
        rx.enable_round_robin(true);

        tx.try_send(1).unwrap();
        tx2.try_send(2).unwrap();

        let mut results = vec![];
        results.push(rx.recv().await.unwrap());
        results.push(rx.recv().await.unwrap());

        results.sort();
        assert_eq!(results, vec![1, 2]);
    }

    #[tokio::test]
    async fn stream_usage() {
        let (tx, mut rx) = channel::<i32>(4);
        for i in 0..3 {
            tx.send(i).await.unwrap();
        }
        let mut results = vec![];
        while let Some(v) = rx.next().await {
            results.push(v);
            if results.len() == 3 {
                break;
            }
        }
        assert_eq!(results, vec![0, 1, 2]);
    }

    #[tokio::test]
    async fn stress_test() {
        let (tx, rx) = channel::<usize>(64);
        let tx2 = tx.clone();

        let t1 = task::spawn(async move {
            for i in 0..1000 {
                tx.send(i).await.unwrap();
            }
        });
        let t2 = task::spawn(async move {
            for i in 1000..2000 {
                tx2.send(i).await.unwrap();
            }
        });

        let mut received = Vec::new();
        while let Some(v) = rx.recv().await {
            received.push(v);
            if received.len() == 2000 {
                break;
            }
        }

        t1.await.unwrap();
        t2.await.unwrap();

        received.sort();
        assert_eq!(received.len(), 2000);
    }
}
