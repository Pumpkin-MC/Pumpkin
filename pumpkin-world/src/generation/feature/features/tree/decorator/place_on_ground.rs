use super::TreeDecorator;
use crate::generation::block_state_provider::BlockStateProvider;
use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::{
    Block,
    tag::{Block::MINECRAFT_LEAVES, Taggable},
};
use pumpkin_util::{
    math::{block_box::BlockBox, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PlaceOnGroundTreeDecorator {
    tries: i32,
    radius: i32,
    height: i32,
    block_state_provider: BlockStateProvider,
}

impl PlaceOnGroundTreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        root_positions: &[BlockPos],
        log_positions: &[BlockPos],
    ) {
        let list = TreeDecorator::get_leaf_litter_positions(root_positions, log_positions);

        let Some(pos) = list.first() else {
            return;
        };

        let i = pos.0.y;
        let mut j = pos.0.x;
        let mut k = pos.0.x;
        let mut l = pos.0.z;
        let mut m = pos.0.z;

        for block_pos_2 in list {
            if block_pos_2.0.y != i {
                continue;
            }
            j = j.min(block_pos_2.0.x);
            k = k.max(block_pos_2.0.x);
            l = l.min(block_pos_2.0.z);
            m = m.max(block_pos_2.0.z);
        }

        let block_box =
            BlockBox::new(j, i, l, k, i, m).expand(self.radius, self.height, self.radius);

        for _n in 0..self.tries {
            let pos = BlockPos::new(
                random.next_inbetween_i32(block_box.min.x, block_box.max.x),
                random.next_inbetween_i32(block_box.min.y, block_box.max.y),
                random.next_inbetween_i32(block_box.min.z, block_box.max.z),
            );
            self.generate_decoration(chunk, pos, random);
        }
    }

    fn generate_decoration<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: BlockPos,
        random: &mut RandomGenerator,
    ) {
        let state = GenerationCache::get_block_state(chunk, &pos.0);
        let up_pos = pos.up();
        let up_state = GenerationCache::get_block_state(chunk, &up_pos.0);
        // TODO
        if (up_state.to_state().is_air() || up_state.to_block() == &Block::VINE)
            && state.to_state().is_full_cube()
            && !state.to_block().has_tag(&MINECRAFT_LEAVES)
        // TODO: using heightmap seems not to work
        {
            chunk.set_block_state(&up_pos.0, self.block_state_provider.get(random, up_pos));
        }
    }
}
