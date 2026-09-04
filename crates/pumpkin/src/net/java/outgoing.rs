//! Tick-thread enqueue is non-blocking; socket write/flush is a dedicated task.
//! Vanilla `suspendFlushing` / `resumeFlushing` write a game tick without flushing,
//! then `Connection.flushChannel` once so `CBlockEvent` and entity motion share a tick.

use std::collections::VecDeque;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

use bytes::Bytes;
use pumpkin_protocol::{
    MAX_PACKET_SIZE, PacketEncodeError, java::packet_encoder::TCPNetworkEncoder,
};
use tokio::io::AsyncWrite;
use tokio::sync::{
    Notify,
    mpsc::{Receiver, error::TryRecvError},
    oneshot,
};
use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;
use tracing::warn;

/// `resumeFlushing` when the FIFO is full: cannot drop the tick barrier.
#[derive(Clone)]
pub struct TickFlush {
    pending: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl TickFlush {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn request(&self) {
        self.pending.store(true, Ordering::Release);
        self.notify.notify_one();
    }

    fn take(&self) -> bool {
        self.pending.swap(false, Ordering::AcqRel)
    }
}

impl Default for TickFlush {
    fn default() -> Self {
        Self::new()
    }
}

/// Off-tick fallback. Play ticks flush from `OutgoingPacket::Flush`.
const TICK_FLUSH_INTERVAL: Duration = Duration::from_millis(50);
const MAX_FRAME_BATCH_DATA_SIZE: usize = MAX_PACKET_SIZE as usize;

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

struct FramePacket {
    data: Bytes,
    completion: Option<oneshot::Sender<()>>,
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

    fn ingest(self, flush_request: &mut FlushRequest, packets: &mut VecDeque<FramePacket>) {
        match self {
            Self::Flush => *flush_request = flush_request.merge(FlushRequest::Always),
            Self::Data { data, completion } => {
                *flush_request = flush_request.merge(if completion.is_some() {
                    FlushRequest::IfNotSuspended
                } else {
                    FlushRequest::None
                });
                packets.push_back(FramePacket { data, completion });
            }
        }
    }
}

fn take_frame_batch(packets: &mut VecDeque<FramePacket>) -> Vec<FramePacket> {
    let mut batch = Vec::new();
    let mut data_len = 0usize;

    while let Some(packet) = packets.pop_front() {
        let next_len = data_len.saturating_add(packet.data.len());
        if !batch.is_empty() && next_len > MAX_FRAME_BATCH_DATA_SIZE {
            packets.push_front(packet);
            break;
        }

        data_len = next_len;
        batch.push(packet);
    }

    batch
}

fn frame_packet_batch<W: AsyncWrite + Unpin>(
    mut writer: TCPNetworkEncoder<W>,
    batch: &[FramePacket],
) -> (TCPNetworkEncoder<W>, Vec<u8>, Option<PacketEncodeError>) {
    let mut frame = Vec::new();
    let mut frame_err = None;
    for packet in batch {
        if let Err(err) = writer.frame_packet(&packet.data, &mut frame) {
            frame_err = Some(err);
            break;
        }
    }
    (writer, frame, frame_err)
}

async fn frame_batch_maybe_offload<W: AsyncWrite + Unpin + Send + 'static>(
    writer: TCPNetworkEncoder<W>,
    packet_batch: Vec<FramePacket>,
) -> Result<
    (
        TCPNetworkEncoder<W>,
        Vec<FramePacket>,
        Vec<u8>,
        Option<PacketEncodeError>,
    ),
    tokio::task::JoinError,
> {
    let needs_offload = packet_batch
        .iter()
        .any(|packet| writer.is_compressing_packet(&packet.data));

    if needs_offload {
        tokio::task::spawn_blocking(move || {
            let (writer, frame, frame_err) = frame_packet_batch(writer, &packet_batch);
            (writer, packet_batch, frame, frame_err)
        })
        .await
    } else {
        let (writer, frame, frame_err) = frame_packet_batch(writer, &packet_batch);
        Ok((writer, packet_batch, frame, frame_err))
    }
}

