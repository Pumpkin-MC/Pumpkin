use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, trace, warn};
use webrtc::{
    data_channel::DataChannel,
    peer_connection::{
        PeerConnection, PeerConnectionBuilder, PeerConnectionEventHandler, RTCConfiguration,
        RTCConfigurationBuilder, RTCIceCandidateInit, RTCIceCandidateType, RTCIceConnectionState,
        RTCIceGatheringState, RTCIceServer, RTCPeerConnectionState, RTCSessionDescription,
        SettingEngine,
    },
};

use super::{
    ice_router::{proxy_answer, proxy_offer},
    identity::{add_server_identity, authenticate_client_offer},
    session::NetherNetSession,
    state::NetherNetState,
};

pub(super) async fn negotiate(
    state: &NetherNetState,
    address: SocketAddr,
    offer: &str,
    candidates: Option<mpsc::UnboundedReceiver<RTCIceCandidateInit>>,
) -> Result<(String, Arc<NetherNetSession>), String> {
    Box::pin(negotiate_inner(state, address, offer, candidates, None)).await
}

pub(super) async fn negotiate_direct(
    state: &NetherNetState,
    address: SocketAddr,
    offer: &str,
    advertised_ip: Option<IpAddr>,
) -> Result<(String, Arc<NetherNetSession>), String> {
    Box::pin(negotiate_inner(state, address, offer, None, advertised_ip)).await
}

#[expect(
    clippy::too_many_lines,
    reason = "Negotiation coordinates the complete peer lifecycle"
)]
async fn negotiate_inner(
    state: &NetherNetState,
    address: SocketAddr,
    offer: &str,
    candidates: Option<mpsc::UnboundedReceiver<RTCIceCandidateInit>>,
    advertised_ip: Option<IpAddr>,
) -> Result<(String, Arc<NetherNetSession>), String> {
    let signaling = if candidates.is_some() { "LAN" } else { "HTTP" };
    let direct_ip = candidates.is_none();
    let advertised_ip = state.external_ip.or(advertised_ip);
    let (offer, client_public_key) = {
        let offer = offer.to_string();
        let require_identity = direct_ip && state.require_client_identity;
        let oidc_verifier = if require_identity {
            Some(
                state
                    .oidc_verifier
                    .as_ref()
                    .and_then(|cell| cell.get())
                    .cloned()
                    .ok_or_else(|| "Bedrock authentication keys are not ready".to_owned())?,
            )
        } else {
            state
                .oidc_verifier
                .as_ref()
                .and_then(|cell| cell.get())
                .cloned()
        };
        let (tx, rx) = tokio::sync::oneshot::channel();
        rayon::spawn(move || {
            let res = authenticate_client_offer(&offer, require_identity, oidc_verifier.as_ref());
            let _ = tx.send(res);
        });
        rx.await.map_err(|_| "Task cancelled".to_string())?
    }?;

    let gathering_notify = Arc::new(tokio::sync::Notify::new());
    let handler = Arc::new(NetherNetEventHandler {
        session: Mutex::new(None),
        address,
        gathering_notify: gathering_notify.clone(),
    });

    let configuration = rtc_configuration(&state.stun_servers);

    let (offer, remote_candidates) = if direct_ip {
        proxy_offer(&offer, state.ice_router.internal_addr())
    } else {
        (offer, Vec::new())
    };
    let peer = Box::pin(build_peer(
        state,
        configuration,
        handler.clone(),
        direct_ip,
        advertised_ip,
    ))
    .await?;
    let session = Arc::new(NetherNetSession::new(
        peer.clone(),
        client_public_key,
        address,
        state.incoming.clone(),
    ));
    *handler.session.lock().await = Some(session.clone());

    let mut candidate_task = None;
    let result: Result<String, String> = async {
        let offer = RTCSessionDescription::offer(offer).map_err(|error| error.to_string())?;
        peer.set_remote_description(offer)
            .await
            .map_err(|error| error.to_string())?;
        if let Some(mut candidates) = candidates {
            let peer = peer.clone();
            candidate_task = Some(tokio::spawn(async move {
                while let Some(candidate) = candidates.recv().await {
                    if let Err(error) = peer.add_ice_candidate(candidate).await {
                        debug!("Failed to add NetherNet LAN ICE candidate: {error}");
                    }
                }
            }));
        }
        let answer = peer
            .create_answer(None)
            .await
            .map_err(|error| error.to_string())?;
        peer.set_local_description(answer)
            .await
            .map_err(|error| error.to_string())?;
        let _ = tokio::time::timeout(Duration::from_secs(2), gathering_notify.notified()).await;
        let answer = peer
            .local_description()
            .await
            .ok_or_else(|| "WebRTC did not produce a local description".to_string())?;
        let answer = remove_component_two_candidates(&answer.sdp);
        let answer = if direct_ip {
            let (answer, ufrag, internal) = proxy_answer(
                &answer,
                advertised_ip,
                state.ice_router.public_addr().port(),
            )?;
            session.set_ice_route(
                state
                    .ice_router
                    .register(ufrag, internal, remote_candidates),
            );
            answer
        } else {
            answer
        };
        trace!(%address, signaling, "Completed NetherNet negotiation");

        let identity_key = state.identity_key.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        rayon::spawn(move || {
            let res = add_server_identity(&answer, &identity_key);
            let _ = tx.send(res);
        });
        rx.await.map_err(|_| "Task cancelled".to_string())?
    }
    .await;

    match result {
        Ok(answer) => Ok((answer, session)),
        Err(error) => {
            if let Some(task) = candidate_task {
                task.abort();
            }
            *handler.session.lock().await = None;
            session.close().await;
            Err(error)
        }
    }
}

