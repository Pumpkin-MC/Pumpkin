//! Tick-thread enqueue is non-blocking; socket write/flush is a dedicated task.
//! Vanilla `suspendFlushing` / `resumeFlushing` write a game tick without flushing,
//! then `Connection.flushChannel` once so `CBlockEvent` and entity motion share a tick.

use std::collections::VecDeque;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::time::Duration;

use bytes::Bytes;
use pumpkin_protocol::{
    MAX_PACKET_SIZE, PacketEncodeError, java::packet_encoder::TCPNetworkEncoder,
};
use tokio::io::AsyncWrite;
use tokio::sync::{
    mpsc::{UnboundedReceiver, error::TryRecvError},
    oneshot,
};
use tokio::time::{Instant, MissedTickBehavior};
use tokio_util::sync::CancellationToken;
use tracing::warn;

use crate::net::decrement_pending_bytes;

/// Max wait for the disconnect flush (`kick_explicit`, writer drain on close).
pub const DISCONNECT_FLUSH_TIMEOUT: Duration = Duration::from_secs(5);

/// Fallback for bytes written while suspended when the hold ends without a `Flush`
const TICK_FLUSH_INTERVAL: Duration = Duration::from_millis(50);
const MAX_FRAME_BATCH_DATA_SIZE: usize = MAX_PACKET_SIZE as usize;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FlushRequest {
    None,
    /// `send_packet_now` / `try_enqueue`. Vanilla `send(..., flush)` is false while suspended.
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

pub enum Completion {
    /// After `write_frame` into the `BufWriter`.
    Framed(oneshot::Sender<()>),
    /// After the TCP flush. Disconnect: `close()` follows, so the flush must be done.
    /// Flushes also while suspended, else a kick awaited inside the tick waits on its own barrier.
    Flushed(oneshot::Sender<()>),
}

pub enum OutgoingPacket {
    Data {
        data: Bytes,
        completion: Option<Completion>,
    },
    /// End-of-tick barrier (`Connection.flushChannel`).
    Flush,
}

struct FramePacket {
    data: Bytes,
    completion: Option<Completion>,
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
            completion: Some(Completion::Framed(completion)),
        }
    }

    pub const fn flushed(data: Bytes, completion: oneshot::Sender<()>) -> Self {
        Self::Data {
            data,
            completion: Some(Completion::Flushed(completion)),
        }
    }

    fn ingest(self, flush_request: &mut FlushRequest, packets: &mut VecDeque<FramePacket>) {
        match self {
            Self::Flush => *flush_request = flush_request.merge(FlushRequest::Always),
            Self::Data { data, completion } => {
                *flush_request = flush_request.merge(match completion {
                    Some(Completion::Flushed(_)) => FlushRequest::Always,
                    // off-tick sends flush at once -> mid-tick ones wait for `Flush`.
                    Some(Completion::Framed(_)) | None => FlushRequest::IfNotSuspended,
                });
                packets.push_back(FramePacket { data, completion });
            }
        }
    }
}

/// A single oversized packet is framed alone.
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

/// Compressing batches are framed on a blocking task.
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

/// Shared, never mutated by the writer loop.
struct WriterCtx {
    close_token: CancellationToken,
    suspend_flushing: Arc<AtomicBool>,
    pending_bytes: Arc<AtomicUsize>,
    id: u64,
}

/// Reason flush left the writer loop.
enum FlushExit {
    /// `close()` during a stalled flush -> Buffer for `drain_on_close`.
    Closing,
    /// Socket error.
    Failed,
}

/// Flush, else leave the writer loop
/// drain on close / stop on socket error.
macro_rules! flush_or_exit {
    ($state:expr, $writer:expr, $ctx:expr) => {
        match $state.flush_and_stamp(&mut $writer, $ctx).await {
            Ok(()) => {}
            Err(FlushExit::Closing) => break,
            Err(FlushExit::Failed) => {
                $ctx.close_token.cancel();
                return;
            }
        }
    };
}

/// Write state between two TCP flushes.
struct FlushState {
    /// Written to the `BufWriter`, not flushed yet.
    unflushed: bool,
    /// Tokio clock so paused-time tests drive the 50ms fallback.
    last_tcp_flush: Instant,
    /// `Completion::Flushed` of written frames. Sent after the next successful flush.
    on_flush: Vec<oneshot::Sender<()>>,
}

