use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{BambooLeaves, BambooLikeProperties, BlockProperties, Integer0To1},
    tag,
    tag::Taggable,
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::generation::proto_chunk::GenerationCache;
use crate::world::BlockRegistryExt;

#[derive(Deserialize)]
pub struct BambooFeature {
    probability: f32,
}

impl BambooFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn BlockRegistryExt,
        _min_y: i8,
        _height: u16,
        _feature: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut i = 0;
        if chunk.is_air(&pos.0) {
            if block_registry.can_place_at(&Block::BAMBOO, chunk, &pos, BlockDirection::Up) {
                let height = random.next_bounded_i32(12) + 5;
                if random.next_f32() < self.probability {
                    let rnd = random.next_bounded_i32(4) + 1;
                    for x in pos.0.x - rnd..pos.0.x + rnd {
                        for z in pos.0.z - rnd..pos.0.z + rnd {
                            let block_below =
                                BlockPos::new(x, chunk.top_block_height_exclusive(x, z) - 1, z);
                            let block = GenerationCache::get_block_state(chunk, &block_below.0);
                            if !block.to_block().has_tag(&tag::Block::MINECRAFT_DIRT) {
                                continue;
                            }
                            chunk.set_block_state(&block_below.0, Block::PODZOL.default_state);
                        }
                    }
                }
                let mut bpos = pos;
                let bamboo = Block::BAMBOO.default_state;
                for _ in 0..height {
                    if chunk.is_air(&bpos.0) {
                        chunk.set_block_state(&bpos.0, bamboo);
                        bpos = bpos.up();
                    } else {
                        break;
                    }
                }
                // Top block
                if bpos.0.y - pos.0.y >= 3 {
                    let mut props = BambooLikeProperties::default(&Block::BAMBOO);
                    props.leaves = BambooLeaves::Large;
                    props.stage = Integer0To1::L1;

                    chunk.set_block_state(
                        &bpos.0,
                        BlockState::from_id(props.to_state_id(&Block::BAMBOO)),
                    );
                    props.stage = Integer0To1::L0;

                    chunk.set_block_state(
                        &bpos.down().0,
                        BlockState::from_id(props.to_state_id(&Block::BAMBOO)),
                    );
                    props.leaves = BambooLeaves::Small;

                    chunk.set_block_state(
                        &bpos.down().down().0,
                        BlockState::from_id(props.to_state_id(&Block::BAMBOO)),
                    );
                }
            }
            i += 1;
        }
        i > 0
    }
}
