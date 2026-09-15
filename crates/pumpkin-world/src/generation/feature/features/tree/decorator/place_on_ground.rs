use super::TreeDecorator;
use crate::generation::proto_chunk::GenerationCache;
use crate::{generation::block_state_provider::BlockStateProvider, world::WorldPortalExt};
use pumpkin_data::{Block, tag::Block::MINECRAFT_LEAVES};
use pumpkin_util::{
    math::{block_box::BlockBox, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

pub struct PlaceOnGroundTreeDecorator {
    pub tries: i32,
    pub radius: i32,
    pub height: i32,
    pub block_state_provider: BlockStateProvider,
}

impl PlaceOnGroundTreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
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

        for block_pos_2 in list.iter() {
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
            self.generate_decoration(chunk, block_registry, pos, random);
        }
    }

    fn generate_decoration<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        pos: BlockPos,
        random: &mut RandomGenerator,
    ) {
        let state = GenerationCache::get_block_state(chunk, &pos.0);
        let ground_pos = pos.down();
        let ground_state = GenerationCache::get_block_state(chunk, &ground_pos.0);
        if (state.to_state().is_air() || state.to_block_id() == Block::VINE)
            && ground_state.to_state().is_full_cube()
            && !ground_state.to_block_id().has_tag(MINECRAFT_LEAVES)
        {
            chunk.set_block_state(
                &pos.0,
                self.block_state_provider
                    .get(random, pos, chunk, block_registry),
            );
        }
    }
}
