use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, LazyLock},
    time::Duration,
};

use aes::{
    Aes256, Block,
    cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit},
};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use tokio::{
    net::UdpSocket,
    sync::{Mutex, mpsc},
};
use tracing::{debug, trace, warn};
use webrtc::peer_connection::RTCIceCandidateInit;

use super::{NetherNetListener, negotiate};
use crate::server::Server;

const DISCOVERY_PORT: u16 = 7551;
const CHECKSUM_SIZE: usize = 32;
const HEADER_SIZE: usize = 18;
const REQUEST_PACKET: u16 = 0;
const RESPONSE_PACKET: u16 = 1;
const MESSAGE_PACKET: u16 = 2;
const SERVER_DATA_VERSION: u8 = 6;

type ConnectionKey = (u64, u64);

static KEY: LazyLock<[u8; 32]> =
    LazyLock::new(|| Sha256::digest(0xdeadbeefu64.to_le_bytes()).into());

pub struct NetherNetDiscovery {
    socket: Arc<UdpSocket>,
    network_id: u64,
    advertisement_id: u64,
    candidates: Arc<Mutex<HashMap<ConnectionKey, mpsc::UnboundedSender<RTCIceCandidateInit>>>>,
}

impl NetherNetDiscovery {
    pub async fn bind(address: SocketAddr, network_id: u64) -> Result<Self, Error> {
        let ip = match address.ip() {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => Ipv4Addr::UNSPECIFIED,
        };
        let socket = UdpSocket::bind((ip, DISCOVERY_PORT)).await?;
        Ok(Self {
            socket: Arc::new(socket),
            network_id,
            advertisement_id: rand::random(),
            candidates: Arc::default(),
        })
    }

    pub async fn receive(
        &self,
        server: &Server,
        listener: &NetherNetListener,
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        let (length, address) = self.socket.recv_from(buffer).await?;
        match decode_packet(&buffer[..length]) {
            Some(Packet::Request) => {
                trace!("Received NetherNet LAN discovery request from {address}");
                self.advertise(server, address).await?;
            }
            Some(Packet::Message {
                sender_id,
                recipient_id,
                data,
            }) if recipient_id == self.network_id => {
                trace!(
                    sender_id,
                    recipient_id,
                    signal = data
                        .split_once(' ')
                        .map_or(data.as_str(), |(signal, _)| signal),
                    "Received NetherNet LAN signal from {address}"
                );
                self.handle_signal(listener, address, sender_id, data).await;
            }
            Some(Packet::Message {
                sender_id,
                recipient_id,
                ..
            }) => trace!(
                sender_id,
                recipient_id,
                expected_recipient_id = self.network_id,
                "Ignored NetherNet LAN signal for another server"
            ),
            None => trace!("Ignored invalid {length}-byte NetherNet LAN packet from {address}"),
        }
        Ok(())
    }

    async fn advertise(&self, server: &Server, address: SocketAddr) -> Result<(), Error> {
        let players = server
            .get_status()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .status_response
            .players
            .as_ref()
            .map_or(0, |players| players.online);
        let game_mode = server
            .defaultgamemode
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .gamemode as u8;
        let response = encode_response(
            self.network_id,
            self.advertisement_id,
            &server.advanced_config.networking.bedrock.motd,
            &server.basic_config.default_level_name,
            game_mode,
            players,
            server.advanced_config.networking.bedrock.max_players,
            server.basic_config.hardcore,
            server.advanced_config.networking.bedrock.online_mode,
        )?;
        self.socket.send_to(&response, address).await?;
        trace!(
            %address,
            network_id = self.network_id,
            advertisement_id = format_args!("{:016x}", self.advertisement_id),
            version = SERVER_DATA_VERSION,
            players,
            max_players = server.advanced_config.networking.bedrock.max_players,
            online_mode = server.advanced_config.networking.bedrock.online_mode,
            "Sent NetherNet LAN discovery response"
        );
        Ok(())
    }

