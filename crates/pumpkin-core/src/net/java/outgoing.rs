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

use super::PacketRun;
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
    /// The ordinary packets a tick queued for this client, in order.
    Batch(Vec<BatchItem>),
}

/// What a tick queues for a client: one packet, or a run of them. A run is taken apart
/// by the writer task, not by the tick thread that queued it.
pub enum BatchItem {
    Packet(Bytes),
    Run(PacketRun),
}

/// What the writer frames: a packet, or the packets of a run still back to back.
struct FramePacket {
    data: Bytes,
    /// Where each packet of a run ends in `data`. `None` for one packet.
    run_ends: Option<Vec<usize>>,
    completion: Option<Completion>,
}

impl FramePacket {
    fn packets(&self) -> impl Iterator<Item = &[u8]> {
        let single = self.run_ends.is_none().then_some(self.data.len());
        let mut start = 0;
        self.run_ends
            .iter()
            .flatten()
            .copied()
            .chain(single)
            .map(move |end| {
                let packet = &self.data[start..end];
                start = end;
                packet
            })
    }
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
                packets.push_back(FramePacket {
                    data,
                    run_ends: None,
                    completion,
                });
            }
            Self::Batch(batch) => {
                *flush_request = flush_request.merge(FlushRequest::IfNotSuspended);
                // A run is framed from where it lies: a handle for each of its packets
                // would cost more than framing them.
                packets.extend(batch.into_iter().map(|item| match item {
                    BatchItem::Packet(data) => FramePacket {
                        data,
                        run_ends: None,
                        completion: None,
                    },
                    BatchItem::Run(run) => FramePacket {
                        data: run.bytes.into(),
                        run_ends: Some(run.ends),
                        completion: None,
                    },
                }));
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
    // Room for what is not compressed, with a short length in front of each packet. The
    // packets of a run are small, so they count as not compressed.
    let room = |packet: &FramePacket| match &packet.run_ends {
        Some(ends) => packet.data.len() + 3 * ends.len(),
        None if writer.is_compressing_packet(&packet.data) => 0,
        None => packet.data.len() + 3,
    };
    let mut frame = Vec::with_capacity(batch.iter().map(room).sum());
    let frame_err = batch
        .iter()
        .flat_map(FramePacket::packets)
        .find_map(|packet| writer.frame_packet(packet, &mut frame).err());
    (writer, frame, frame_err)
}

/// Compressing fewer bytes than this takes less time than handing the batch to a blocking
/// thread, and every hand-off goes through the one lock of the blocking pool.
const OFFLOAD_COMPRESSION_BYTES: usize = 16 * 1024;

fn offloads_framing<W: AsyncWrite + Unpin>(
    writer: &TCPNetworkEncoder<W>,
    batch: &[FramePacket],
) -> bool {
    let compressed_bytes: usize = batch
        .iter()
        .flat_map(FramePacket::packets)
        .filter(|packet| writer.is_compressing_packet(packet))
        .map(<[u8]>::len)
        .sum();
    compressed_bytes >= OFFLOAD_COMPRESSION_BYTES
}

