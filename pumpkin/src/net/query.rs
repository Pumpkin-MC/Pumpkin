use std::{
    collections::HashMap,
    ffi::{CString, NulError},
    net::SocketAddr,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use pumpkin_protocol::query::{
    CBasicStatus, CFullStatus, CHandshake, PacketType, RawQueryPacket, SHandshake, SStatusRequest,
};
use pumpkin_world::CURRENT_MC_VERSION;
use rand::Rng;
use tokio::{net::UdpSocket, sync::RwLock, time};

use crate::{PLUGIN_MANAGER, SHOULD_STOP, STOP_INTERRUPT, server::Server};

pub async fn start_query_handler(server: Arc<Server>, query_addr: SocketAddr) {
    let socket = Arc::new(
        UdpSocket::bind(query_addr)
            .await
            .expect("Unable to bind to address"),
    );

    // Challenge tokens are bound to the IP address and port
    let valid_challenge_tokens = Arc::new(RwLock::new(HashMap::new()));
    let valid_challenge_tokens_clone = valid_challenge_tokens.clone();
    // All challenge tokens ever created are expired every 30 seconds
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;
            valid_challenge_tokens_clone.write().await.clear();
        }
    });

    log::info!(
        "Server query running on port {}",
        socket
            .local_addr()
            .expect("Unable to find running address!")
            .port()
    );

    while !SHOULD_STOP.load(Ordering::Relaxed) {
        let socket = socket.clone();
        let valid_challenge_tokens = valid_challenge_tokens.clone();
        let server = server.clone();
        let mut buf = vec![0; 1024];

        let recv_result = tokio::select! {
            result = socket.recv_from(&mut buf) => Some(result),
            () = STOP_INTERRUPT.notified() => None,
        };

        let Some(Ok((_, addr))) = recv_result else {
            break;
        };

        tokio::spawn(async move {
            if let Err(err) = handle_packet(
                buf,
                valid_challenge_tokens,
                server,
                socket,
                addr,
                query_addr,
            )
            .await
            {
                log::error!("Interior 0 bytes found! Cannot encode query response! {err}");
            }
        });
    }
}

// Errors of packets that don't meet the format aren't returned since we won't handle them anyway
// The only errors that are thrown are because of a null terminator in a CString
// since those errors need to be corrected by server owner
#[inline]
async fn handle_packet(
    buf: Vec<u8>,
    clients: Arc<RwLock<HashMap<i32, SocketAddr>>>,
    server: Arc<Server>,
    socket: Arc<UdpSocket>,
    addr: SocketAddr,
    bound_addr: SocketAddr,
) -> Result<(), NulError> {
    if let Ok(mut raw_packet) = RawQueryPacket::decode(buf).await {
        match raw_packet.packet_type {
            PacketType::Handshake => {
                if let Ok(packet) = SHandshake::decode(&mut raw_packet).await {
                    let challenge_token = rand::rng().random_range(1..=i32::MAX);
                    let response = CHandshake {
                        session_id: packet.session_id,
                        challenge_token,
                    };

                    // Ignore all errors since we don't want the query handler to crash
                    // Protocol also ignores all errors and just doesn't respond
                    let _ = socket
                        .send_to(response.encode().await.as_slice(), addr)
                        .await;

                    clients.write().await.insert(challenge_token, addr);
                }
            }
            PacketType::Status => {
                if let Ok(packet) = SStatusRequest::decode(&mut raw_packet).await
                    && clients
                        .read()
                        .await
                        .get(&packet.challenge_token)
                        .is_some_and(|token_bound_ip: &SocketAddr| token_bound_ip == &addr)
                {
                    if packet.is_full_request {
                        // Get 4 players
                        let mut players: Vec<CString> = Vec::new();
                        for world in server.worlds.read().await.iter() {
                            let mut world_players = world
                                .players
                                .read()
                                .await
                                // Although there is no documented limit, we will limit to 4 players
                                .values()
                                .take(4 - players.len())
                                .map(|player| {
                                    CString::new(player.gameprofile.name.as_str()).unwrap()
                                })
                                .collect::<Vec<_>>();

                            players.append(&mut world_players); // Append players from this world

                            if players.len() >= 4 {
                                break; // Stop if we've collected 4 players
                            }
                        }

                        let plugins = PLUGIN_MANAGER
                            .active_plugins()
                            .await
                            .into_iter()
                            .map(|meta| meta.name.to_string())
                            .reduce(|acc, name| format!("{acc}, {name}"))
                            .unwrap_or_default();

                        let response = CFullStatus {
                            session_id: packet.session_id,
                            hostname: CString::new(server.basic_config.motd.as_str())?,
                            version: CString::new(CURRENT_MC_VERSION)?,
                            plugins: CString::new(plugins)?,
                            map: CString::new("world")?, // TODO: Get actual world name
                            num_players: server.get_player_count().await,
                            max_players: server.basic_config.max_players as usize,
                            host_port: bound_addr.port(),
                            host_ip: CString::new(bound_addr.ip().to_string())?,
                            players,
                        };

                        let _ = socket
                            .send_to(response.encode().await.as_slice(), addr)
                            .await;
                    } else {
                        let response = CBasicStatus {
                            session_id: packet.session_id,
                            motd: CString::new(server.basic_config.motd.as_str())?,
                            map: CString::new("world")?,
                            num_players: server.get_player_count().await,
                            max_players: server.basic_config.max_players as usize,
                            host_port: bound_addr.port(),
                            host_ip: CString::new(bound_addr.ip().to_string())?,
                        };

                        let _ = socket
                            .send_to(response.encode().await.as_slice(), addr)
                            .await;
                    }
                }
            }
        }
    }
    Ok(())
}
