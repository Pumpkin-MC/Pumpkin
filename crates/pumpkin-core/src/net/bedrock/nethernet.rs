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
pub(super) async fn test_session(
    address: std::net::SocketAddr,
) -> Result<std::sync::Arc<NetherNetSession>, webrtc::error::Error> {
    use std::sync::Arc;
    use webrtc::peer_connection::{PeerConnectionBuilder, PeerConnectionEventHandler};

    struct QueueTestHandler;
    #[async_trait::async_trait]
    impl PeerConnectionEventHandler for QueueTestHandler {}

    let peer = Box::pin(
        PeerConnectionBuilder::new()
            .with_udp_addrs(vec![address])
            .with_handler(Arc::new(QueueTestHandler))
            .build(),
    )
    .await?;
    let (incoming, _) = tokio::sync::mpsc::channel(1);
    Ok(Arc::new(NetherNetSession::new(
        Arc::new(peer),
        None,
        address,
        incoming,
    )))
}
