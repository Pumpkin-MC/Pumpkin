use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use bytes::Bytes;
use pumpkin_util::p384::PublicKey;
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use webrtc::{
    data_channel::{RTCDataChannel, data_channel_message::DataChannelMessage},
    peer_connection::RTCPeerConnection,
};

pub const RELIABLE_CHANNEL: &str = "ReliableDataChannel";
pub const UNRELIABLE_CHANNEL: &str = "UnreliableDataChannel";

// NetherNet splits encoded packets that exceed 10,000 bytes into application-level
// segments. Larger SCTP messages are rejected by some Bedrock clients.
const MAX_FRAGMENT_SIZE: usize = 10_000;
const MAX_FRAGMENTS: usize = 256;

pub type IncomingSession = (Arc<NetherNetSession>, SocketAddr);

/// A WebRTC connection carrying complete Bedrock batch packets.
pub struct NetherNetSession {
    peer: Arc<RTCPeerConnection>,
    reliable: RwLock<Option<Arc<RTCDataChannel>>>,
    unreliable: RwLock<Option<Arc<RTCDataChannel>>>,
    fragments: Mutex<FragmentBuffer>,
    packets: Mutex<mpsc::Receiver<Bytes>>,
    packet_sender: mpsc::Sender<Bytes>,
    accepted: AtomicBool,
    closed: CancellationToken,
    client_public_key: PublicKey,
    address: SocketAddr,
    incoming: mpsc::Sender<IncomingSession>,
}

impl NetherNetSession {
    pub fn new(
        peer: Arc<RTCPeerConnection>,
        client_public_key: PublicKey,
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
            accepted: AtomicBool::new(false),
            closed: CancellationToken::new(),
            client_public_key,
            address,
            incoming,
        }
    }

    pub async fn attach_channel(self: &Arc<Self>, channel: Arc<RTCDataChannel>) {
        let reliable = match channel.label() {
            RELIABLE_CHANNEL => {
                *self.reliable.write().await = Some(channel.clone());
                true
            }
            UNRELIABLE_CHANNEL => {
                *self.unreliable.write().await = Some(channel.clone());
                false
            }
            label => {
                debug!("Ignoring unknown NetherNet data channel {label:?}");
                return;
            }
        };

        let session = self.clone();
        channel.on_message(Box::new(move |message: DataChannelMessage| {
            let session = session.clone();
            Box::pin(async move {
                if let Err(error) = session.receive_segment(reliable, message.data).await {
                    warn!(
                        "Invalid NetherNet message from {}: {error}",
                        session.address
                    );
                    session.close().await;
                }
            })
        }));

        if !reliable {
            return;
        }

        let session = self.clone();
        channel.on_close(Box::new(move || {
            let session = session.clone();
            Box::pin(async move {
                session.close().await;
            })
        }));

        // An inbound channel may already be open by the time this handler is registered.
        if channel.ready_state()
            == webrtc::data_channel::data_channel_state::RTCDataChannelState::Open
        {
            self.accept().await;
        } else {
            let session = self.clone();
            channel.on_open(Box::new(move || {
                Box::pin(async move {
                    session.accept().await;
                })
            }));
        }
    }

    async fn accept(self: &Arc<Self>) {
        if self.accepted.swap(true, Ordering::AcqRel) {
            return;
        }
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

    async fn receive_segment(&self, reliable: bool, data: Bytes) -> Result<(), String> {
        let (&remaining, payload) = data
            .split_first()
            .ok_or_else(|| "empty data-channel message".to_string())?;
        if payload.is_empty() {
            return Err("empty NetherNet packet segment".to_string());
        }
        if !reliable {
            if remaining != 0 {
                return Err("fragmented unreliable message".to_string());
            }
            return self.deliver(Bytes::copy_from_slice(payload)).await;
        }

        let packet = {
            let mut fragments = self.fragments.lock().await;
            fragments.push(remaining, payload)?
        };
        match packet {
            Some(packet) => self.deliver(packet).await,
            None => Ok(()),
        }
    }

    async fn deliver(&self, packet: Bytes) -> Result<(), String> {
        self.packet_sender
            .send(packet)
            .await
            .map_err(|_| "connection is closed".to_string())
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
        if segment_count > MAX_FRAGMENTS {
            return Err("Bedrock batch is too large for NetherNet".to_string());
        }
        for (index, chunk) in data.chunks(MAX_FRAGMENT_SIZE).enumerate() {
            let mut segment = Vec::with_capacity(chunk.len() + 1);
            segment.push((segment_count - index - 1) as u8);
            segment.extend_from_slice(chunk);
            channel
                .send(&Bytes::from(segment))
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
        let mut segment = Vec::with_capacity(data.len() + 1);
        segment.push(0);
        segment.extend_from_slice(&data);
        channel
            .send(&Bytes::from(segment))
            .await
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub const fn client_public_key(&self) -> &PublicKey {
        &self.client_public_key
    }

    pub fn is_closed(&self) -> bool {
        self.closed.is_cancelled()
    }

    pub fn mark_closed(&self) {
        self.closed.cancel();
    }

    /// Tearing the peer down inline would deadlock when this is called from one of
    /// its own callbacks, so it is closed on a detached task.
    #[allow(clippy::unused_async)]
    pub async fn close(&self) {
        if self.closed.is_cancelled() {
            return;
        }
        self.closed.cancel();
        let peer = self.peer.clone();
        tokio::spawn(async move {
            let _ = peer.close().await;
        });
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

    #[test]
    fn fragments_round_trip() {
        let mut fragments = FragmentBuffer::default();
        assert!(fragments.push(2, b"one").unwrap().is_none());
        assert!(fragments.push(1, b"two").unwrap().is_none());
        assert_eq!(fragments.push(0, b"three").unwrap().unwrap(), "onetwothree");
    }

    #[test]
    fn outbound_payloads_are_split_at_the_nethernet_limit() {
        let payload = vec![0; MAX_FRAGMENT_SIZE + 1];
        let chunks = payload.chunks(MAX_FRAGMENT_SIZE).collect::<Vec<_>>();

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].len(), 10_000);
        assert_eq!(chunks[1].len(), 1);
    }

    #[test]
    fn rejects_out_of_order_fragments_and_recovers() {
        let mut fragments = FragmentBuffer::default();
        assert!(fragments.push(2, b"one").unwrap().is_none());
        assert!(fragments.push(0, b"three").is_err());
        assert_eq!(fragments.push(0, b"complete").unwrap().unwrap(), "complete");
    }
}
