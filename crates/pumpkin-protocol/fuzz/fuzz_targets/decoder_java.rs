#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin_protocol::ServerPacket;
use pumpkin_protocol::java::{
    client::{
        config::CFinishConfig,
        login::CLoginDisconnect,
        play::{CBlockEntityData, CPlayerPosition, CSetEquipment, CUpdateEntityPosRot},
        status::{CPingResponse, CStatusResponse},
    },
    packet_decoder::TCPNetworkDecoder,
    server::{
        config::{
            SAcknowledgeFinishConfig, SClientInformationConfig, SConfigCookieResponse,
            SConfigResourcePack, SCustomClickAction as SConfigCustomClickAction,
            SKeepAlive as SConfigKeepAlive, SKnownPacks, SPluginMessage,
        },
        handshake::SHandShake,
        login::{
            SEncryptionResponse, SLoginAcknowledged, SLoginCookieResponse, SLoginPluginResponse,
            SLoginStart,
        },
        play::{
            SAttack, SBundleItemSelected, SChangeGameMode, SChatCommand, SChatMessage, SChunkBatch,
            SClickSlot, SClientCommand, SClientInformationPlay, SClientTickEnd, SCloseContainer,
            SCommandSuggestion, SConfirmTeleport, SContainerButtonClick, SCookieResponse,
            SCustomClickAction as SPlayCustomClickAction, SCustomPayload, SDebugSampleSubscription,
            SDebugSubscriptionRequest, SEditBook, SInteract, SJigsawGenerate,
            SKeepAlive as SPlayKeepAlive, SMoveVehicle, SPaddleBoat, SPickItemFromBlock,
            SPickItemFromEntity, SPlaceRecipe, SPlayPingRequest, SPlayerAbilities, SPlayerAction,
            SPlayerCommand, SPlayerInput, SPlayerLoaded, SPlayerPosition, SPlayerPositionRotation,
            SPlayerRotation, SPlayerSession, SRecipeBookChangeSettings, SRecipeBookSeenRecipe,
            SRenameItem, SSeenAdvancement, SSelectTrade, SSetBeacon, SSetCommandBlock,
            SSetCreativeSlot, SSetHeldItem, SSetJigsawBlock, SSetPlayerGround, SSetTestBlock,
            SSwingArm, STeleportToEntity, STestInstanceBlockAction, SUpdateSign, SUseItem,
            SUseItemOn,
        },
        status::{SStatusPingRequest, SStatusRequest},
    },
};
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Cursor;
use tokio::runtime::Runtime;

const TARGET_VERSION: JavaMinecraftVersion = JavaMinecraftVersion::V_26_1;

// ---------------------------------------------------------------------------
// Helper: run every known ServerPacket::read against the same payload.
// Uses a slice and the Version enum as required by the ServerPacket signature.
// ---------------------------------------------------------------------------
fn fuzz_all_deserializers(payload: &[u8]) {
    macro_rules! run_read {
        ($($packet:ty),* $(,)?) => {
            $(
                let mut slice = payload;
                let _ = <$packet>::read(&mut slice, &TARGET_VERSION);
            )*
        };
    }

    run_read!(
        // Handshake
        SHandShake,
        // Status
        SStatusPingRequest,
        SStatusRequest,
        // Login
        SLoginStart,
        SEncryptionResponse,
        SLoginPluginResponse,
        SLoginCookieResponse,
        SLoginAcknowledged,
        // Config
        SAcknowledgeFinishConfig,
        SClientInformationConfig,
        SConfigCookieResponse,
        SConfigCustomClickAction,
        SConfigKeepAlive,
        SKnownPacks,
        SPluginMessage,
        SConfigResourcePack,
        // Play
        SAttack,
        SBundleItemSelected,
        SChangeGameMode,
        SChatCommand,
        SChatMessage,
        SChunkBatch,
        SClickSlot,
        SClientCommand,
        SClientInformationPlay,
        SClientTickEnd,
        SCloseContainer,
        SCommandSuggestion,
        SConfirmTeleport,
        SContainerButtonClick,
        SCookieResponse,
        SPlayCustomClickAction,
        SCustomPayload,
        SDebugSampleSubscription,
        SDebugSubscriptionRequest,
        SEditBook,
        SInteract,
        SJigsawGenerate,
        SPlayKeepAlive,
        SMoveVehicle,
        SPaddleBoat,
        SPickItemFromBlock,
        SPickItemFromEntity,
        SPlayPingRequest,
        SPlaceRecipe,
        SPlayerAbilities,
        SPlayerAction,
        SPlayerCommand,
        SPlayerInput,
        SPlayerLoaded,
        SPlayerPosition,
        SPlayerPositionRotation,
        SPlayerRotation,
        SPlayerSession,
        SRecipeBookChangeSettings,
        SRecipeBookSeenRecipe,
        SRenameItem,
        SSeenAdvancement,
        SSelectTrade,
        SSetBeacon,
        SSetCommandBlock,
        SSetCreativeSlot,
        SSetHeldItem,
        SSetJigsawBlock,
        SSetPlayerGround,
        SSetTestBlock,
        SSwingArm,
        STeleportToEntity,
        STestInstanceBlockAction,
        SUpdateSign,
        SUseItem,
        SUseItemOn,
        // Clientbound packets implementing ServerPacket
        CBlockEntityData,
        CFinishConfig,
        CLoginDisconnect,
        CPlayerPosition,
        CSetEquipment,
        CUpdateEntityPosRot,
        CPingResponse,
        CStatusResponse,
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
                let _ = decoder.set_encryption(&aes_key);
            }
            3 => {
                decoder.set_compression(256);
                let mut aes_key = [0u8; 16];
                aes_key.copy_from_slice(key);
                let _ = decoder.set_encryption(&aes_key);
            }
            _ => {}
        }
        let _ = decoder.get_raw_packet().await;
    });

    // --- Path 2: Individual Packet Deserializers ---------------------------
    fuzz_all_deserializers(deser_bytes);
});
