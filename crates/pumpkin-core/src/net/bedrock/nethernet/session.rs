use std::{
    future::Future,
    net::SocketAddr,
    sync::{
        Arc, Mutex as StdMutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    time::Duration,
};

use bytes::{BufMut, Bytes, BytesMut};
use pumpkin_auth::p384::PublicKey;
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use webrtc::{
    data_channel::{DataChannel, DataChannelEvent},
    peer_connection::PeerConnection,
};

use super::ice_router::Registration;

const RELIABLE_CHANNEL: &str = "ReliableDataChannel";
const UNRELIABLE_CHANNEL: &str = "UnreliableDataChannel";
// NetherNet splits encoded packets that exceed 10,000 bytes into application-level
// segments. Larger SCTP messages are rejected by some Bedrock clients.
const MAX_FRAGMENT_SIZE: usize = 10_000;
const RELIABLE_FLUSH_POLL_INTERVAL: Duration = Duration::from_millis(5);

pub(super) type IncomingSession = (Arc<NetherNetSession>, SocketAddr);

async fn wait_for_reliable_delivery<F, Fut>(
    timeout: Duration,
    mut outstanding_bytes: F,
) -> Result<(), String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<usize, String>>,
{
    tokio::time::timeout(timeout, async {
        loop {
            match outstanding_bytes().await {
                Ok(0) => return Ok(()),
                Ok(_) => tokio::time::sleep(RELIABLE_FLUSH_POLL_INTERVAL).await,
                Err(error) => return Err(error),
            }
        }
    })
    .await
    .map_err(|_| "timed out waiting for reliable messages to be acknowledged".to_string())?
}

/// A WebRTC connection carrying complete Bedrock batch packets.
pub struct NetherNetSession {
    peer: Arc<dyn PeerConnection>,
    reliable: RwLock<Option<Arc<dyn DataChannel>>>,
    unreliable: RwLock<Option<Arc<dyn DataChannel>>>,
    fragments: Mutex<FragmentBuffer>,
    packets: Mutex<mpsc::Receiver<Bytes>>,
    packet_sender: mpsc::Sender<Bytes>,
    open_channels: AtomicU8,
    accepted: AtomicBool,
    closed: CancellationToken,
    client_public_key: Option<PublicKey>,
    address: SocketAddr,
    incoming: mpsc::Sender<IncomingSession>,
    ice_route: StdMutex<Option<Registration>>,
}

impl NetherNetSession {
    pub(super) fn new(
        peer: Arc<dyn PeerConnection>,
        client_public_key: Option<PublicKey>,
        address: SocketAddr,
        incoming: mpsc::Sender<IncomingSession>,
    ) -> Self {
        let (packet_sender, packets) = mpsc::channel(4096);
        Self {
            peer,
            reliable: RwLock::new(None),
            unreliable: RwLock::new(None),
            fragments: Mutex::new(FragmentBuffer::default()),
            packets: Mutex::new(packets),
            packet_sender,
            open_channels: AtomicU8::new(0),
            accepted: AtomicBool::new(false),
            closed: CancellationToken::new(),
            client_public_key,
            address,
            incoming,
            ice_route: StdMutex::new(None),
        }
    }

    pub(super) fn set_ice_route(&self, route: Registration) {
        if !self.closed.is_cancelled() {
            *self
                .ice_route
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(route);
        }
    }

    pub(super) async fn attach_channel(
        self: &Arc<Self>,
        channel: Arc<dyn DataChannel>,
    ) -> Result<(), String> {
        let label = channel.label().await.map_err(|e| e.to_string())?;
        let ordered = channel.ordered().await.map_err(|e| e.to_string())?;
        let protocol = channel.protocol().await.map_err(|e| e.to_string())?;
        let negotiated = channel.negotiated().await.map_err(|e| e.to_string())?;
        let max_packet_lifetime = channel
            .max_packet_life_time()
            .await
            .map_err(|e| e.to_string())?;
        let max_retransmits = channel.max_retransmits().await.map_err(|e| e.to_string())?;

        let has_default_parameters =
            protocol.is_empty() && !negotiated && max_packet_lifetime.is_none();
        let bit = match label.as_str() {
            RELIABLE_CHANNEL if ordered && has_default_parameters && max_retransmits.is_none() => {
                *self.reliable.write().await = Some(channel.clone());
                1
            }
            UNRELIABLE_CHANNEL
                if !ordered
                    && has_default_parameters
                    && (max_retransmits.is_none() || max_retransmits == Some(0)) =>
            {
                *self.unreliable.write().await = Some(channel.clone());
                2
            }
            label => return Err(format!("invalid channel {label:?}")),
        };

        let session = self.clone();
        tokio::spawn(async move {
            let mut opened = false;
            while let Some(event) = channel.poll().await {
                match event {
                    DataChannelEvent::OnOpen => {
                        opened = true;
                        session.channel_opened(bit).await;
                    }
                    DataChannelEvent::OnMessage(msg) => {
                        if !opened {
                            opened = true;
                            session.channel_opened(bit).await;
                        }
                        if let Err(error) = session.receive_segment(bit, msg.data.into()).await {
                            warn!(
                                "Invalid NetherNet message from {}: {error}",
                                session.address
                            );
                            break;
                        }
                    }
                    DataChannelEvent::OnClose => break,
                    DataChannelEvent::OnError => {
                        warn!(address = %session.address, "Failed to read NetherNet data channel");
                        break;
                    }
                    _ => {}
                }
            }
            session.close().await;
        });

        Ok(())
    }

    async fn channel_opened(self: &Arc<Self>, bit: u8) {
        let open = self.open_channels.fetch_or(bit, Ordering::AcqRel) | bit;
        if open == 3 && !self.accepted.swap(true, Ordering::AcqRel) {
            debug!(
                "Accepted Bedrock NetherNet connection from {}",
                self.address
            );
            if self
                .incoming
                .send((self.clone(), self.address))
                .await
                .is_err()
            {
                self.close().await;
            }
        }
    }

    async fn receive_segment(&self, channel: u8, data: Bytes) -> Result<(), String> {
        let (&remaining, payload) = data
            .split_first()
            .ok_or_else(|| "empty data-channel message".to_string())?;
        if payload.is_empty() {
            return Err("empty NetherNet packet segment".to_string());
        }
        if channel == 2 {
            if remaining != 0 {
                return Err("fragmented unreliable message".to_string());
            }
            self.packet_sender
                .send(Bytes::copy_from_slice(payload))
                .await
                .map_err(|_| "connection is closed".to_string())?;
            return Ok(());
        }

        let packet = {
            let mut fragments = self.fragments.lock().await;
            fragments.push(remaining, payload)?
        };
        if let Some(packet) = packet {
            self.packet_sender
                .send(packet)
                .await
                .map_err(|_| "connection is closed".to_string())?;
        }
        Ok(())
    }

    pub async fn recv(&self) -> Option<Bytes> {
        let mut packets = self.packets.lock().await;
        tokio::select! {
            () = self.closed.cancelled() => None,
            packet = packets.recv() => packet,
        }
    }

    pub async fn send(&self, data: Bytes) -> Result<(), String> {
        if self.is_closed() {
            return Err("connection is closed".to_string());
        }
        let channel = self
            .reliable
            .read()
            .await
            .clone()
            .ok_or_else(|| "reliable channel is not open".to_string())?;
        let segment_count = data.len().div_ceil(MAX_FRAGMENT_SIZE).max(1);
        if segment_count > 256 {
            return Err("Bedrock batch is too large for NetherNet".to_string());
        }
        for (index, chunk) in data.chunks(MAX_FRAGMENT_SIZE).enumerate() {
            let mut segment = BytesMut::with_capacity(chunk.len() + 1);
            segment.put_u8((segment_count - index - 1) as u8);
            segment.extend_from_slice(chunk);
            channel
                .send(segment)
                .await
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub async fn send_unreliable(&self, data: Bytes) -> Result<(), String> {
        if self.is_closed() {
            return Err("connection is closed".to_string());
        }
        if data.len() > MAX_FRAGMENT_SIZE {
            return Err("unreliable NetherNet packet is too large".to_string());
        }
        let channel = self
            .unreliable
            .read()
            .await
            .clone()
            .ok_or_else(|| "unreliable channel is not open".to_string())?;
        let mut segment = BytesMut::with_capacity(data.len() + 1);
        segment.put_u8(0);
        segment.extend_from_slice(&data);
        channel
            .send(segment)
            .await
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    /// Waits for reliable messages to be acknowledged; [`DataChannel::send`] only queues them,
    /// so closing immediately afterwards could discard the final message.
    pub(in crate::net::bedrock) async fn flush_reliable(
        &self,
        timeout: Duration,
    ) -> Result<(), String> {
        let channel = self
            .reliable
            .read()
            .await
            .clone()
            .ok_or_else(|| "reliable channel is not open".to_string())?;

        wait_for_reliable_delivery(timeout, || {
            let channel = channel.clone();
            async move {
                channel
                    .outstanding_bytes()
                    .await
                    .map_err(|error| error.to_string())
            }
        })
        .await
    }

    pub const fn client_public_key(&self) -> Option<&PublicKey> {
        self.client_public_key.as_ref()
    }

    pub fn is_closed(&self) -> bool {
        self.closed.is_cancelled()
    }

    pub(super) fn mark_closed(&self) {
        if !self.closed.is_cancelled() {
            self.closed.cancel();
            self.ice_route
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
        }
    }

    pub async fn close(&self) {
        if self.closed.is_cancelled() {
            return;
        }
        self.closed.cancel();
        self.ice_route
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let _ = self.peer.close().await;
    }
}

#[derive(Default)]
struct FragmentBuffer {
    next_remaining: Option<u8>,
    data: Vec<u8>,
}

impl FragmentBuffer {
    fn push(&mut self, remaining: u8, payload: &[u8]) -> Result<Option<Bytes>, String> {
        match self.next_remaining {
            None if remaining > 0 => self.next_remaining = Some(remaining - 1),
            None => return Ok(Some(Bytes::copy_from_slice(payload))),
            Some(expected) if expected == remaining => {
                self.next_remaining = remaining.checked_sub(1);
            }
            Some(expected) => {
                self.next_remaining = None;
                self.data.clear();
                return Err(format!(
                    "out-of-order fragment: expected {expected}, got {remaining}"
                ));
            }
        }
        self.data.extend_from_slice(payload);
        if remaining == 0 {
            self.next_remaining = None;
            return Ok(Some(Bytes::from(std::mem::take(&mut self.data))));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[tokio::test]
    async fn reliable_delivery_waits_until_no_bytes_are_outstanding() {
        let polls = Arc::new(AtomicUsize::new(0));
        let result = wait_for_reliable_delivery(Duration::from_secs(1), || {
            let polls = polls.clone();
            async move {
                let poll = polls.fetch_add(1, Ordering::Relaxed);
                Ok(usize::from(poll == 0))
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(polls.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn reliable_delivery_wait_is_bounded() {
        let result = wait_for_reliable_delivery(Duration::from_millis(1), || async { Ok(1) }).await;

        assert_eq!(
            result.unwrap_err(),
            "timed out waiting for reliable messages to be acknowledged"
        );
    }

    #[test]
    fn fragments_round_trip() {
        let mut fragments = FragmentBuffer::default();
        assert!(fragments.push(2, b"one").unwrap().is_none());
        assert!(fragments.push(1, b"two").unwrap().is_none());
        assert_eq!(fragments.push(0, b"three").unwrap().unwrap(), "onetwothree");
    }

    #[test]
    fn rejects_out_of_order_fragments_and_recovers() {
        let mut fragments = FragmentBuffer::default();
        assert!(fragments.push(2, b"one").unwrap().is_none());
        assert!(fragments.push(0, b"three").is_err());
        assert_eq!(fragments.push(0, b"complete").unwrap().unwrap(), "complete");
    }
}
