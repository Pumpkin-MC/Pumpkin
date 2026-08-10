use std::{
    fs::OpenOptions,
    io::{ErrorKind, Write},
    net::{IpAddr, SocketAddr},
    path::Path as FsPath,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{
    Router,
    body::Bytes,
    extract::{ConnectInfo, DefaultBodyLimit, Path, State},
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose};
use pumpkin_util::jwt::Jwks;
use pumpkin_util::p384::{
    PublicKey,
    ecdsa::{
        Signature, SigningKey,
        signature::{Signer, Verifier},
    },
    pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey},
};
use serde_json::{Value, json};
use tokio::{
    net::TcpListener,
    sync::{Mutex, RwLock, mpsc},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, trace, warn};
use webrtc::{
    api::{API, APIBuilder, media_engine::MediaEngine, setting_engine::SettingEngine},
    data_channel::RTCDataChannel,
    ice::{
        network_type::NetworkType,
        udp_mux::{UDPMuxDefault, UDPMuxParams},
        udp_network::UDPNetwork,
    },
    ice_transport::ice_candidate::RTCIceCandidateInit,
    ice_transport::{ice_candidate_type::RTCIceCandidateType, ice_server::RTCIceServer},
    peer_connection::{
        RTCPeerConnection, configuration::RTCConfiguration,
        peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
};

use crate::STOP_INTERRUPT;
use crate::net::bedrock::status::IceSocket;

pub mod discovery;

const RELIABLE_CHANNEL: &str = "ReliableDataChannel";
const UNRELIABLE_CHANNEL: &str = "UnreliableDataChannel";
// NetherNet splits encoded packets that exceed 10,000 bytes into application-level
// segments. Larger SCTP messages are rejected by some Bedrock clients.
const MAX_FRAGMENT_SIZE: usize = 10_000;
// Bedrock may send its login batch as one maximum-sized NetherNet segment. This
// exceeds webrtc-rs's 65,535-byte callback buffer when the skin data is large.
const MAX_INBOUND_MESSAGE_SIZE: usize = 262_144;
const MAX_SDP_SIZE: usize = 1 << 20;

type IncomingSession = (Arc<NetherNetSession>, SocketAddr);

/// Accepts Bedrock `NetherNet` connections negotiated through Mojang's HTTP endpoint.
pub struct NetherNetListener {
    incoming: Mutex<mpsc::Receiver<IncomingSession>>,
    local_addr: SocketAddr,
    state: EndpointState,
}

#[derive(Clone)]
struct EndpointState {
    incoming: mpsc::Sender<IncomingSession>,
    api: Arc<API>,
    identity_key: Arc<SigningKey>,
    require_client_identity: bool,
    oidc_verifier: Option<Arc<(String, Jwks)>>,
    stun_servers: Arc<[String]>,
}

impl NetherNetListener {
    pub async fn bind(
        address: SocketAddr,
        ice_socket: IceSocket,
        external_ip: Option<IpAddr>,
        identity_key: Arc<SigningKey>,
        require_client_identity: bool,
        oidc_verifier: Option<Arc<(String, Jwks)>>,
        stun_servers: Vec<String>,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind(address).await?;
        let local_addr = listener.local_addr()?;
        let ice_local_addr = ice_socket.local_addr()?;
        let (incoming, receiver) = mpsc::channel(128);
        let state = EndpointState {
            incoming,
            api: Arc::new(build_api(ice_socket, external_ip)?),
            identity_key,
            require_client_identity,
            oidc_verifier,
            stun_servers: stun_servers.into(),
        };
        let router = Router::new()
            .route("/v1/join", get(ping))
            .route("/v1/join/{network_id}", post(join))
            .layer(DefaultBodyLimit::max(MAX_SDP_SIZE))
            .with_state(state.clone());

        tokio::spawn(async move {
            let result = axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(STOP_INTERRUPT.clone().cancelled_owned())
            .await;
            if let Err(error) = result {
                warn!("NetherNet signaling server stopped: {error}");
            }
        });

        info!("Bedrock NetherNet signaling is listening on {local_addr}");
        info!("Bedrock NetherNet ICE is listening on {ice_local_addr}");
        Ok(Self {
            incoming: Mutex::new(receiver),
            local_addr,
            state,
        })
    }

    pub async fn accept(&self) -> Option<IncomingSession> {
        self.incoming.lock().await.recv().await
    }

    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

fn build_api<C>(ice_socket: C, external_ip: Option<IpAddr>) -> std::io::Result<API>
where
    C: webrtc::util::Conn + Send + Sync + 'static,
{
    let ice_ip = webrtc::util::Conn::local_addr(&ice_socket)
        .map_err(|error| std::io::Error::other(error.to_string()))?
        .ip();
    if external_ip.is_some_and(|external_ip| external_ip.is_ipv4() != ice_ip.is_ipv4()) {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "NetherNet external IP and ICE address must use the same address family",
        ));
    }
    let mut media_engine = MediaEngine::default();
    media_engine
        .register_default_codecs()
        .map_err(|error| std::io::Error::other(error.to_string()))?;

    let udp_mux = UDPMuxDefault::new(UDPMuxParams::new(ice_socket));
    let mut setting_engine = SettingEngine::default();
    setting_engine.detach_data_channels();
    setting_engine.set_udp_network(UDPNetwork::Muxed(udp_mux));
    setting_engine.set_network_types(vec![if ice_ip.is_ipv4() {
        NetworkType::Udp4
    } else {
        NetworkType::Udp6
    }]);
    if let Some(external_ip) = external_ip {
        let selected_ip = OnceLock::new();
        setting_engine.set_ip_filter(Box::new(move |ip| selected_ip.get_or_init(|| ip) == &ip));
        setting_engine.set_nat_1to1_ips(vec![external_ip.to_string()], RTCIceCandidateType::Host);
    }

    Ok(APIBuilder::new()
        .with_media_engine(media_engine)
        .with_setting_engine(setting_engine)
        .build())
}