impl FlushState {
    fn new() -> Self {
        Self {
            unflushed: false,
            last_tcp_flush: Instant::now(),
            on_flush: Vec::new(),
        }
    }

    /// `Ok(true)` if TCP flush ran.
    async fn flush<W: AsyncWrite + Unpin>(
        &mut self,
        writer: &mut TCPNetworkEncoder<W>,
        ctx: &WriterCtx,
    ) -> Result<bool, FlushExit> {
        let did_flush = self.unflushed;
        if did_flush {
            // Flush first: a `try_kick` racing this flush still gets out on a healthy socket.
            let flushed = tokio::select! {
                biased;
                res = writer.flush() => res,
                // Stalled flush: `drain_on_close` resumes it
                // bounded by DISCONNECT_FLUSH_TIMEOUT
                () = ctx.close_token.cancelled() => return Err(FlushExit::Closing),
            };
            if let Err(err) = flushed {
                if !ctx.close_token.is_cancelled() {
                    warn!("Failed to flush packets for client {}: {err}", ctx.id);
                }
                return Err(FlushExit::Failed);
            }
            self.unflushed = false;
            for completion in self.on_flush.drain(..) {
                let _ = completion.send(());
            }
        }
        Ok(did_flush)
    }

    async fn flush_and_stamp<W: AsyncWrite + Unpin>(
        &mut self,
        writer: &mut TCPNetworkEncoder<W>,
        ctx: &WriterCtx,
    ) -> Result<(), FlushExit> {
        if self.flush(writer, ctx).await? {
            self.last_tcp_flush = Instant::now();
        }
        Ok(())
    }

    fn should_flush_now(
        &self,
        flush_request: FlushRequest,
        disconnected: bool,
        ctx: &WriterCtx,
    ) -> bool {
        let suspended = ctx.suspend_flushing.load(Ordering::Acquire);
        let fallback_due =
            self.unflushed && !suspended && self.last_tcp_flush.elapsed() >= TICK_FLUSH_INTERVAL;
        flush_request.should_flush(suspended) || disconnected || fallback_due
    }
}

/// What woke the writer loop.
enum WriterStep {
    Packet(OutgoingPacket),
    /// 50ms elapsed with unflushed bytes.
    Flush,
    Stop,
}

async fn next_step(
    packet_receiver: &mut UnboundedReceiver<OutgoingPacket>,
    flush_interval: &mut tokio::time::Interval,
    state: &FlushState,
    ctx: &WriterCtx,
) -> WriterStep {
    // Order: close, packets, 50ms fallback.
    tokio::select! {
        biased;
        () = ctx.close_token.cancelled() => WriterStep::Stop,
        res = packet_receiver.recv() => res.map_or(WriterStep::Stop, WriterStep::Packet),
        _ = flush_interval.tick(), if state.unflushed
            && !ctx.suspend_flushing.load(Ordering::Acquire) => WriterStep::Flush,
    }
}

