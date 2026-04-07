#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin_protocol::bedrock::packet_decoder::UDPNetworkDecoder;
use pumpkin_protocol::bedrock::server::{
    client_cache_status::SClientCacheStatus,
    command_request::SCommandRequest,
    container_close::SContainerClose,
    interaction::SInteraction,
    loading_screen::SLoadingScreen,
    login::SLogin,
    player_auth_input::SPlayerAuthInput,
    raknet::{
        connection::SConnectionRequest,
        open_connection::{SOpenConnectionRequest1, SOpenConnectionRequest2},
        unconnected_ping::SUnconnectedPing,
    },
    request_chunk_radius::SRequestChunkRadius,
    request_network_settings::SRequestNetworkSettings,
    text::SText,
};
use pumpkin_protocol::serial::PacketRead;
use std::io::Cursor;

// ---------------------------------------------------------------------------
// Helper: Run every Serverbound packet's read method.
// ---------------------------------------------------------------------------
fn fuzz_serverbound_packets(payload: &[u8]) {
    let mut cursor = Cursor::new(payload);

    macro_rules! run_read {
        ($($packet:ty),* $(,)?) => {
            $(
                cursor.set_position(0);
                // Attempt to read as a standard Game Packet (with version)
                let _ = <$packet>::read(&mut cursor);
            )*
        };
    }

    // Standard Bedrock Serverbound Packets
    run_read!(
        SClientCacheStatus,
        SCommandRequest,
        SContainerClose,
        SInteraction,
        SLoadingScreen,
        SLogin,
        SPlayerAuthInput,
        SRequestChunkRadius,
        SRequestNetworkSettings,
        SText,
    );

    // RakNet Handshake Packets (Usually read without version)
    cursor.set_position(0);
    let _ = SConnectionRequest::read(&mut cursor);
    cursor.set_position(0);
    let _ = SOpenConnectionRequest1::read(&mut cursor);
    cursor.set_position(0);
    let _ = SOpenConnectionRequest2::read(&mut cursor);
    cursor.set_position(0);
    let _ = SUnconnectedPing::read(&mut cursor);
}

// ---------------------------------------------------------------------------
// Fuzz Target
// ---------------------------------------------------------------------------
fuzz_target!(|data: &[u8]| {
    if data.len() < 20 {
        return;
    }

    // Split data for decoder configuration vs raw payload
    let threshold_raw = data[0];
    let key: [u8; 16] = data[1..17].try_into().unwrap();
    let stream_data = &data[17..];

    let mut decoder = UDPNetworkDecoder::new();

    // Setup Decoder
    if threshold_raw > 0 {
        // Assuming your CompressionThreshold is a wrapper around u32
        decoder.set_compression((threshold_raw as u32).try_into().unwrap());
    }
    decoder.set_encryption(&key);

    // 1. Fuzz the Decoder (Framing/VarInts/Bitmasks)
    let decoder_cursor = Cursor::new(stream_data.to_vec());
    if let Ok(raw_packet) = decoder.get_game_packet(decoder_cursor) {
        // If framed correctly, fuzz the internal payload
        fuzz_serverbound_packets(&raw_packet.payload);
    }

    // 2. Shotgun Fuzz
    // Pass raw data directly to all readers to catch panics in the parsers
    fuzz_serverbound_packets(stream_data);
});
