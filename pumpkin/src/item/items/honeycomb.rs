use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use crate::world::BlockFlags;
use async_trait::async_trait;
use pumpkin_data::block::{Block, BlockProperties, OakDoorLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use std::collections::HashMap;
use std::sync::LazyLock;

pub struct HoneyCombItem;
pub static UNWAXED2WAXED: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    HashMap::from([
        (Block::COPPER_BLOCK.id, Block::WAXED_COPPER_BLOCK.id),
        (Block::EXPOSED_COPPER.id, Block::WAXED_EXPOSED_COPPER.id),
        (Block::WEATHERED_COPPER.id, Block::WAXED_WEATHERED_COPPER.id),
        (Block::OXIDIZED_COPPER.id, Block::WAXED_OXIDIZED_COPPER.id),
        (Block::CUT_COPPER.id, Block::WAXED_CUT_COPPER.id),
        (
            Block::EXPOSED_CUT_COPPER.id,
            Block::WAXED_EXPOSED_CUT_COPPER.id,
        ),
        (
            Block::WEATHERED_CUT_COPPER.id,
            Block::WAXED_WEATHERED_CUT_COPPER.id,
        ),
        (
            Block::OXIDIZED_CUT_COPPER.id,
            Block::WAXED_OXIDIZED_CUT_COPPER.id,
        ),
        (Block::CUT_COPPER_SLAB.id, Block::WAXED_CUT_COPPER_SLAB.id),
        (
            Block::EXPOSED_CUT_COPPER_SLAB.id,
            Block::WAXED_EXPOSED_CUT_COPPER_SLAB.id,
        ),
        (
            Block::WEATHERED_CUT_COPPER_SLAB.id,
            Block::WAXED_WEATHERED_CUT_COPPER_SLAB.id,
        ),
        (
            Block::OXIDIZED_CUT_COPPER_SLAB.id,
            Block::WAXED_OXIDIZED_CUT_COPPER_SLAB.id,
        ),
        (
            Block::CUT_COPPER_STAIRS.id,
            Block::WAXED_CUT_COPPER_STAIRS.id,
        ),
        (
            Block::EXPOSED_CUT_COPPER_STAIRS.id,
            Block::WAXED_EXPOSED_CUT_COPPER_STAIRS.id,
        ),
        (
            Block::WEATHERED_CUT_COPPER_STAIRS.id,
            Block::WAXED_WEATHERED_CUT_COPPER_STAIRS.id,
        ),
        (
            Block::OXIDIZED_CUT_COPPER_STAIRS.id,
            Block::WAXED_OXIDIZED_CUT_COPPER_STAIRS.id,
        ),
        (Block::CHISELED_COPPER.id, Block::WAXED_CHISELED_COPPER.id),
        (
            Block::EXPOSED_CHISELED_COPPER.id,
            Block::WAXED_EXPOSED_CHISELED_COPPER.id,
        ),
        (
            Block::WEATHERED_CHISELED_COPPER.id,
            Block::WAXED_WEATHERED_CHISELED_COPPER.id,
        ),
        (
            Block::OXIDIZED_CHISELED_COPPER.id,
            Block::WAXED_OXIDIZED_CHISELED_COPPER.id,
        ),
        (Block::COPPER_DOOR.id, Block::WAXED_COPPER_DOOR.id),
        (
            Block::EXPOSED_COPPER_DOOR.id,
            Block::WAXED_EXPOSED_COPPER_DOOR.id,
        ),
        (
            Block::WEATHERED_COPPER_DOOR.id,
            Block::WAXED_WEATHERED_COPPER_DOOR.id,
        ),
        (
            Block::OXIDIZED_COPPER_DOOR.id,
            Block::WAXED_OXIDIZED_COPPER_DOOR.id,
        ),
        (Block::COPPER_TRAPDOOR.id, Block::WAXED_COPPER_TRAPDOOR.id),
        (
            Block::EXPOSED_COPPER_TRAPDOOR.id,
            Block::WAXED_EXPOSED_COPPER_TRAPDOOR.id,
        ),
        (
            Block::WEATHERED_COPPER_TRAPDOOR.id,
            Block::WAXED_WEATHERED_COPPER_TRAPDOOR.id,
        ),
        (
            Block::OXIDIZED_COPPER_TRAPDOOR.id,
            Block::WAXED_OXIDIZED_COPPER_TRAPDOOR.id,
        ),
        (Block::COPPER_GRATE.id, Block::WAXED_COPPER_GRATE.id),
        (
            Block::EXPOSED_COPPER_GRATE.id,
            Block::WAXED_EXPOSED_COPPER_GRATE.id,
        ),
        (
            Block::WEATHERED_COPPER_GRATE.id,
            Block::WAXED_WEATHERED_COPPER_GRATE.id,
        ),
        (
            Block::OXIDIZED_COPPER_GRATE.id,
            Block::WAXED_OXIDIZED_COPPER_GRATE.id,
        ),
        (Block::COPPER_BULB.id, Block::WAXED_COPPER_BULB.id),
        (
            Block::EXPOSED_COPPER_BULB.id,
            Block::WAXED_EXPOSED_COPPER_BULB.id,
        ),
        (
            Block::WEATHERED_COPPER_BULB.id,
            Block::WAXED_WEATHERED_COPPER_BULB.id,
        ),
        (
            Block::OXIDIZED_COPPER_BULB.id,
            Block::WAXED_OXIDIZED_COPPER_BULB.id,
        ),
    ])
});
pub static WAXED2UNWAXED: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    UNWAXED2WAXED
        .iter()
        .map(|(k, v)| (*v, *k))
        .collect::<HashMap<_, _>>()
});

impl ItemMetadata for HoneyCombItem {
    fn ids() -> Box<[u16]> {
        [Item::HONEYCOMB.id].into()
    }
}

#[async_trait]
impl PumpkinItem for HoneyCombItem {
    async fn use_on_block(
        &self,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        _face: &BlockDirection,
        block: &Block,
        _server: &Server,
    ) {
        let world = player.world().await;

        // First we try to strip the block. by getting his equivalent and applying it the axis.
        let replacement_block = UNWAXED2WAXED.get(&block.id);
        // If there is a strip equivalent.
        if replacement_block.is_some() {
            // get block state of the old log.
            // get the log properties
            // create new properties for the new log.
            let new_block = &Block::from_id(*replacement_block.unwrap()).unwrap();

            let new_state_id = if block.is_tagged_with("#minecraft:doors").is_some()
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

            // TODO Implements trapdoors
            world
                .set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL)
                .await;
            return;
        }
    }
}
