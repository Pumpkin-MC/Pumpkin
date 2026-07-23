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
        // Snap Y to motion-blocking-no-leaves surface (vanilla PlaceOnGroundDecorator).
        let surface_y = chunk.top_motion_blocking_block_no_leaves_height_exclusive(pos.0.x, pos.0.z);
        // height exclusive → solid top is surface_y - 1; place litter at surface_y.
        let ground = BlockPos::new(pos.0.x, surface_y - 1, pos.0.z);
        let place_at = BlockPos::new(pos.0.x, surface_y, pos.0.z);

        let state = GenerationCache::get_block_state(chunk, &ground.0);
        let up_state = GenerationCache::get_block_state(chunk, &place_at.0);
        if (up_state.to_state().is_air() || up_state.to_block_id() == Block::VINE)
            && state.to_state().is_full_cube()
            && !state.to_block_id().has_tag(MINECRAFT_LEAVES)
        {
            chunk.set_block_state(
                &place_at.0,
                self.block_state_provider
                    .get(random, place_at, chunk, block_registry),
            );
        }
    }
}
