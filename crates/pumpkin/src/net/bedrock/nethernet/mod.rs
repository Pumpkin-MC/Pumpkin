mod crypto;
mod discovery;
mod identity;
mod session;
mod signal;

pub use identity::load_or_create_identity_key;
pub use session::NetherNetSession;

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use pumpkin_util::{GameMode, jwt::Jwks, p384::ecdsa::SigningKey};
use tokio::{
    net::UdpSocket,
    sync::{Mutex, mpsc},
};
use tracing::{debug, info, warn};
use webrtc::{
    api::{APIBuilder, media_engine::MediaEngine},
    ice_transport::{ice_candidate::RTCIceCandidateInit, ice_server::RTCIceServer},
    peer_connection::{
        RTCPeerConnection, configuration::RTCConfiguration,
        peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
};

use self::{
    discovery::{
        CONNECTION_TYPE_LAN_SIGNALING, DiscoveryPacket, ServerData, TRANSPORT_LAYER_NETHERNET,
    },
    identity::{add_server_identity, verify_and_strip_identity},
    session::IncomingSession,
    signal::Signal,
};
use crate::{STOP_INTERRUPT, server::Server};

/// Liveness probe clients send before opening a connection.
const PING_MESSAGE: &str = "Ping";

const DATAGRAM_BUFFER_SIZE: usize = 65535;
const NEGOTIATION_TIMEOUT: Duration = Duration::from_secs(15);
const MAINTENANCE_INTERVAL: Duration = Duration::from_secs(1);
const GATHERING_TIMEOUT: Duration = Duration::from_secs(10);

/// Accepts Bedrock `NetherNet` connections negotiated over the LAN discovery protocol.
pub struct NetherNetListener {
    incoming: Mutex<mpsc::Receiver<IncomingSession>>,
    local_addr: SocketAddr,
}

impl NetherNetListener {
    pub async fn bind(
        server: Arc<Server>,
        address: SocketAddr,
        identity_key: Arc<SigningKey>,
        oidc_verifier: Option<Arc<(String, Jwks)>>,
        stun_servers: Vec<String>,
    ) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(address).await?;
        socket.set_broadcast(true)?;
        let local_addr = socket.local_addr()?;
        let network_id = server.server_guid;
        let (incoming, receiver) = mpsc::channel(128);

        let transport = Arc::new(Transport {
            socket,
            network_id,
            world_id: format!("{:016x}", server.server_guid),
            identity_key,
            oidc_verifier,
            stun_servers: stun_servers.into(),
            server,
            incoming,
            negotiations: Mutex::new(HashMap::new()),
        });
        tokio::spawn(Transport::run(transport));

        info!("Bedrock NetherNet discovery is listening on {local_addr} (network ID {network_id})");
        Ok(Self {
            incoming: Mutex::new(receiver),
            local_addr,
        })
    }

    pub async fn accept(&self) -> Option<IncomingSession> {
        self.incoming.lock().await.recv().await
    }

    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

struct Negotiation {
    peer: Arc<RTCPeerConnection>,
    address: SocketAddr,
    network_id: u64,
    /// Candidates buffered until the remote description is applied.
    candidates: Mutex<Option<Vec<String>>>,
    /// Whether the client asserted an identity and therefore expects one back.
    assert_identity: bool,
    started: Instant,
}

struct Transport {
    socket: UdpSocket,
    network_id: u64,
    world_id: String,
    identity_key: Arc<SigningKey>,
    oidc_verifier: Option<Arc<(String, Jwks)>>,
    stun_servers: Arc<[String]>,
    server: Arc<Server>,
    incoming: mpsc::Sender<IncomingSession>,
    negotiations: Mutex<HashMap<u64, Arc<Negotiation>>>,
}

impl Transport {
    async fn run(self: Arc<Self>) {
        let mut buffer = vec![0; DATAGRAM_BUFFER_SIZE];
        let mut maintenance = tokio::time::interval(MAINTENANCE_INTERVAL);
        loop {
            tokio::select! {
                () = STOP_INTERRUPT.cancelled() => break,
                _ = maintenance.tick() => self.expire_negotiations().await,
                result = self.socket.recv_from(&mut buffer) => match result {
                    Ok((length, address)) => {
                        self.clone().handle_datagram(&buffer[..length], address).await;
                    }
                    Err(error) => {
                        warn!("NetherNet discovery socket failed: {error}");
                        break;
                    }
                },
            }
        }
    }

    async fn handle_datagram(self: Arc<Self>, datagram: &[u8], address: SocketAddr) {
        let Some((packet, sender_id)) = discovery::unmarshal(datagram) else {
            debug!("Ignoring invalid NetherNet discovery datagram from {address}");
            return;
        };
        if sender_id == self.network_id {
            return;
        }

        match packet {
            DiscoveryPacket::Request => {
                let application_data = self.server_data().await;
                self.send_packet(&DiscoveryPacket::Response { application_data }, address)
                    .await;
            }
            DiscoveryPacket::Message { recipient_id, data } => {
                if recipient_id != self.network_id {
                    debug!(
                        "Ignoring NetherNet message from {address} addressed to network {recipient_id}"
                    );
                    return;
                }
                // Clients probe a server with an empty or "Ping" message before connecting.
                if data.is_empty() || data == PING_MESSAGE {
                    return;
                }
                let Some(signal) = Signal::parse(&data) else {
                    debug!("Ignoring malformed NetherNet signal from {address}");
                    return;
                };
                self.handle_signal(signal, sender_id, address).await;
            }
            DiscoveryPacket::Response { .. } => {}
        }
    }

    async fn handle_signal(
        self: Arc<Self>,
        signal: Signal,
        sender_network_id: u64,
        address: SocketAddr,
    ) {
        debug!(
            "NetherNet signal {} from {address} (connection {})",
            signal.kind, signal.connection_id
        );
        match signal.kind.as_str() {
            signal::TYPE_OFFER => self.handle_offer(signal, sender_network_id, address).await,
            signal::TYPE_CANDIDATE => self.handle_candidate(signal).await,
            signal::TYPE_ERROR => {
                debug!(
                    "Remote NetherNet error on connection {}: {}",
                    signal.connection_id, signal.data
                );
                self.drop_negotiation(signal.connection_id).await;
            }
            _ => {}
        }
    }

    async fn handle_offer(
        self: Arc<Self>,
        signal: Signal,
        sender_network_id: u64,
        address: SocketAddr,
    ) {
        let connection_id = signal.connection_id;
        if self.negotiations.lock().await.contains_key(&connection_id) {
            return;
        }

        let (offer, client_public_key) =
            match verify_and_strip_identity(&signal.data, self.oidc_verifier.as_deref()) {
                Ok(identity) => identity,
                Err(error) => {
                    warn!("Rejecting NetherNet connection {connection_id} from {address}: {error}");
                    self.send_signal(
                        &Signal::new(signal::TYPE_ERROR, connection_id, error),
                        sender_network_id,
                        address,
                    )
                    .await;
                    return;
                }
            };

        let assert_identity = client_public_key.is_some();
        let peer = match self.create_peer_connection().await {
            Ok(peer) => peer,
            Err(error) => {
                warn!("Failed to create a NetherNet peer connection: {error}");
                return;
            }
        };

        let session = Arc::new(NetherNetSession::new(
            peer.clone(),
            client_public_key,
            address,
            self.incoming.clone(),
        ));

        let channel_session = session.clone();
        peer.on_data_channel(Box::new(move |channel| {
            let session = channel_session.clone();
            Box::pin(async move {
                session.attach_channel(channel).await;
            })
        }));

        let transport = self.clone();
        peer.on_peer_connection_state_change(Box::new(move |state| {
            let transport = transport.clone();
            let session = session.clone();
            Box::pin(async move {
                match state {
                    RTCPeerConnectionState::Connected => {
                        transport.forget_negotiation(connection_id).await;
                    }
                    RTCPeerConnectionState::Failed
                    | RTCPeerConnectionState::Disconnected
                    | RTCPeerConnectionState::Closed => {
                        transport.forget_negotiation(connection_id).await;
                        session.mark_closed();
                    }
                    _ => {}
                }
            })
        }));

        let negotiation = Arc::new(Negotiation {
            peer,
            address,
            network_id: sender_network_id,
            candidates: Mutex::new(Some(Vec::new())),
            assert_identity,
            started: Instant::now(),
        });
        self.negotiations
            .lock()
            .await
            .insert(connection_id, negotiation.clone());

        tokio::spawn(async move {
            if let Err(error) = self.negotiate(&negotiation, connection_id, offer).await {
                warn!("NetherNet negotiation {connection_id} with {address} failed: {error}");
                self.send_signal(
                    &Signal::new(signal::TYPE_ERROR, connection_id, error),
                    sender_network_id,
                    address,
                )
                .await;
                self.drop_negotiation(connection_id).await;
            }
        });
    }

    async fn negotiate(
        &self,
        negotiation: &Negotiation,
        connection_id: u64,
        offer: String,
    ) -> Result<(), String> {
        let peer = &negotiation.peer;
        let offer = RTCSessionDescription::offer(offer).map_err(|error| error.to_string())?;
        peer.set_remote_description(offer)
            .await
            .map_err(|error| error.to_string())?;

        let buffered = negotiation
            .candidates
            .lock()
            .await
            .take()
            .unwrap_or_default();
        for candidate in buffered {
            add_candidate(peer, candidate).await;
        }

        let answer = peer
            .create_answer(None)
            .await
            .map_err(|error| error.to_string())?;
        let mut gathering_complete = peer.gathering_complete_promise().await;
        peer.set_local_description(answer)
            .await
            .map_err(|error| error.to_string())?;
        tokio::time::timeout(GATHERING_TIMEOUT, gathering_complete.recv())
            .await
            .map_err(|_| "timed out gathering ICE candidates".to_string())?;

        let answer = peer
            .local_description()
            .await
            .ok_or_else(|| "WebRTC did not produce a local description".to_string())?;

        // The identity assertion is over a kilobyte, which pushes the answer past the MTU and
        // makes it rely on IP fragmentation. Only send it back to clients that asserted one.
        let answer_sdp = if negotiation.assert_identity {
            add_server_identity(&answer.sdp, &self.identity_key)?
        } else {
            answer.sdp.clone()
        };
        self.send_signal(
            &Signal::new(signal::TYPE_ANSWER, connection_id, answer_sdp),
            negotiation.network_id,
            negotiation.address,
        )
        .await;

        for candidate in local_candidates(&answer.sdp) {
            self.send_signal(
                &Signal::new(signal::TYPE_CANDIDATE, connection_id, candidate),
                negotiation.network_id,
                negotiation.address,
            )
            .await;
        }
        Ok(())
    }

    async fn handle_candidate(&self, signal: Signal) {
        let Some(negotiation) = self
            .negotiations
            .lock()
            .await
            .get(&signal.connection_id)
            .cloned()
        else {
            return;
        };

        let mut buffered = negotiation.candidates.lock().await;
        if let Some(candidates) = buffered.as_mut() {
            candidates.push(signal.data);
            return;
        }
        drop(buffered);
        add_candidate(&negotiation.peer, signal.data).await;
    }

    async fn create_peer_connection(&self) -> Result<Arc<RTCPeerConnection>, String> {
        let mut media_engine = MediaEngine::default();
        media_engine
            .register_default_codecs()
            .map_err(|error| error.to_string())?;
        let api = APIBuilder::new().with_media_engine(media_engine).build();
        let peer = api
            .new_peer_connection(RTCConfiguration {
                ice_servers: (!self.stun_servers.is_empty())
                    .then(|| RTCIceServer {
                        urls: self.stun_servers.to_vec(),
                        ..Default::default()
                    })
                    .into_iter()
                    .collect(),
                ..Default::default()
            })
            .await
            .map_err(|error| error.to_string())?;
        Ok(Arc::new(peer))
    }

    async fn server_data(&self) -> Vec<u8> {
        let config = &self.server.advanced_config.networking.bedrock;
        let player_count = self
            .server
            .get_status()
            .lock()
            .await
            .status_response
            .players
            .as_ref()
            .map_or(0, |players| players.online) as i32;
        let gamemode = self.server.defaultgamemode.lock().await.gamemode;
        let game_type = match gamemode {
            GameMode::Creative => 1,
            GameMode::Adventure => 2,
            _ => 0,
        };

        ServerData {
            server_name: &config.motd,
            level_name: &self.server.basic_config.default_level_name,
            game_type,
            player_count,
            max_player_count: config.max_players as i32,
            editor_world: false,
            hardcore: self.server.basic_config.hardcore,
            accepts_online_auth: config.online_mode,
            accepts_self_signed_auth: !config.online_mode,
            world_id: &self.world_id,
            transport_layer: TRANSPORT_LAYER_NETHERNET,
            connection_type: CONNECTION_TYPE_LAN_SIGNALING,
        }
        .encode()
    }

    async fn send_signal(&self, signal: &Signal, recipient_id: u64, address: SocketAddr) {
        self.send_packet(
            &DiscoveryPacket::Message {
                recipient_id,
                data: signal.to_string(),
            },
            address,
        )
        .await;
    }

    async fn send_packet(&self, packet: &DiscoveryPacket, address: SocketAddr) {
        let datagram = discovery::marshal(packet, self.network_id);
        if let Err(error) = self.socket.send_to(&datagram, address).await {
            debug!("Failed to send a NetherNet datagram to {address}: {error}");
        }
    }

    async fn forget_negotiation(&self, connection_id: u64) {
        self.negotiations.lock().await.remove(&connection_id);
    }

    async fn drop_negotiation(&self, connection_id: u64) {
        let negotiation = self.negotiations.lock().await.remove(&connection_id);
        if let Some(negotiation) = negotiation {
            let _ = negotiation.peer.close().await;
        }
    }

    async fn expire_negotiations(&self) {
        let expired = {
            let mut negotiations = self.negotiations.lock().await;
            let now = Instant::now();
            let mut expired = Vec::new();
            negotiations.retain(|_, negotiation| {
                if now.duration_since(negotiation.started) >= NEGOTIATION_TIMEOUT {
                    expired.push(negotiation.clone());
                    return false;
                }
                true
            });
            expired
        };
        for negotiation in expired {
            debug!(
                "Dropping stale NetherNet negotiation with {}",
                negotiation.address
            );
            let _ = negotiation.peer.close().await;
        }
    }
}

async fn add_candidate(peer: &RTCPeerConnection, candidate: String) {
    let init = RTCIceCandidateInit {
        candidate,
        ..Default::default()
    };
    if let Err(error) = peer.add_ice_candidate(init).await {
        debug!("Ignoring invalid NetherNet ICE candidate: {error}");
    }
}

fn local_candidates(sdp: &str) -> Vec<String> {
    sdp.lines()
        .filter_map(|line| line.trim_end().strip_prefix("a="))
        .filter(|line| line.starts_with("candidate:"))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_candidates_are_reported_without_their_attribute_prefix() {
        let sdp =
            "v=0\r\na=candidate:1 1 udp 2 127.0.0.1 50000 typ host\r\na=fingerprint:sha-256 AA\r\n";
        assert_eq!(
            local_candidates(sdp),
            vec!["candidate:1 1 udp 2 127.0.0.1 50000 typ host".to_string()]
        );
    }
}
