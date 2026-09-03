use pumpkin_data::{Block, BlockState, block_properties::BrownMushroomBlockLikeProperties};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;

pub struct HugeRedMushroomFeature;

impl HugeRedMushroomFeature {
    const FOLIAGE_RADIUS: i32 = 2;

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
        let tree_height = super::huge_brown_mushroom::mushroom_tree_height(random);

        let radius = Self::FOLIAGE_RADIUS;
        for i in (tree_height - 3)..=tree_height {
            let j = if i < tree_height { radius } else { radius - 1 };
            let k = radius - 2;

            for l in -j..=j {
                for m in -j..=j {
                    let on_x_edge = l == -j || l == j;
                    let on_z_edge = m == -j || m == j;

                    if i < tree_height && on_x_edge == on_z_edge {
                        continue;
                    }

                    let props = BrownMushroomBlockLikeProperties {
                        up: i >= tree_height - 1,
                        down: false,
                        west: l < -k,
                        east: l > k,
                        north: m < -k,
                        south: m > k,
                    };
                    let state_id = props.to_state_id(&Block::RED_MUSHROOM_BLOCK);
                    let cap_pos = BlockPos::new(pos.0.x + l, pos.0.y + i, pos.0.z + m);
                    chunk.set_block_state(&cap_pos.0, BlockState::from_id(state_id));
                }
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
