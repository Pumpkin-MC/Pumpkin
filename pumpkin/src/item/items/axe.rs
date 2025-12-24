use std::pin::Pin;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::{OakDoorLikeProperties, PaleOakWoodLikeProperties};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::BlockFlags;

pub struct AxeItem;

impl ItemMetadata for AxeItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_AXES.1.to_vec().into_boxed_slice()
    }
}

impl ItemBehaviour for AxeItem {
    #[allow(clippy::too_many_lines)]
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
            // I tried to follow mojang order of doing things.
            let world = player.world();
            let replacement_block = try_use_axe(block);
            // First we try to strip the block. by getting his equivalent and applying it the axis.

            // If there is a strip equivalent.
            if replacement_block != 0 {
                let new_block = &Block::from_id(replacement_block);
                let new_state_id = if block.has_tag(&tag::Block::MINECRAFT_LOGS) {
                    let log_information = world.get_block_state_id(&location).await;
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
                // TODO Implements trapdoors when It's implemented
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
fn try_use_axe(block: &Block) -> u16 {
    // Trying to get the strip equivalent
    let block_id = get_stripped_equivalent(block);
    if block_id != 0 {
        return block_id;
    }
    // Else decrease the level of oxidation
    let block_id = get_deoxidized_equivalent(block);
    if block_id != 0 {
        return block_id;
    }
    // Else unwax the block
    get_unwaxed_equivalent(block)
}

fn get_stripped_equivalent(block: &Block) -> u16 {
    match block.id {
        id if id == Block::OAK_LOG.id => Block::STRIPPED_OAK_LOG.id,
        id if id == Block::SPRUCE_LOG.id => Block::STRIPPED_SPRUCE_LOG.id,
        id if id == Block::BIRCH_LOG.id => Block::STRIPPED_BIRCH_LOG.id,
        id if id == Block::JUNGLE_LOG.id => Block::STRIPPED_JUNGLE_LOG.id,
        id if id == Block::ACACIA_LOG.id => Block::STRIPPED_ACACIA_LOG.id,
        id if id == Block::DARK_OAK_LOG.id => Block::STRIPPED_DARK_OAK_LOG.id,
        id if id == Block::MANGROVE_LOG.id => Block::STRIPPED_MANGROVE_LOG.id,
        id if id == Block::CHERRY_LOG.id => Block::STRIPPED_CHERRY_LOG.id,
        id if id == Block::PALE_OAK_LOG.id => Block::STRIPPED_PALE_OAK_LOG.id,

        id if id == Block::OAK_WOOD.id => Block::STRIPPED_OAK_WOOD.id,
        id if id == Block::SPRUCE_WOOD.id => Block::STRIPPED_SPRUCE_WOOD.id,
        id if id == Block::BIRCH_WOOD.id => Block::STRIPPED_BIRCH_WOOD.id,
        id if id == Block::JUNGLE_WOOD.id => Block::STRIPPED_JUNGLE_WOOD.id,
        id if id == Block::ACACIA_WOOD.id => Block::STRIPPED_ACACIA_WOOD.id,
        id if id == Block::DARK_OAK_WOOD.id => Block::STRIPPED_DARK_OAK_WOOD.id,
        id if id == Block::MANGROVE_WOOD.id => Block::STRIPPED_MANGROVE_WOOD.id,
        id if id == Block::CHERRY_WOOD.id => Block::STRIPPED_CHERRY_WOOD.id,
        id if id == Block::PALE_OAK_WOOD.id => Block::STRIPPED_PALE_OAK_WOOD.id,
        _ => 0,
    }
}

fn get_deoxidized_equivalent(block: &Block) -> u16 {
    match block.id {
        id if id == Block::OXIDIZED_COPPER.id => Block::WEATHERED_COPPER.id,
        id if id == Block::WEATHERED_COPPER.id => Block::EXPOSED_COPPER.id,
        id if id == Block::EXPOSED_COPPER.id => Block::COPPER_BLOCK.id,
        id if id == Block::OXIDIZED_CHISELED_COPPER.id => Block::WEATHERED_CHISELED_COPPER.id,
        id if id == Block::WEATHERED_CHISELED_COPPER.id => Block::EXPOSED_CHISELED_COPPER.id,
        id if id == Block::EXPOSED_CHISELED_COPPER.id => Block::CHISELED_COPPER.id,
        id if id == Block::OXIDIZED_COPPER_GRATE.id => Block::WEATHERED_COPPER_GRATE.id,
        id if id == Block::WEATHERED_COPPER_GRATE.id => Block::EXPOSED_COPPER_GRATE.id,
        id if id == Block::EXPOSED_COPPER_GRATE.id => Block::COPPER_GRATE.id,
        id if id == Block::OXIDIZED_CUT_COPPER.id => Block::WEATHERED_CUT_COPPER.id,
        id if id == Block::WEATHERED_CUT_COPPER.id => Block::EXPOSED_CUT_COPPER.id,
        id if id == Block::EXPOSED_CUT_COPPER.id => Block::CUT_COPPER.id,
        id if id == Block::OXIDIZED_CUT_COPPER_STAIRS.id => Block::WEATHERED_CUT_COPPER_STAIRS.id,
        id if id == Block::WEATHERED_CUT_COPPER_STAIRS.id => Block::EXPOSED_CUT_COPPER_STAIRS.id,
        id if id == Block::EXPOSED_CUT_COPPER_STAIRS.id => Block::CUT_COPPER_STAIRS.id,
        id if id == Block::OXIDIZED_CUT_COPPER_SLAB.id => Block::WEATHERED_CUT_COPPER_SLAB.id,
        id if id == Block::WEATHERED_CUT_COPPER_SLAB.id => Block::EXPOSED_CUT_COPPER_SLAB.id,
        id if id == Block::EXPOSED_CUT_COPPER_SLAB.id => Block::CUT_COPPER_SLAB.id,
        id if id == Block::OXIDIZED_COPPER_BULB.id => Block::WEATHERED_COPPER_BULB.id,
        id if id == Block::WEATHERED_COPPER_BULB.id => Block::EXPOSED_COPPER_BULB.id,
        id if id == Block::EXPOSED_COPPER_BULB.id => Block::COPPER_BULB.id,
        id if id == Block::OXIDIZED_COPPER_DOOR.id => Block::WEATHERED_COPPER_DOOR.id,
        id if id == Block::WEATHERED_COPPER_DOOR.id => Block::EXPOSED_COPPER_DOOR.id,
        id if id == Block::EXPOSED_COPPER_DOOR.id => Block::COPPER_DOOR.id,
        id if id == Block::OXIDIZED_COPPER_TRAPDOOR.id => Block::WEATHERED_COPPER_TRAPDOOR.id,
        id if id == Block::WEATHERED_COPPER_TRAPDOOR.id => Block::EXPOSED_COPPER_TRAPDOOR.id,
        id if id == Block::EXPOSED_COPPER_TRAPDOOR.id => Block::COPPER_TRAPDOOR.id,
        _ => 0,
    }
}

fn get_unwaxed_equivalent(block: &Block) -> u16 {
    match &block.id {
        id if id == &Block::WAXED_OXIDIZED_COPPER.id => Block::OXIDIZED_COPPER.id,
        id if id == &Block::WAXED_WEATHERED_COPPER.id => Block::WEATHERED_COPPER.id,
        id if id == &Block::WAXED_EXPOSED_COPPER.id => Block::EXPOSED_COPPER.id,
        id if id == &Block::WAXED_COPPER_BLOCK.id => Block::COPPER_BLOCK.id,
        id if id == &Block::WAXED_OXIDIZED_CHISELED_COPPER.id => Block::OXIDIZED_CHISELED_COPPER.id,
        id if id == &Block::WAXED_WEATHERED_CHISELED_COPPER.id => {
            Block::WEATHERED_CHISELED_COPPER.id
        }
        id if id == &Block::WAXED_EXPOSED_CHISELED_COPPER.id => Block::EXPOSED_CHISELED_COPPER.id,
        id if id == &Block::WAXED_CHISELED_COPPER.id => Block::CHISELED_COPPER.id,
        id if id == &Block::WAXED_COPPER_GRATE.id => Block::COPPER_GRATE.id,
        id if id == &Block::WAXED_OXIDIZED_COPPER_GRATE.id => Block::OXIDIZED_COPPER_GRATE.id,
        id if id == &Block::WAXED_WEATHERED_COPPER_GRATE.id => Block::WEATHERED_COPPER_GRATE.id,
        id if id == &Block::WAXED_EXPOSED_COPPER_GRATE.id => Block::EXPOSED_COPPER_GRATE.id,
        id if id == &Block::WAXED_OXIDIZED_CUT_COPPER.id => Block::OXIDIZED_CUT_COPPER.id,
        id if id == &Block::WAXED_WEATHERED_CUT_COPPER.id => Block::WEATHERED_CUT_COPPER.id,
        id if id == &Block::WAXED_EXPOSED_CUT_COPPER.id => Block::EXPOSED_CUT_COPPER.id,
        id if id == &Block::WAXED_CUT_COPPER.id => Block::CUT_COPPER.id,
        id if id == &Block::WAXED_OXIDIZED_CUT_COPPER_STAIRS.id => {
            Block::OXIDIZED_CUT_COPPER_STAIRS.id
        }
        id if id == &Block::WAXED_WEATHERED_CUT_COPPER_STAIRS.id => {
            Block::WEATHERED_CUT_COPPER_STAIRS.id
        }
        id if id == &Block::WAXED_EXPOSED_CUT_COPPER_STAIRS.id => {
            Block::EXPOSED_CUT_COPPER_STAIRS.id
        }
        id if id == &Block::WAXED_CUT_COPPER_STAIRS.id => Block::CUT_COPPER_STAIRS.id,
        id if id == &Block::WAXED_OXIDIZED_CUT_COPPER_SLAB.id => Block::OXIDIZED_CUT_COPPER_SLAB.id,
        id if id == &Block::WAXED_WEATHERED_CUT_COPPER_SLAB.id => {
            Block::WEATHERED_CUT_COPPER_SLAB.id
        }
        id if id == &Block::WAXED_EXPOSED_CUT_COPPER_SLAB.id => Block::EXPOSED_CUT_COPPER_SLAB.id,
        id if id == &Block::WAXED_CUT_COPPER_SLAB.id => Block::CUT_COPPER_SLAB.id,
        id if id == &Block::WAXED_OXIDIZED_COPPER_BULB.id => Block::OXIDIZED_COPPER_BULB.id,
        id if id == &Block::WAXED_WEATHERED_COPPER_BULB.id => Block::WEATHERED_COPPER_BULB.id,
        id if id == &Block::WAXED_EXPOSED_COPPER_BULB.id => Block::EXPOSED_COPPER_BULB.id,
        id if id == &Block::WAXED_COPPER_BULB.id => Block::COPPER_BULB.id,
        id if id == &Block::WAXED_OXIDIZED_COPPER_DOOR.id => Block::OXIDIZED_COPPER_DOOR.id,
        id if id == &Block::WAXED_WEATHERED_COPPER_DOOR.id => Block::WEATHERED_COPPER_DOOR.id,
        id if id == &Block::WAXED_EXPOSED_COPPER_DOOR.id => Block::EXPOSED_COPPER_DOOR.id,
        id if id == &Block::WAXED_COPPER_DOOR.id => Block::COPPER_DOOR.id,
        id if id == &Block::WAXED_OXIDIZED_COPPER_TRAPDOOR.id => Block::OXIDIZED_COPPER_TRAPDOOR.id,
        id if id == &Block::WAXED_WEATHERED_COPPER_TRAPDOOR.id => {
            Block::WEATHERED_COPPER_TRAPDOOR.id
        }
        id if id == &Block::WAXED_EXPOSED_COPPER_TRAPDOOR.id => Block::EXPOSED_COPPER_TRAPDOOR.id,
        id if id == &Block::WAXED_COPPER_TRAPDOOR.id => Block::COPPER_TRAPDOOR.id,
        _ => 0,
    }
}
