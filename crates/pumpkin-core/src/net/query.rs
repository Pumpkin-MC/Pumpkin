use std::{
    ffi::{CString, NulError},
    net::{IpAddr, SocketAddr},
    sync::{Arc, atomic::Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use hmac::{Hmac, KeyInit, Mac};
use pumpkin_protocol::query::{
    CBasicStatus, CFullStatus, CHandshake, PacketType, RawQueryPacket, SHandshake, SStatusRequest,
};
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_world::CURRENT_MC_VERSION;
use rand::Rng;
use sha2::Sha256;
use tokio::net::UdpSocket;
use tracing::{error, info};

use crate::{SHOULD_STOP, STOP_INTERRUPT, server::Server};

/// Token lifetime. One is accepted for the window it was minted in and the previous one.
const TOKEN_WINDOW: u64 = 30;

/// Derives the challenge token for a source address. Query is UDP and the source is
/// forgeable, so deriving leaves no table a flood of spoofed handshakes could fill.
fn challenge_token(key: &Hmac<Sha256>, addr: SocketAddr, window: u64) -> i32 {
    let mut mac = key.clone();
    match addr.ip() {
        IpAddr::V4(ip) => mac.update(&ip.octets()),
        IpAddr::V6(ip) => mac.update(&ip.octets()),
    }
    mac.update(&addr.port().to_be_bytes());
    mac.update(&window.to_be_bytes());

    let bytes = mac.finalize().into_bytes();
    let value = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    // The wire format is a signed int, and vanilla never sends 0.
    let token = (value & 0x7FFF_FFFF) as i32;
    if token == 0 { 1 } else { token }
}

fn current_window() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() / TOKEN_WINDOW)
}

pub async fn start_query_handler(server: Arc<Server>, query_addr: SocketAddr) {
    let Ok(socket) = UdpSocket::bind(query_addr).await else {
        error!("Unable to bind query UDP socket");
        return;
    };

    // Per process, so a token cannot be reused on another server or across a restart.
    let mut secret = [0u8; 32];
    rand::rng().fill_bytes(&mut secret);
    let Ok(token_key) = <Hmac<Sha256> as KeyInit>::new_from_slice(&secret) else {
        error!("Unable to derive query challenge token key");
        return;
    };

    if let Ok(local_addr) = socket.local_addr() {
        info!(
            "Server query running on port {}",
            TextComponent::text(format!("{}", local_addr.port()))
                .color_named(NamedColor::DarkBlue)
                .to_pretty_console()
        );
    }

    // Reused across packets; handling inline keeps a datagram flood from piling up tasks.
    let mut buf = vec![0; 1024];

    while !SHOULD_STOP.load(Ordering::Relaxed) {
        let recv_result = tokio::select! {
            result = socket.recv_from(&mut buf) => Some(result),
            () = STOP_INTERRUPT.cancelled() => None,
        };

        let Some(Ok((length, addr))) = recv_result else {
            break;
        };

        if let Err(err) = handle_packet(
            buf[..length].to_vec(),
            &token_key,
            &server,
            &socket,
            addr,
            query_addr,
        )
        .await
        {
            error!("Interior 0 bytes found! Cannot encode query response! {err}");
        }
    }
}

// Errors of packets that don't meet the format aren't returned since we won't handle them anyway
// The only errors that are thrown are because of a null terminator in a CString
// since those errors need to be corrected by server owner
#[expect(clippy::too_many_lines)]
#[inline]
async fn handle_packet(
    buf: Vec<u8>,
    token_key: &Hmac<Sha256>,
    server: &Server,
    socket: &UdpSocket,
    addr: SocketAddr,
    bound_addr: SocketAddr,
) -> Result<(), NulError> {
    if let Ok(mut raw_packet) = RawQueryPacket::decode(buf).await {
        match raw_packet.packet_type {
            PacketType::Handshake => {
                if let Ok(packet) = SHandshake::decode(&mut raw_packet).await {
                    let challenge_token = challenge_token(token_key, addr, current_window());
                    let response = CHandshake {
                        session_id: packet.session_id,
                        challenge_token,
                    };

                    // Ignore all errors since we don't want the query handler to crash
                    // Protocol also ignores all errors and just doesn't respond
                    // Dropped rather than awaited: an unwritable socket would hold the
                    // receive loop, and with it the shutdown check, for an answer a client
                    // will retry anyway.
                    if let Some(encoded) = response.encode() {
                        let _ = socket.try_send_to(encoded.as_slice(), addr);
                    }
                }
            }
            PacketType::Status => {
                if let Ok(packet) = SStatusRequest::decode(&mut raw_packet).await
                    // The minting window may have rolled over, so the previous is accepted too.
                    && {
                        let window = current_window();
                        packet.challenge_token == challenge_token(token_key, addr, window)
                            || packet.challenge_token
                                == challenge_token(token_key, addr, window.saturating_sub(1))
                    }
                {
                    if packet.is_full_request {
                        // Get 4 players
                        let mut players: Vec<CString> = Vec::new();
                        for world in server.worlds.load().iter() {
                            let mut world_players = world
                                .players
                                .load()
                                // Although there is no documented limit, we will limit to 4 players
                                .iter()
                                .take(4 - players.len())
                                .filter_map(|player| {
                                    CString::new(player.gameprofile.name.as_str()).ok()
                                })
                                .collect::<Vec<_>>();

                            players.append(&mut world_players); // Append players from this world

                            if players.len() >= 4 {
                                break; // Stop if we've collected 4 players
                            }
                        }

                        let plugins = server
                            .plugin_manager
                            .active_plugins()
                            .into_iter()
                            .map(|meta| meta.name)
                            .reduce(|acc, name| format!("{acc}, {name}"))
                            .unwrap_or_default();

                        let response = CFullStatus {
                            session_id: packet.session_id,
                            hostname: CString::new(
                                server.advanced_config.networking.java.motd.as_str(),
                            )?,
                            version: CString::new(CURRENT_MC_VERSION)?,
                            plugins: CString::new(plugins)?,
                            map: CString::new(
                                server
                                    .worlds
                                    .load()
                                    .first()
                                    .map_or("world", |w| w.get_world_name()),
                            )?,
                            num_players: server.get_player_count(),
                            max_players: server.advanced_config.networking.java.max_players
                                as usize,
                            host_port: bound_addr.port(),
                            host_ip: CString::new(bound_addr.ip().to_string())?,
                            players,
                        };

                        if let Some(encoded) = response.encode() {
                            let _ = socket.try_send_to(encoded.as_slice(), addr);
                        }
                    } else {
                        let response = CBasicStatus {
                            session_id: packet.session_id,
                            motd: CString::new(
                                server.advanced_config.networking.java.motd.as_str(),
                            )?,
                            map: CString::new(
                                server
                                    .worlds
                                    .load()
                                    .first()
                                    .map_or("world", |w| w.get_world_name()),
                            )?,
                            num_players: server.get_player_count(),
                            max_players: server.advanced_config.networking.java.max_players
                                as usize,
                            host_port: bound_addr.port(),
                            host_ip: CString::new(bound_addr.ip().to_string())?,
                        };

                        if let Some(encoded) = response.encode() {
                            let _ = socket.try_send_to(encoded.as_slice(), addr);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
