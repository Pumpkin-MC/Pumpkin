use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::block::UseWithItemArgs;
use crate::block::entities::BlockEntity;
use crate::block::entities::sign::SignBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::OakDoorLikeProperties;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::Taggable;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, tag};
use pumpkin_data::{BlockDirection, BlockId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
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
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();

            // First we try to strip the block. by getting his equivalent and applying it the axis.
            let replacement = get_waxed_equivalent(block.id);
            // If there is a strip equivalent.
            if let Some(replacement) = replacement {
                // get block state of the old log.
                // get the log properties
                // create new properties for the new log.
                let new_block = replacement.to_block();

                let new_state_id = if block.has_tag(&tag::Block::MINECRAFT_DOORS)
                    && block.has_tag(&tag::Block::MINECRAFT_DOORS)
                {
                    // get block state of the old log.
                    let door_information = world.get_block_state_id(&location);
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
    pub fn apply_to_sign(
        &self,
        args: &UseWithItemArgs<'_>,
        block_entity: &Arc<dyn BlockEntity>,
        sign_entity: &SignBlockEntity,
    ) -> BlockActionResult {
        sign_entity.is_waxed.store(true, Ordering::Relaxed);

        args.world.update_block_entity(block_entity);
        args.world
            .sync_world_event(WorldEvent::ParticlesAndSoundWaxOn, *args.position, 0);

        BlockActionResult::Success
    }
}

const fn get_waxed_equivalent(id: BlockId) -> Option<BlockId> {
    match id {
        BlockId::OXIDIZED_COPPER => Some(BlockId::WAXED_OXIDIZED_COPPER),
        BlockId::WEATHERED_COPPER => Some(BlockId::WAXED_WEATHERED_COPPER),
        BlockId::EXPOSED_COPPER => Some(BlockId::WAXED_EXPOSED_COPPER),
        BlockId::COPPER_BLOCK => Some(BlockId::WAXED_COPPER_BLOCK),
        BlockId::OXIDIZED_CHISELED_COPPER => Some(BlockId::WAXED_OXIDIZED_CHISELED_COPPER),
        BlockId::WEATHERED_CHISELED_COPPER => Some(BlockId::WAXED_WEATHERED_CHISELED_COPPER),
        BlockId::EXPOSED_CHISELED_COPPER => Some(BlockId::WAXED_EXPOSED_CHISELED_COPPER),
        BlockId::CHISELED_COPPER => Some(BlockId::WAXED_CHISELED_COPPER),
        BlockId::OXIDIZED_COPPER_GRATE => Some(BlockId::WAXED_OXIDIZED_COPPER_GRATE),
        BlockId::WEATHERED_COPPER_GRATE => Some(BlockId::WAXED_WEATHERED_COPPER_GRATE),
        BlockId::EXPOSED_COPPER_GRATE => Some(BlockId::WAXED_EXPOSED_COPPER_GRATE),
        BlockId::COPPER_GRATE => Some(BlockId::WAXED_COPPER_GRATE),
        BlockId::OXIDIZED_CUT_COPPER => Some(BlockId::WAXED_OXIDIZED_CUT_COPPER),
        BlockId::WEATHERED_CUT_COPPER => Some(BlockId::WAXED_WEATHERED_CUT_COPPER),
        BlockId::EXPOSED_CUT_COPPER => Some(BlockId::WAXED_EXPOSED_CUT_COPPER),
        BlockId::CUT_COPPER => Some(BlockId::WAXED_CUT_COPPER),
        BlockId::OXIDIZED_CUT_COPPER_STAIRS => Some(BlockId::WAXED_OXIDIZED_CUT_COPPER_STAIRS),
        BlockId::WEATHERED_CUT_COPPER_STAIRS => Some(BlockId::WAXED_WEATHERED_CUT_COPPER_STAIRS),
        BlockId::EXPOSED_CUT_COPPER_STAIRS => Some(BlockId::WAXED_EXPOSED_CUT_COPPER_STAIRS),
        BlockId::CUT_COPPER_STAIRS => Some(BlockId::WAXED_CUT_COPPER_STAIRS),
        BlockId::OXIDIZED_CUT_COPPER_SLAB => Some(BlockId::WAXED_OXIDIZED_CUT_COPPER_SLAB),
        BlockId::WEATHERED_CUT_COPPER_SLAB => Some(BlockId::WAXED_WEATHERED_CUT_COPPER_SLAB),
        BlockId::EXPOSED_CUT_COPPER_SLAB => Some(BlockId::WAXED_EXPOSED_CUT_COPPER_SLAB),
        BlockId::CUT_COPPER_SLAB => Some(BlockId::WAXED_CUT_COPPER_SLAB),
        BlockId::OXIDIZED_COPPER_BULB => Some(BlockId::WAXED_OXIDIZED_COPPER_BULB),
        BlockId::WEATHERED_COPPER_BULB => Some(BlockId::WAXED_WEATHERED_COPPER_BULB),
        BlockId::EXPOSED_COPPER_BULB => Some(BlockId::WAXED_EXPOSED_COPPER_BULB),
        BlockId::COPPER_BULB => Some(BlockId::WAXED_COPPER_BULB),
        BlockId::OXIDIZED_COPPER_DOOR => Some(BlockId::WAXED_OXIDIZED_COPPER_DOOR),
        BlockId::WEATHERED_COPPER_DOOR => Some(BlockId::WAXED_WEATHERED_COPPER_DOOR),
        BlockId::EXPOSED_COPPER_DOOR => Some(BlockId::WAXED_EXPOSED_COPPER_DOOR),
        BlockId::COPPER_DOOR => Some(BlockId::WAXED_COPPER_DOOR),
        BlockId::OXIDIZED_COPPER_TRAPDOOR => Some(BlockId::WAXED_OXIDIZED_COPPER_TRAPDOOR),
        BlockId::WEATHERED_COPPER_TRAPDOOR => Some(BlockId::WAXED_WEATHERED_COPPER_TRAPDOOR),
        BlockId::EXPOSED_COPPER_TRAPDOOR => Some(BlockId::WAXED_EXPOSED_COPPER_TRAPDOOR),
        BlockId::COPPER_TRAPDOOR => Some(BlockId::WAXED_COPPER_TRAPDOOR),
        _ => None,
    }
}
