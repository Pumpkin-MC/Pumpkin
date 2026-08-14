use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;

pub struct DripstoneClusterFeature;

impl DripstoneClusterFeature {
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut generated = false;
        let radius = 3;

        for dx in -radius..=radius {
            for dz in -radius..=radius {
                if dx * dx + dz * dz <= radius * radius {
                    let mut current_pos = BlockPos::new(pos.0.x + dx, pos.0.y, pos.0.z + dz);
                    while chunk.is_air(&current_pos.0)
                        && current_pos.0.y > chunk.bottom_y() as i32 + 2
                    {
                        current_pos = current_pos.down();
                    }
                    if super::gen_dripstone(chunk, current_pos) {
                        generated = true;
                        let drip_pos = current_pos.up();
                        if chunk.is_air(&drip_pos.0) {
                            chunk.set_block_state(
                                &drip_pos.0,
                                Block::POINTED_DRIPSTONE.default_state,
                            );
                        }
                    }
                }
            }
        }
        generated
    }
}
