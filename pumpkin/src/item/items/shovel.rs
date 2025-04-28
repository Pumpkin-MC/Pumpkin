use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use crate::world::BlockFlags;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

pub struct ShovelItem;

impl ItemMetadata for ShovelItem {
    fn ids() -> Box<[u16]> {
        Item::get_tag_values("#minecraft:shovels")
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
impl PumpkinItem for ShovelItem {
    async fn use_on_block(
        &self,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        face: &BlockDirection,
        block: &Block,
        _server: &Server,
    ) {
        // Yes, Minecraft does hardcode these
        if block == &Block::GRASS_BLOCK
            || block == &Block::DIRT
            || block == &Block::COARSE_DIRT
            || block == &Block::ROOTED_DIRT
            || block == &Block::PODZOL
            || block == &Block::MYCELIUM
        {
            let world = player.world().await;
            if face != &BlockDirection::Down
                && world
                    .get_block_state(&location.up())
                    .await
                    .unwrap()
                    .is_air()
            {
                world
                    .set_block_state(
                        &location,
                        Block::DIRT_PATH.default_state_id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        }
        if block == &Block::CAMPFIRE || block == &Block::SOUL_CAMPFIRE {
            // TODO Implements campfire
        }
    }
}
