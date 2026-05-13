#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin_protocol::ServerPacket;
use pumpkin_protocol::java::{
    packet_decoder::TCPNetworkDecoder,
    server::{
        config::{
            SClientInformationConfig, SConfigCookieResponse, SConfigResourcePack, SKnownPacks,
            SPluginMessage,
        },
        handshake::SHandShake,
        login::{SEncryptionResponse, SLoginCookieResponse, SLoginPluginResponse, SLoginStart},
        play::{
            SChangeGameMode, SChatCommand, SChatMessage, SChunkBatch, SClickSlot, SClientCommand,
            SClientInformationPlay, SCloseContainer, SCommandSuggestion, SConfirmTeleport,
            SContainerButtonClick, SCookieResponse, SCustomPayload, SInteract, SKeepAlive,
            SMoveVehicle, SPaddleBoat, SPickItemFromBlock, SPlayPingRequest, SPlayerAbilities,
            SPlayerAction, SPlayerCommand, SPlayerInput, SPlayerLoaded, SPlayerPosition,
            SPlayerPositionRotation, SPlayerRotation, SPlayerSession, SSetCommandBlock,
            SSetCreativeSlot, SSetHeldItem, SSetPlayerGround, SSwingArm, SUpdateSign, SUseItem,
            SUseItemOn,
        },
        status::SStatusPingRequest,
    },
};
use pumpkin_util::version::MinecraftVersion;
use std::io::Cursor;
use tokio::runtime::Runtime;

const TARGET_VERSION: MinecraftVersion = MinecraftVersion::V_26_1;

// ---------------------------------------------------------------------------
// Helper: run every known ServerPacket::read against the same payload.
// Uses a Cursor and the Version enum as required by the new signature.
// ---------------------------------------------------------------------------
fn fuzz_all_deserializers(payload: &[u8]) {
    let mut cursor = Cursor::new(payload);

    macro_rules! run_read {
        ($($packet:ty),* $(,)?) => {
            $(
                cursor.set_position(0);
                let _ = <$packet>::read(&mut cursor, &TARGET_VERSION);
            )*
        };
    }

    run_read!(
        // Handshake
        SHandShake,
        // Status
        SStatusPingRequest,
        // Login
        SLoginStart,
        SEncryptionResponse,
        SLoginPluginResponse,
        SLoginCookieResponse,
        // Config
        SClientInformationConfig,
        SPluginMessage,
        SKnownPacks,
        SConfigCookieResponse,
        SConfigResourcePack,
        // Play
        SConfirmTeleport,
        SChangeGameMode,
        SChatCommand,
        SChatMessage,
        SClientInformationPlay,
        SClientCommand,
        SPlayerInput,
        SMoveVehicle,
        SPaddleBoat,
        SInteract,
        SKeepAlive,
        SPlayerPosition,
        SPlayerPositionRotation,
        SPlayerRotation,
        SSetPlayerGround,
        SPickItemFromBlock,
        SPlayerAbilities,
        SPlayerAction,
        SSetCommandBlock,
        SPlayerCommand,
        SPlayerLoaded,
        SPlayPingRequest,
        SClickSlot,
        SContainerButtonClick,
        SSetHeldItem,
        SSetCreativeSlot,
        SSwingArm,
        SUpdateSign,
        SUseItemOn,
        SUseItem,
        SCommandSuggestion,
        SCookieResponse,
        SCloseContainer,
        SChunkBatch,
        SPlayerSession,
        SCustomPayload,
    );
}

// ---------------------------------------------------------------------------
// Fuzz target
// ---------------------------------------------------------------------------
fuzz_target!(|data: &[u8]| {
    if data.len() < 18 {
        return;
    }

    let mode = data[0] % 4;
    let key = &data[2..18];
    let rest = &data[18..];

    let split = if rest.is_empty() {
        0
    } else {
        (data[1] as usize) % rest.len()
    };
    let (decoder_bytes, deser_bytes) = rest.split_at(split);

    // --- Path 1: decoder (framing / encryption / compression) --------------
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut decoder = TCPNetworkDecoder::new(Cursor::new(decoder_bytes));
        match mode {
            1 => {
                decoder.set_compression(256);
            }
            2 => {
                let mut aes_key = [0u8; 16];
                aes_key.copy_from_slice(key);
                decoder.set_encryption(&aes_key);
            }
            3 => {
                decoder.set_compression(256);
                let mut aes_key = [0u8; 16];
                aes_key.copy_from_slice(key);
                decoder.set_encryption(&aes_key);
            }
            _ => {}
        }
        let _ = decoder.get_raw_packet().await;
    });

    // --- Path 2: Individual Packet Deserializers ---------------------------
    fuzz_all_deserializers(deser_bytes);
});
