use std::{
    io::{Error, ErrorKind},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::LazyLock,
};

use aes::{
    Aes256, Block,
    cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit},
};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use tokio::net::UdpSocket;

use crate::server::Server;

const DISCOVERY_PORT: u16 = 7551;
const CHECKSUM_SIZE: usize = 32;
const HEADER_SIZE: usize = 18;
const REQUEST_PACKET: u16 = 0;
const RESPONSE_PACKET: u16 = 1;
const SERVER_DATA_VERSION: u8 = 6;

static KEY: LazyLock<[u8; 32]> =
    LazyLock::new(|| Sha256::digest(0xdeadbeefu64.to_le_bytes()).into());

pub struct NetherNetDiscovery {
    socket: UdpSocket,
    advertisement_id: u64,
}

impl NetherNetDiscovery {
    pub async fn bind(address: SocketAddr) -> Result<Self, Error> {
        let ip = match address.ip() {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => Ipv4Addr::UNSPECIFIED,
        };
        let socket = UdpSocket::bind((ip, DISCOVERY_PORT)).await?;
        Ok(Self {
            socket,
            advertisement_id: rand::random(),
        })
    }

    pub async fn receive(&self, server: &Server, buffer: &mut [u8]) -> Result<(), Error> {
        let (length, address) = self.socket.recv_from(buffer).await?;
        if decode_request(&buffer[..length]).is_none() {
            return Ok(());
        }

        let players = server
            .get_status()
            .lock()
            .await
            .status_response
            .players
            .as_ref()
            .map_or(0, |players| players.online);
        let game_mode = server.defaultgamemode.lock().await.gamemode as u8;
        let response = encode_response(
            server.server_guid,
            self.advertisement_id,
            &server.advanced_config.networking.bedrock.motd,
            &server.basic_config.default_level_name,
            game_mode,
            players,
            server.advanced_config.networking.bedrock.max_players,
            server.basic_config.hardcore,
        )?;
        self.socket.send_to(&response, address).await?;
        Ok(())
    }

    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        self.socket.local_addr()
    }
}

fn decode_request(data: &[u8]) -> Option<u64> {
    let encrypted = data.get(CHECKSUM_SIZE..)?;
    if encrypted.is_empty() || encrypted.len() % 16 != 0 {
        return None;
    }

    let mut payload = data.get(CHECKSUM_SIZE..)?.to_vec();
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

    let mut mac = Hmac::<Sha256>::new_from_slice(KEY.as_slice()).ok()?;
    mac.update(&payload);
    mac.verify_slice(data.get(..CHECKSUM_SIZE)?).ok()?;

    let declared_length = usize::from(u16::from_le_bytes(payload.get(..2)?.try_into().ok()?));
    if declared_length != payload.len() - 2
        || declared_length != HEADER_SIZE
        || u16::from_le_bytes(payload.get(2..4)?.try_into().ok()?) != REQUEST_PACKET
        || payload.get(12..20)? != [0; 8]
    {
        return None;
    }
    Some(u64::from_le_bytes(payload.get(4..12)?.try_into().ok()?))
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
    server_data.extend_from_slice(&[0, u8::from(hardcore), 0, 0]);
    push_string(&mut server_data, &format!("{advertisement_id:016x}"))?;
    server_data.extend_from_slice(&[2 << 1, 4 << 1]);

    let application_data = hex::encode(server_data);
    let application_length = u32::try_from(application_data.len())
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "NetherNet MOTD is too long"))?;
    let packet_length = HEADER_SIZE + size_of::<u32>() + application_data.len();
    let packet_length = u16::try_from(packet_length)
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "NetherNet MOTD is too long"))?;

    let mut payload = Vec::with_capacity(2 + usize::from(packet_length) + 16);
    payload.extend_from_slice(&packet_length.to_le_bytes());
    payload.extend_from_slice(&RESPONSE_PACKET.to_le_bytes());
    payload.extend_from_slice(&network_id.to_le_bytes());
    payload.extend_from_slice(&[0; 8]);
    payload.extend_from_slice(&application_length.to_le_bytes());
    payload.extend_from_slice(application_data.as_bytes());

    let mut mac = Hmac::<Sha256>::new_from_slice(KEY.as_slice())
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
    Aes256::new_from_slice(KEY.as_slice())
        .expect("AES-256 key has the correct length")
        .encrypt_blocks(blocks);
}

fn decrypt(data: &mut [u8]) -> Option<()> {
    let (blocks, remainder) = Block::slice_as_chunks_mut(data);
    if !remainder.is_empty() {
        return None;
    }
    Aes256::new_from_slice(KEY.as_slice())
        .ok()?
        .decrypt_blocks(blocks);
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
        assert_eq!(decode_request(&request), Some(99));
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
        )
        .unwrap();
        let mut payload = response[CHECKSUM_SIZE..].to_vec();
        decrypt(&mut payload).unwrap();
        let padding = usize::from(*payload.last().unwrap());
        payload.truncate(payload.len() - padding);
        let application_length = u32::from_le_bytes(payload[20..24].try_into().unwrap()) as usize;
        let server_data = hex::decode(&payload[24..24 + application_length]).unwrap();
        assert_eq!(
            hex::encode(server_data),
            "0610446564696361746564205365727665720e4372656174697665206c6576656c\
             02000000000a0000000000000010396262363462636631343732376264620408"
        );
    }
}
