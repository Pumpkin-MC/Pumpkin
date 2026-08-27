//! Outgoing Java packet writer: tick-thread enqueue stays non-blocking, socket
//! write/flush runs on a dedicated task.
//!
//! Vanilla (`ServerCommonPacketListenerImpl.suspendFlushing` /
//! `resumeFlushing`, `Connection.tick`) writes every packet of a game tick
//! without flushing, then flushes once at tick end (~50ms at 20 TPS). Flushing
//! every N packets splits a tick across TCP flushes and lets tick N+1 mix in.

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

/// Vanilla game-tick length (`MinecraftServer` 20 TPS). Socket flush cadence
/// matches `Connection.tick` / `resumeFlushing`, not a packet-count batch.
const TICK_FLUSH_INTERVAL: Duration = Duration::from_millis(50);

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

    /// Returns `None` on a write/flush failure. `Some(true)` means flush now
    /// (tick barrier or high-priority `send_packet_now`).
    async fn handle(&mut self, packet: OutgoingPacket) -> Option<bool> {
        match packet {
            OutgoingPacket::Flush => Some(true),
            OutgoingPacket::Data { data, completion } => {
                if !self.write_data(data).await {
                    return None;
                }
                let immediate = completion.is_some();
                if let Some(completion) = completion {
                    self.pending_completions.push(completion);
                }
                Some(immediate)
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

        let Some(mut flush_now) = state.handle(first).await else {
            close_token.cancel();
            break;
        };

        // Drain whatever is already queued. No packet-count cap: splitting at 64
        // is what mixed tick N with N+1.
        loop {
            match try_recv_next(&mut priority_packet_receiver, &mut packet_receiver) {
                Ok(packet) => {
                    let Some(immediate) = state.handle(packet).await else {
                        close_token.cancel();
                        return;
                    };
                    flush_now |= immediate;
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

        if flush_now && !state.flush().await {
            close_token.cancel();
            break;
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
