use crate::entity::player::Player;
use crate::item::items::honeycomb::WAXED2UNWAXED;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use crate::world::BlockFlags;
use async_trait::async_trait;
use pumpkin_data::block::{
    Block, BlockProperties, OakDoorLikeProperties, PaleOakWoodLikeProperties,
};
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use std::collections::HashMap;
use std::sync::LazyLock;

pub struct AxeItem;

// Need review. Since I can't verify if a block is equal to another (like mojang did) I'm going to do it with there id.

static STRIPPED_BLOCKS: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    HashMap::from([
        (Block::OAK_WOOD.id, Block::STRIPPED_OAK_WOOD.id),
        (Block::OAK_LOG.id, Block::STRIPPED_OAK_LOG.id),
        (Block::DARK_OAK_WOOD.id, Block::STRIPPED_DARK_OAK_WOOD.id),
        (Block::DARK_OAK_LOG.id, Block::STRIPPED_DARK_OAK_LOG.id),
        (Block::PALE_OAK_WOOD.id, Block::STRIPPED_PALE_OAK_WOOD.id),
        (Block::PALE_OAK_LOG.id, Block::STRIPPED_PALE_OAK_LOG.id),
        (Block::ACACIA_WOOD.id, Block::STRIPPED_ACACIA_WOOD.id),
        (Block::ACACIA_LOG.id, Block::STRIPPED_ACACIA_LOG.id),
        (Block::CHERRY_WOOD.id, Block::STRIPPED_CHERRY_WOOD.id),
        (Block::CHERRY_LOG.id, Block::STRIPPED_CHERRY_LOG.id),
        (Block::BIRCH_WOOD.id, Block::STRIPPED_BIRCH_WOOD.id),
        (Block::BIRCH_LOG.id, Block::STRIPPED_BIRCH_LOG.id),
        (Block::JUNGLE_WOOD.id, Block::STRIPPED_JUNGLE_WOOD.id),
        (Block::JUNGLE_LOG.id, Block::STRIPPED_JUNGLE_LOG.id),
        (Block::SPRUCE_WOOD.id, Block::STRIPPED_SPRUCE_WOOD.id),
        (Block::SPRUCE_LOG.id, Block::STRIPPED_SPRUCE_LOG.id),
        (Block::WARPED_STEM.id, Block::STRIPPED_WARPED_STEM.id),
        (Block::WARPED_HYPHAE.id, Block::STRIPPED_WARPED_HYPHAE.id),
        (Block::CRIMSON_STEM.id, Block::STRIPPED_CRIMSON_STEM.id),
        (Block::CRIMSON_HYPHAE.id, Block::STRIPPED_CRIMSON_HYPHAE.id),
        (Block::MANGROVE_WOOD.id, Block::STRIPPED_MANGROVE_WOOD.id),
        (Block::MANGROVE_LOG.id, Block::STRIPPED_MANGROVE_LOG.id),
        (Block::BAMBOO_BLOCK.id, Block::STRIPPED_BAMBOO_BLOCK.id),
    ])
});

