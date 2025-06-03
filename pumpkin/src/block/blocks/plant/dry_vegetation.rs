use async_trait::async_trait;
use pumpkin_data::tag::Tagable;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

pub struct DryVegetationBlock;

impl BlockMetadata for DryVegetationBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &["dead_bush", "tall_dry_grass", "short_dry_grass"]
    }
}

#[async_trait]
impl PumpkinBlock for DryVegetationBlock {
    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        let block_below = block_accessor.get_block(&block_pos.down()).await;
        block_below
            .is_tagged_with("minecraft:dry_vegetation_may_place_on")
            .unwrap()
    }
}