fn drain_until_barrier(
    first: OutgoingPacket,
    packet_receiver: &mut UnboundedReceiver<OutgoingPacket>,
) -> (FlushRequest, VecDeque<FramePacket>, bool) {
    let mut flush_request = FlushRequest::None;
    let mut packets = VecDeque::new();
    let mut disconnected = false;
    match first {
        OutgoingPacket::Flush => flush_request = FlushRequest::Always,
        data @ OutgoingPacket::Data { .. } => {
            data.ingest(&mut flush_request, &mut packets);
            loop {
                match packet_receiver.try_recv() {
                    Ok(OutgoingPacket::Flush) => {
                        flush_request = flush_request.merge(FlushRequest::Always);
                        break;
                    }
                    Ok(packet) => packet.ingest(&mut flush_request, &mut packets),
                    Err(TryRecvError::Empty) => break,
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
    on_flush: &mut Vec<oneshot::Sender<()>>,
    ctx: &WriterCtx,
) -> Option<TCPNetworkEncoder<W>> {
    let (close_token, id) = (&ctx.close_token, ctx.id);
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

        let written_bytes: usize = returned_batch.iter().map(|packet| packet.data.len()).sum();
        decrement_pending_bytes(&ctx.pending_bytes, written_bytes);

        // The frame is in the `BufWriter`
        // `Framed` releases now
        // `Flushed` waits for the TCP flush
        for packet in returned_batch {
            match packet.completion {
                Some(Completion::Framed(completion)) => {
                    let _ = completion.send(());
                }
                Some(Completion::Flushed(completion)) => on_flush.push(completion),
                None => {}
            }
        }
    }

    Some(writer)
}

enum WriteOutcome<W: AsyncWrite + Unpin> {
    Open(TCPNetworkEncoder<W>),
    /// `close()` mid-write. Batch finished, drain gets the rest of the budget.
    Closing(TCPNetworkEncoder<W>, Instant),
    Failed,
}

/// `write_queued_frames`, raced against `close()`. On close the same future keeps
/// running under `DISCONNECT_FLUSH_TIMEOUT`: dropping `write_all` cuts a frame in half.
async fn write_or_close<W: AsyncWrite + Unpin + Send + 'static>(
    writer: TCPNetworkEncoder<W>,
    packets: VecDeque<FramePacket>,
    on_flush: &mut Vec<oneshot::Sender<()>>,
    ctx: &WriterCtx,
) -> WriteOutcome<W> {
    let write = write_queued_frames(writer, packets, on_flush, ctx);
    tokio::pin!(write);
    tokio::select! {
        biased;
        res = &mut write => res.map_or(WriteOutcome::Failed, WriteOutcome::Open),
        () = ctx.close_token.cancelled() => {
            let deadline = Instant::now() + DISCONNECT_FLUSH_TIMEOUT;
            match tokio::time::timeout_at(deadline, write).await {
                Ok(Some(writer)) => WriteOutcome::Closing(writer, deadline),
                // Stalled past the budget, or socket error.
                _ => WriteOutcome::Failed,
            }
        }
    }
}

pub async fn run_outgoing_packet_writer<W: AsyncWrite + Unpin + Send + 'static>(
    mut packet_receiver: UnboundedReceiver<OutgoingPacket>,
    mut writer: TCPNetworkEncoder<W>,
    close_token: CancellationToken,
    suspend_flushing: Arc<AtomicBool>,
    pending_bytes: Arc<AtomicUsize>,
    id: u64,
) {
    let ctx = WriterCtx {
        close_token,
        suspend_flushing,
        pending_bytes,
        id,
    };
    let mut state = FlushState::new();

    let mut flush_interval = tokio::time::interval(TICK_FLUSH_INTERVAL);
    flush_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    // `interval` fires immediately; skip so an empty connection is not flushed.
    flush_interval.tick().await;

    loop {
        if ctx.close_token.is_cancelled() {
            break;
        }

        let first = match next_step(&mut packet_receiver, &mut flush_interval, &state, &ctx).await {
            WriterStep::Packet(packet) => packet,
            WriterStep::Flush => {
                flush_or_exit!(state, writer, &ctx);
                continue;
            }
            WriterStep::Stop => break,
        };

        let (flush_request, packets_to_frame, disconnected) =
            drain_until_barrier(first, &mut packet_receiver);

        if !packets_to_frame.is_empty() {
            match write_or_close(writer, packets_to_frame, &mut state.on_flush, &ctx).await {
                WriteOutcome::Open(returned) => writer = returned,
                WriteOutcome::Closing(returned, deadline) => {
                    state.unflushed = true;
                    drain_on_close(packet_receiver, returned, state, &ctx, deadline).await;
                    return;
                }
                WriteOutcome::Failed => {
                    ctx.close_token.cancel();
                    return;
                }
            }
            state.unflushed = true;
        }

        if state.should_flush_now(flush_request, disconnected, &ctx) {
            flush_or_exit!(state, writer, &ctx);
        }

        // Flushed above already, so skip the final flush.
        if disconnected {
            return;
        }
    }

    let deadline = Instant::now() + DISCONNECT_FLUSH_TIMEOUT;
    drain_on_close(packet_receiver, writer, state, &ctx, deadline).await;
}

/// close only after the disconnect is sent. `try_kick` enqueues it and
/// closes at once, so frame and flush what was admitted before `close()`.
/// A write or flush stalled at close time resumes here. Socket errors return before this.
async fn drain_on_close<W: AsyncWrite + Unpin + Send + 'static>(
    mut packet_receiver: UnboundedReceiver<OutgoingPacket>,
    mut writer: TCPNetworkEncoder<W>,
    mut state: FlushState,
    ctx: &WriterCtx,
    deadline: Instant,
) {
    packet_receiver.close();
    let mut packets = VecDeque::new();
    // Closing: flushes once below, suspended or not.
    let mut flush_request = FlushRequest::None;
    while let Ok(packet) = packet_receiver.try_recv() {
        packet.ingest(&mut flush_request, &mut packets);
    }

    // Stalled peer give up like `kick_explicit`.
    let _ = tokio::time::timeout_at(deadline, async move {
        if !packets.is_empty() {
            writer = write_queued_frames(writer, packets, &mut state.on_flush, ctx).await?;
            state.unflushed = true;
        }
        if state.unflushed && writer.flush().await.is_ok() {
            for completion in state.on_flush.drain(..) {
                let _ = completion.send(());
            }
        }
        Some(())
    })
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;
    use std::task::{Context, Poll};
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

    /// Writes land in the `BufWriter`, the TCP flush never finishes.
    struct StalledFlushWriter {
        flush_polls: Arc<AtomicUsize>,
    }

    impl AsyncWrite for StalledFlushWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            self.flush_polls.fetch_add(1, Ordering::SeqCst);
            Poll::Pending
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    /// Socket stalls until open: TCP flush always, writes unless `writes_pass`.
    #[derive(Default)]
    struct SocketGate {
        open: AtomicBool,
        writes_pass: bool,
        waker: std::sync::Mutex<Option<std::task::Waker>>,
        flushed: AtomicUsize,
    }

    impl SocketGate {
        fn stalled_flush() -> Arc<Self> {
            Arc::new(Self {
                writes_pass: true,
                ..Self::default()
            })
        }

        fn stalled_write() -> Arc<Self> {
            Arc::new(Self::default())
        }

        /// true once open, else parks waker
        fn poll_open(&self, cx: &Context<'_>) -> bool {
            *self
                .waker
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(cx.waker().clone());
            self.open.load(Ordering::SeqCst)
        }

        fn open(&self) {
            self.open.store(true, Ordering::SeqCst);
            let waker = self
                .waker
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            if let Some(waker) = waker {
                waker.wake();
            }
        }
    }

    struct GatedWriter {
        writes: Arc<std::sync::Mutex<Vec<u8>>>,
        gate: Arc<SocketGate>,
    }

    impl AsyncWrite for GatedWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            if !self.gate.writes_pass && !self.gate.poll_open(cx) {
                return Poll::Pending;
            }
            self.writes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            if !self.gate.poll_open(cx) {
                return Poll::Pending;
            }
            self.gate.flushed.fetch_add(1, Ordering::SeqCst);
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
        rx: UnboundedReceiver<OutgoingPacket>,
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
            Arc::new(AtomicUsize::new(0)),
            0,
        )
        .await;
    }

    fn spawn_gated_writer(
        rx: UnboundedReceiver<OutgoingPacket>,
        writes: Arc<std::sync::Mutex<Vec<u8>>>,
        gate: Arc<SocketGate>,
        close: CancellationToken,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(run_outgoing_packet_writer(
            rx,
            TCPNetworkEncoder::new(GatedWriter { writes, gate }),
            close,
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicUsize::new(0)),
            0,
        ))
    }

    fn spawn_writer(
        rx: UnboundedReceiver<OutgoingPacket>,
        writes: Arc<std::sync::Mutex<Vec<u8>>>,
        flushes: Arc<AtomicUsize>,
        suspend: Arc<AtomicBool>,
        close: CancellationToken,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(run_writer(rx, writes, flushes, suspend, close))
    }

    #[tokio::test(start_paused = true)]
    async fn tick_barrier_flushes_once_not_every_sixty_four() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(false));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend, close.clone());

        for i in 0..200u8 {
            tx.send(packet(i)).unwrap();
        }
        tx.send(OutgoingPacket::Flush).unwrap();

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

    #[tokio::test(start_paused = true)]
    async fn off_tick_enqueue_flushes_immediately() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(false));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend, close.clone());

        tx.send(packet(1)).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(
            flushes.load(Ordering::SeqCst),
            1,
            "keep-alive / pong outside the tick must not wait for the 50ms fallback"
        );

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn suspend_flushing_holds_the_fifty_ms_flush_until_resume() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend.clone(), close.clone());

        tx.send(packet(1)).unwrap();
        tokio::time::sleep(Duration::from_millis(80)).await;
        assert_eq!(
            flushes.load(Ordering::SeqCst),
            0,
            "mid-tick 50ms timer must not flush while suspendFlushing is set"
        );

        suspend.store(false, Ordering::Release);
        tx.send(OutgoingPacket::Flush).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(flushes.load(Ordering::SeqCst), 1);

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn send_packet_now_does_not_flush_while_suspended() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend, close.clone());

        tx.send(packet(1)).unwrap();
        let (done_tx, done_rx) = oneshot::channel();
        tx.send(OutgoingPacket::high_priority(
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

        tx.send(OutgoingPacket::Flush).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(flushes.load(Ordering::SeqCst), 1);

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }

    /// Unsuspended, `high_priority` requests a flush -> the completion is tied to
    /// `write_frame`, so a stalled TCP flush must not hold it back.
    #[tokio::test(start_paused = true)]
    async fn send_packet_now_completes_before_the_tcp_flush() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let flush_polls = Arc::new(AtomicUsize::new(0));
        let close = CancellationToken::new();

        let writer = tokio::spawn(run_outgoing_packet_writer(
            rx,
            TCPNetworkEncoder::new(StalledFlushWriter {
                flush_polls: flush_polls.clone(),
            }),
            close.clone(),
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicUsize::new(0)),
            0,
        ));

        let (done_tx, done_rx) = oneshot::channel();
        tx.send(OutgoingPacket::high_priority(
            Bytes::from_static(&[1]),
            done_tx,
        ))
        .unwrap();

        tokio::time::timeout(Duration::from_millis(50), done_rx)
            .await
            .expect("send_packet_now must not wait for the TCP flush")
            .expect("writer dropped");

        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(
            flush_polls.load(Ordering::SeqCst),
            1,
            "the flush was attempted and is still pending"
        );

        // Close during the stall
        writer.abort();
    }

    /// `kick_explicit` closes on completion: must not fire while the frame sits in the buffer.
    #[tokio::test(start_paused = true)]
    async fn disconnect_completes_only_after_the_tcp_flush() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let flush_polls = Arc::new(AtomicUsize::new(0));
        let close = CancellationToken::new();

        let writer = tokio::spawn(run_outgoing_packet_writer(
            rx,
            TCPNetworkEncoder::new(StalledFlushWriter {
                flush_polls: flush_polls.clone(),
            }),
            close.clone(),
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicUsize::new(0)),
            0,
        ));

        let (done_tx, mut done_rx) = oneshot::channel();
        tx.send(OutgoingPacket::flushed(Bytes::from_static(&[1]), done_tx))
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(flush_polls.load(Ordering::SeqCst), 1, "flush attempted");
        assert!(
            matches!(done_rx.try_recv(), Err(oneshot::error::TryRecvError::Empty)),
            "disconnect completion must wait for the stalled TCP flush"
        );

        // Stalled past `DISCONNECT_FLUSH_TIMEOUT`.
        close.cancel();
        writer.abort();
        let _ = writer.await;
        assert!(done_rx.await.is_err(), "failed flush drops the completion");
    }

    /// Busy connection: `try_kick` lands while an earlier flush is stalled.
    /// Disconnect must still go out once the socket drains.
    #[tokio::test(start_paused = true)]
    async fn close_during_stalled_flush_still_sends_the_disconnect() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let gate = SocketGate::stalled_flush();
        let close = CancellationToken::new();
        let writer = spawn_gated_writer(rx, writes.clone(), gate.clone(), close.clone());

        tx.send(packet(1)).unwrap();
        tx.send(OutgoingPacket::Flush).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;

        tx.send(packet(0xDC)).unwrap();
        close.cancel();
        tokio::time::sleep(Duration::from_millis(20)).await;
        gate.open();

        tokio::time::timeout(Duration::from_millis(50), writer)
            .await
            .expect("drain must finish once the socket drains")
            .unwrap();
        assert_eq!(
            writes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .last(),
            Some(&0xDC),
            "disconnect written after the stalled flush"
        );
        assert_eq!(gate.flushed.load(Ordering::SeqCst), 1);
    }

    /// Full socket buffer: `try_kick` lands mid-write. Frame finishes whole, then the disconnect.
    #[tokio::test(start_paused = true)]
    async fn close_during_stalled_write_finishes_the_frame() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let gate = SocketGate::stalled_write();
        let close = CancellationToken::new();
        let writer = spawn_gated_writer(rx, writes.clone(), gate.clone(), close.clone());

        tx.send(packet(1)).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;

        tx.send(packet(0xDC)).unwrap();
        close.cancel();
        tokio::time::sleep(Duration::from_millis(20)).await;
        gate.open();

        tokio::time::timeout(Duration::from_millis(50), writer)
            .await
            .expect("drain must finish once the socket drains")
            .unwrap();
        assert_eq!(
            *writes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            [1, 1, 1, 0xDC],
            "stalled frame written whole, disconnect behind it"
        );
        assert_eq!(gate.flushed.load(Ordering::SeqCst), 1);
    }

    /// If Peer never reads `close()`, must still end the writer within the disconnect budget
    #[tokio::test(start_paused = true)]
    async fn close_bounds_a_stalled_write() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let close = CancellationToken::new();
        let writer = spawn_gated_writer(rx, writes, SocketGate::stalled_write(), close.clone());

        tx.send(packet(1)).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        close.cancel();

        tokio::time::timeout(DISCONNECT_FLUSH_TIMEOUT + Duration::from_secs(1), writer)
            .await
            .expect("stalled write must give up at DISCONNECT_FLUSH_TIMEOUT")
            .unwrap();
    }

    /// `try_kick`: enqueue mid-tick, `close()` at once. The disconnect must still go out.
    #[tokio::test(start_paused = true)]
    async fn close_writes_and_flushes_the_queued_disconnect() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes.clone(), flushes.clone(), suspend, close.clone());
        tokio::time::sleep(Duration::from_millis(10)).await;

        tx.send(packet(0xDC)).unwrap();
        close.cancel();
        tokio::time::timeout(Duration::from_millis(50), writer)
            .await
            .expect("drain on close must not hang on a healthy socket")
            .unwrap();

        assert_eq!(flushes.load(Ordering::SeqCst), 1, "flushed despite suspend");
        assert_eq!(
            writes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .last(),
            Some(&0xDC),
            "disconnect written before the writer stops"
        );
    }

    /// `close()` before the writer ever ran: the queue is still drained.
    #[tokio::test(start_paused = true)]
    async fn close_before_start_still_drains_the_queue() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let close = CancellationToken::new();

        for i in 1..=3 {
            tx.send(packet(i)).unwrap();
        }
        close.cancel();
        run_writer(
            rx,
            writes.clone(),
            flushes.clone(),
            Arc::new(AtomicBool::new(false)),
            close,
        )
        .await;

        let written = writes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let payloads: Vec<u8> = written.chunks(2).map(|frame| frame[1]).collect();
        assert_eq!(payloads, [1, 2, 3], "one [len, id] frame per queued packet");
        assert_eq!(flushes.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn send_packet_now_does_not_overtake_queued_tick_packets() {
        const TICK: u8 = 0xAA;
        const NOW: u8 = 0xBB;

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes.clone(), flushes, suspend, close.clone());

        tx.send(packet(TICK)).unwrap();
        let (done_tx, done_rx) = oneshot::channel();
        tx.send(OutgoingPacket::high_priority(
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

    #[tokio::test(start_paused = true)]
    async fn flush_stops_drain_so_later_packets_are_the_next_tick() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let writes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));
        let suspend = Arc::new(AtomicBool::new(true));
        let close = CancellationToken::new();

        let writer = spawn_writer(rx, writes, flushes.clone(), suspend, close.clone());

        tx.send(packet(1)).unwrap();
        tx.send(OutgoingPacket::Flush).unwrap();
        tx.send(packet(2)).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(
            flushes.load(Ordering::SeqCst),
            1,
            "Flush must not pull the next tick's packets into this TCP flush"
        );

        tx.send(OutgoingPacket::Flush).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(flushes.load(Ordering::SeqCst), 2);

        drop(tx);
        close.cancel();
        writer.await.unwrap();
    }
}
