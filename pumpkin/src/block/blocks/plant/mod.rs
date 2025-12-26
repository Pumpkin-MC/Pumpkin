use pumpkin_data::{Block, tag, tag::Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};

pub mod bamboo;
pub mod bamboo_sapling;
pub mod bush;
pub mod crop;
pub mod dry_vegetation;
pub mod flower;
pub mod flowerbed;
pub mod leaf_litter;
pub mod lily_pad;
pub mod mushroom_plant;
pub mod roots;
pub mod sapling;
pub mod sea_grass;
pub mod sea_pickles;
pub mod segmented;
pub mod short_plant;
pub mod sugar_cane;
pub mod tall_plant;

trait PlantBlockBase {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        block.has_tag(&tag::Block::MINECRAFT_DIRT) || block == &Block::FARMLAND
    }

    async fn get_state_for_neighbor_update(
        &self,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        block_state: BlockStateId,
    ) -> BlockStateId {
        if !self.can_place_at(block_accessor, block_pos).await {
            return Block::AIR.default_state.id;
        }
        block_state
    }

    async fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        self.can_plant_on_top(block_accessor, &block_pos.down())
            .await
    }
}
