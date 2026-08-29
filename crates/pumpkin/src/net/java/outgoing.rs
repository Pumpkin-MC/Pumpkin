//! Tick-thread enqueue is non-blocking; socket write/flush is a dedicated task.
//! Vanilla `suspendFlushing` / `resumeFlushing` write a game tick without flushing,
//! then `Connection.flushChannel` once so `CBlockEvent` and entity motion share a tick.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use bytes::Bytes;
use pumpkin_protocol::java::packet_encoder::TCPNetworkEncoder;
use tokio::io::AsyncWrite;
use tokio::sync::{
    mpsc::{Receiver, error::TryRecvError},
    oneshot,
};
use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;
use tracing::warn;

/// Off-tick fallback. Play ticks flush from `OutgoingPacket::Flush`.
const TICK_FLUSH_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, PartialEq, Eq)]
enum FlushRequest {
    None,
    /// `send_packet_now`. Vanilla `send(..., flush)` is false while suspended.
    IfNotSuspended,
    /// `Connection.flushChannel`. Always, including while still suspended.
    Always,
}

impl FlushRequest {
    const fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Always, _) | (_, Self::Always) => Self::Always,
            (Self::IfNotSuspended, _) | (_, Self::IfNotSuspended) => Self::IfNotSuspended,
            (Self::None, Self::None) => Self::None,
        }
    }

    const fn should_flush(self, suspended: bool) -> bool {
        match self {
            Self::Always => true,
            Self::IfNotSuspended => !suspended,
            Self::None => false,
        }
    }
}

pub enum OutgoingPacket {
    Data {
        data: Bytes,
        completion: Option<oneshot::Sender<()>>,
    },
    /// End-of-tick barrier (`Connection.flushChannel`).
    Flush,
}

impl OutgoingPacket {
    pub const fn normal(data: Bytes) -> Self {
        Self::Data {
            data,
            completion: None,
        }
    }

    pub const fn high_priority(data: Bytes, completion: oneshot::Sender<()>) -> Self {
        Self::Data {
            data,
            completion: Some(completion),
        }
    }
}

struct WriterState<W: AsyncWrite + Unpin> {
    writer: TCPNetworkEncoder<W>,
    unflushed: bool,
    pending_completions: Vec<oneshot::Sender<()>>,
    close_token: CancellationToken,
    id: u64,
}

impl<W: AsyncWrite + Unpin> WriterState<W> {
    async fn write_data(&mut self, data: Bytes) -> bool {
        if let Err(err) = self.writer.write_packet(data).await {
            if !self.close_token.is_cancelled() {
                warn!("Failed to send packet to client {}: {err}", self.id);
            }
            return false;
        }
        self.unflushed = true;
        true
    }

    async fn flush(&mut self) -> bool {
        if self.unflushed {
            if let Err(err) = self.writer.flush().await {
                if !self.close_token.is_cancelled() {
                    warn!("Failed to flush packets for client {}: {err}", self.id);
                }
                return false;
            }
            self.unflushed = false;
        }
        for completion in self.pending_completions.drain(..) {
            let _ = completion.send(());
        }
        true
    }

    /// Completions wait for TCP flush when we flush; complete immediately while suspended.
    fn complete_pending(&mut self) {
        for completion in self.pending_completions.drain(..) {
            let _ = completion.send(());
        }
    }

    /// Returns `None` on a write/flush failure.
    async fn handle(&mut self, packet: OutgoingPacket) -> Option<FlushRequest> {
        match packet {
            OutgoingPacket::Flush => Some(FlushRequest::Always),
            OutgoingPacket::Data { data, completion } => {
                if !self.write_data(data).await {
                    return None;
                }
                let request = if completion.is_some() {
                    FlushRequest::IfNotSuspended
                } else {
                    FlushRequest::None
                };
                if let Some(completion) = completion {
                    self.pending_completions.push(completion);
                }
                Some(request)
            }
        }
    }
}

