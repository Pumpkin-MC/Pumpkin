pub mod discovery;
mod endpoint;
mod ice_router;
pub mod ice_socket;
mod identity;
mod peer;
mod session;
mod state;

pub use endpoint::NetherNetListener;
pub use ice_socket::IceSocket;
pub use identity::load_or_create_identity_key;
pub use session::NetherNetSession;

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc, time::Duration};

    use tokio::sync::{Mutex, mpsc};
    use webrtc::peer_connection::{PeerConnectionBuilder, PeerConnectionEventHandler};

    use super::NetherNetSession;
    use crate::net::{PacketRateLimiter, bedrock::BedrockClient};
    use pumpkin_protocol::RawPacket;

    #[tokio::test]
    async fn admission_queue_preserves_input_and_releases_senders_on_close()
    -> Result<(), Box<dyn std::error::Error>> {
        struct Handler;
        #[async_trait::async_trait]
        impl PeerConnectionEventHandler for Handler {}

        let _ = rustls::crypto::ring::default_provider().install_default();
        let peer = Box::pin(
            PeerConnectionBuilder::new()
                .with_handler(Arc::new(Handler))
                .with_udp_addrs(vec!["127.0.0.1:0"])
                .build(),
        )
        .await?;
        let (incoming, _sessions) = mpsc::channel(1);
        let address = "127.0.0.1:19132".parse()?;
        let session = Arc::new(NetherNetSession::new(
            Arc::new(peer),
            None,
            address,
            incoming,
        ));
        let client = BedrockClient::new(
            1,
            session,
            address,
            Arc::new(Mutex::new(HashMap::new())),
            PacketRateLimiter::new(false, 0.0, 0.0),
            std::sync::Weak::new(),
        );
        let mut receiver = client
            .incoming_game_packet_recv
            .lock()
            .await
            .take()
            .ok_or("No receiver")?;
        let capacity = client.incoming_game_packet_send.capacity();
        let packet = |id| RawPacket {
            id,
            payload: bytes::Bytes::new(),
        };
        for _ in 0..capacity {
            client.handle_game_packet(packet(7)).await?;
        }
        let pending = client.handle_game_packet(packet(8));
        tokio::pin!(pending);
        assert!(
            tokio::time::timeout(Duration::from_millis(10), &mut pending)
                .await
                .is_err()
        );
        assert_eq!(receiver.recv().await.ok_or("No queued input")?.id, 7);
        tokio::time::timeout(Duration::from_secs(1), pending).await??;
        for _ in 1..capacity {
            assert_eq!(receiver.recv().await.ok_or("Lost queued input")?.id, 7);
        }
        assert_eq!(receiver.recv().await.ok_or("Lost pending input")?.id, 8);

        for _ in 0..capacity {
            client.handle_game_packet(packet(7)).await?;
        }
        let pending = client.handle_game_packet(packet(9));
        tokio::pin!(pending);
        assert!(
            tokio::time::timeout(Duration::from_millis(10), &mut pending)
                .await
                .is_err()
        );
        client.close().await;
        let result = tokio::time::timeout(Duration::from_secs(1), pending).await?;
        assert_eq!(
            result.err().ok_or("Accepted input after close")?.kind(),
            std::io::ErrorKind::Interrupted
        );
        assert_eq!(receiver.len(), capacity);
        Ok(())
    }
}