fn complete_pending(pending_completions: &mut Vec<oneshot::Sender<()>>) {
    for completion in pending_completions.drain(..) {
        let _ = completion.send(());
    }
}

/// `None` on socket error. `Some(true)` if TCP flush ran.
async fn flush_writer<W: AsyncWrite + Unpin>(
    writer: &mut TCPNetworkEncoder<W>,
    unflushed: &mut bool,
    pending_completions: &mut Vec<oneshot::Sender<()>>,
    close_token: &CancellationToken,
    id: u64,
) -> Option<bool> {
    let did_flush = *unflushed;
    if did_flush {
        if let Err(err) = writer.flush().await {
            if !close_token.is_cancelled() {
                warn!("Failed to flush packets for client {id}: {err}");
            }
            return None;
        }
        *unflushed = false;
    }
    complete_pending(pending_completions);
    Some(did_flush)
}

async fn flush_and_stamp<W: AsyncWrite + Unpin>(
    writer: &mut TCPNetworkEncoder<W>,
    unflushed: &mut bool,
    pending_completions: &mut Vec<oneshot::Sender<()>>,
    close_token: &CancellationToken,
    last_tcp_flush: &mut Instant,
    id: u64,
) -> bool {
    match flush_writer(writer, unflushed, pending_completions, close_token, id).await {
        Some(true) => {
            *last_tcp_flush = Instant::now();
            true
        }
        Some(false) => true,
        None => false,
    }
}

