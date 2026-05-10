#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin_protocol::java::packet_encoder::TCPNetworkEncoder;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::ser::NetworkWriteExt;
use pumpkin_protocol::ServerPacket;
use pumpkin_protocol::java::client::play::CPlayerPosition;
use pumpkin_protocol::packet::MultiVersionJavaPacket;
use pumpkin_protocol::ClientPacket;
use pumpkin_util::version::MinecraftVersion;
use std::io::Cursor;
use tokio::runtime::Runtime;

const TARGET_VERSION: MinecraftVersion = MinecraftVersion::V_1_21_4;

fuzz_target!(|data: &[u8]| {
    if data.len() < 20 {
        return;
    }

    let compression_threshold = data[0] as usize;
    let compression_level = (data[1] % 10) as u32;
    let encryption_key: [u8; 16] = data[2..18].try_into().unwrap();
    let use_compression = data[18] % 2 == 0;
    let use_encryption = data[19] % 2 == 0;
    let packet_data = &data[20..];

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut out = Vec::new();
        let mut encoder = TCPNetworkEncoder::new(&mut out);

        if use_compression {
            encoder.set_compression((compression_threshold, compression_level));
        }
        if use_encryption {
            encoder.set_encryption(&encryption_key);
        }

        // 1. Fuzz with raw bytes (simulating various packet payloads)
        let mut buf = Vec::new();
        let packet_id = if packet_data.is_empty() { 0 } else { packet_data[0] as i32 };
        let _ = buf.write_var_int(&VarInt(packet_id));
        buf.extend_from_slice(packet_data);
        let _ = encoder.write_packet(buf.into()).await;

        // 2. Fuzz with actual packets if they can be partially read
        let mut cursor = Cursor::new(packet_data);
        if let Ok(packet) = CPlayerPosition::read(&mut cursor, &TARGET_VERSION) {
             let mut packet_buf = Vec::new();
             let id = CPlayerPosition::to_id(TARGET_VERSION);
             let _ = packet_buf.write_var_int(&VarInt(id));
             let _ = packet.write_packet_data(&mut packet_buf, &TARGET_VERSION);
             let _ = encoder.write_packet(packet_buf.into()).await;
        }

        let _ = encoder.flush().await;
    });
});
