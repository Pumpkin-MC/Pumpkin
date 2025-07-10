use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::{OakDoorLikeProperties, PaleOakWoodLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

pub struct AxeItem;

impl ItemMetadata for AxeItem {
    fn ids() -> Box<[u16]> {
        Item::get_tag_values("#minecraft:axes")
            .expect("This is a valid vanilla tag")
            .iter()
            .map(|key| {
                Item::from_registry_key(key)
                    .expect("We just got this key from the registry")
                    .id
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

#[async_trait]
impl PumpkinItem for AxeItem {
    #[allow(clippy::too_many_lines)]
    async fn use_on_block(
        &self,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        block: &Block,
        _server: &Server,
    ) {
        // I tried to follow mojang order of doing things.
        let world = player.world().await;
        let replacement_block = try_use_axe(block);
        // First we try to strip the block. by getting his equivalent and applying it the axis.

        // If there is a strip equivalent.
        if let Some(replacement_block) = replacement_block {
            let new_block = Block::from_id(replacement_block);
            let new_block = &new_block;
            let new_state_id = if block.is_tagged_with("#minecraft:logs") == Some(true) {
                let log_information = world.get_block_state_id(&location).await;
                let log_props = PaleOakWoodLikeProperties::from_state_id(log_information, block);
                // create new properties for the new log.
                let mut new_log_properties = PaleOakWoodLikeProperties::default(new_block);
                new_log_properties.axis = log_props.axis;

                // create new properties for the new log.

                // Set old axis to the new log.
                new_log_properties.axis = log_props.axis;
                new_log_properties.to_state_id(new_block)
            }
            // Let's check if It's a door
            else if block.is_tagged_with("#minecraft:doors") == Some(true) {
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
            return;
        }
    }
}
fn try_use_axe(block: &Block) -> Option<u16> {
    // Trying to get the strip equivalent
    let block_id = get_stripped_equivalent(block);
    if block_id.is_some() {
        return block_id;
    }
    // Else decrease the level of oxidation
    let block_id = get_deoxidized_equivalent(block);
    if block_id.is_some() {
        return block_id;
    }
    // Else unwax the block
    let block_id = get_unwaxed_equivalent(block);
    if block_id.is_some() {
        return block_id;
    }
    None
}

fn get_stripped_equivalent(block: &Block) -> Option<u16> {
    let new_block_id = match &block.id {
        id if id == &Block::OAK_LOG.id => Block::STRIPPED_OAK_LOG.id,
        id if id == &Block::SPRUCE_LOG.id => Block::STRIPPED_SPRUCE_LOG.id,
        id if id == &Block::BIRCH_LOG.id => Block::STRIPPED_BIRCH_LOG.id,
        id if id == &Block::JUNGLE_LOG.id => Block::STRIPPED_JUNGLE_LOG.id,
        id if id == &Block::ACACIA_LOG.id => Block::STRIPPED_ACACIA_LOG.id,
        id if id == &Block::DARK_OAK_LOG.id => Block::STRIPPED_DARK_OAK_LOG.id,
        id if id == &Block::MANGROVE_LOG.id => Block::STRIPPED_MANGROVE_LOG.id,
        id if id == &Block::CHERRY_LOG.id => Block::STRIPPED_CHERRY_LOG.id,
        id if id == &Block::PALE_OAK_LOG.id => Block::STRIPPED_PALE_OAK_LOG.id,

        id if id == &Block::OAK_WOOD.id => Block::STRIPPED_OAK_WOOD.id,
        id if id == &Block::SPRUCE_WOOD.id => Block::STRIPPED_SPRUCE_WOOD.id,
        id if id == &Block::BIRCH_WOOD.id => Block::STRIPPED_BIRCH_WOOD.id,
        id if id == &Block::JUNGLE_WOOD.id => Block::STRIPPED_JUNGLE_WOOD.id,
        id if id == &Block::ACACIA_WOOD.id => Block::STRIPPED_ACACIA_WOOD.id,
        id if id == &Block::DARK_OAK_WOOD.id => Block::STRIPPED_DARK_OAK_WOOD.id,
        id if id == &Block::MANGROVE_WOOD.id => Block::STRIPPED_MANGROVE_WOOD.id,
        id if id == &Block::CHERRY_WOOD.id => Block::STRIPPED_CHERRY_WOOD.id,
        id if id == &Block::PALE_OAK_WOOD.id => Block::STRIPPED_PALE_OAK_WOOD.id,
        _ => block.id,
    };
    if new_block_id == block.id {
        return None;
    }
    Some(new_block_id)
}

fn get_deoxidized_equivalent(block: &Block) -> Option<u16> {
    match &block.id {
        id if id == &Block::OXIDIZED_COPPER.id => Some(Block::WEATHERED_COPPER.id),
        id if id == &Block::WEATHERED_COPPER.id => Some(Block::EXPOSED_COPPER.id),
        id if id == &Block::EXPOSED_COPPER.id => Some(Block::COPPER_BLOCK.id),
        id if id == &Block::OXIDIZED_CHISELED_COPPER.id => {
            Some(Block::WEATHERED_CHISELED_COPPER.id)
        }
        id if id == &Block::WEATHERED_CHISELED_COPPER.id => Some(Block::EXPOSED_CHISELED_COPPER.id),
        id if id == &Block::EXPOSED_CHISELED_COPPER.id => Some(Block::CHISELED_COPPER.id),
        id if id == &Block::OXIDIZED_COPPER_GRATE.id => Some(Block::WEATHERED_COPPER_GRATE.id),
        id if id == &Block::WEATHERED_COPPER_GRATE.id => Some(Block::EXPOSED_COPPER_GRATE.id),
        id if id == &Block::EXPOSED_COPPER_GRATE.id => Some(Block::COPPER_GRATE.id),
        id if id == &Block::OXIDIZED_CUT_COPPER.id => Some(Block::WEATHERED_CUT_COPPER.id),
        id if id == &Block::WEATHERED_CUT_COPPER.id => Some(Block::EXPOSED_CUT_COPPER.id),
        id if id == &Block::EXPOSED_CUT_COPPER.id => Some(Block::CUT_COPPER.id),
        id if id == &Block::OXIDIZED_CUT_COPPER_STAIRS.id => {
            Some(Block::WEATHERED_CUT_COPPER_STAIRS.id)
        }
        id if id == &Block::WEATHERED_CUT_COPPER_STAIRS.id => {
            Some(Block::EXPOSED_CUT_COPPER_STAIRS.id)
        }
        id if id == &Block::EXPOSED_CUT_COPPER_STAIRS.id => Some(Block::CUT_COPPER_STAIRS.id),
        id if id == &Block::OXIDIZED_CUT_COPPER_SLAB.id => {
            Some(Block::WEATHERED_CUT_COPPER_SLAB.id)
        }
        id if id == &Block::WEATHERED_CUT_COPPER_SLAB.id => Some(Block::EXPOSED_CUT_COPPER_SLAB.id),
        id if id == &Block::EXPOSED_CUT_COPPER_SLAB.id => Some(Block::CUT_COPPER_SLAB.id),
        id if id == &Block::OXIDIZED_COPPER_BULB.id => Some(Block::WEATHERED_COPPER_BULB.id),
        id if id == &Block::WEATHERED_COPPER_BULB.id => Some(Block::EXPOSED_COPPER_BULB.id),
        id if id == &Block::EXPOSED_COPPER_BULB.id => Some(Block::COPPER_BULB.id),
        id if id == &Block::OXIDIZED_COPPER_DOOR.id => Some(Block::WEATHERED_COPPER_DOOR.id),
        id if id == &Block::WEATHERED_COPPER_DOOR.id => Some(Block::EXPOSED_COPPER_DOOR.id),
        id if id == &Block::EXPOSED_COPPER_DOOR.id => Some(Block::COPPER_DOOR.id),
        id if id == &Block::OXIDIZED_COPPER_TRAPDOOR.id => {
            Some(Block::WEATHERED_COPPER_TRAPDOOR.id)
        }
        id if id == &Block::WEATHERED_COPPER_TRAPDOOR.id => Some(Block::EXPOSED_COPPER_TRAPDOOR.id),
        id if id == &Block::EXPOSED_COPPER_TRAPDOOR.id => Some(Block::COPPER_TRAPDOOR.id),
        _ => None,
    }
}

fn get_unwaxed_equivalent(block: &Block) -> Option<u16> {
    match &block.id {
        id if id == &Block::WAXED_OXIDIZED_COPPER.id => Some(Block::OXIDIZED_COPPER.id),
        id if id == &Block::WAXED_WEATHERED_COPPER.id => Some(Block::WEATHERED_COPPER.id),
        id if id == &Block::WAXED_EXPOSED_COPPER.id => Some(Block::EXPOSED_COPPER.id),
        id if id == &Block::WAXED_COPPER_BLOCK.id => Some(Block::COPPER_BLOCK.id),
        id if id == &Block::WAXED_OXIDIZED_CHISELED_COPPER.id => {
            Some(Block::OXIDIZED_CHISELED_COPPER.id)
        }
        id if id == &Block::WAXED_WEATHERED_CHISELED_COPPER.id => {
            Some(Block::WEATHERED_CHISELED_COPPER.id)
        }
        id if id == &Block::WAXED_EXPOSED_CHISELED_COPPER.id => {
            Some(Block::EXPOSED_CHISELED_COPPER.id)
        }
        id if id == &Block::WAXED_CHISELED_COPPER.id => Some(Block::CHISELED_COPPER.id),
        id if id == &Block::WAXED_COPPER_GRATE.id => Some(Block::COPPER_GRATE.id),
        id if id == &Block::WAXED_OXIDIZED_COPPER_GRATE.id => Some(Block::OXIDIZED_COPPER_GRATE.id),
        id if id == &Block::WAXED_WEATHERED_COPPER_GRATE.id => {
            Some(Block::WEATHERED_COPPER_GRATE.id)
        }
        id if id == &Block::WAXED_EXPOSED_COPPER_GRATE.id => Some(Block::EXPOSED_COPPER_GRATE.id),
        id if id == &Block::WAXED_OXIDIZED_CUT_COPPER.id => Some(Block::OXIDIZED_CUT_COPPER.id),
        id if id == &Block::WAXED_WEATHERED_CUT_COPPER.id => Some(Block::WEATHERED_CUT_COPPER.id),
        id if id == &Block::WAXED_EXPOSED_CUT_COPPER.id => Some(Block::EXPOSED_CUT_COPPER.id),
        id if id == &Block::WAXED_CUT_COPPER.id => Some(Block::CUT_COPPER.id),
        id if id == &Block::WAXED_OXIDIZED_CUT_COPPER_STAIRS.id => {
            Some(Block::OXIDIZED_CUT_COPPER_STAIRS.id)
        }
        id if id == &Block::WAXED_WEATHERED_CUT_COPPER_STAIRS.id => {
            Some(Block::WEATHERED_CUT_COPPER_STAIRS.id)
        }
        id if id == &Block::WAXED_EXPOSED_CUT_COPPER_STAIRS.id => {
            Some(Block::EXPOSED_CUT_COPPER_STAIRS.id)
        }
        id if id == &Block::WAXED_CUT_COPPER_STAIRS.id => Some(Block::CUT_COPPER_STAIRS.id),
        id if id == &Block::WAXED_OXIDIZED_CUT_COPPER_SLAB.id => {
            Some(Block::OXIDIZED_CUT_COPPER_SLAB.id)
        }
        id if id == &Block::WAXED_WEATHERED_CUT_COPPER_SLAB.id => {
            Some(Block::WEATHERED_CUT_COPPER_SLAB.id)
        }
        id if id == &Block::WAXED_EXPOSED_CUT_COPPER_SLAB.id => {
            Some(Block::EXPOSED_CUT_COPPER_SLAB.id)
        }
        id if id == &Block::WAXED_CUT_COPPER_SLAB.id => Some(Block::CUT_COPPER_SLAB.id),
        id if id == &Block::WAXED_OXIDIZED_COPPER_BULB.id => Some(Block::OXIDIZED_COPPER_BULB.id),
        id if id == &Block::WAXED_WEATHERED_COPPER_BULB.id => Some(Block::WEATHERED_COPPER_BULB.id),
        id if id == &Block::WAXED_EXPOSED_COPPER_BULB.id => Some(Block::EXPOSED_COPPER_BULB.id),
        id if id == &Block::WAXED_COPPER_BULB.id => Some(Block::COPPER_BULB.id),
        id if id == &Block::WAXED_OXIDIZED_COPPER_DOOR.id => Some(Block::OXIDIZED_COPPER_DOOR.id),
        id if id == &Block::WAXED_WEATHERED_COPPER_DOOR.id => Some(Block::WEATHERED_COPPER_DOOR.id),
        id if id == &Block::WAXED_EXPOSED_COPPER_DOOR.id => Some(Block::EXPOSED_COPPER_DOOR.id),
        id if id == &Block::WAXED_COPPER_DOOR.id => Some(Block::COPPER_DOOR.id),
        id if id == &Block::WAXED_OXIDIZED_COPPER_TRAPDOOR.id => {
            Some(Block::OXIDIZED_COPPER_TRAPDOOR.id)
        }
        id if id == &Block::WAXED_WEATHERED_COPPER_TRAPDOOR.id => {
            Some(Block::WEATHERED_COPPER_TRAPDOOR.id)
        }
        id if id == &Block::WAXED_EXPOSED_COPPER_TRAPDOOR.id => {
            Some(Block::EXPOSED_COPPER_TRAPDOOR.id)
        }
        id if id == &Block::WAXED_COPPER_TRAPDOOR.id => Some(Block::COPPER_TRAPDOOR.id),
        _ => None,
    }
}
