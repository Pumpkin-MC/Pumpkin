use pumpkin_data::BlockDirection;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::ProtoChunk;

#[derive(Deserialize)]
pub struct SmallDripstoneFeature {
    chance_of_taller_dripstone: f32,
    chance_of_directional_spread: f32,
    chance_of_spread_radius2: f32,
    chance_of_spread_radius3: f32,
}

impl SmallDripstoneFeature {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if let Some(dir) = Self::get_direction(chunk, pos, random) {
            let pos = pos.offset(dir.opposite().to_offset());
            self.gen_dripstone_blocks(chunk, pos, random);
            // TODO
            return true;
        }
        false
    }

    fn get_direction(
        chunk: &mut ProtoChunk,
        pos: BlockPos,
        random: &mut RandomGenerator,
    ) -> Option<BlockDirection> {
        let up = super::can_replace(&chunk.get_block_state(&pos.up().0).to_block());
        let down: bool = super::can_replace(&chunk.get_block_state(&pos.down().0).to_block());
        if up && down {
            return if random.next_bool() {
                Some(BlockDirection::Down)
            } else {
                Some(BlockDirection::Up)
            };
        }
        if up {
            return Some(BlockDirection::Down);
        }
        if down {
            return Some(BlockDirection::Up);
        }
        None
    }

    fn gen_dripstone_blocks(
        &self,
        chunk: &mut ProtoChunk,
        pos: BlockPos,
        random: &mut RandomGenerator,
    ) {
        super::gen_dripstone(chunk, pos);
        for dir in BlockDirection::horizontal() {
            if random.next_f32() > self.chance_of_directional_spread {
                continue;
            }
            let pos = pos.offset(dir.to_offset());
            super::gen_dripstone(chunk, pos);
            if random.next_f32() > self.chance_of_spread_radius2 {
                continue;
            }
            let pos = pos.offset(BlockDirection::random(random).to_offset());
            super::gen_dripstone(chunk, pos);
            if random.next_f32() > self.chance_of_spread_radius3 {
                continue;
            }
            let pos = pos.offset(BlockDirection::random(random).to_offset());
            super::gen_dripstone(chunk, pos);
        }
    }
}