pub fn load_or_create_identity_key(path: &FsPath) -> std::io::Result<Arc<SigningKey>> {
    loop {
        match std::fs::read(path) {
            Ok(bytes) => {
                let key = SigningKey::from_pkcs8_der(&bytes).map_err(|error| {
                    std::io::Error::new(
                        ErrorKind::InvalidData,
                        format!("invalid NetherNet identity key: {error}"),
                    )
                })?;
                return Ok(Arc::new(key));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let key = loop {
                    let bytes = rand::random::<[u8; 48]>();
                    if let Ok(key) = SigningKey::from_slice(&bytes) {
                        break key;
                    }
                };
                let document = key
                    .to_pkcs8_der()
                    .map_err(|error| std::io::Error::other(error.to_string()))?;
                let mut options = OpenOptions::new();
                options.write(true).create_new(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600)
                };
                match options.open(path) {
                    Ok(mut file) => {
                        file.write_all(document.as_bytes())?;
                        file.sync_all()?;
                        return Ok(Arc::new(key));
                    }
                    Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(error),
                }
            }
            Err(error) => return Err(error),
        }
    }
}

async fn ping(ConnectInfo(address): ConnectInfo<SocketAddr>) -> StatusCode {
    trace!(%address, "Accepted NetherNet capability probe");
    StatusCode::OK
}

async fn join(
    State(state): State<EndpointState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Path(network_id): Path<String>,
    offer: Bytes,
) -> Response {
    trace!(%address, %network_id, length = offer.len(), "Received NetherNet SDP offer");
    if offer.is_empty() {
        debug!(%address, %network_id, "Rejected empty NetherNet SDP offer");
        return (StatusCode::BAD_REQUEST, "Missing SDP offer").into_response();
    }
    let Ok(offer) = String::from_utf8(offer.to_vec()) else {
        debug!(%address, %network_id, "Rejected non-UTF-8 NetherNet SDP offer");
        return (StatusCode::BAD_REQUEST, "SDP offer must be UTF-8").into_response();
    };

    match negotiate(&state, address, &offer, None).await {
        Ok((answer, _session)) => {
            trace!(%address, %network_id, length = answer.len(), "Returning NetherNet SDP answer");
            let mut response = (StatusCode::OK, answer).into_response();
            response
                .headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static("application/sdp"));
            response
        }
        Err(error) => {
            warn!("NetherNet negotiation with {address} failed: {error}");
            (StatusCode::BAD_REQUEST, error).into_response()
        }
    }
}

