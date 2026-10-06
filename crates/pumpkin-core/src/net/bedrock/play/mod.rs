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
use pumpkin_protocol::{
    bedrock::{
        client::{
            chunk_radius_updated::CChunkRadiusUpdated, container_open::CContainerOpen,
            inventory_content::CInventoryContent, player_hotbar::CPlayerHotbar,
            update_block::CUpdateBlock,
        },
        network_item::{
            ContainerName, FullContainerName, NetworkItemDescriptor, NetworkItemStackDescriptor,
        },
        server::{
            actor_event::{ActorEventID, SActorEvent},
            animate::{AnimateAction, SAnimate},
            block_pick_request::SBlockPickRequest,
            command_request::SCommandRequest,
            container_close::SContainerClose,
            emote::SEmote,
            emote_list::SEmoteList,
            interact::{Action, SInteract},
            inventory_transaction::{
                SInventoryTransaction, TransactionData, WINDOW_ID_ARMOUR, WINDOW_ID_INVENTORY,
                WINDOW_ID_OFF_HAND,
            },
            mob_equipment::SMobEquipment,
            player_action::{PlayerActionType as PlayerAction, SPlayerAction},
            player_auth_input::{InputData, SPlayerAuthInput},
            request_chunk_radius::SRequestChunkRadius,
            respawn::SRespawn,
            set_local_player_as_initialized::SSetLocalPlayerAsInitialized,
            text::SText,
        },
    },
    codec::{var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong},
    java::{
        client::play::{
            Animation, CEntityAnimation, CSetSelectedSlot, CSwingArm, CSystemChatMessage,
        },
        server::play::ActionType,
    },
};
use pumpkin_util::{GameMode, Hand, math::position::BlockPos, text::TextComponent};

use pumpkin_inventory::Inventory;
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
        player_interact_entity_event::PlayerInteractEntityEvent,
        player_interact_event::{InteractAction, PlayerInteractEvent},
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
        WINDOW_ID_INVENTORY => {
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
        WINDOW_ID_ARMOUR => {
            if slot < 4 {
                // Armor: Bedrock 0-3 -> Screen Handler 5-8
                Some(slot as usize + 5)
            } else {
                None
            }
        }
        WINDOW_ID_OFF_HAND => {
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

/// Bedrock container id, name and slot of a Java player-screen slot, the inverse of
/// [`map_bedrock_slot_to_screen_handler`]. Like Geyser's `PlayerInventoryTranslator`.
pub(crate) const fn bedrock_inventory_slot(
    player_screen_slot: i16,
) -> Option<(u32, ContainerName, u32)> {
    match player_screen_slot {
        5..=8 => Some((
            WINDOW_ID_ARMOUR as u32,
            ContainerName::Armor,
            (player_screen_slot - 5) as u32,
        )),
        9..=35 => Some((
            WINDOW_ID_INVENTORY as u32,
            ContainerName::Inventory,
            player_screen_slot as u32,
        )),
        36..=44 => Some((
            WINDOW_ID_INVENTORY as u32,
            ContainerName::Inventory,
            (player_screen_slot - 36) as u32,
        )),
        45 => Some((WINDOW_ID_OFF_HAND as u32, ContainerName::Offhand, 0)),
        _ => None,
    }
}

/// Same as [`bedrock_inventory_slot`] for a `PlayerInventory` index
pub(crate) const fn bedrock_player_inventory_slot(
    index: usize,
) -> Option<(u32, ContainerName, u32)> {
    match index {
        // Hotbar first on both sides
        0..=35 => Some((
            WINDOW_ID_INVENTORY as u32,
            ContainerName::Inventory,
            index as u32,
        )),
        // Feet to head here, head to feet on Bedrock
        36..=39 => bedrock_inventory_slot((44 - index) as i16),
        PlayerInventory::OFF_HAND_SLOT => bedrock_inventory_slot(45),
        _ => None,
    }
}

/// Sends one slot of the Bedrock player's own containers
pub(crate) fn send_bedrock_inventory_slot(
    bedrock: &BedrockClient,
    (container_id, container_name, slot): (u32, ContainerName, u32),
    stack: &ItemStack,
) {
    use pumpkin_protocol::bedrock::client::inventory_slot::CInventorySlot;

    bedrock.try_enqueue_client_packet(&CInventorySlot {
        container_id: VarUInt(container_id),
        slot: VarUInt(slot),
        full_container_name: Some(FullContainerName {
            container_name,
            dynamic_id: None,
        }),
        storage_item: None,
        item: NetworkItemStackDescriptor::from(stack),
    });
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

#[cfg(test)]
mod tests {
    use super::{
        ContainerName, WINDOW_ID_ARMOUR, WINDOW_ID_OFF_HAND, bedrock_inventory_slot,
        bedrock_player_inventory_slot, map_bedrock_slot_to_screen_handler,
    };

    #[test]
    fn player_screen_slots_round_trip_through_bedrock() {
        for slot in 0..=45 {
            let Some((container_id, _, bedrock_slot)) = bedrock_inventory_slot(slot) else {
                // The crafting grid has no Bedrock inventory slot
                assert!(slot < 5);
                continue;
            };
            assert_eq!(
                map_bedrock_slot_to_screen_handler(container_id as i32, bedrock_slot),
                Some(slot as usize)
            );
        }
    }

    #[test]
    fn player_inventory_slots_map_to_bedrock_containers() {
        assert_eq!(
            bedrock_player_inventory_slot(4),
            Some((0, ContainerName::Inventory, 4))
        );
        // Chest
        assert_eq!(
            bedrock_player_inventory_slot(38),
            Some((WINDOW_ID_ARMOUR as u32, ContainerName::Armor, 1))
        );
        // Head
        assert_eq!(
            bedrock_player_inventory_slot(39),
            Some((WINDOW_ID_ARMOUR as u32, ContainerName::Armor, 0))
        );
        assert_eq!(
            bedrock_player_inventory_slot(40),
            Some((WINDOW_ID_OFF_HAND as u32, ContainerName::Offhand, 0))
        );
    }
}