/// Batches with much to compress are framed on a blocking task.
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
    if offloads_framing(&writer, &packet_batch) {
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
        data @ (OutgoingPacket::Data { .. } | OutgoingPacket::Batch(_)) => {
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
        let (returned_writer, returned_batch, mut frame, frame_err) =
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

        if let Err(err) = writer.write_frame(&mut frame).await {
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
    use super::super::{JavaClient, PacketRun, pending::PendingConnection};
    use super::*;
    use crate::net::{GameProfile, PacketRateLimiter, PlayerConfig};
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::ConnectionState;
    use pumpkin_protocol::java::client::play::CPlayDisconnect;
    use pumpkin_protocol::java::packet_decoder::TCPNetworkDecoder;
    use pumpkin_protocol::packet::MultiVersionJavaPacket;
    use pumpkin_util::text::TextComponent;
    use std::net::{Ipv4Addr, SocketAddr};
    use std::pin::Pin;
    use std::sync::Weak;
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

    /// A connected `JavaClient` and what its client end receives.
    fn java_client() -> (
        Arc<JavaClient>,
        TCPNetworkDecoder<tokio::io::ReadHalf<tokio::io::DuplexStream>>,
    ) {
        let (client_end, server_end) = tokio::io::duplex(1 << 20);
        let (read, write) = tokio::io::split(server_end);
        let pending = PendingConnection::from_transport(
            Box::new(read),
            Box::new(write),
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            0,
            PacketRateLimiter::new(false, 0.0, 0.0),
            Weak::new(),
        );
        let profile = GameProfile {
            id: uuid::Uuid::nil(),
            name: "player".to_string(),
            properties: arc_swap::ArcSwap::default(),
            profile_actions: None,
        };
        let mut client = JavaClient::from_pending(pending, profile, PlayerConfig::default());
        client.connection_state.store(ConnectionState::Play);
        client.start_outgoing_packet_task();
        let (read, _write) = tokio::io::split(client_end);
        (Arc::new(client), TCPNetworkDecoder::new(read))
    }

    /// A packet with id `id` and `payload` as its only field.
    fn numbered(id: u8, payload: u32) -> Bytes {
        let mut data = vec![id];
        data.extend_from_slice(&payload.to_be_bytes());
        data.into()
    }

    #[tokio::test]
    async fn a_tick_reaches_the_client_in_the_order_it_was_queued() {
        let (client, mut received) = java_client();
        client.suspend_flushing();
        client.try_enqueue_packet(numbered(1, 0));
        client.try_enqueue_packet(numbered(2, 0));
        // Does not wait for the end of the tick, and must not overtake what was queued before.
        assert!(client.send_packet_now_data(numbered(3, 0)).await);
        client.try_enqueue_packet(numbered(4, 0));
        client.resume_flushing();

        for id in 1..=4 {
            assert_eq!(received.get_raw_packet().await.unwrap().id, id);
        }
    }

    #[test]
    fn only_batches_with_much_to_compress_leave_the_writer_task() {
        let mut writer = TCPNetworkEncoder::new(tokio::io::sink());
        writer.set_compression((256, 4));
        let batch = |sizes: &[usize]| -> Vec<FramePacket> {
            sizes
                .iter()
                .map(|&size| FramePacket {
                    data: Bytes::from(vec![0; size]),
                    run_ends: None,
                    completion: None,
                })
                .collect()
        };
        // Below the compression threshold nothing is compressed, however many packets.
        assert!(!offloads_framing(&writer, &batch(&[200; 500])));
        assert!(!offloads_framing(&writer, &batch(&[200, 4096, 200])));
        assert!(offloads_framing(&writer, &batch(&[200, 64 * 1024])));
        assert!(offloads_framing(&writer, &batch(&[4096; 4])));

        // A run counts packet by packet, like the same packets one by one.
        let run = |size: usize, packets: usize| -> Vec<FramePacket> {
            vec![FramePacket {
                data: Bytes::from(vec![0; size * packets]),
                run_ends: Some((1..=packets).map(|packet| packet * size).collect()),
                completion: None,
            }]
        };
        assert!(!offloads_framing(&writer, &run(200, 500)));
        assert!(offloads_framing(&writer, &run(300, 100)));
    }

    #[tokio::test]
    async fn a_run_is_framed_packet_by_packet_also_when_some_are_compressed() {
        let sizes = [5usize, 300, 7, 256, 255];
        let mut packets = PacketRun::default();
        for (id, &size) in (1u8..).zip(&sizes) {
            let mut packet = vec![id];
            packet.resize(size, id);
            packets.push(&packet);
        }
        let batch = [
            FramePacket {
                data: packets.bytes.into(),
                run_ends: Some(packets.ends),
                completion: None,
            },
            FramePacket {
                data: numbered(6, 0),
                run_ends: None,
                completion: None,
            },
            // A run of nothing frames nothing.
            FramePacket {
                data: Bytes::new(),
                run_ends: Some(Vec::new()),
                completion: None,
            },
        ];
        let mut writer = TCPNetworkEncoder::new(tokio::io::sink());
        writer.set_compression((256, 4));
        let (_, frame, frame_err) = frame_packet_batch(writer, &batch);
        assert!(frame_err.is_none());

        let mut received = TCPNetworkDecoder::new(std::io::Cursor::new(frame));
        received.set_compression(256);
        for (id, &size) in (1..).zip(&sizes) {
            let packet = received.get_raw_packet().await.unwrap();
            // The id is a one-byte `VarInt`, the rest is the payload.
            assert_eq!((packet.id, packet.payload.len()), (id, size - 1));
        }
        assert_eq!(received.get_raw_packet().await.unwrap().id, 6);
        assert!(received.get_raw_packet().await.is_err());
    }

    fn run(ids: std::ops::RangeInclusive<u8>) -> PacketRun {
        let mut run = PacketRun::default();
        for id in ids {
            run.push(&numbered(id, 0));
        }
        run
    }

    #[tokio::test]
    async fn a_run_arrives_packet_by_packet_where_it_was_queued() {
        let (client, mut received) = java_client();
        client.try_enqueue_run(run(1..=2));
        client.suspend_flushing();
        client.try_enqueue_packet(numbered(3, 0));
        client.try_enqueue_run(run(4..=6));
        client.try_enqueue_packet(numbered(7, 0));
        client.resume_flushing();

        for id in 1..=7 {
            assert_eq!(received.get_raw_packet().await.unwrap().id, id);
        }
        // Each packet of a run counted once: nothing stays reserved after the writer is done.
        client.send_packet_now_data(numbered(8, 0)).await;
        assert_eq!(client.pending_bytes.load(Ordering::Acquire), 0);
    }

    #[tokio::test]
    async fn several_runs_are_queued_as_one_with_every_packet_in_order() {
        let (client, mut received) = java_client();
        // Outside play no plugin sees a packet, whatever plugins other tests register.
        client.connection_state.store(ConnectionState::Config);
        client.suspend_flushing();
        let mut runs = vec![run(1..=2), run(3..=3), run(4..=6)];
        client.try_enqueue_runs(&mut runs);
        assert!(runs.is_empty());
        // One buffer for the writer, however many the tick threads filled.
        assert_eq!(client.lock_tick_batch().len(), 1);
        client.resume_flushing();

        for id in 1..=6 {
            assert_eq!(received.get_raw_packet().await.unwrap().id, id);
        }
        client.send_packet_now_data(numbered(7, 0)).await;
        assert_eq!(client.pending_bytes.load(Ordering::Acquire), 0);
    }

    struct SeesPackets;
    impl crate::plugin::EventHandler<crate::plugin::server::packet::PacketSentEvent> for SeesPackets {}

    #[tokio::test]
    async fn a_run_still_arrives_when_a_plugin_watches_packets() {
        let plugins = crate::plugin::PluginManager::new(Vec::new());
        plugins.register(
            Arc::new(SeesPackets),
            crate::plugin::EventPriority::Normal,
            false,
        );
        let (client, mut received) = java_client();
        client.try_enqueue_run(run(1..=3));

        for id in 1..=3 {
            assert_eq!(received.get_raw_packet().await.unwrap().id, id);
        }
    }

    #[tokio::test]
    async fn a_kick_mid_tick_follows_the_packets_queued_before_it() {
        let (client, mut received) = java_client();
        client.suspend_flushing();
        client.try_enqueue_packet(numbered(1, 0));
        client.try_kick(&TextComponent::text("bye"));

        assert_eq!(received.get_raw_packet().await.unwrap().id, 1);
        assert_eq!(
            received.get_raw_packet().await.unwrap().id,
            CPlayDisconnect::to_id(CURRENT_MC_VERSION)
        );
    }

    #[tokio::test]
    async fn closing_mid_tick_still_sends_what_the_tick_queued() {
        let (client, mut received) = java_client();
        client.suspend_flushing();
        client.try_enqueue_packet(numbered(1, 0));
        client.close();

        assert_eq!(received.get_raw_packet().await.unwrap().id, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_packet_racing_the_end_of_a_tick_is_neither_first_nor_left_behind() {
        const ROUNDS: u32 = 10_000;
        let (client, mut received) = java_client();
        let runtime = tokio::runtime::Handle::current();

        for round in 0..ROUNDS {
            client.suspend_flushing();
            client.try_enqueue_packet(numbered(1, round));
            let start = Arc::new(AtomicBool::new(false));
            let racer = {
                let (client, start, runtime) = (client.clone(), start.clone(), runtime.clone());
                tokio::task::spawn_blocking(move || {
                    while !start.load(Ordering::Acquire) {
                        std::hint::spin_loop();
                    }
                    // One that goes out at once could overtake the tick, an ordinary one
                    // could stay behind with no later tick or packet to carry it out.
                    if round % 2 == 0 {
                        runtime.block_on(client.send_packet_now_data(numbered(2, round)));
                    } else {
                        client.try_enqueue_packet(numbered(2, round));
                    }
                })
            };
            start.store(true, Ordering::Release);
            client.resume_flushing();
            racer.await.unwrap();

            for id in 1..=2 {
                let packet =
                    tokio::time::timeout(Duration::from_secs(5), received.get_raw_packet())
                        .await
                        .expect("left behind after the tick ended")
                        .unwrap();
                assert_eq!(
                    (packet.id, &packet.payload[..]),
                    (id, &round.to_be_bytes()[..])
                );
            }
        }
    }
}
