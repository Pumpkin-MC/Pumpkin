use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{DoubleBlockHalf, TallSeagrassLikeProperties},
    tag,
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct SeagrassFeature {
    pub probability: f32,
}

impl SeagrassFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut placed_any = false;
        let x = random.next_bounded_i32(8) - random.next_bounded_i32(8);
        let z = random.next_bounded_i32(8) - random.next_bounded_i32(8);
        let y = chunk.ocean_floor_height_exclusive(pos.0.x + x, pos.0.z + z);
        let grass_pos = BlockPos::new(pos.0.x + x, y, pos.0.z + z);

        if GenerationCache::get_block_state(chunk, &grass_pos.0).to_block_id() == Block::WATER
            && Self::can_survive(chunk, grass_pos)
        {
            let is_tall = random.next_f64() < self.probability as f64;
            if is_tall {
                let above = grass_pos.up();
                if GenerationCache::get_block_state(chunk, &above.0).to_block_id() == Block::WATER {
                    let mut upper_props =
                        TallSeagrassLikeProperties::default(&Block::TALL_SEAGRASS);
                    upper_props.half = DoubleBlockHalf::Upper;
                    let upper_state =
                        BlockState::from_id(upper_props.to_state_id(&Block::TALL_SEAGRASS));

                    chunk.set_block_state(&grass_pos.0, Block::TALL_SEAGRASS.default_state);
                    chunk.set_block_state(&above.0, upper_state);
                    placed_any = true;
                }
            } else {
                chunk.set_block_state(&grass_pos.0, Block::SEAGRASS.default_state);
                placed_any = true;
            }
        }

        placed_any
    }

    fn can_survive<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
        let below = pos.down();
        let below_state = GenerationCache::get_block_state(chunk, &below.0);
        let below_id = below_state.to_block_id();

        if below_id.has_tag(tag::Block::MINECRAFT_CANNOT_SUPPORT_SEAGRASS) {
            return false;
        }

        below_state.to_state().is_side_solid(BlockDirection::Up)
    }
}
