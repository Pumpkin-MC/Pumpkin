#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin_protocol::{
    ServerPacket,
    java::{
        packet_decoder::TCPNetworkDecoder,
        server::{
            config::{
                SClientInformationConfig, SConfigCookieResponse, SConfigResourcePack, SKnownPacks,
                SPluginMessage,
            },
            handshake::SHandShake,
            login::{SEncryptionResponse, SLoginCookieResponse, SLoginPluginResponse, SLoginStart},
            play::{
                SChangeGameMode, SChatCommand, SChatMessage, SChunkBatch, SClickSlot,
                SClientCommand, SClientInformationPlay, SCloseContainer, SCommandSuggestion,
                SConfirmTeleport, SCookieResponse, SCustomPayload, SInteract, SKeepAlive,
                SMoveVehicle, SPaddleBoat, SPickItemFromBlock, SPlayPingRequest, SPlayerAbilities,
                SPlayerAction, SPlayerCommand, SPlayerInput, SPlayerLoaded, SPlayerPosition,
                SPlayerPositionRotation, SPlayerRotation, SPlayerSession, SSetCommandBlock,
                SSetCreativeSlot, SSetHeldItem, SSetPlayerGround, SSwingArm, SUpdateSign, SUseItem,
                SUseItemOn,
            },
            status::SStatusPingRequest,
        },
    },
};
use std::io::Cursor;
use tokio::runtime::Runtime;

// ---------------------------------------------------------------------------
// Helper: run every known ServerPacket::read against the same payload.
// We don't care about the result — only that nothing panics.
// ---------------------------------------------------------------------------
fn fuzz_all_deserializers(payload: &[u8]) {
    // Handshake
    let _ = SHandShake::read(payload);

    // Status
    //let _ = SStatusRequest::read(payload);
    let _ = SStatusPingRequest::read(payload);

    // Login
    let _ = SLoginStart::read(payload);
    let _ = SEncryptionResponse::read(payload);
    let _ = SLoginPluginResponse::read(payload);
    //let _ = SLoginAcknowledged::read(payload);
    let _ = SLoginCookieResponse::read(payload);

    // Config
    let _ = SClientInformationConfig::read(payload);
    let _ = SPluginMessage::read(payload);
    //let _ = SAcknowledgeFinishConfig::read(payload);
    let _ = SKnownPacks::read(payload);
    let _ = SConfigCookieResponse::read(payload);
    let _ = SConfigResourcePack::read(payload);

    // Play
    let _ = SConfirmTeleport::read(payload);
    let _ = SChangeGameMode::read(payload);
    let _ = SChatCommand::read(payload);
    let _ = SChatMessage::read(payload);
    let _ = SClientInformationPlay::read(payload);
    let _ = SClientCommand::read(payload);
    let _ = SPlayerInput::read(payload);
    let _ = SMoveVehicle::read(payload);
    let _ = SPaddleBoat::read(payload);
    let _ = SInteract::read(payload);
    let _ = SKeepAlive::read(payload);
    let _ = SPlayerPosition::read(payload);
    let _ = SPlayerPositionRotation::read(payload);
    let _ = SPlayerRotation::read(payload);
    let _ = SSetPlayerGround::read(payload);
    let _ = SPickItemFromBlock::read(payload);
    let _ = SPlayerAbilities::read(payload);
    let _ = SPlayerAction::read(payload);
    let _ = SSetCommandBlock::read(payload);
    let _ = SPlayerCommand::read(payload);
    let _ = SPlayerLoaded::read(payload);
    let _ = SPlayPingRequest::read(payload);
    let _ = SClickSlot::read(payload);
    let _ = SSetHeldItem::read(payload);
    let _ = SSetCreativeSlot::read(payload);
    let _ = SSwingArm::read(payload);
    let _ = SUpdateSign::read(payload);
    let _ = SUseItemOn::read(payload);
    let _ = SUseItem::read(payload);
    let _ = SCommandSuggestion::read(payload);
    let _ = SCookieResponse::read(payload);
    let _ = SCloseContainer::read(payload);
    let _ = SChunkBatch::read(payload);
    let _ = SPlayerSession::read(payload);
    let _ = SCustomPayload::read(payload);
}

// ---------------------------------------------------------------------------
// Fuzz target
// ---------------------------------------------------------------------------
fuzz_target!(|data: &[u8]| {
    if data.len() < 18 {
        return;
    }

    // Byte layout:
    //  [0]     = decoder mode  (0=plain, 1=compressed, 2=encrypted, 3=both)
    //  [1]     = split point % rest.len() — how much goes to the decoder vs
    //            the deserializers, letting the fuzzer explore both paths
    //            independently with different byte patterns
    //  [2..18] = 16-byte AES key candidate
    //  [18..]  = raw bytes fed to both fuzzing paths
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

    fuzz_all_deserializers(deser_bytes);
});