async fn negotiate(
    state: &EndpointState,
    address: SocketAddr,
    offer: &str,
    candidates: Option<mpsc::UnboundedReceiver<RTCIceCandidateInit>>,
) -> Result<(String, Arc<NetherNetSession>), String> {
    let signaling = if candidates.is_some() { "LAN" } else { "HTTP" };
    trace!(%address, signaling, "Starting NetherNet negotiation");
    let (offer, client_public_key) = authenticate_client_offer(
        offer,
        state.require_client_identity,
        state.oidc_verifier.as_deref(),
    )?;
    trace!(
        %address,
        signaling,
        authenticated = client_public_key.is_some(),
        candidates = ?candidate_summary(&offer),
        "Received NetherNet ICE candidates"
    );

    let peer = Arc::new(
        state
            .api
            .new_peer_connection(RTCConfiguration {
                ice_servers: (!state.stun_servers.is_empty())
                    .then(|| RTCIceServer {
                        urls: state.stun_servers.to_vec(),
                        ..Default::default()
                    })
                    .into_iter()
                    .collect(),
                ..Default::default()
            })
            .await
            .map_err(|error| error.to_string())?,
    );
    let session = Arc::new(NetherNetSession::new(
        peer.clone(),
        client_public_key,
        address,
        state.incoming.clone(),
    ));
    register_peer_callbacks(&peer, &session, address);

    let offer = RTCSessionDescription::offer(offer).map_err(|error| error.to_string())?;
    peer.set_remote_description(offer)
        .await
        .map_err(|error| error.to_string())?;
    trace!(%address, signaling, "Applied NetherNet remote description");
    if let Some(mut candidates) = candidates {
        let peer = peer.clone();
        tokio::spawn(async move {
            while let Some(candidate) = candidates.recv().await {
                if let Err(error) = peer.add_ice_candidate(candidate).await {
                    debug!("Failed to add NetherNet LAN ICE candidate: {error}");
                }
            }
        });
    }
    let answer = peer
        .create_answer(None)
        .await
        .map_err(|error| error.to_string())?;
    let mut gathering_complete = peer.gathering_complete_promise().await;
    peer.set_local_description(answer)
        .await
        .map_err(|error| error.to_string())?;
    trace!(%address, signaling, "Gathering NetherNet ICE candidates");
    tokio::time::timeout(Duration::from_secs(10), gathering_complete.recv())
        .await
        .map_err(|_| "Timed out gathering ICE candidates".to_string())?;
    let answer = peer
        .local_description()
        .await
        .ok_or_else(|| "WebRTC did not produce a local description".to_string())?;
    let answer = remove_component_two_candidates(&answer.sdp);
    trace!(
        %address,
        signaling,
        candidates = ?candidate_summary(&answer),
        "Gathered NetherNet ICE candidates"
    );
    trace!(%address, signaling, "Completed NetherNet negotiation");
    Ok((add_server_identity(&answer, &state.identity_key)?, session))
}

fn register_peer_callbacks(
    peer: &RTCPeerConnection,
    session: &Arc<NetherNetSession>,
    address: SocketAddr,
) {
    let session_for_channels = session.clone();
    peer.on_data_channel(Box::new(move |channel| {
        let session = session_for_channels.clone();
        Box::pin(async move {
            trace!(
                %address,
                label = channel.label(),
                ordered = channel.ordered(),
                negotiated = channel.negotiated(),
                max_retransmits = ?channel.max_retransmits(),
                "Received NetherNet data channel"
            );
            if let Err(error) = session.attach_channel(channel).await {
                warn!("Rejected NetherNet data channel: {error}");
                session.close().await;
            }
        })
    }));

    let session_for_state = session.clone();
    peer.on_peer_connection_state_change(Box::new(move |connection_state| {
        let session = session_for_state.clone();
        Box::pin(async move {
            trace!(?connection_state, %address, "NetherNet peer connection state changed");
            if matches!(
                connection_state,
                RTCPeerConnectionState::Failed
                    | RTCPeerConnectionState::Disconnected
                    | RTCPeerConnectionState::Closed
            ) {
                session.mark_closed();
            }
        })
    }));

    peer.on_ice_connection_state_change(Box::new(move |connection_state| {
        Box::pin(async move {
            trace!(?connection_state, %address, "NetherNet ICE connection state changed");
        })
    }));
}