    async fn handle_signal(
        &self,
        listener: &NetherNetListener,
        address: SocketAddr,
        sender_id: u64,
        data: String,
    ) {
        if data == "Ping" {
            return;
        }
        let mut parts = data.splitn(3, ' ');
        let (Some(signal_type), Some(connection_id), Some(data)) =
            (parts.next(), parts.next(), parts.next())
        else {
            trace!(%address, sender_id, "Ignored empty or malformed NetherNet LAN signal");
            return;
        };
        let Ok(connection_id) = connection_id.parse::<u64>() else {
            trace!(%address, sender_id, "Ignored NetherNet LAN signal with invalid connection ID");
            return;
        };
        let key = (sender_id, connection_id);

        match signal_type {
            "CONNECTREQUEST" => {
                trace!(%address, sender_id, connection_id, "Accepted NetherNet LAN connection request");
                let (candidate_sender, candidate_receiver) = mpsc::unbounded_channel();
                self.candidates.lock().await.insert(key, candidate_sender);

                let state = listener.state.clone();
                let socket = self.socket.clone();
                let candidates = self.candidates.clone();
                let offer = data.to_owned();
                let network_id = self.network_id;
                tokio::spawn(async move {
                    let signal = match Box::pin(negotiate(
                        &state,
                        address,
                        &offer,
                        Some(candidate_receiver),
                    ))
                    .await
                    {
                        Ok((answer, _session)) => {
                            format!("CONNECTRESPONSE {connection_id} {answer}")
                        }
                        Err(error) => {
                            warn!("NetherNet LAN negotiation with {address} failed: {error}");
                            format!("CONNECTERROR {connection_id} 11")
                        }
                    };
                    match encode_message(network_id, sender_id, &signal) {
                        Ok(response) => {
                            if let Err(error) = socket.send_to(&response, address).await {
                                warn!("Failed to send NetherNet LAN signal to {address}: {error}");
                            } else {
                                trace!(%address, sender_id, connection_id, "Sent NetherNet LAN connection response");
                            }
                        }
                        Err(error) => warn!("Failed to encode NetherNet LAN signal: {error}"),
                    }
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    candidates.lock().await.remove(&key);
                });
            }
            "CANDIDATEADD" => {
                let candidate =
                    serde_json::from_str(data).unwrap_or_else(|_| RTCIceCandidateInit {
                        candidate: data.to_owned(),
                        ..Default::default()
                    });
                if let Some(sender) = self.candidates.lock().await.get(&key) {
                    let _ = sender.send(candidate);
                    trace!(%address, sender_id, connection_id, "Forwarded NetherNet LAN ICE candidate");
                } else {
                    trace!(%address, sender_id, connection_id, "Ignored NetherNet LAN ICE candidate for unknown connection");
                }
            }
            signal_type => debug!("Ignoring NetherNet LAN signal {signal_type}"),
        }
    }

    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        self.socket.local_addr()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Packet {
    Request,
    Message {
        sender_id: u64,
        recipient_id: u64,
        data: String,
    },
}

fn decode_packet(data: &[u8]) -> Option<Packet> {
    let encrypted = data.get(CHECKSUM_SIZE..)?;
    if encrypted.is_empty() || !encrypted.len().is_multiple_of(16) {
        return None;
    }

    let mut payload = encrypted.to_vec();
    decrypt(&mut payload)?;
    let padding = usize::from(*payload.last()?);
    if padding == 0
        || padding > 16
        || payload.len() < padding
        || payload[payload.len() - padding..]
            .iter()
            .any(|byte| usize::from(*byte) != padding)
    {
        return None;
    }
    payload.truncate(payload.len() - padding);

    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(KEY.as_slice()).ok()?;
    mac.update(&payload);
    mac.verify_slice(data.get(..CHECKSUM_SIZE)?).ok()?;

    let declared_length = usize::from(u16::from_le_bytes(payload.get(..2)?.try_into().ok()?));
    if ![payload.len(), payload.len() - 2].contains(&declared_length)
        || payload.get(12..20)? != [0; 8]
    {
        return None;
    }
    let packet_id = u16::from_le_bytes(payload.get(2..4)?.try_into().ok()?);
    let sender_id = u64::from_le_bytes(payload.get(4..12)?.try_into().ok()?);
    let body = payload.get(20..)?;
    match packet_id {
        REQUEST_PACKET if body.is_empty() => Some(Packet::Request),
        MESSAGE_PACKET => {
            let recipient_id = u64::from_le_bytes(body.get(..8)?.try_into().ok()?);
            let length = u32::from_le_bytes(body.get(8..12)?.try_into().ok()?) as usize;
            let data = body.get(12..12usize.checked_add(length)?)?;
            if body.len() != 12 + length {
                return None;
            }
            Some(Packet::Message {
                sender_id,
                recipient_id,
                data: String::from_utf8(data.to_vec()).ok()?,
            })
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn encode_response(
    network_id: u64,
    advertisement_id: u64,
    server_name: &str,
    level_name: &str,
    game_mode: u8,
    player_count: u32,
    max_player_count: u32,
    hardcore: bool,
    online_mode: bool,
) -> Result<Vec<u8>, Error> {
    let mut server_data = vec![SERVER_DATA_VERSION];
    push_string(&mut server_data, server_name)?;
    push_string(&mut server_data, level_name)?;
    server_data.push(game_mode << 1);
    server_data.extend_from_slice(
        &i32::try_from(player_count)
            .unwrap_or(i32::MAX)
            .to_le_bytes(),
    );
    server_data.extend_from_slice(
        &i32::try_from(max_player_count)
            .unwrap_or(i32::MAX)
            .to_le_bytes(),
    );
    server_data.extend_from_slice(&[0, u8::from(hardcore), 0, u8::from(!online_mode)]);
    push_string(&mut server_data, &format!("{advertisement_id:016x}"))?;
    server_data.extend_from_slice(&[2 << 1, 4 << 1]);

    let application_data = hex::encode(server_data);
    let application_length = u32::try_from(application_data.len())
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "NetherNet MOTD is too long"))?;
    let mut body = Vec::with_capacity(size_of::<u32>() + application_data.len());
    body.extend_from_slice(&application_length.to_le_bytes());
    body.extend_from_slice(application_data.as_bytes());
    encode_packet(RESPONSE_PACKET, network_id, &body)
}

fn encode_message(network_id: u64, recipient_id: u64, data: &str) -> Result<Vec<u8>, Error> {
    let length = u32::try_from(data.len())
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "NetherNet signal is too long"))?;
    let mut body = Vec::with_capacity(size_of::<u64>() + size_of::<u32>() + data.len());
    body.extend_from_slice(&recipient_id.to_le_bytes());
    body.extend_from_slice(&length.to_le_bytes());
    body.extend_from_slice(data.as_bytes());
    encode_packet(MESSAGE_PACKET, network_id, &body)
}

