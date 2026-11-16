use std::{
    io::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::net::UdpSocket;

/// Public UDP socket used by Bedrock's `NetherNet` ICE agent.
pub struct IceSocket {
    socket: Arc<UdpSocket>,
}

impl IceSocket {
    pub async fn bind(address: SocketAddr) -> Result<Self, Error> {
        let ip = match address.ip() {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => Ipv4Addr::UNSPECIFIED,
        };
        let socket = Arc::new(UdpSocket::bind((ip, address.port())).await?);
        Ok(Self { socket })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        self.socket.local_addr()
    }

    pub async fn recv_from(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        self.socket.recv_from(buffer).await
    }

    pub async fn send_to(&self, buffer: &[u8], target: SocketAddr) -> Result<usize, Error> {
        self.socket.send_to(buffer, target).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ice_socket_sends_and_receives() {
        let server = IceSocket::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        client
            .send_to(b"ping", server.local_addr().unwrap())
            .await
            .unwrap();

        let mut buffer = [0; 16];
        let (length, address) = server.recv_from(&mut buffer).await.unwrap();
        assert_eq!(&buffer[..length], b"ping");
        assert_eq!(address, client.local_addr().unwrap());

        server
            .send_to(b"pong", client.local_addr().unwrap())
            .await
            .unwrap();

        let (length, address) = client.recv_from(&mut buffer).await.unwrap();
        assert_eq!(&buffer[..length], b"pong");
        assert_eq!(address, server.local_addr().unwrap());
    }
}
