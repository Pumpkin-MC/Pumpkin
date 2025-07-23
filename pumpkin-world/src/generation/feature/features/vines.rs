use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockProperties, get_state_by_state_id},
};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;

use crate::{ProtoChunk, world::BlockRegistryExt};

#[derive(Deserialize)]
pub struct VinesFeature;

impl VinesFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn BlockRegistryExt,
        _min_y: i8,
        _height: u16,
        _feature: &str,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !chunk.is_air(&pos.0) {
            return false;
        }
        for dir in BlockDirection::all() {
            // TODO
            if dir == BlockDirection::Down
                || !chunk
                    .get_block_state(&pos.offset(dir.to_offset()).0)
                    .to_state()
                    .is_full_cube()
            {
                continue;
            }
            let mut vine =
                pumpkin_data::block_properties::VineLikeProperties::default(&Block::VINE);
            vine.north = dir == BlockDirection::North;
            vine.east = dir == BlockDirection::East;
            vine.south = dir == BlockDirection::South;
            vine.west = dir == BlockDirection::West;
            vine.up = dir == BlockDirection::Up;
            chunk.set_block_state(
                &pos.0,
                get_state_by_state_id(vine.to_state_id(&Block::VINE)),
            );
            return true;
        }
        false
    }
}