fn encode_packet(packet_id: u16, network_id: u64, body: &[u8]) -> Result<Vec<u8>, Error> {
    let packet_length = u16::try_from(size_of::<u16>() + HEADER_SIZE + body.len())
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "NetherNet packet is too long"))?;
    let mut payload = Vec::with_capacity(usize::from(packet_length) + 16);
    payload.extend_from_slice(&packet_length.to_le_bytes());
    payload.extend_from_slice(&packet_id.to_le_bytes());
    payload.extend_from_slice(&network_id.to_le_bytes());
    payload.extend_from_slice(&[0; 8]);
    payload.extend_from_slice(body);

    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(KEY.as_slice())
        .map_err(|_| Error::other("invalid NetherNet discovery key"))?;
    mac.update(&payload);
    let checksum = mac.finalize().into_bytes();

    let padding = 16 - payload.len() % 16;
    payload.resize(payload.len() + padding, padding as u8);
    encrypt(&mut payload);

    let mut response = Vec::with_capacity(CHECKSUM_SIZE + payload.len());
    response.extend_from_slice(&checksum);
    response.extend_from_slice(&payload);
    Ok(response)
}

fn push_string(buffer: &mut Vec<u8>, value: &str) -> Result<(), Error> {
    let length = u8::try_from(value.len())
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "NetherNet MOTD field is too long"))?;
    buffer.push(length);
    buffer.extend_from_slice(value.as_bytes());
    Ok(())
}

fn encrypt(data: &mut [u8]) {
    let (blocks, remainder) = Block::slice_as_chunks_mut(data);
    debug_assert!(remainder.is_empty());
    Aes256::new((&*KEY).into()).encrypt_blocks(blocks);
}

fn decrypt(data: &mut [u8]) -> Option<()> {
    let (blocks, remainder) = Block::slice_as_chunks_mut(data);
    if !remainder.is_empty() {
        return None;
    }
    Aes256::new((&*KEY).into()).decrypt_blocks(blocks);
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_reference_discovery_request() {
        let request = hex::decode(
            "b3eca3eb83a6fcb079faf2eae2bf8abbaadb5906bc42bd63a0056274a26e013f\
             6b9027ddcd144fe3ada066f65b76e8faff0f8709e13bf6858e2051c321db615e",
        )
        .unwrap();
        assert_eq!(decode_packet(&request), Some(Packet::Request));
    }

    #[test]
    fn decodes_current_discovery_request() {
        let request = hex::decode(
            "ba539dab685df1c59ba406ec900f48e8249d28821c6e7149309d3fc289002201\
             ee5a9a7b4fec49402ac54510cb744a96ff0f8709e13bf6858e2051c321db615e",
        )
        .unwrap();
        assert_eq!(decode_packet(&request), Some(Packet::Request));
    }

    #[test]
    fn encodes_current_server_data() {
        let response = encode_response(
            99,
            0x9bb64bcf14727bdb,
            "Dedicated Server",
            "Creative level",
            1,
            0,
            10,
            false,
            false,
        )
        .unwrap();
        let mut payload = response[CHECKSUM_SIZE..].to_vec();
        decrypt(&mut payload).unwrap();
        let padding = usize::from(*payload.last().unwrap());
        payload.truncate(payload.len() - padding);
        assert_eq!(
            usize::from(u16::from_le_bytes(payload[..2].try_into().unwrap())),
            payload.len()
        );
        let application_length = u32::from_le_bytes(payload[20..24].try_into().unwrap()) as usize;
        let server_data = hex::decode(&payload[24..24 + application_length]).unwrap();
        assert_eq!(
            hex::encode(server_data),
            "0610446564696361746564205365727665720e4372656174697665206c6576656c\
             02000000000a0000000000000110396262363462636631343732376264620408"
        );
    }

    #[test]
    fn message_packet_roundtrip() {
        let message = encode_message(99, 42, "CONNECTREQUEST 7 offer").unwrap();
        assert_eq!(
            decode_packet(&message),
            Some(Packet::Message {
                sender_id: 99,
                recipient_id: 42,
                data: "CONNECTREQUEST 7 offer".to_owned(),
            })
        );
    }
}
