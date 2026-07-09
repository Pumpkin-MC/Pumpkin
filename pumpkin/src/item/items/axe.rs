use std::pin::Pin;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::{OakDoorLikeProperties, PaleOakWoodLikeProperties};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_data::{BlockDirection, BlockId};
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct AxeItem;

impl ItemMetadata for AxeItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_AXES.1.into()
    }
}

impl ItemBehaviour for AxeItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // I tried to follow mojang order of doing things.
            let world = player.world();
            let replacement_block = try_use_axe(block.id);
            // First we try to strip the block. by getting his equivalent and applying it the axis.

            // If there is a strip equivalent.
            let changed = if let Some(replacement) = replacement_block {
                let new_block = replacement.to_block();
                let new_state_id = if block.has_tag(&tag::Block::MINECRAFT_LOGS) {
                    let log_information = world.get_block_state_id(&location);
                    let log_props =
                        PaleOakWoodLikeProperties::from_state_id(log_information, block);
                    // create new properties for the new log.
                    let mut new_log_properties = PaleOakWoodLikeProperties::default(new_block);
                    new_log_properties.axis = log_props.axis;

                    // create new properties for the new log.

                    // Set old axis to the new log.
                    new_log_properties.axis = log_props.axis;
                    new_log_properties.to_state_id(new_block)
                }
                // Let's check if It's a door
                else if block.has_tag(&tag::Block::MINECRAFT_DOORS) {
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
                // TODO Implements trapdoors when It's implemented
                world
                    .set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL)
                    .await;
                true
            } else {
                false
            };

            if changed && player.gamemode.load() != GameMode::Creative {
                // TODO: Handle DamageResult::Broken to broadcast item break and update player slot.
                let _ = item.damage_item(1);
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
const fn try_use_axe(id: BlockId) -> Option<BlockId> {
    // Trying to get the strip equivalent
    if let Some(block) = get_stripped_equivalent(id) {
        return Some(block);
    }
    // Else decrease the level of oxidation
    if let Some(block) = get_deoxidized_equivalent(id) {
        return Some(block);
    }
    // Else unwax the block
    get_unwaxed_equivalent(id)
}

const fn get_stripped_equivalent(id: BlockId) -> Option<BlockId> {
    match id {
        BlockId::OAK_LOG => Some(BlockId::STRIPPED_OAK_LOG),
        BlockId::SPRUCE_LOG => Some(BlockId::STRIPPED_SPRUCE_LOG),
        BlockId::BIRCH_LOG => Some(BlockId::STRIPPED_BIRCH_LOG),
        BlockId::JUNGLE_LOG => Some(BlockId::STRIPPED_JUNGLE_LOG),
        BlockId::ACACIA_LOG => Some(BlockId::STRIPPED_ACACIA_LOG),
        BlockId::DARK_OAK_LOG => Some(BlockId::STRIPPED_DARK_OAK_LOG),
        BlockId::MANGROVE_LOG => Some(BlockId::STRIPPED_MANGROVE_LOG),
        BlockId::CHERRY_LOG => Some(BlockId::STRIPPED_CHERRY_LOG),
        BlockId::PALE_OAK_LOG => Some(BlockId::STRIPPED_PALE_OAK_LOG),
        BlockId::OAK_WOOD => Some(BlockId::STRIPPED_OAK_WOOD),
        BlockId::SPRUCE_WOOD => Some(BlockId::STRIPPED_SPRUCE_WOOD),
        BlockId::BIRCH_WOOD => Some(BlockId::STRIPPED_BIRCH_WOOD),
        BlockId::JUNGLE_WOOD => Some(BlockId::STRIPPED_JUNGLE_WOOD),
        BlockId::ACACIA_WOOD => Some(BlockId::STRIPPED_ACACIA_WOOD),
        BlockId::DARK_OAK_WOOD => Some(BlockId::STRIPPED_DARK_OAK_WOOD),
        BlockId::MANGROVE_WOOD => Some(BlockId::STRIPPED_MANGROVE_WOOD),
        BlockId::CHERRY_WOOD => Some(BlockId::STRIPPED_CHERRY_WOOD),
        BlockId::PALE_OAK_WOOD => Some(BlockId::STRIPPED_PALE_OAK_WOOD),
        _ => None,
    }
}

const fn get_deoxidized_equivalent(id: BlockId) -> Option<BlockId> {
    match id {
        BlockId::OXIDIZED_COPPER => Some(BlockId::WEATHERED_COPPER),
        BlockId::WEATHERED_COPPER => Some(BlockId::EXPOSED_COPPER),
        BlockId::EXPOSED_COPPER => Some(BlockId::COPPER_BLOCK),
        BlockId::OXIDIZED_CHISELED_COPPER => Some(BlockId::WEATHERED_CHISELED_COPPER),
        BlockId::WEATHERED_CHISELED_COPPER => Some(BlockId::EXPOSED_CHISELED_COPPER),
        BlockId::EXPOSED_CHISELED_COPPER => Some(BlockId::CHISELED_COPPER),
        BlockId::OXIDIZED_COPPER_GRATE => Some(BlockId::WEATHERED_COPPER_GRATE),
        BlockId::WEATHERED_COPPER_GRATE => Some(BlockId::EXPOSED_COPPER_GRATE),
        BlockId::EXPOSED_COPPER_GRATE => Some(BlockId::COPPER_GRATE),
        BlockId::OXIDIZED_CUT_COPPER => Some(BlockId::WEATHERED_CUT_COPPER),
        BlockId::WEATHERED_CUT_COPPER => Some(BlockId::EXPOSED_CUT_COPPER),
        BlockId::EXPOSED_CUT_COPPER => Some(BlockId::CUT_COPPER),
        BlockId::OXIDIZED_CUT_COPPER_STAIRS => Some(BlockId::WEATHERED_CUT_COPPER_STAIRS),
        BlockId::WEATHERED_CUT_COPPER_STAIRS => Some(BlockId::EXPOSED_CUT_COPPER_STAIRS),
        BlockId::EXPOSED_CUT_COPPER_STAIRS => Some(BlockId::CUT_COPPER_STAIRS),
        BlockId::OXIDIZED_CUT_COPPER_SLAB => Some(BlockId::WEATHERED_CUT_COPPER_SLAB),
        BlockId::WEATHERED_CUT_COPPER_SLAB => Some(BlockId::EXPOSED_CUT_COPPER_SLAB),
        BlockId::EXPOSED_CUT_COPPER_SLAB => Some(BlockId::CUT_COPPER_SLAB),
        BlockId::OXIDIZED_COPPER_BULB => Some(BlockId::WEATHERED_COPPER_BULB),
        BlockId::WEATHERED_COPPER_BULB => Some(BlockId::EXPOSED_COPPER_BULB),
        BlockId::EXPOSED_COPPER_BULB => Some(BlockId::COPPER_BULB),
        BlockId::OXIDIZED_COPPER_DOOR => Some(BlockId::WEATHERED_COPPER_DOOR),
        BlockId::WEATHERED_COPPER_DOOR => Some(BlockId::EXPOSED_COPPER_DOOR),
        BlockId::EXPOSED_COPPER_DOOR => Some(BlockId::COPPER_DOOR),
        BlockId::OXIDIZED_COPPER_TRAPDOOR => Some(BlockId::WEATHERED_COPPER_TRAPDOOR),
        BlockId::WEATHERED_COPPER_TRAPDOOR => Some(BlockId::EXPOSED_COPPER_TRAPDOOR),
        BlockId::EXPOSED_COPPER_TRAPDOOR => Some(BlockId::COPPER_TRAPDOOR),
        _ => None,
    }
}

const fn get_unwaxed_equivalent(id: BlockId) -> Option<BlockId> {
    match id {
        BlockId::WAXED_OXIDIZED_COPPER => Some(BlockId::OXIDIZED_COPPER),
        BlockId::WAXED_WEATHERED_COPPER => Some(BlockId::WEATHERED_COPPER),
        BlockId::WAXED_EXPOSED_COPPER => Some(BlockId::EXPOSED_COPPER),
        BlockId::WAXED_COPPER_BLOCK => Some(BlockId::COPPER_BLOCK),
        BlockId::WAXED_OXIDIZED_CHISELED_COPPER => Some(BlockId::OXIDIZED_CHISELED_COPPER),
        BlockId::WAXED_WEATHERED_CHISELED_COPPER => Some(BlockId::WEATHERED_CHISELED_COPPER),
        BlockId::WAXED_EXPOSED_CHISELED_COPPER => Some(BlockId::EXPOSED_CHISELED_COPPER),
        BlockId::WAXED_CHISELED_COPPER => Some(BlockId::CHISELED_COPPER),
        BlockId::WAXED_COPPER_GRATE => Some(BlockId::COPPER_GRATE),
        BlockId::WAXED_OXIDIZED_COPPER_GRATE => Some(BlockId::OXIDIZED_COPPER_GRATE),
        BlockId::WAXED_WEATHERED_COPPER_GRATE => Some(BlockId::WEATHERED_COPPER_GRATE),
        BlockId::WAXED_EXPOSED_COPPER_GRATE => Some(BlockId::EXPOSED_COPPER_GRATE),
        BlockId::WAXED_OXIDIZED_CUT_COPPER => Some(BlockId::OXIDIZED_CUT_COPPER),
        BlockId::WAXED_WEATHERED_CUT_COPPER => Some(BlockId::WEATHERED_CUT_COPPER),
        BlockId::WAXED_EXPOSED_CUT_COPPER => Some(BlockId::EXPOSED_CUT_COPPER),
        BlockId::WAXED_CUT_COPPER => Some(BlockId::CUT_COPPER),
        BlockId::WAXED_OXIDIZED_CUT_COPPER_STAIRS => Some(BlockId::OXIDIZED_CUT_COPPER_STAIRS),
        BlockId::WAXED_WEATHERED_CUT_COPPER_STAIRS => Some(BlockId::WEATHERED_CUT_COPPER_STAIRS),
        BlockId::WAXED_EXPOSED_CUT_COPPER_STAIRS => Some(BlockId::EXPOSED_CUT_COPPER_STAIRS),
        BlockId::WAXED_CUT_COPPER_STAIRS => Some(BlockId::CUT_COPPER_STAIRS),
        BlockId::WAXED_OXIDIZED_CUT_COPPER_SLAB => Some(BlockId::OXIDIZED_CUT_COPPER_SLAB),
        BlockId::WAXED_WEATHERED_CUT_COPPER_SLAB => Some(BlockId::WEATHERED_CUT_COPPER_SLAB),
        BlockId::WAXED_EXPOSED_CUT_COPPER_SLAB => Some(BlockId::EXPOSED_CUT_COPPER_SLAB),
        BlockId::WAXED_CUT_COPPER_SLAB => Some(BlockId::CUT_COPPER_SLAB),
        BlockId::WAXED_OXIDIZED_COPPER_BULB => Some(BlockId::OXIDIZED_COPPER_BULB),
        BlockId::WAXED_WEATHERED_COPPER_BULB => Some(BlockId::WEATHERED_COPPER_BULB),
        BlockId::WAXED_EXPOSED_COPPER_BULB => Some(BlockId::EXPOSED_COPPER_BULB),
        BlockId::WAXED_COPPER_BULB => Some(BlockId::COPPER_BULB),
        BlockId::WAXED_OXIDIZED_COPPER_DOOR => Some(BlockId::OXIDIZED_COPPER_DOOR),
        BlockId::WAXED_WEATHERED_COPPER_DOOR => Some(BlockId::WEATHERED_COPPER_DOOR),
        BlockId::WAXED_EXPOSED_COPPER_DOOR => Some(BlockId::EXPOSED_COPPER_DOOR),
        BlockId::WAXED_COPPER_DOOR => Some(BlockId::COPPER_DOOR),
        BlockId::WAXED_OXIDIZED_COPPER_TRAPDOOR => Some(BlockId::OXIDIZED_COPPER_TRAPDOOR),
        BlockId::WAXED_WEATHERED_COPPER_TRAPDOOR => Some(BlockId::WEATHERED_COPPER_TRAPDOOR),
        BlockId::WAXED_EXPOSED_COPPER_TRAPDOOR => Some(BlockId::EXPOSED_COPPER_TRAPDOOR),
        BlockId::WAXED_COPPER_TRAPDOOR => Some(BlockId::COPPER_TRAPDOOR),
        _ => None,
    }
}
