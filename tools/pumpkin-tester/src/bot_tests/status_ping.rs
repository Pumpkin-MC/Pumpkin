use std::{net::SocketAddr, time::Instant};

use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_protocol::{
    ConnectionState, ServerPacket,
    codec::var_int::VarInt,
    java::{
        client::status::{CPingResponse, CStatusResponse},
        packet_decoder::TCPNetworkDecoder,
        packet_encoder::TCPNetworkEncoder,
        server::{
            handshake::SHandShake,
            status::{SStatusPingRequest, SStatusRequest},
        },
    },
    packet::MultiVersionJavaPacket,
};
use tokio::{
    io::{BufReader, BufWriter},
    net::TcpStream,
    time::timeout,
};

use crate::bot::{PACKET_TIMEOUT, send_framed_packet};

pub async fn run_bot_status_ping_test(
    addr: SocketAddr,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let slp_start = Instant::now();
    let stream = TcpStream::connect(addr)
        .await
        .map_err(|err| format!("Status ping failed to connect to {addr}: {err}"))?;
    let _ = stream.set_nodelay(true);
    let (read_half, write_half) = stream.into_split();
    let mut reader = TCPNetworkDecoder::new(BufReader::new(read_half));
    let mut writer = TCPNetworkEncoder::new(BufWriter::new(write_half));

    send_framed_packet(
        &mut writer,
        &SHandShake {
            protocol_version: VarInt(CURRENT_MC_VERSION.protocol_version()),
            server_address: "127.0.0.1".into(),
            server_port: addr.port(),
            next_state: ConnectionState::Status,
        },
    )
    .await?;

    send_framed_packet(&mut writer, &SStatusRequest).await?;

    let status_raw = timeout(PACKET_TIMEOUT, reader.get_raw_packet())
        .await
        .map_err(|_| "Timed out waiting for CStatusResponse".to_string())?
        .map_err(|err| format!("Failed to decode CStatusResponse: {err}"))?;
    if status_raw.id != CStatusResponse::to_id(CURRENT_MC_VERSION) {
        return Err(format!(
            "Unexpected status packet id {}, expected {}",
            status_raw.id,
            CStatusResponse::to_id(CURRENT_MC_VERSION)
        ));
    }
    let mut status_slice = &status_raw.payload[..];
    let status = CStatusResponse::read(&mut status_slice, &CURRENT_MC_VERSION)
        .map_err(|err| format!("Failed to parse CStatusResponse: {err}"))?;
    let parsed: serde_json::Value = serde_json::from_str(&status.json_response)
        .map_err(|err| format!("Invalid status JSON: {err}"))?;
    let protocol = parsed
        .get("version")
        .and_then(|v| v.get("protocol"))
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| "Status JSON missing version.protocol".to_string())?;
    let expected_protocol = i64::from(CURRENT_MC_VERSION.protocol_version());
    if protocol != expected_protocol {
        return Err(format!(
            "Status protocol mismatch: expected {expected_protocol}, got {protocol}"
        ));
    }

    let expected_ping = 0x1234_5678_9ABC_DEF0i64;
    send_framed_packet(
        &mut writer,
        &SStatusPingRequest {
            payload: expected_ping,
        },
    )
    .await?;

    let pong_raw = timeout(PACKET_TIMEOUT, reader.get_raw_packet())
        .await
        .map_err(|_| "Timed out waiting for CPingResponse".to_string())?
        .map_err(|err| format!("Failed to decode CPingResponse: {err}"))?;
    if pong_raw.id != CPingResponse::to_id(CURRENT_MC_VERSION) {
        return Err(format!(
            "Unexpected ping response packet id {}, expected {}",
            pong_raw.id,
            CPingResponse::to_id(CURRENT_MC_VERSION)
        ));
    }
    let mut pong_slice = &pong_raw.payload[..];
    let pong = CPingResponse::read(&mut pong_slice, &CURRENT_MC_VERSION)
        .map_err(|err| format!("Failed to parse CPingResponse: {err}"))?;
    if pong.payload != expected_ping {
        return Err(format!(
            "Ping payload mismatch: expected {expected_ping:#x}, got {:#x}",
            pong.payload
        ));
    }

    summary_lines.push(format!(
        "[PASS] bot:status_ping ({}ms)",
        slp_start.elapsed().as_millis()
    ));
    Ok(())
}