fn candidate_summary(sdp: &str) -> Vec<String> {
    sdp.lines()
        .filter_map(|line| line.strip_prefix("a=candidate:"))
        .map(|candidate| {
            let fields = candidate.split_whitespace().collect::<Vec<_>>();
            match fields.as_slice() {
                [
                    foundation,
                    component,
                    protocol,
                    _,
                    address,
                    port,
                    "typ",
                    kind,
                    ..,
                ] => {
                    format!("{foundation}/{component} {protocol} {address}:{port} {kind}")
                }
                _ => "malformed candidate".to_owned(),
            }
        })
        .collect()
}

fn remove_component_two_candidates(sdp: &str) -> String {
    let mut filtered = String::with_capacity(sdp.len());
    for line in sdp.lines().filter(|line| {
        line.strip_prefix("a=candidate:")
            .and_then(|candidate| candidate.split_whitespace().nth(1))
            != Some("2")
    }) {
        filtered.push_str(line);
        filtered.push_str("\r\n");
    }
    filtered
}

/// A WebRTC connection carrying complete Bedrock batch packets.
pub struct NetherNetSession {
    #[allow(dead_code)]
    peer: Arc<RTCPeerConnection>,
    reliable: RwLock<Option<Arc<RTCDataChannel>>>,
    unreliable: RwLock<Option<Arc<RTCDataChannel>>>,
    fragments: Mutex<FragmentBuffer>,
    packets: Mutex<mpsc::Receiver<Bytes>>,
    packet_sender: mpsc::Sender<Bytes>,
    open_channels: AtomicU8,
    accepted: AtomicBool,
    closed: CancellationToken,
    client_public_key: Option<PublicKey>,
    address: SocketAddr,
    incoming: mpsc::Sender<IncomingSession>,
}

