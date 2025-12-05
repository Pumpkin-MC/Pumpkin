use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::block::UseWithItemArgs;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::OakDoorLikeProperties;
use pumpkin_data::item::Item;
use pumpkin_data::tag::Taggable;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::entities::BlockEntity;
use pumpkin_world::block::entities::sign::SignBlockEntity;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::BlockFlags;

pub struct HoneyCombItem;

impl ItemMetadata for HoneyCombItem {
    fn ids() -> Box<[u16]> {
        [Item::HONEYCOMB.id].into()
    }
}

impl ItemBehaviour for HoneyCombItem {
    fn use_on_block<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();

            // First we try to strip the block. by getting his equivalent and applying it the axis.
            let replacement_block = get_waxed_equivalent(block);
            // If there is a strip equivalent.
            if let Some(replacement_block) = replacement_block {
                // get block state of the old log.
                // get the log properties
                // create new properties for the new log.
                let new_block = &Block::from_id(replacement_block);

                let new_state_id = if block.has_tag(&tag::Block::MINECRAFT_DOORS)
                    && block.has_tag(&tag::Block::MINECRAFT_DOORS)
                {
                    // get block state of the old log.
                    let door_information = world.get_block_state_id(&location).await;
                    // get the log properties
                    let door_props = OakDoorLikeProperties::from_state_id(door_information, block);
                    // create new properties for the new log.
                    let mut new_door_properties = OakDoorLikeProperties::default(new_block);
                    // Set old axis to the new log.
                    new_door_properties.facing = door_props.facing;
                    new_door_properties.open = door_props.open;
                    new_door_properties.half = door_props.half;
                    new_door_properties.hinge = door_props.hinge;
                    new_door_properties.powered = door_props.powered;
                    new_door_properties.to_state_id(new_block)
                } else {
                    new_block.default_state.id
                };

                // TODO Implements trapdoors
                world
                    .set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl HoneyCombItem {
    pub async fn apply_to_sign(
        &self,
        args: &UseWithItemArgs<'_>,
        block_entity: &Arc<dyn BlockEntity>,
        sign_entity: &SignBlockEntity,
    ) -> BlockActionResult {
        sign_entity.is_waxed.store(true, Ordering::Relaxed);

        args.world.update_block_entity(block_entity).await;
        args.world
            .sync_world_event(WorldEvent::BlockWaxed, *args.position, 0)
            .await;

        BlockActionResult::Success
    }
}

fn get_waxed_equivalent(block: &Block) -> Option<u16> {
    match &block.id {
        id if id == &Block::OXIDIZED_COPPER.id => Some(Block::WAXED_OXIDIZED_COPPER.id),
        id if id == &Block::WEATHERED_COPPER.id => Some(Block::WAXED_WEATHERED_COPPER.id),
        id if id == &Block::EXPOSED_COPPER.id => Some(Block::WAXED_EXPOSED_COPPER.id),
        id if id == &Block::COPPER_BLOCK.id => Some(Block::WAXED_COPPER_BLOCK.id),
        id if id == &Block::OXIDIZED_CHISELED_COPPER.id => {
            Some(Block::WAXED_OXIDIZED_CHISELED_COPPER.id)
        }
        id if id == &Block::WEATHERED_CHISELED_COPPER.id => {
            Some(Block::WAXED_WEATHERED_CHISELED_COPPER.id)
        }
        id if id == &Block::EXPOSED_CHISELED_COPPER.id => {
            Some(Block::WAXED_EXPOSED_CHISELED_COPPER.id)
        }
        id if id == &Block::CHISELED_COPPER.id => Some(Block::WAXED_CHISELED_COPPER.id),
        id if id == &Block::OXIDIZED_COPPER_GRATE.id => Some(Block::WAXED_OXIDIZED_COPPER_GRATE.id),
        id if id == &Block::WEATHERED_COPPER_GRATE.id => {
            Some(Block::WAXED_WEATHERED_COPPER_GRATE.id)
        }
        id if id == &Block::EXPOSED_COPPER_GRATE.id => Some(Block::WAXED_EXPOSED_COPPER_GRATE.id),
        id if id == &Block::COPPER_GRATE.id => Some(Block::WAXED_COPPER_GRATE.id),
        id if id == &Block::OXIDIZED_CUT_COPPER.id => Some(Block::WAXED_OXIDIZED_CUT_COPPER.id),
        id if id == &Block::WEATHERED_CUT_COPPER.id => Some(Block::WAXED_WEATHERED_CUT_COPPER.id),
        id if id == &Block::EXPOSED_CUT_COPPER.id => Some(Block::WAXED_EXPOSED_CUT_COPPER.id),
        id if id == &Block::CUT_COPPER.id => Some(Block::WAXED_CUT_COPPER.id),
        id if id == &Block::OXIDIZED_CUT_COPPER_STAIRS.id => {
            Some(Block::WAXED_OXIDIZED_CUT_COPPER_STAIRS.id)
        }
        id if id == &Block::WEATHERED_CUT_COPPER_STAIRS.id => {
            Some(Block::WAXED_WEATHERED_CUT_COPPER_STAIRS.id)
        }
        id if id == &Block::EXPOSED_CUT_COPPER_STAIRS.id => {
            Some(Block::WAXED_EXPOSED_CUT_COPPER_STAIRS.id)
        }
        id if id == &Block::CUT_COPPER_STAIRS.id => Some(Block::WAXED_CUT_COPPER_STAIRS.id),
        id if id == &Block::OXIDIZED_CUT_COPPER_SLAB.id => {
            Some(Block::WAXED_OXIDIZED_CUT_COPPER_SLAB.id)
        }
        id if id == &Block::WEATHERED_CUT_COPPER_SLAB.id => {
            Some(Block::WAXED_WEATHERED_CUT_COPPER_SLAB.id)
        }
        id if id == &Block::EXPOSED_CUT_COPPER_SLAB.id => {
            Some(Block::WAXED_EXPOSED_CUT_COPPER_SLAB.id)
        }
        id if id == &Block::CUT_COPPER_SLAB.id => Some(Block::WAXED_CUT_COPPER_SLAB.id),
        id if id == &Block::OXIDIZED_COPPER_BULB.id => Some(Block::WAXED_OXIDIZED_COPPER_BULB.id),
        id if id == &Block::WEATHERED_COPPER_BULB.id => Some(Block::WAXED_WEATHERED_COPPER_BULB.id),
        id if id == &Block::EXPOSED_COPPER_BULB.id => Some(Block::WAXED_EXPOSED_COPPER_BULB.id),
        id if id == &Block::COPPER_BULB.id => Some(Block::WAXED_COPPER_BULB.id),
        id if id == &Block::OXIDIZED_COPPER_DOOR.id => Some(Block::WAXED_OXIDIZED_COPPER_DOOR.id),
        id if id == &Block::WEATHERED_COPPER_DOOR.id => Some(Block::WAXED_WEATHERED_COPPER_DOOR.id),
        id if id == &Block::EXPOSED_COPPER_DOOR.id => Some(Block::WAXED_EXPOSED_COPPER_DOOR.id),
        id if id == &Block::COPPER_DOOR.id => Some(Block::WAXED_COPPER_DOOR.id),
        id if id == &Block::OXIDIZED_COPPER_TRAPDOOR.id => {
            Some(Block::WAXED_OXIDIZED_COPPER_TRAPDOOR.id)
        }
        id if id == &Block::WEATHERED_COPPER_TRAPDOOR.id => {
            Some(Block::WAXED_WEATHERED_COPPER_TRAPDOOR.id)
        }
        id if id == &Block::EXPOSED_COPPER_TRAPDOOR.id => {
            Some(Block::WAXED_EXPOSED_COPPER_TRAPDOOR.id)
        }
        id if id == &Block::COPPER_TRAPDOOR.id => Some(Block::WAXED_COPPER_TRAPDOOR.id),
        _ => None,
    }
}
