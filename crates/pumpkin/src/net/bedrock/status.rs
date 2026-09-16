use std::{io::Error, net::SocketAddr, sync::Arc};

use bytes::Bytes;
use tokio::{
    net::UdpSocket,
    sync::{Mutex, mpsc},
};
use tracing::trace;

/// UDP socket shared by `RakNet` and `NetherNet` ICE.
pub struct BedrockUdpSocket {
    socket: Arc<UdpSocket>,
    ice_packets: mpsc::Sender<(Bytes, SocketAddr)>,
}

/// The `NetherNet` ICE view of the shared Bedrock UDP socket.
pub struct IceSocket {
    socket: Arc<UdpSocket>,
    packets: Mutex<mpsc::Receiver<(Bytes, SocketAddr)>>,
}

impl BedrockUdpSocket {
    pub async fn bind(address: SocketAddr) -> Result<(Self, IceSocket), Error> {
        let socket = Arc::new(UdpSocket::bind(address).await?);
        let (ice_packets, packets) = mpsc::channel(1024);
        Ok((
            Self {
                socket: socket.clone(),
                ice_packets,
            },
            IceSocket {
                socket,
                packets: Mutex::new(packets),
            },
        ))
    }

    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        self.socket.local_addr()
    }

    #[must_use]
    pub const fn socket(&self) -> &Arc<UdpSocket> {
        &self.socket
    }

    pub async fn recv_from(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        self.socket.recv_from(buffer).await
    }

    /// Hands a datagram that is not a `RakNet` packet to the ICE agent.
    pub fn forward_to_ice(&self, packet: &[u8], client: SocketAddr) {
        if self
            .ice_packets
            .try_send((Bytes::copy_from_slice(packet), client))
            .is_err()
        {
            trace!(%client, "Dropped Bedrock ICE datagram because its queue is unavailable");
        }
    }
}

impl IceSocket {
    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        self.socket.local_addr()
    }

    pub async fn recv_from(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        let (packet, address) = self.packets.lock().await.recv().await.ok_or_else(|| {
            Error::new(std::io::ErrorKind::BrokenPipe, "Bedrock UDP socket closed")
        })?;
        let length = buffer.len().min(packet.len());
        buffer[..length].copy_from_slice(&packet[..length]);
        Ok((length, address))
    }

    pub async fn send_to(&self, buffer: &[u8], target: SocketAddr) -> Result<usize, Error> {
        self.socket.send_to(buffer, target).await
    }
}