fn try_recv_next(
    priority: &mut Receiver<OutgoingPacket>,
    normal: &mut Receiver<OutgoingPacket>,
) -> Result<OutgoingPacket, TryRecvError> {
    match priority.try_recv() {
        Ok(packet) => Ok(packet),
        Err(TryRecvError::Empty) => normal.try_recv(),
        Err(TryRecvError::Disconnected) => match normal.try_recv() {
            Ok(packet) => Ok(packet),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => {
                Err(TryRecvError::Disconnected)
            }
        },
    }
}

pub async fn run_outgoing_packet_writer<W: AsyncWrite + Unpin>(
    mut packet_receiver: Receiver<OutgoingPacket>,
    mut priority_packet_receiver: Receiver<OutgoingPacket>,
    writer: TCPNetworkEncoder<W>,
    close_token: CancellationToken,
    suspend_flushing: Arc<AtomicBool>,
    id: u64,
) {
    let mut flush_interval = tokio::time::interval(TICK_FLUSH_INTERVAL);
    flush_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    // `interval` fires immediately; skip so an empty connection is not flushed.
    flush_interval.tick().await;

    let mut state = WriterState {
        writer,
        unflushed: false,
        pending_completions: Vec::new(),
        close_token: close_token.clone(),
        id,
    };

    loop {
        if close_token.is_cancelled() {
            break;
        }

        let recv_result = tokio::select! {
            biased;
            () = close_token.cancelled() => None,
            res = priority_packet_receiver.recv() => res,
            res = packet_receiver.recv() => res,
            _ = flush_interval.tick(), if state.unflushed
                && !suspend_flushing.load(Ordering::Acquire) =>
            {
                if !state.flush().await {
                    close_token.cancel();
                    break;
                }
                continue;
            }
        };

        let Some(first) = recv_result else {
            break;
        };

        let Some(mut flush_request) = state.handle(first).await else {
            close_token.cancel();
            break;
        };

        // Drain whatever is already queued. No packet-count cap: splitting at 64
        // is what mixed tick N with N+1.
        loop {
            match try_recv_next(&mut priority_packet_receiver, &mut packet_receiver) {
                Ok(packet) => {
                    let Some(request) = state.handle(packet).await else {
                        close_token.cancel();
                        return;
                    };
                    flush_request = flush_request.merge(request);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if !state.flush().await {
                        close_token.cancel();
                    }
                    return;
                }
            }
        }

        let suspended = suspend_flushing.load(Ordering::Acquire);
        if flush_request.should_flush(suspended) {
            if !state.flush().await {
                close_token.cancel();
                break;
            }
        } else {
            state.complete_pending();
        }
    }

    let _ = state.flush().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;
    use std::sync::atomic::AtomicUsize;
    use std::task::{Context, Poll};
    use std::time::Instant;

    struct RecordingWriter {
        flushes: Arc<AtomicUsize>,
    }

    impl AsyncWrite for RecordingWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            self.flushes.fetch_add(1, Ordering::SeqCst);
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    fn packet(n: u8) -> OutgoingPacket {
        OutgoingPacket::normal(Bytes::from(vec![n]))
    }

    async fn run_writer(
        rx: Receiver<OutgoingPacket>,
        pri_rx: Receiver<OutgoingPacket>,
        flushes: Arc<AtomicUsize>,
        suspend: Arc<AtomicBool>,
        close: CancellationToken,
    ) {
        run_outgoing_packet_writer(
            rx,
            pri_rx,
            TCPNetworkEncoder::new(RecordingWriter { flushes }),
            close,
            suspend,
            0,
        )
        .await;
    }

    #[tokio::test]
    async fn tick_barrier_flushes_once_not_every_sixty_four() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let (pri_tx, pri_rx) = tokio::sync::mpsc::channel(4096);
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(false));
        let close = CancellationToken::new();

        let writer = tokio::spawn(run_writer(
            rx,
            pri_rx,
            flushes.clone(),
            suspend,
            close.clone(),
        ));

        for i in 0..200u8 {
            tx.try_send(packet(i)).unwrap();
        }
        tx.try_send(OutgoingPacket::Flush).unwrap();

        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(
            flushes.load(Ordering::SeqCst),
            1,
            "200 packets plus a tick barrier must be one flush, not 200/64 batches"
        );

        drop(tx);
        drop(pri_tx);
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn fifty_ms_interval_flushes_when_no_tick_barrier() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let (_pri_tx, pri_rx) = tokio::sync::mpsc::channel(4096);
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(false));
        let close = CancellationToken::new();

        let writer = tokio::spawn(run_writer(
            rx,
            pri_rx,
            flushes.clone(),
            suspend,
            close.clone(),
        ));

        tx.try_send(packet(1)).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(flushes.load(Ordering::SeqCst), 0);

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(
            flushes.load(Ordering::SeqCst) >= 1,
            "unflushed packets must flush on the 50ms tick cadence"
        );

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn suspend_flushing_holds_the_fifty_ms_flush_until_resume() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let (_pri_tx, pri_rx) = tokio::sync::mpsc::channel(4096);
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = tokio::spawn(run_writer(
            rx,
            pri_rx,
            flushes.clone(),
            suspend.clone(),
            close.clone(),
        ));

        tx.try_send(packet(1)).unwrap();
        tokio::time::sleep(Duration::from_millis(80)).await;
        assert_eq!(
            flushes.load(Ordering::SeqCst),
            0,
            "mid-tick 50ms timer must not flush while suspendFlushing is set"
        );

        suspend.store(false, Ordering::Release);
        tx.try_send(OutgoingPacket::Flush).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(flushes.load(Ordering::SeqCst), 1);

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn high_priority_does_not_flush_while_suspended() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let (pri_tx, pri_rx) = tokio::sync::mpsc::channel(4096);
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = tokio::spawn(run_writer(
            rx,
            pri_rx,
            flushes.clone(),
            suspend.clone(),
            close.clone(),
        ));

        tx.try_send(packet(1)).unwrap();
        let (done_tx, done_rx) = oneshot::channel();
        pri_tx
            .try_send(OutgoingPacket::high_priority(
                Bytes::from_static(&[2]),
                done_tx,
            ))
            .unwrap();
        tokio::time::timeout(Duration::from_millis(50), done_rx)
            .await
            .expect("send_packet_now must complete without waiting for tick-end flush")
            .expect("writer dropped");
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(
            flushes.load(Ordering::SeqCst),
            0,
            "high-priority must not flush CBlockEvent / entity motion mid-tick"
        );

        tx.try_send(OutgoingPacket::Flush).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(flushes.load(Ordering::SeqCst), 1);

        drop(tx);
        drop(pri_tx);
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn flush_barrier_flushes_while_still_suspended() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let (_pri_tx, pri_rx) = tokio::sync::mpsc::channel(4096);
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = tokio::spawn(run_writer(
            rx,
            pri_rx,
            flushes.clone(),
            suspend,
            close.clone(),
        ));

        tx.try_send(packet(1)).unwrap();
        tx.try_send(OutgoingPacket::Flush).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(
            flushes.load(Ordering::SeqCst),
            1,
            "resumeFlushing queues Flush before lifting suspendFlushing"
        );

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn tick_thread_enqueue_is_isolated_from_writer() {
        let (tx, rx) = tokio::sync::mpsc::channel::<OutgoingPacket>(4096);
        let (_pri_tx, pri_rx) = tokio::sync::mpsc::channel(4096);
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = tokio::spawn(run_writer(
            rx,
            pri_rx,
            flushes.clone(),
            suspend,
            close.clone(),
        ));

        let start = Instant::now();
        for i in 0..200u8 {
            tx.try_send(packet(i)).unwrap();
        }
        assert!(
            start.elapsed() < Duration::from_millis(100),
            "try_send must not wait on socket write/flush"
        );

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }
}
