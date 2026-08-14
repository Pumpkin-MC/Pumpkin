use pumpkin_protocol::bedrock::server::text::SText;
use pumpkin_util::{Hand, PermissionLvl};
use rsa::pkcs1v15::{Signature as RsaPkcs1v15Signature, VerifyingKey};
use rsa::signature::Verifier;
use sha1::Sha1;
use std::num::NonZeroU8;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{Level, debug, error, info, trace, warn};

use crate::block::BlockHitResult;
use crate::block::registry::BlockActionResult;
use crate::block::{self};
use crate::entity::EntityBase;
use crate::entity::equipment_break_status;
use crate::entity::player::statistics::{CustomStatistic, StatisticCategory};
use crate::entity::player::{ChatMode, ChatSession, MINE_BLOCK_EXHAUSTION, Player};
use crate::error::PumpkinError;
use crate::log_at_level;
use crate::net::PlayerConfig;
use crate::net::java::JavaClient;
use crate::plugin::player::changed_main_hand::PlayerChangedMainHandEvent;
use crate::plugin::player::fish::{PlayerFishEvent, PlayerFishState};
use crate::plugin::player::item_held::PlayerItemHeldEvent;
use crate::plugin::player::player_chat::PlayerChatEvent;
use crate::plugin::player::player_command_send::PlayerCommandSendEvent;
use crate::plugin::player::player_interact_entity_event::PlayerInteractEntityEvent;
use crate::plugin::player::player_interact_event::{InteractAction, PlayerInteractEvent};
use crate::plugin::player::player_interact_unknown_entity_event::PlayerInteractUnknownEntityEvent;
use crate::plugin::player::player_move::PlayerMoveEvent;
use crate::plugin::player::player_toggle_flight_event::PlayerToggleFlightEvent;
use crate::plugin::player::player_toggle_sneak_event::PlayerToggleSneakEvent;

use crate::block::entities::command_block::CommandBlockEntity;
use crate::block::entities::jigsaw_block::JigsawBlockEntity;
use crate::block::entities::sign::SignBlockEntity;
use crate::plugin::player::player_toggle_sprint_event::PlayerToggleSprintEvent;
use crate::server::{Server, seasonal_events};
use crate::world::{BlockBreakingProgress, World, chunker};
use pumpkin_data::block_properties::{BlockProperties, CommandBlockLikeProperties};
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    BlocksAttacksImpl, ConsumableImpl, DataComponentImpl, EquipmentSlot, EquippableImpl, FoodImpl,
    WritableBookContentImpl, WrittenBookContentImpl,
};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Advancement, Block, BlockDirection, translation};
use pumpkin_inventory::InventoryError;
use pumpkin_inventory::merchant::merchant_screen_handler::MerchantScreenHandler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler};
use pumpkin_protocol::bedrock::client::CMovePlayer;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::{
    CBlockUpdate, CCommandSuggestions, CEntityPositionSync, CHeadRot, COpenSignEditor,
    CPingResponse, CPlayerInfoUpdate, CPlayerPosition, CSetCamera, CSetSelectedSlot,
    CSystemChatMessage, CUpdateEntityPos, CUpdateEntityPosRot, CUpdateEntityRot, InitChat,
    PlayerAction,
};
use pumpkin_protocol::java::server::play::{
    Action, ActionType, CommandBlockMode, FLAG_ON_GROUND, SAttack, SBundleItemSelected,
    SChangeGameMode, SChatCommand, SChatMessage, SChunkBatch, SClientCommand,
    SClientInformationPlay, SCloseContainer, SCommandSuggestion, SConfirmTeleport,
    SCookieResponse as SPCookieResponse, SEditBook, SInteract, SJigsawGenerate, SKeepAlive,
    SMoveVehicle, SPaddleBoat, SPickItemFromBlock, SPickItemFromEntity, SPlaceRecipe,
    SPlayPingRequest, SPlayerAbilities, SPlayerAction, SPlayerCommand, SPlayerInput,
    SPlayerPosition, SPlayerPositionRotation, SPlayerRotation, SPlayerSession,
    SRecipeBookChangeSettings, SRecipeBookSeenRecipe, SSeenAdvancement, SSelectTrade,
    SSetCommandBlock, SSetCreativeSlot, SSetHeldItem, SSetJigsawBlock, SSetPlayerGround,
    SSetTestBlock, SSwingArm, STeleportToEntity, STestInstanceBlockAction, SUpdateSign, SUseItem,
    SUseItemOn, Status,
};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{polynomial_rolling_hash, position::BlockPos, wrap_degrees};
use pumpkin_util::{GameMode, text::TextComponent};
use pumpkin_world::generation::structure::structures::jigsaw::JigsawJointType;
use pumpkin_world::world::BlockFlags;
use tokio::sync::Mutex;

/// In secure chat mode, Player will be kicked if they send a chat message with a timestamp that is older than this (in ms)
/// Vanilla: 2 minutes
const CHAT_MESSAGE_MAX_AGE: i64 = 1000 * 60 * 2;

