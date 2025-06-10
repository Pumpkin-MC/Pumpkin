use pumpkin_data::Block;
use pumpkin_util::{
    math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{ProtoChunk, generation::block_state_provider::BlockStateProvider};

use super::TreeDecorator;

#[derive(Deserialize)]
pub struct PlaceOnGroundTreeDecorator {
    tries: i32,
    radius: i32,
    height: i32,
    block_state_provider: BlockStateProvider,
}

impl PlaceOnGroundTreeDecorator {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        root_positions: Vec<BlockPos>,
        log_positions: Vec<BlockPos>,
    ) {
        let list = TreeDecorator::get_leaf_litter_positions(root_positions, log_positions);

        if list.is_empty() {
            return;
        }
        let pos = list.first().unwrap();
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

        let block_box = BoundingBox::new(
            Vector3::new(j as f64, i as f64, l as f64),
            Vector3::new(k as f64, i as f64, m as f64),
        )
        .expand(self.radius as f64, self.height as f64, self.radius as f64);

        for _n in 0..self.tries {
            let pos = BlockPos::new(
                random.next_inbetween_i32(block_box.min.x as i32, block_box.max.x as i32),
                random.next_inbetween_i32(block_box.min.y as i32, block_box.max.y as i32),
                random.next_inbetween_i32(block_box.min.z as i32, block_box.max.z as i32),
            );
            self.generate_decoration(chunk, pos, random);
        }
    }

    fn generate_decoration(
        &self,
        chunk: &mut ProtoChunk,
        pos: BlockPos,
        random: &mut RandomGenerator,
    ) {
        let state = chunk.get_block_state(&pos.0);
        let pos = pos.up();
        let up_state = chunk.get_block_state(&pos.0);

        // TODO
        if (up_state.to_state().is_air() || up_state.to_block() == Block::VINE)
            && state.to_state().is_full_cube()
        {
            chunk.set_block_state(&pos.0, &self.block_state_provider.get(random, pos));
        }
    }
}
