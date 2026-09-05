use pumpkin_data::{Block, BlockDirection, BlockState, block_properties::KelpLikeProperties};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct WeepingVinesFeature;

impl WeepingVinesFeature {
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !chunk.is_air(&pos.0) {
            return false;
        }

        let above = pos.up();
        let above_id = GenerationCache::get_block_state(chunk, &above.0).to_block_id();
        if above_id != Block::NETHERRACK.id && above_id != Block::NETHER_WART_BLOCK.id {
            return false;
        }

        Self::place_roof_nether_wart(chunk, random, pos);
        Self::place_roof_weeping_vines(chunk, random, pos);
        true
    }

    fn place_roof_nether_wart<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        origin: BlockPos,
    ) {
        chunk.set_block_state(&origin.0, Block::NETHER_WART_BLOCK.default_state);

        for _ in 0..200 {
            let place_pos = origin.add(
                random.next_bounded_i32(6) - random.next_bounded_i32(6),
                random.next_bounded_i32(2) - random.next_bounded_i32(5),
                random.next_bounded_i32(6) - random.next_bounded_i32(6),
            );

            if chunk.is_air(&place_pos.0) {
                let mut neighbours = 0;
                for dir in BlockDirection::all() {
                    let neighbour_pos = place_pos.offset(dir.to_offset());
                    let id =
                        GenerationCache::get_block_state(chunk, &neighbour_pos.0).to_block_id();
                    if id == Block::NETHERRACK.id || id == Block::NETHER_WART_BLOCK.id {
                        neighbours += 1;
                    }
                    if neighbours > 1 {
                        break;
                    }
                }

                if neighbours == 1 {
                    chunk.set_block_state(&place_pos.0, Block::NETHER_WART_BLOCK.default_state);
                }
            }
        }
    }

    fn place_roof_weeping_vines<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        origin: BlockPos,
    ) {
        for _ in 0..100 {
            let place_pos = origin.add(
                random.next_bounded_i32(8) - random.next_bounded_i32(8),
                random.next_bounded_i32(2) - random.next_bounded_i32(7),
                random.next_bounded_i32(8) - random.next_bounded_i32(8),
            );

            if chunk.is_air(&place_pos.0) {
                let above = place_pos.up();
                let above_id = GenerationCache::get_block_state(chunk, &above.0).to_block_id();
                if above_id == Block::NETHERRACK.id || above_id == Block::NETHER_WART_BLOCK.id {
                    let mut vine_height = random.next_bounded_i32(8) + 1;
                    if random.next_bounded_i32(6) == 0 {
                        vine_height *= 2;
                    }
                    if random.next_bounded_i32(5) == 0 {
                        vine_height = 1;
                    }

                    Self::place_weeping_vines_column(chunk, random, place_pos, vine_height, 17, 25);
                }
            }
        }
    }

    pub fn place_weeping_vines_column<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        mut place_pos: BlockPos,
        total_height: i32,
        min_age: i32,
        max_age: i32,
    ) {
        for height in 0..=total_height {
            if chunk.is_air(&place_pos.0) {
                if height == total_height || !chunk.is_air(&place_pos.down().0) {
                    let age = random.next_inbetween_i32(min_age, max_age);
                    let mut props = KelpLikeProperties::default(&Block::WEEPING_VINES);
                    props.age = age as u8;
                    let state_id = props.to_state_id(&Block::WEEPING_VINES);
                    chunk.set_block_state(&place_pos.0, BlockState::from_id(state_id));
                    break;
                }

                chunk.set_block_state(&place_pos.0, Block::WEEPING_VINES_PLANT.default_state);
            }

            place_pos = place_pos.down();
        }
    }
}
