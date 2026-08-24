use std::{
    num::NonZero,
    sync::{Arc, atomic::Ordering},
};

use pumpkin_data::{
    data_component_impl::{
        BlocksAttacksImpl, ConsumableImpl, ConsumeAnimation, EquipmentSlot, EquippableImpl,
        FoodImpl,
    },
    item_stack::ItemStack,
};
use pumpkin_inventory::{
    player::player_inventory::PlayerInventory,
    screen_handler::{InventoryPlayer, ScreenHandler},
};
use pumpkin_protocol::bedrock::{
    client::{inventory_content::CInventoryContent, respawn::CRespawn},
    network_item::{
        ContainerName, FullContainerName, NetworkItemDescriptor, NetworkItemStackDescriptor,
    },
    respawn::RespawnState,
};
use pumpkin_protocol::{
    bedrock::{
        client::{
            chunk_radius_update::CChunkRadiusUpdate, container_open::CContainerOpen,
            player_hotbar::CPlayerHotbar, update_block::CUpdateBlock,
        },
        server::{
            actor_event::{ActorEventType, SActorEvent},
            animate::{AnimateAction, SAnimate},
            block_pick_request::SBlockPickRequest,
            command_request::SCommandRequest,
            container_close::SContainerClose,
            emote::SEmote,
            emote_list::SEmoteList,
            interaction::{Action, SInteraction},
            inventory_transaction::{SInventoryTransaction, TransactionData},
            mob_equipment::SMobEquipment,
            player_action::{Action as PlayerAction, SPlayerAction},
            player_auth_input::{InputData, SPlayerAuthInput},
            request_chunk_radius::SRequestChunkRadius,
            respawn::SRespawn,
            set_local_player_as_initialized::SSetLocalPlayerAsInitialized,
            text::SText,
        },
    },
    codec::{var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong},
    java::client::play::{Animation, CEntityAnimation, CSetSelectedSlot, CSystemChatMessage},
};
use pumpkin_util::{GameMode, Hand, math::position::BlockPos, text::TextComponent};

use pumpkin_world::inventory::Inventory;
use pumpkin_world::world::BlockFlags;

use crate::{
    block::{BlockHitResult, registry::BlockActionResult},
    entity::{
        EntityBase,
        player::{MINE_BLOCK_EXHAUSTION, Player},
    },
    net::{DisconnectReason, bedrock::BedrockClient},
    plugin::player::{
        item_held::PlayerItemHeldEvent,
        player_chat::PlayerChatEvent,
        player_command_send::PlayerCommandSendEvent,
        player_interact_event::{InteractAction, PlayerInteractEvent},
        player_toggle_flight_event::PlayerToggleFlightEvent,
    },
    server::{Server, seasonal_events},
    world::{BlockBreakingProgress, chunker},
};
use pumpkin_data::BlockDirection;
use tracing::{debug, info};

const MIN_PREDICTED_BREAK_PROGRESS: f32 = 0.65;

fn descriptor_to_stack(desc: &NetworkItemDescriptor) -> ItemStack {
    if desc.id.0 == 0 || desc.stack_size == 0 {
        ItemStack::EMPTY.clone()
    } else {
        pumpkin_data::item::JavaToBedrockItemMapping::from_bedrock(
            desc.id.0 as i16,
            desc.aux_value.0,
        )
        .map_or_else(
            || {
                tracing::warn!(
                    "Failed to map bedrock item id {} and data {} to Java item",
                    desc.id.0,
                    desc.aux_value.0
                );
                ItemStack::EMPTY.clone()
            },
            |mapping| ItemStack::new(desc.stack_size as u8, mapping.java_item),
        )
    }
}

const fn map_bedrock_slot_to_screen_handler(window_id: i32, slot: u32) -> Option<usize> {
    match window_id {
        0 => {
            // WINDOW_ID_INVENTORY
            if slot < 9 {
                // Hotbar: Bedrock 0-8 -> Screen Handler 36-44
                Some(slot as usize + 36)
            } else if slot < 36 {
                // Main Inventory: Bedrock 9-35 -> Screen Handler 9-35
                Some(slot as usize)
            } else {
                None
            }
        }
        120 => {
            // WINDOW_ID_ARMOUR
            if slot < 4 {
                // Armor: Bedrock 0-3 -> Screen Handler 5-8
                Some(slot as usize + 5)
            } else {
                None
            }
        }
        119 => {
            // WINDOW_ID_OFF_HAND
            if slot == 0 {
                // Offhand: Bedrock 0 -> Screen Handler 45
                Some(45)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub mod actor_event;
pub mod animate;
pub mod block_pick_request;
pub mod chat_command;
pub mod chat_message;
pub mod container_close;
pub mod emote;
pub mod emote_list;
pub mod interaction;
pub mod inventory_action;
pub mod item_stack_request;
pub(super) use item_stack_request::record_update;
pub mod mob_equipment;
pub mod modal_form_response;
pub mod player_action;
pub mod player_auth_input;
pub mod player_block_action;
pub mod request_ability;
pub mod request_chunk_radius;
pub mod respawn;
pub mod set_local_player_as_initialized;