#[derive(Debug, Error)]
pub enum BlockPlacingError {
    BlockOutOfReach,
    InvalidHand,
    InvalidBlockFace,
    BlockOutOfWorld,
    InvalidGamemode,
}

impl std::fmt::Display for BlockPlacingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PumpkinError for BlockPlacingError {
    fn is_kick(&self) -> bool {
        match self {
            Self::BlockOutOfReach | Self::BlockOutOfWorld | Self::InvalidGamemode => false,
            Self::InvalidBlockFace | Self::InvalidHand => true,
        }
    }

    fn severity(&self) -> Level {
        match self {
            Self::BlockOutOfWorld | Self::InvalidGamemode => Level::TRACE,
            Self::BlockOutOfReach | Self::InvalidBlockFace | Self::InvalidHand => Level::WARN,
        }
    }

    fn client_kick_reason(&self) -> Option<String> {
        match self {
            Self::BlockOutOfReach | Self::BlockOutOfWorld | Self::InvalidGamemode => None,
            Self::InvalidBlockFace => Some("Invalid block face".into()),
            Self::InvalidHand => Some("Invalid hand".into()),
        }
    }
}

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("sent an oversized message")]
    OversizedMessage,
    #[error("sent a message with illegal characters")]
    IllegalCharacters,
    #[error("sent a chat with invalid/no signature")]
    UnsignedChat,
    #[error("has too many unacknowledged chats queued")]
    TooManyPendingChats,
    #[error("sent a chat that couldn't be validated")]
    ChatValidationFailed,
    #[error("sent a chat with an out of order timestamp")]
    OutOfOrderChat,
    #[error("has an expired public key")]
    ExpiredPublicKey,
    #[error("attempted to initialize a session with an invalid public key")]
    InvalidPublicKey,
}

impl PumpkinError for ChatError {
    fn is_kick(&self) -> bool {
        true
    }

    fn severity(&self) -> Level {
        Level::WARN
    }

    fn client_kick_reason(&self) -> Option<String> {
        match self {
            Self::OversizedMessage => Some("Chat message too long".into()),
            Self::IllegalCharacters => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_ILLEGAL_CHARACTERS,
                    translation::java::MULTIPLAYER_DISCONNECT_ILLEGAL_CHARACTERS,
                    [],
                )
                .get_text(),
            ),
            Self::UnsignedChat => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_UNSIGNED_CHAT,
                    translation::java::MULTIPLAYER_DISCONNECT_UNSIGNED_CHAT,
                    [],
                )
                .get_text(),
            ),
            Self::TooManyPendingChats => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_TOO_MANY_PENDING_CHATS,
                    translation::java::MULTIPLAYER_DISCONNECT_TOO_MANY_PENDING_CHATS,
                    [],
                )
                .get_text(),
            ),
            Self::ChatValidationFailed => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_CHAT_VALIDATION_FAILED,
                    translation::java::MULTIPLAYER_DISCONNECT_CHAT_VALIDATION_FAILED,
                    [],
                )
                .get_text(),
            ),
            Self::OutOfOrderChat => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_OUT_OF_ORDER_CHAT,
                    translation::java::MULTIPLAYER_DISCONNECT_OUT_OF_ORDER_CHAT,
                    [],
                )
                .get_text(),
            ),
            Self::ExpiredPublicKey => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_EXPIRED_PUBLIC_KEY,
                    translation::java::MULTIPLAYER_DISCONNECT_EXPIRED_PUBLIC_KEY,
                    [],
                )
                .get_text(),
            ),
            Self::InvalidPublicKey => Some(
                TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_INVALID_PUBLIC_KEY_SIGNATURE,
                    translation::java::MULTIPLAYER_DISCONNECT_INVALID_PUBLIC_KEY_SIGNATURE,
                    [],
                )
                .get_text(),
            ),
        }
    }
}

pub mod attack;
pub mod bundle_item_selected;
pub mod change_game_mode;
pub mod chat_command;
pub mod chat_message;
pub mod chunk_batch;
pub mod client_command;
pub mod client_information;
pub mod close_container;
pub mod command_suggestion;
pub mod confirm_teleport;
pub mod cookie_response;
pub mod debug_sample_subscription;
pub mod debug_subscription_request;
pub mod edit_book;
pub mod interact;
pub mod jigsaw_generate;
pub mod keep_alive;
pub mod move_vehicle;
pub mod paddle_boat;
pub mod pick_item;
pub mod ping_request;
pub mod place_recipe;
pub mod player_abilities;
pub mod player_action;
pub mod player_command;
pub mod player_ground;
pub mod player_input;
pub mod player_loaded;
pub mod player_position;
pub mod player_rotation;
pub mod recipe_book_change_settings;
pub mod recipe_book_seen_recipe;
pub mod seen_advancement;
pub mod select_trade;
pub mod set_command_block;
pub mod set_creative_slot;
pub mod set_held_item;
pub mod set_jigsaw_block;
pub mod set_test_block;
pub mod swing_arm;
pub mod teleport_to_entity;
pub mod test_instance_block_action;
pub mod update_sign;
pub mod use_item;
pub mod use_item_on;
