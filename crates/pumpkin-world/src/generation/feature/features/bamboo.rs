use pumpkin_data::{
    Block, BlockState,
    block_properties::{BambooLeaves, BambooLikeProperties},
    tag,
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;

pub struct BambooFeature {
    pub probability: f32,
}

impl BambooFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut placed = 0;
        if chunk.is_air(&pos.0)
            && block_registry.can_place_at(&Block::BAMBOO, Block::BAMBOO.default_state, chunk, &pos)
        {
            let height = random.next_bounded_i32(12) + 5;
            if random.next_f32() < self.probability {
                let r = random.next_bounded_i32(4) + 1;
                for xx in pos.0.x - r..=pos.0.x + r {
                    for zz in pos.0.z - r..=pos.0.z + r {
                        let xd = xx - pos.0.x;
                        let zd = zz - pos.0.z;
                        if xd * xd + zd * zd <= r * r {
                            let podzol_y = chunk.top_block_height_exclusive(xx, zz) - 1;
                            let podzol_pos = BlockPos::new(xx, podzol_y, zz);
                            let block_id = GenerationCache::get_block_state(chunk, &podzol_pos.0)
                                .to_block_id();
                            if block_id
                                .has_tag(tag::Block::MINECRAFT_BENEATH_BAMBOO_PODZOL_REPLACEABLE)
                            {
                                chunk.set_block_state(&podzol_pos.0, Block::PODZOL.default_state);
                            }
                        }
                    }
                }
            }

            let trunk_props = BambooLikeProperties {
                age: 1,
                leaves: BambooLeaves::None,
                stage: 0,
            };
            let trunk_state = BlockState::from_id(trunk_props.to_state_id(&Block::BAMBOO));

            let mut bpos = pos;
            for _ in 0..height {
                if chunk.is_air(&bpos.0) {
                    chunk.set_block_state(&bpos.0, trunk_state);
                    bpos = bpos.up();
                } else {
                    break;
                }
            }

            if bpos.0.y - pos.0.y >= 3 {
                let final_large_props = BambooLikeProperties {
                    age: 1,
                    leaves: BambooLeaves::Large,
                    stage: 1,
                };
                chunk.set_block_state(
                    &bpos.down().0,
                    BlockState::from_id(final_large_props.to_state_id(&Block::BAMBOO)),
                );

                let top_large_props = BambooLikeProperties {
                    age: 1,
                    leaves: BambooLeaves::Large,
                    stage: 0,
                };
                chunk.set_block_state(
                    &bpos.down().down().0,
                    BlockState::from_id(top_large_props.to_state_id(&Block::BAMBOO)),
                );

                let top_small_props = BambooLikeProperties {
                    age: 1,
                    leaves: BambooLeaves::Small,
                    stage: 0,
                };
                chunk.set_block_state(
                    &bpos.down().down().down().0,
                    BlockState::from_id(top_small_props.to_state_id(&Block::BAMBOO)),
                );
            }

            placed += 1;
        }

        placed > 0
    }
}