fn drain_until_barrier(
    first: OutgoingPacket,
    packet_receiver: &mut Receiver<OutgoingPacket>,
    tick_flush: &TickFlush,
) -> (FlushRequest, VecDeque<FramePacket>, bool) {
    let mut flush_request = FlushRequest::None;
    let mut packets = VecDeque::new();
    let mut disconnected = false;
    match first {
        OutgoingPacket::Flush => flush_request = FlushRequest::Always,
        data @ OutgoingPacket::Data { .. } => {
            data.ingest(&mut flush_request, &mut packets);
            loop {
                if tick_flush.take() {
                    flush_request = flush_request.merge(FlushRequest::Always);
                    break;
                }
                match packet_receiver.try_recv() {
                    Ok(OutgoingPacket::Flush) => {
                        flush_request = flush_request.merge(FlushRequest::Always);
                        break;
                    }
                    Ok(packet) => packet.ingest(&mut flush_request, &mut packets),
                    Err(TryRecvError::Empty) => {
                        if tick_flush.take() {
                            flush_request = flush_request.merge(FlushRequest::Always);
                        }
                        break;
                    }
                    Err(TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }
        }
    }
    (flush_request, packets, disconnected)
}

async fn write_queued_frames<W: AsyncWrite + Unpin + Send + 'static>(
    mut writer: TCPNetworkEncoder<W>,
    mut packets_to_frame: VecDeque<FramePacket>,
    pending_completions: &mut Vec<oneshot::Sender<()>>,
    close_token: &CancellationToken,
    id: u64,
) -> Option<TCPNetworkEncoder<W>> {
    while !packets_to_frame.is_empty() {
        let frame_batch = take_frame_batch(&mut packets_to_frame);
        let (returned_writer, returned_batch, frame, frame_err) =
            match frame_batch_maybe_offload(writer, frame_batch).await {
                Ok(result) => result,
                Err(err) => {
                    if !close_token.is_cancelled() {
                        warn!("Packet framing task failed for client {id}: {err}");
                    }
                    return None;
                }
            };
        writer = returned_writer;

        if let Some(err) = frame_err {
            if !close_token.is_cancelled() {
                warn!("Failed to frame packet for client {id}: {err}");
            }
            return None;
        }

        if let Err(err) = writer.write_frame(&frame).await {
            if !close_token.is_cancelled() {
                warn!("Failed to send packet batch to client {id}: {err}");
            }
            return None;
        }

        for packet in returned_batch {
            if let Some(completion) = packet.completion {
                pending_completions.push(completion);
            }
        }
    }

    Some(writer)
}

pub async fn run_outgoing_packet_writer<W: AsyncWrite + Unpin + Send + 'static>(
    mut packet_receiver: Receiver<OutgoingPacket>,
    mut writer: TCPNetworkEncoder<W>,
    close_token: CancellationToken,
    suspend_flushing: Arc<AtomicBool>,
    tick_flush: TickFlush,
    id: u64,
) {
    let mut flush_interval = tokio::time::interval(TICK_FLUSH_INTERVAL);
    flush_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    // `interval` fires immediately; skip so an empty connection is not flushed.
    flush_interval.tick().await;

    let mut unflushed = false;
    let mut pending_completions = Vec::new();
    let mut last_tcp_flush = Instant::now();

    loop {
        if close_token.is_cancelled() {
            break;
        }

        if tick_flush.take() {
            if !flush_and_stamp(
                &mut writer,
                &mut unflushed,
                &mut pending_completions,
                &close_token,
                &mut last_tcp_flush,
                id,
            )
            .await
            {
                close_token.cancel();
                break;
            }
            continue;
        }

        let recv_result = tokio::select! {
            biased;
            () = close_token.cancelled() => None,
            () = tick_flush.notify.notified() => {
                continue;
            }
            res = packet_receiver.recv() => res,
            _ = flush_interval.tick(), if unflushed
                && !suspend_flushing.load(Ordering::Acquire) =>
            {
                if !flush_and_stamp(
                    &mut writer,
                    &mut unflushed,
                    &mut pending_completions,
                    &close_token,
                    &mut last_tcp_flush,
                    id,
                )
                .await
                {
                    close_token.cancel();
                    break;
                }
                continue;
            }
        };

        let Some(first) = recv_result else {
            break;
        };

        let (flush_request, packets_to_frame, disconnected) =
            drain_until_barrier(first, &mut packet_receiver, &tick_flush);

        if !packets_to_frame.is_empty() {
            let Some(returned) = write_queued_frames(
                writer,
                packets_to_frame,
                &mut pending_completions,
                &close_token,
                id,
            )
            .await
            else {
                close_token.cancel();
                return;
            };
            writer = returned;
            unflushed = true;
        }

        let suspended = suspend_flushing.load(Ordering::Acquire);
        let fallback_due =
            unflushed && !suspended && last_tcp_flush.elapsed() >= TICK_FLUSH_INTERVAL;
        if flush_request.should_flush(suspended) || disconnected || fallback_due {
            if !flush_and_stamp(
                &mut writer,
                &mut unflushed,
                &mut pending_completions,
                &close_token,
                &mut last_tcp_flush,
                id,
            )
            .await
            {
                close_token.cancel();
                break;
            }
        } else {
            complete_pending(&mut pending_completions);
        }

        if disconnected {
            return;
        }
    }

    let _ = writer.flush().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;
    use std::sync::atomic::AtomicUsize;
    use std::task::{Context, Poll};
    use std::time::Instant;

    struct RecordingWriter {
        writes: Arc<std::sync::Mutex<Vec<u8>>>,
        flushes: Arc<AtomicUsize>,
    }

    impl AsyncWrite for RecordingWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            self.writes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .extend_from_slice(buf);
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
        writes: Arc<std::sync::Mutex<Vec<u8>>>,
        flushes: Arc<AtomicUsize>,
        suspend: Arc<AtomicBool>,
        close: CancellationToken,
    ) {
        run_outgoing_packet_writer(
            rx,
            TCPNetworkEncoder::new(RecordingWriter { writes, flushes }),
            close,
            suspend,
            TickFlush::new(),
            0,
        )
        .await;
    }

    async fn run_writer_with_tick_flush(
        rx: Receiver<OutgoingPacket>,
        writes: Arc<std::sync::Mutex<Vec<u8>>>,
        flushes: Arc<AtomicUsize>,
        suspend: Arc<AtomicBool>,
        tick_flush: TickFlush,
        close: CancellationToken,
    ) {
        run_outgoing_packet_writer(
            rx,
            TCPNetworkEncoder::new(RecordingWriter { writes, flushes }),
            close,
            suspend,
            tick_flush,
            0,
        )
        .await;
    }

    fn spawn_writer(
        rx: Receiver<OutgoingPacket>,
        writes: Arc<std::sync::Mutex<Vec<u8>>>,
        flushes: Arc<AtomicUsize>,
        suspend: Arc<AtomicBool>,
        close: CancellationToken,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(run_writer(rx, writes, flushes, suspend, close))
    }

    #[tokio::test]
    async fn tick_barrier_flushes_once_not_every_sixty_four() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(false));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend, close.clone());

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
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn fifty_ms_interval_flushes_when_no_tick_barrier() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(false));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend, close.clone());

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
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend.clone(), close.clone());

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
    async fn send_packet_now_does_not_flush_while_suspended() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend, close.clone());

        tx.try_send(packet(1)).unwrap();
        let (done_tx, done_rx) = oneshot::channel();
        tx.try_send(OutgoingPacket::high_priority(
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
            "send_packet_now must not flush CBlockEvent / entity motion mid-tick"
        );

        tx.try_send(OutgoingPacket::Flush).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(flushes.load(Ordering::SeqCst), 1);

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn send_packet_now_does_not_overtake_queued_tick_packets() {
        const TICK: u8 = 0xAA;
        const NOW: u8 = 0xBB;

        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes.clone(), flushes, suspend, close.clone());

        tx.try_send(packet(TICK)).unwrap();
        let (done_tx, done_rx) = oneshot::channel();
        tx.try_send(OutgoingPacket::high_priority(
            Bytes::from_static(&[NOW]),
            done_tx,
        ))
        .unwrap();
        tokio::time::timeout(Duration::from_millis(50), done_rx)
            .await
            .expect("send_packet_now must complete")
            .expect("writer dropped");

        let written = writes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let tick_at = written.iter().position(|&b| b == TICK);
        let now_at = written.iter().position(|&b| b == NOW);
        assert!(
            tick_at.is_some() && now_at.is_some() && tick_at < now_at,
            "FIFO: tick packet {TICK:#x} must be written before send_packet_now {NOW:#x}, got {written:?}"
        );

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn flush_barrier_flushes_while_still_suspended() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend, close.clone());

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
    async fn flush_stops_drain_so_later_packets_are_the_next_tick() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend, close.clone());

        tx.try_send(packet(1)).unwrap();
        tx.try_send(OutgoingPacket::Flush).unwrap();
        tx.try_send(packet(2)).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(
            flushes.load(Ordering::SeqCst),
            1,
            "Flush must not pull the next tick's packets into this TCP flush"
        );

        tx.try_send(OutgoingPacket::Flush).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(flushes.load(Ordering::SeqCst), 2);

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn lagged_writer_flushes_once_per_queued_tick_barrier() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend, close.clone());

        tx.try_send(packet(1)).unwrap();
        tx.try_send(OutgoingPacket::Flush).unwrap();
        tx.try_send(packet(2)).unwrap();
        tx.try_send(OutgoingPacket::Flush).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(
            flushes.load(Ordering::SeqCst),
            2,
            "each Flush must be its own TCP flush even if both ticks were already queued"
        );

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn tick_thread_enqueue_is_isolated_from_writer() {
        let (tx, rx) = tokio::sync::mpsc::channel::<OutgoingPacket>(4096);
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes, suspend, close.clone());

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

    #[tokio::test]
    async fn pending_tick_flush_flushes_when_fifo_has_no_room_for_barrier() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let tick_flush = TickFlush::new();
        let close = CancellationToken::new();

        let writer = tokio::spawn(run_writer_with_tick_flush(
            rx,
            writes,
            flushes.clone(),
            suspend,
            tick_flush.clone(),
            close.clone(),
        ));

        tx.try_send(packet(1)).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(flushes.load(Ordering::SeqCst), 0);

        tick_flush.request();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(
            flushes.load(Ordering::SeqCst),
            1,
            "resumeFlushing must flush even when try_send(Flush) cannot enqueue"
        );

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn fifty_ms_flushes_while_packets_keep_arriving() {
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(false));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend, close.clone());

        for i in 0..16u8 {
            tx.try_send(packet(i)).unwrap();
            tokio::time::sleep(Duration::from_millis(8)).await;
        }
        assert!(
            flushes.load(Ordering::SeqCst) >= 1,
            "busy recv must not starve the 50ms fallback flush"
        );

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }
}