impl NetherNetSession {
    fn new(
        peer: Arc<RTCPeerConnection>,
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
        }
    }

    async fn attach_channel(self: &Arc<Self>, channel: Arc<RTCDataChannel>) -> Result<(), String> {
        let has_default_parameters = channel.protocol().is_empty()
            && !channel.negotiated()
            && channel.max_packet_lifetime().is_none();
        let bit = match channel.label() {
            RELIABLE_CHANNEL
                if channel.ordered()
                    && has_default_parameters
                    && channel.max_retransmits().is_none() =>
            {
                *self.reliable.write().await = Some(channel.clone());
                1
            }
            UNRELIABLE_CHANNEL
                if !channel.ordered()
                    && has_default_parameters
                    && channel.max_retransmits() == Some(0) =>
            {
                *self.unreliable.write().await = Some(channel.clone());
                2
            }
            label => return Err(format!("invalid channel {label:?}")),
        };

        let session = self.clone();
        let channel_for_open = channel.clone();
        channel.on_open(Box::new(move || {
            Box::pin(async move {
                let detached = match channel_for_open.detach().await {
                    Ok(channel) => channel,
                    Err(error) => {
                        warn!(%error, address = %session.address, "Failed to detach NetherNet data channel");
                        session.close().await;
                        return;
                    }
                };
                session.channel_opened(bit).await;
                tokio::spawn(async move {
                    let mut buffer = vec![0; MAX_INBOUND_MESSAGE_SIZE];
                    loop {
                        match detached.read_data_channel(&mut buffer).await {
                            Ok((0, _)) => break,
                            Ok((length, _)) => {
                                if let Err(error) = session
                                    .receive_segment(
                                        bit,
                                        Bytes::copy_from_slice(&buffer[..length]),
                                    )
                                    .await
                                {
                                    warn!(
                                        "Invalid NetherNet message from {}: {error}",
                                        session.address
                                    );
                                    break;
                                }
                            }
                            Err(error) => {
                                warn!(%error, address = %session.address, "Failed to read NetherNet data channel");
                                break;
                            }
                        }
                    }
                    session.close().await;
                });
            })
        }));
        Ok(())
    }

    async fn channel_opened(self: &Arc<Self>, bit: u8) {
        let open = self.open_channels.fetch_or(bit, Ordering::AcqRel) | bit;
        trace!(
            address = %self.address,
            channel = if bit == 1 { "reliable" } else { "unreliable" },
            both_open = open == 3,
            "NetherNet data channel opened"
        );
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

    pub const fn client_public_key(&self) -> Option<&PublicKey> {
        self.client_public_key.as_ref()
    }

    pub fn is_closed(&self) -> bool {
        self.closed.is_cancelled()
    }

    fn mark_closed(&self) {
        if !self.closed.is_cancelled() {
            trace!(address = %self.address, "NetherNet session closed");
            self.closed.cancel();
        }
    }

    #[allow(clippy::unused_async)]
    pub async fn close(&self) {
        if self.closed.is_cancelled() {
            return;
        }
        self.closed.cancel();
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

fn verify_and_strip_identity(
    offer: &str,
    oidc_verifier: Option<&(String, Jwks)>,
) -> Result<(String, PublicKey), String> {
    let identity = offer
        .lines()
        .find_map(|line| line.strip_prefix("a=identity:"))
        .ok_or_else(|| "SDP offer is missing its identity assertion".to_string())?;
    let identity = general_purpose::STANDARD
        .decode(identity)
        .map_err(|error| format!("invalid identity encoding: {error}"))?;
    let identity: Value = serde_json::from_slice(&identity)
        .map_err(|error| format!("invalid identity JSON: {error}"))?;
    let assertion = identity["assertion"]
        .as_str()
        .ok_or_else(|| "identity assertion is missing".to_string())?;
    let assertion: Value = serde_json::from_str(assertion)
        .map_err(|error| format!("invalid nested identity assertion: {error}"))?;
    let token = assertion["token"]
        .as_str()
        .ok_or_else(|| "identity token is missing".to_string())?;
    if let Some((issuer, keys)) = oidc_verifier {
        if identity["idp"]["protocol"] != "default"
            || identity["idp"]["domain"].as_str().is_none_or(str::is_empty)
        {
            return Err("invalid identity provider".to_string());
        }
        pumpkin_util::jwt::verify_oidc_token(token, issuer, keys)
            .map_err(|error| format!("invalid GameServerToken: {error}"))?;
    } else {
        validate_token_expiration(token)?;
    }
    let public_key = pumpkin_util::jwt::extract_cpk_from_token(token)
        .map_err(|error| format!("invalid identity public key: {error}"))?;
    let fingerprints = assertion["fingerprints"]
        .as_str()
        .ok_or_else(|| "fingerprint assertion is missing".to_string())?;
    verify_fingerprint_assertion(fingerprints, offer, &public_key)?;

    let mut stripped = offer
        .lines()
        .filter(|line| !line.starts_with("a=identity:"))
        .collect::<Vec<_>>()
        .join("\r\n");
    stripped.push_str("\r\n");
    Ok((stripped, public_key))
}

fn authenticate_client_offer(
    offer: &str,
    require_identity: bool,
    oidc_verifier: Option<&(String, Jwks)>,
) -> Result<(String, Option<PublicKey>), String> {
    if offer.lines().any(|line| line.starts_with("a=identity:")) {
        let (offer, public_key) = verify_and_strip_identity(offer, oidc_verifier)?;
        return Ok((offer, Some(public_key)));
    }
    if require_identity {
        return Err("SDP offer is missing its identity assertion".to_string());
    }
    Ok((offer.to_owned(), None))
}

fn validate_token_expiration(token: &str) -> Result<(), String> {
    let payload = token
        .split('.')
        .nth(1)
        .ok_or_else(|| "malformed identity token".to_string())?;
    let payload = general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|error| format!("invalid identity token payload: {error}"))?;
    let payload: Value = serde_json::from_slice(&payload)
        .map_err(|error| format!("invalid identity token claims: {error}"))?;
    let expiration = payload["exp"]
        .as_i64()
        .ok_or_else(|| "identity token has no expiration".to_string())?;
    if expiration < unix_time() {
        return Err("identity token has expired".to_string());
    }
    Ok(())
}

fn verify_fingerprint_assertion(
    assertion: &str,
    sdp: &str,
    public_key: &PublicKey,
) -> Result<(), String> {
    let mut parts = assertion.split('.');
    let header = parts.next().ok_or_else(|| "malformed JWS".to_string())?;
    let detached = parts.next().ok_or_else(|| "malformed JWS".to_string())?;
    let signature = parts.next().ok_or_else(|| "malformed JWS".to_string())?;
    if !detached.is_empty() || parts.next().is_some() {
        return Err("fingerprint assertion is not a detached JWS".to_string());
    }
    let header_json = general_purpose::URL_SAFE_NO_PAD
        .decode(header)
        .map_err(|error| format!("invalid fingerprint header: {error}"))?;
    let header_json: Value = serde_json::from_slice(&header_json)
        .map_err(|error| format!("invalid fingerprint header: {error}"))?;
    if header_json["alg"] != "ES384" {
        return Err("fingerprint assertion must use ES384".to_string());
    }
    let payload = fingerprint_payload(sdp)?;
    let payload_b64 = general_purpose::URL_SAFE_NO_PAD.encode(payload);
    let signature = general_purpose::URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|error| format!("invalid fingerprint signature: {error}"))?;
    let signature = Signature::from_slice(&signature)
        .map_err(|error| format!("invalid ES384 signature: {error}"))?;
    let verifying_key = pumpkin_util::p384::ecdsa::VerifyingKey::from(public_key);
    verifying_key
        .verify(format!("{header}.{payload_b64}").as_bytes(), &signature)
        .map_err(|_| "fingerprint signature verification failed".to_string())
}

