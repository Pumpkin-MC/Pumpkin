use pumpkin_data::{Block, BlockDirection, BlockState, block_properties::VineLikeProperties};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;

pub struct VinesFeature;

impl VinesFeature {
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        _block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !chunk.is_air(&pos.0) {
            return false;
        }

        for dir in BlockDirection::all() {
            if dir == BlockDirection::Down {
                continue;
            }

            let neighbor = pos.offset(dir.to_offset());
            let neighbor_state = GenerationCache::get_block_state(chunk, &neighbor.0);
            if !neighbor_state.to_state().is_side_solid(dir.opposite()) {
                continue;
            }

            let mut vine = VineLikeProperties::default(&Block::VINE);
            vine.north = dir == BlockDirection::North;
            vine.east = dir == BlockDirection::East;
            vine.south = dir == BlockDirection::South;
            vine.west = dir == BlockDirection::West;
            vine.up = dir == BlockDirection::Up;
            chunk.set_block_state(&pos.0, BlockState::from_id(vine.to_state_id(&Block::VINE)));
            return true;
        }

        false
    }
}
