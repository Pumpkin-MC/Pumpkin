use pumpkin_data::{Block, BlockDirection, BlockState, block_properties::KelpLikeProperties, tag};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct KelpFeature;

impl KelpFeature {
    #[allow(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut placed = 0;

        let y = chunk.ocean_floor_height_exclusive(pos.0.x, pos.0.z);
        let mut kelp_pos = BlockPos::new(pos.0.x, y, pos.0.z);

        if GenerationCache::get_block_state(chunk, &kelp_pos.0).to_block_id() == Block::WATER {
            let height = 1 + random.next_bounded_i32(10);

            for h in 0..=height {
                let above = kelp_pos.up();
                let can_survive = Self::can_survive(chunk, kelp_pos);

                if GenerationCache::get_block_state(chunk, &kelp_pos.0).to_block_id()
                    == Block::WATER
                    && GenerationCache::get_block_state(chunk, &above.0).to_block_id()
                        == Block::WATER
                    && can_survive
                {
                    if h == height {
                        let age = (random.next_bounded_i32(4) + 20) as u8;
                        let mut props = KelpLikeProperties::default(&Block::KELP);
                        props.age = age;
                        let state_id = props.to_state_id(&Block::KELP);
                        chunk.set_block_state(&kelp_pos.0, BlockState::from_id(state_id));
                        placed += 1;
                    } else {
                        chunk.set_block_state(&kelp_pos.0, Block::KELP_PLANT.default_state);
                    }
                } else if h > 0 {
                    let below = kelp_pos.down();
                    let below_below = below.down();
                    let below_below_id =
                        GenerationCache::get_block_state(chunk, &below_below.0).to_block_id();
                    if Self::can_survive(chunk, below) && below_below_id != Block::KELP.id {
                        let age = (random.next_bounded_i32(4) + 20) as u8;
                        let mut props = KelpLikeProperties::default(&Block::KELP);
                        props.age = age;
                        let state_id = props.to_state_id(&Block::KELP);
                        chunk.set_block_state(&below.0, BlockState::from_id(state_id));
                        placed += 1;
                    }
                    break;
                }

                kelp_pos = kelp_pos.up();
            }
        }

        placed > 0
    }

    fn can_survive<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
        let below = pos.down();
        let below_state = GenerationCache::get_block_state(chunk, &below.0);
        let below_id = below_state.to_block_id();

        if below_id.has_tag(tag::Block::MINECRAFT_CANNOT_SUPPORT_KELP) {
            return false;
        }

        below_id == Block::KELP.id
            || below_id == Block::KELP_PLANT.id
            || below_state.to_state().is_side_solid(BlockDirection::Up)
    }
}