async fn build_peer(
    state: &NetherNetState,
    configuration: RTCConfiguration,
    handler: Arc<NetherNetEventHandler>,
    direct_ip: bool,
    advertised_ip: Option<IpAddr>,
) -> Result<Arc<dyn PeerConnection>, String> {
    let ice_bind_addr = if direct_ip {
        // Direct connections are relayed through the router's loopback socket.
        // Binding every interface would make the inner ICE source differ from
        // the loopback route registered after SDP generation.
        SocketAddr::new(state.ice_router.internal_addr().ip(), 0)
    } else {
        SocketAddr::new(state.ice_local_addr.ip(), 0)
    };
    let mut setting_engine = SettingEngine::default();
    if direct_ip && let Some(external_ip) = advertised_ip {
        setting_engine.set_nat_1to1_ips(vec![external_ip.to_string()], RTCIceCandidateType::Host);
    }
    Ok(Arc::new(
        Box::pin(
            PeerConnectionBuilder::new()
                .with_configuration(configuration)
                .with_setting_engine(setting_engine)
                .with_handler(handler)
                .with_udp_addrs(vec![ice_bind_addr])
                .build(),
        )
        .await
        .map_err(|error| error.to_string())?,
    ))
}

fn rtc_configuration(stun_servers: &[String]) -> RTCConfiguration {
    if stun_servers.is_empty() {
        RTCConfigurationBuilder::default().build()
    } else {
        RTCConfigurationBuilder::default()
            .with_ice_servers(vec![RTCIceServer {
                urls: stun_servers.to_vec(),
                ..Default::default()
            }])
            .build()
    }
}

struct NetherNetEventHandler {
    session: Mutex<Option<Arc<NetherNetSession>>>,
    address: SocketAddr,
    gathering_notify: Arc<tokio::sync::Notify>,
}

#[async_trait]
impl PeerConnectionEventHandler for NetherNetEventHandler {
    async fn on_data_channel(&self, channel: Arc<dyn DataChannel>) {
        if let Some(session) = self.session.lock().await.as_ref()
            && let Err(error) = session.attach_channel(channel).await
        {
            warn!("Rejected NetherNet data channel: {error}");
            session.close().await;
        }
    }

    async fn on_connection_state_change(&self, connection_state: RTCPeerConnectionState) {
        trace!(?connection_state, address = %self.address, "NetherNet peer connection state changed");
        if matches!(
            connection_state,
            RTCPeerConnectionState::Failed
                | RTCPeerConnectionState::Disconnected
                | RTCPeerConnectionState::Closed
        ) && let Some(session) = self.session.lock().await.as_ref()
        {
            session.mark_closed();
        }
    }

    async fn on_ice_connection_state_change(&self, connection_state: RTCIceConnectionState) {
        trace!(?connection_state, address = %self.address, "NetherNet ICE connection state changed");
    }

    async fn on_ice_gathering_state_change(&self, state: RTCIceGatheringState) {
        trace!(?state, address = %self.address, "NetherNet ICE gathering state changed");
        if state == RTCIceGatheringState::Complete {
            self.gathering_notify.notify_waiters();
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_channel_sdp_only_advertises_component_one() {
        let sdp = "v=0\r\na=candidate:1 1 udp 1 192.0.2.1 19134 typ host\r\na=candidate:1 2 udp 1 192.0.2.1 19134 typ host\r\na=end-of-candidates\r\n";
        assert_eq!(
            remove_component_two_candidates(sdp),
            "v=0\r\na=candidate:1 1 udp 1 192.0.2.1 19134 typ host\r\na=end-of-candidates\r\n"
        );
    }
}
