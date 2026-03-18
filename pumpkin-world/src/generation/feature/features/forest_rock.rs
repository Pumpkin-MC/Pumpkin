use pumpkin_data::tag;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::{block::BlockStateCodec, generation::proto_chunk::GenerationCache};

pub struct ForestRockFeature {
    pub state: BlockStateCodec,
}

impl ForestRockFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut pos = pos;
        loop {
            if pos.0.y <= chunk.bottom_y() as i32 + 3 {
                return false;
            }
            let below_state = GenerationCache::get_block_state(chunk, &pos.down().0);
            let below_id = below_state.to_block_id();
            if !chunk.is_air(&pos.down().0)
                && (tag::Block::MINECRAFT_DIRT.1.contains(&below_id)
                    || tag::Block::MINECRAFT_BASE_STONE_OVERWORLD
                        .1
                        .contains(&below_id))
            {
                break;
            }
            pos = pos.down();
        }

        let block_state = self.state.get_state();

        for _ in 0..3 {
            let xr = random.next_bounded_i32(2);
            let yr = random.next_bounded_i32(2);
            let zr = random.next_bounded_i32(2);
            let threshold = (xr + yr + zr) as f32 * 0.333_f32 + 0.5_f32;
            let threshold_sq = threshold * threshold;

            let start = BlockPos::new(pos.0.x - xr, pos.0.y - yr, pos.0.z - zr);
            let end = BlockPos::new(pos.0.x + xr, pos.0.y + yr, pos.0.z + zr);

            for candidate in BlockPos::iterate(start, end) {
                let dx = (candidate.0.x - pos.0.x) as f32;
                let dy = (candidate.0.y - pos.0.y) as f32;
                let dz = (candidate.0.z - pos.0.z) as f32;
                if dx * dx + dy * dy + dz * dz <= threshold_sq {
                    chunk.set_block_state(&candidate.0, block_state);
                }
            }

            pos = pos.add(
                -1 + random.next_bounded_i32(2),
                -random.next_bounded_i32(2),
                -1 + random.next_bounded_i32(2),
            );
        }

        true
    }
}
