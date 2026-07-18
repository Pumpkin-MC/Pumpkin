use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator, random::RandomImpl};

use crate::generation::proto_chunk::GenerationCache;

pub struct DeltaFeatureFeature;

impl DeltaFeatureFeature {
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let size = random.next_bounded_i32(3) + 2;
        let mut generated = false;

        for dx in -size..=size {
            for dy in -size..=size {
                for dz in -size..=size {
                    if dx * dx + dy * dy + dz * dz <= size * size {
                        let target = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
                        let block_id =
                            GenerationCache::get_block_state(chunk, &target.0).to_block_id();
                        if block_id == Block::NETHERRACK.id
                            || block_id == Block::LAVA.id
                            || block_id == Block::AIR.id
                        {
                            let replacement = if random.next_f32() < 0.6 {
                                Block::BASALT
                            } else {
                                Block::BLACKSTONE
                            };
                            chunk.set_block_state(&target.0, replacement.default_state);
                            generated = true;
                        }
                    }
                }
            }
        }
        generated
    }
}
