use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use super::TreeDecorator;
use crate::{
    generation::{block_state_provider::BlockStateProvider, proto_chunk::GenerationCache},
    world::WorldPortalExt,
};

pub struct AlterGroundTreeDecorator {
    pub provider: BlockStateProvider,
}

impl AlterGroundTreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        root_positions: &[BlockPos],
        log_positions: &[BlockPos],
    ) {
        let list = TreeDecorator::get_leaf_litter_positions(root_positions, log_positions);
        let Some(first) = list.first() else {
            return;
        };

        let min_y = list.iter().map(|p| p.0.y).min().unwrap_or(first.0.y);

        for pos in list.iter().filter(|p| p.0.y == min_y) {
            self.place_circle(chunk, block_registry, random, pos.west().north());
            self.place_circle(chunk, block_registry, random, pos.east().east().north());
            self.place_circle(chunk, block_registry, random, pos.west().south().south());
            self.place_circle(
                chunk,
                block_registry,
                random,
                pos.east().east().south().south(),
            );

            for _ in 0..5 {
                let placement = random.next_bounded_i32(64);
                let xx = placement % 8;
                let zz = placement / 8;
                if xx == 0 || xx == 7 || zz == 0 || zz == 7 {
                    self.place_circle(chunk, block_registry, random, pos.add(-3 + xx, 0, -3 + zz));
                }
            }
        }
    }

    fn place_circle<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) {
        for xx in -2..=2i32 {
            for zz in -2..=2i32 {
                if xx.abs() != 2 || zz.abs() != 2 {
                    self.place_block_at(chunk, block_registry, random, pos.add(xx, 0, zz));
                }
            }
        }
    }

    fn place_block_at<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) {
        for dy in (-3..=2).rev() {
            let cursor = pos.up_height(dy);
            if let Some(replace_with) =
                self.provider
                    .get_optional(block_registry, chunk, random, cursor)
            {
                chunk.set_block_state(&cursor.0, replace_with);
                break;
            }

            if !chunk.is_air(&cursor.0) && dy < 0 {
                break;
            }
        }
    }
}