static DECREASE_OXIDATION: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    HashMap::from([
        (Block::EXPOSED_COPPER.id, Block::COPPER_BLOCK.id),
        (Block::WEATHERED_COPPER.id, Block::EXPOSED_COPPER.id),
        (Block::OXIDIZED_COPPER.id, Block::WEATHERED_COPPER.id),
        (Block::EXPOSED_CUT_COPPER.id, Block::CUT_COPPER.id),
        (Block::WEATHERED_CUT_COPPER.id, Block::EXPOSED_CUT_COPPER.id),
        (
            Block::OXIDIZED_CUT_COPPER.id,
            Block::WEATHERED_CUT_COPPER.id,
        ),
        (Block::EXPOSED_CHISELED_COPPER.id, Block::CHISELED_COPPER.id),
        (
            Block::WEATHERED_CHISELED_COPPER.id,
            Block::EXPOSED_CHISELED_COPPER.id,
        ),
        (
            Block::OXIDIZED_CHISELED_COPPER.id,
            Block::WEATHERED_CHISELED_COPPER.id,
        ),
        (Block::EXPOSED_CUT_COPPER_SLAB.id, Block::CUT_COPPER_SLAB.id),
        (
            Block::WEATHERED_CUT_COPPER_SLAB.id,
            Block::EXPOSED_CUT_COPPER_SLAB.id,
        ),
        (
            Block::OXIDIZED_CUT_COPPER_SLAB.id,
            Block::WEATHERED_CUT_COPPER_SLAB.id,
        ),
        (
            Block::EXPOSED_CUT_COPPER_STAIRS.id,
            Block::CUT_COPPER_STAIRS.id,
        ),
        (
            Block::WEATHERED_CUT_COPPER_STAIRS.id,
            Block::EXPOSED_CUT_COPPER_STAIRS.id,
        ),
        (
            Block::OXIDIZED_CUT_COPPER_STAIRS.id,
            Block::WEATHERED_CUT_COPPER_STAIRS.id,
        ),
        (Block::EXPOSED_COPPER_DOOR.id, Block::COPPER_DOOR.id),
        (
            Block::WEATHERED_COPPER_DOOR.id,
            Block::EXPOSED_COPPER_DOOR.id,
        ),
        (
            Block::OXIDIZED_COPPER_DOOR.id,
            Block::WEATHERED_COPPER_DOOR.id,
        ),
        (Block::EXPOSED_COPPER_TRAPDOOR.id, Block::COPPER_TRAPDOOR.id),
        (
            Block::WEATHERED_COPPER_TRAPDOOR.id,
            Block::EXPOSED_COPPER_TRAPDOOR.id,
        ),
        (
            Block::OXIDIZED_COPPER_TRAPDOOR.id,
            Block::WEATHERED_COPPER_TRAPDOOR.id,
        ),
        (Block::EXPOSED_COPPER_GRATE.id, Block::COPPER_GRATE.id),
        (
            Block::WEATHERED_COPPER_GRATE.id,
            Block::EXPOSED_COPPER_GRATE.id,
        ),
        (
            Block::OXIDIZED_COPPER_GRATE.id,
            Block::WEATHERED_COPPER_GRATE.id,
        ),
        (Block::EXPOSED_COPPER_BULB.id, Block::COPPER_BULB.id),
        (
            Block::WEATHERED_COPPER_BULB.id,
            Block::EXPOSED_COPPER_BULB.id,
        ),
        (
            Block::OXIDIZED_COPPER_BULB.id,
            Block::WEATHERED_COPPER_BULB.id,
        ),
    ])
});

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
        _face: &BlockDirection,
        block: &Block,
        _server: &Server,
    ) {
        // I tried to follow mojang order of doing things.
        let world = player.world().await;
        let replacement_block = try_use_axe(block);
        // First we try to strip the block. by getting his equivalent and applying it the axis.

        // If there is a strip equivalent.
        if replacement_block.is_some() {
            let new_block = Block::from_id(*replacement_block.unwrap());
            let new_block = &new_block.unwrap();
            let new_state_id = if block.is_tagged_with("#minecraft:logs").is_some()
                && block.is_tagged_with("#minecraft:logs").unwrap()
            {
                let log_information = world.get_block_state_id(&location).await.unwrap();
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
            else if block.is_tagged_with("#minecraft:doors").is_some()
                && block.is_tagged_with("#minecraft:doors").unwrap()
            {
                // get block state of the old log.
                let door_information = world.get_block_state_id(&location).await.unwrap();
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
                new_block.default_state_id
            };
            // TODO Implements trapdoors when It's implemented
            world
                .set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL)
                .await;
            return;
        }
    }
}
fn try_use_axe(block: &Block) -> Option<&u16> {
    // Trying to get the strip equivalent
    let block_id = STRIPPED_BLOCKS.get(&block.id);
    if block_id.is_some() {
        return block_id;
    }
    // Else decrease the level of oxidation
    let block_id = DECREASE_OXIDATION.get(&block.id);
    if block_id.is_some() {
        return block_id;
    }
    // Else unwax the block
    let block_id = WAXED2UNWAXED.get(&block.id);
    if block_id.is_some() {
        return block_id;
    }
    None
}