fn add_server_identity(sdp: &str, key: &SigningKey) -> Result<String, String> {
    let public_key = PublicKey::from(key.verifying_key());
    let public_der = public_key
        .to_public_key_der()
        .map_err(|error| error.to_string())?;
    let public_key = general_purpose::STANDARD.encode(public_der.as_bytes());
    let now = unix_time();
    let token = sign_jws(
        key,
        &json!({"alg": "ES384", "x5u": public_key}),
        &json!({"exp": now + 60, "iat": now, "cpk": public_key}),
    )?;
    let fingerprints = sign_detached(key, fingerprint_payload(sdp)?);
    let assertion = serde_json::to_string(&json!({
        "fingerprints": fingerprints,
        "token": token,
    }))
    .map_err(|error| error.to_string())?;
    let identity = json!({
        "assertion": assertion,
        "idp": {"domain": "self", "protocol": "default"},
    });
    let identity = general_purpose::STANDARD
        .encode(serde_json::to_vec(&identity).map_err(|error| error.to_string())?);

    let marker = "m=application";
    let position = sdp
        .find(marker)
        .ok_or_else(|| "answer SDP has no application section".to_string())?;
    let mut answer = String::with_capacity(sdp.len() + identity.len() + 14);
    answer.push_str(&sdp[..position]);
    answer.push_str("a=identity:");
    answer.push_str(&identity);
    answer.push_str("\r\n");
    answer.push_str(&sdp[position..]);
    Ok(answer)
}

fn fingerprint_payload(sdp: &str) -> Result<Vec<u8>, String> {
    let fingerprints = sdp
        .lines()
        .filter_map(|line| line.strip_prefix("a=fingerprint:"))
        .map(|fingerprint| {
            let (algorithm, digest) = fingerprint
                .split_once(' ')
                .ok_or_else(|| "malformed DTLS fingerprint".to_string())?;
            Ok(json!({"algorithm": algorithm, "digest": digest}))
        })
        .collect::<Result<Vec<_>, String>>()?;
    if fingerprints.is_empty() {
        return Err("SDP has no DTLS fingerprint".to_string());
    }
    serde_json::to_vec(&json!({
        "fingerprint": fingerprints,
    }))
    .map_err(|error| error.to_string())
}

fn sign_jws(key: &SigningKey, header: &Value, payload: &Value) -> Result<String, String> {
    let header = general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(header).map_err(|error| error.to_string())?);
    let payload = general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(payload).map_err(|error| error.to_string())?);
    let input = format!("{header}.{payload}");
    let signature: Signature = key.sign(input.as_bytes());
    Ok(format!(
        "{input}.{}",
        general_purpose::URL_SAFE_NO_PAD.encode(signature.to_bytes())
    ))
}

fn sign_detached(key: &SigningKey, payload: Vec<u8>) -> String {
    let header = general_purpose::URL_SAFE_NO_PAD.encode(b"{\"alg\":\"ES384\"}");
    let payload = general_purpose::URL_SAFE_NO_PAD.encode(payload);
    let signature: Signature = key.sign(format!("{header}.{payload}").as_bytes());
    format!(
        "{header}..{}",
        general_purpose::URL_SAFE_NO_PAD.encode(signature.to_bytes())
    )
}

