use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use crate::world::BlockFlags;
use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::item::Item;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
pub struct HoeItem;

impl ItemMetadata for HoeItem {
    const IDS: &'static [u16] = &[
        Item::WOODEN_HOE.id,
        Item::STONE_HOE.id,
        Item::GOLDEN_HOE.id,
        Item::DIAMOND_HOE.id,
        Item::NETHERITE_HOE.id,
    ];
}

#[async_trait]
impl PumpkinItem for HoeItem {
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
            || block == &Block::DIRT_PATH
            || block == &Block::DIRT
            || block == &Block::COARSE_DIRT
            || block == &Block::ROOTED_DIRT
        {
            let world = player.world().await;
            if face != &BlockDirection::Down
                && world.get_block_state(&location.up()).await.unwrap().air
            {
                world
                    .set_block_state(
                        &location,
                        Block::FARMLAND.default_state_id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        }
        // TODO: implement hanging_roots
    }
}
