use pumpkin_data::{Block, BlockState, block_properties::BrownMushroomBlockLikeProperties};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub fn mushroom_tree_height(random: &mut RandomGenerator) -> i32 {
    let mut height = random.next_bounded_i32(3) + 4;
    if random.next_bounded_i32(12) == 0 {
        height *= 2;
    }
    height
}

pub struct HugeBrownMushroomFeature;

impl HugeBrownMushroomFeature {
    const FOLIAGE_RADIUS: i32 = 3;

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
        let tree_height = mushroom_tree_height(random);

        let radius = Self::FOLIAGE_RADIUS;
        let cap_y = pos.0.y + tree_height;
        for j in -radius..=radius {
            for k in -radius..=radius {
                let on_x_edge = j == -radius || j == radius;
                let on_z_edge = k == -radius || k == radius;

                if on_x_edge && on_z_edge {
                    continue;
                }

                let props = BrownMushroomBlockLikeProperties {
                    up: true,
                    down: false,
                    west: j == -radius || (on_z_edge && j == 1 - radius),
                    east: j == radius || (on_z_edge && j == radius - 1),
                    north: k == -radius || (on_x_edge && k == 1 - radius),
                    south: k == radius || (on_x_edge && k == radius - 1),
                };
                let state_id = props.to_state_id(&Block::BROWN_MUSHROOM_BLOCK);
                let cap_pos = BlockPos::new(pos.0.x + j, cap_y, pos.0.z + k);
                chunk.set_block_state(&cap_pos.0, BlockState::from_id(state_id));
            }
        }

        let stem_props = BrownMushroomBlockLikeProperties {
            up: false,
            down: false,
            north: true,
            east: true,
            south: true,
            west: true,
        };
        let stem_state = BlockState::from_id(stem_props.to_state_id(&Block::MUSHROOM_STEM));
        for i in 0..tree_height {
            let stem_pos = BlockPos::new(pos.0.x, pos.0.y + i, pos.0.z);
            chunk.set_block_state(&stem_pos.0, stem_state);
        }

        true
    }
}