fn unix_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::UdpSocket;
    use webrtc::{
        api::setting_engine::SctpMaxMessageSize,
        data_channel::data_channel_init::RTCDataChannelInit,
    };

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

    #[test]
    fn server_identity_is_valid_and_verifiable() {
        let key = SigningKey::from_slice(&[7; 48]).unwrap();
        let sdp = "v=0\r\nt=0 0\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\na=fingerprint:sha-256 AA:BB\r\n";
        let answer = add_server_identity(sdp, &key).unwrap();
        let (_, public_key) = verify_and_strip_identity(&answer, None).unwrap();
        assert_eq!(public_key, PublicKey::from(key.verifying_key()));
    }

    #[test]
    fn fingerprint_payload_contains_every_sdp_fingerprint() {
        let sdp = "v=0\r\na=fingerprint:sha-256 AA:BB\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\na=fingerprint:sha-384 CC:DD\r\n";
        assert_eq!(
            fingerprint_payload(sdp).unwrap(),
            br#"{"fingerprint":[{"algorithm":"sha-256","digest":"AA:BB"},{"algorithm":"sha-384","digest":"CC:DD"}]}"#,
        );
    }

    #[test]
    fn data_channel_sdp_only_advertises_component_one() {
        let sdp = "v=0\r\na=candidate:1 1 udp 1 192.0.2.1 19134 typ host\r\na=candidate:1 2 udp 1 192.0.2.1 19134 typ host\r\na=end-of-candidates\r\n";
        assert_eq!(
            remove_component_two_candidates(sdp),
            "v=0\r\na=candidate:1 1 udp 1 192.0.2.1 19134 typ host\r\na=end-of-candidates\r\n"
        );
    }

    #[test]
    fn summarizes_candidates_without_credentials() {
        let sdp = "a=ice-ufrag:secret\r\na=ice-pwd:also-secret\r\n\
                   a=candidate:123 1 udp 2130706431 192.0.2.1 19132 typ host\r\n";
        assert_eq!(candidate_summary(sdp), ["123/1 udp 192.0.2.1:19132 host"]);
    }

    #[test]
    fn configured_oidc_validation_rejects_untrusted_identity_tokens() {
        let key = SigningKey::from_slice(&[7; 48]).unwrap();
        let sdp = "v=0\r\nt=0 0\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\na=fingerprint:sha-256 AA:BB\r\n";
        let answer = add_server_identity(sdp, &key).unwrap();
        let verifier = (
            "https://issuer.example".to_string(),
            Jwks { keys: Vec::new() },
        );
        assert!(verify_and_strip_identity(&answer, Some(&verifier)).is_err());
    }

    #[test]
    fn offline_mode_accepts_an_unverified_identity_provider() {
        let key = SigningKey::from_slice(&[7; 48]).unwrap();
        let sdp = "v=0\r\nt=0 0\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\na=fingerprint:sha-256 AA:BB\r\n";
        let answer = add_server_identity(sdp, &key).unwrap();
        let encoded = answer
            .lines()
            .find_map(|line| line.strip_prefix("a=identity:"))
            .unwrap();
        let mut identity: Value =
            serde_json::from_slice(&general_purpose::STANDARD.decode(encoded).unwrap()).unwrap();
        identity["idp"] = json!({"domain": "", "protocol": "offline"});
        let offline_identity =
            general_purpose::STANDARD.encode(serde_json::to_vec(&identity).unwrap());
        let offer = answer.replace(encoded, &offline_identity);

        verify_and_strip_identity(&offer, None).unwrap();
        let verifier = (
            "https://issuer.example".to_string(),
            Jwks { keys: Vec::new() },
        );
        assert_eq!(
            verify_and_strip_identity(&offer, Some(&verifier)).unwrap_err(),
            "invalid identity provider"
        );
    }

    #[test]
    fn offline_mode_accepts_an_offer_without_identity() {
        let offer = "v=0\r\nt=0 0\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n";
        let (offer, public_key) = authenticate_client_offer(offer, false, None).unwrap();
        assert_eq!(
            offer,
            "v=0\r\nt=0 0\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n"
        );
        assert!(public_key.is_none());
    }

    #[test]
    fn online_mode_rejects_an_offer_without_identity() {
        let error = authenticate_client_offer("v=0\r\n", true, None).unwrap_err();
        assert_eq!(error, "SDP offer is missing its identity assertion");
    }

    async fn receive_packet(session: &NetherNetSession) -> Bytes {
        tokio::time::timeout(Duration::from_secs(5), session.recv())
            .await
            .unwrap()
            .unwrap()
    }

    async fn receive_bytes(receiver: &mut mpsc::Receiver<Bytes>) -> Bytes {
        tokio::time::timeout(Duration::from_secs(5), receiver.recv())
            .await
            .unwrap()
            .unwrap()
    }

    fn test_client_api() -> API {
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs().unwrap();
        let mut setting_engine = SettingEngine::default();
        setting_engine.set_sctp_max_message_size_can_send(SctpMaxMessageSize::Unbounded);
        APIBuilder::new()
            .with_media_engine(media_engine)
            .with_setting_engine(setting_engine)
            .build()
    }

    #[tokio::test]
    async fn negotiates_channels_and_receives_a_packet() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
        let client = Arc::new(
            test_client_api()
                .new_peer_connection(RTCConfiguration::default())
                .await
                .unwrap(),
        );
        let reliable = client
            .create_data_channel(
                RELIABLE_CHANNEL,
                Some(RTCDataChannelInit {
                    ordered: Some(true),
                    ..Default::default()
                }),
            )
            .await
            .unwrap();
        let unreliable = client
            .create_data_channel(
                UNRELIABLE_CHANNEL,
                Some(RTCDataChannelInit {
                    ordered: Some(false),
                    max_retransmits: Some(0),
                    ..Default::default()
                }),
            )
            .await
            .unwrap();
        let (unreliable_sender, mut unreliable_receiver) = mpsc::channel(1);
        unreliable.on_message(Box::new(move |message| {
            let sender = unreliable_sender.clone();
            Box::pin(async move {
                let _ = sender.send(message.data).await;
            })
        }));
        let offer = client.create_offer(None).await.unwrap();
        let mut gathering_complete = client.gathering_complete_promise().await;
        client.set_local_description(offer).await.unwrap();
        gathering_complete.recv().await;
        let offer = client.local_description().await.unwrap();
        let client_key = SigningKey::from_slice(&[8; 48]).unwrap();
        let offer = add_server_identity(&offer.sdp, &client_key).unwrap();
        let (incoming, mut receiver) = mpsc::channel(1);
        let server_key = Arc::new(SigningKey::from_slice(&[9; 48]).unwrap());
        let ice_socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        let ice_port = ice_socket.local_addr().unwrap().port();
        let state = EndpointState {
            incoming,
            api: Arc::new(build_api(ice_socket, None).unwrap()),
            identity_key: server_key.clone(),
            require_client_identity: true,
            oidc_verifier: None,
            stun_servers: Arc::from([]),
        };
        let (answer, server_session) =
            negotiate(&state, "127.0.0.1:19132".parse().unwrap(), &offer, None)
                .await
                .unwrap();
        let (answer, public_key) = verify_and_strip_identity(&answer, None).unwrap();
        assert_eq!(public_key, PublicKey::from(server_key.verifying_key()));
        assert!(answer.contains(&format!(" {ice_port} typ host")));
        let answer = answer.replace(
            "a=sctp-port:5000\r\n",
            "a=sctp-port:5000\r\na=max-message-size:262144\r\n",
        );
        client
            .set_remote_description(RTCSessionDescription::answer(answer).unwrap())
            .await
            .unwrap();
        let Ok(Some((session, _))) =
            tokio::time::timeout(Duration::from_secs(5), receiver.recv()).await
        else {
            panic!(
                "connection did not open; client={:?}, server={:?}",
                client.connection_state(),
                server_session.peer.connection_state(),
            );
        };
        reliable
            .send(&Bytes::from_static(b"\0hello"))
            .await
            .unwrap();
        let packet = receive_packet(&session).await;
        assert_eq!(packet, b"hello".as_slice());
        let large_packet = vec![42; 100_000];
        let mut segment = Vec::with_capacity(large_packet.len() + 1);
        segment.push(0);
        segment.extend_from_slice(&large_packet);
        reliable.send(&Bytes::from(segment)).await.unwrap();
        let packet = receive_packet(&session).await;
        assert_eq!(packet, large_packet);
        session
            .send_unreliable(Bytes::from_static(b"world"))
            .await
            .unwrap();
        let packet = receive_bytes(&mut unreliable_receiver).await;
        assert_eq!(packet, b"\0world".as_slice());
        session.close().await;
        client.close().await.unwrap();
    }
}
