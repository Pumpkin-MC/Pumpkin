#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin_protocol::bedrock::packet_decoder::BedrockBatchDecoder;
use pumpkin_protocol::bedrock::server::{
    SAnimate, SBlockPickRequest, SClientCacheBlobStatus, SClientCacheStatus, SCommandRequest,
    SContainerClose, SEmote, SEmoteList, SInteraction, SInventoryTransaction, SItemStackRequest,
    SLoadingScreen, SLogin, SMobEquipment, SModalFormResponse, SPacketViolationWarning,
    SPlayerAction, SPlayerAuthInput, SPlayerHotbar, SRequestAbility, SRequestChunkRadius,
    SRequestNetworkSettings, SResourcePackResponse, SRespawn, SSetLocalPlayerAsInitialized,
    SSetPlayerInventoryOptions, SText,
};
use pumpkin_protocol::serial::{PacketRead, PacketReadSlice};
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
                let _ = <$packet>::read(&mut cursor);
            )*
        };
    }

    macro_rules! run_read_slice {
        ($($packet:ty),* $(,)?) => {
            $(
                let mut slice = payload;
                let _ = <$packet>::read_slice(&mut slice);
            )*
        };
    }

    // Standard Bedrock Serverbound Packets (PacketRead)
    run_read!(
        SAnimate,
        SBlockPickRequest,
        SClientCacheBlobStatus,
        SClientCacheStatus,
        SCommandRequest,
        SContainerClose,
        SEmote,
        SEmoteList,
        SInteraction,
        SInventoryTransaction,
        SItemStackRequest,
        SLoadingScreen,
        SLogin,
        SMobEquipment,
        SModalFormResponse,
        SPacketViolationWarning,
        SPlayerAction,
        SPlayerAuthInput,
        SPlayerHotbar,
        SRequestAbility,
        SRequestChunkRadius,
        SRequestNetworkSettings,
        SResourcePackResponse,
        SRespawn,
        SSetLocalPlayerAsInitialized,
        SSetPlayerInventoryOptions,
        SText,
    );

    // Bedrock Serverbound Packets supporting zero-copy PacketReadSlice
    run_read_slice!(SCommandRequest, SEmote, SModalFormResponse, SText,);
}

// ---------------------------------------------------------------------------
// Fuzz Target
// ---------------------------------------------------------------------------
fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }

    // Split data for decoder configuration vs raw payload
    let threshold_raw = data[0];
    let stream_data = &data[1..];

    let mut decoder = BedrockBatchDecoder::new();

    // Setup Decoder
    if threshold_raw > 0 {
        // Assuming your CompressionThreshold is a wrapper around u32
        decoder.set_compression((threshold_raw as u32).try_into().unwrap());
    }
    // 1. Fuzz the Decoder (Framing/VarInts/Bitmasks)
    let mut decoder_cursor = Cursor::new(stream_data.to_vec());
    if let Ok(raw_packet) = decoder.get_game_packet(&mut decoder_cursor) {
        // If framed correctly, fuzz the internal payload
        fuzz_serverbound_packets(&raw_packet.payload);
    }

    // 2. Shotgun Fuzz
    // Pass raw data directly to all readers to catch panics in the parsers
    fuzz_serverbound_packets(stream_data);
});
