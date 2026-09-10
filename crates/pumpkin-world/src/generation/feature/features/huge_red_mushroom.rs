use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockState, block_properties::BrownMushroomBlockLikeProperties, tag};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use super::huge_brown_mushroom::place_mushroom_block;
use crate::generation::proto_chunk::GenerationCache;

pub struct HugeRedMushroomFeature;

impl HugeRedMushroomFeature {
    const FOLIAGE_RADIUS: i32 = 2;

    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        min_y: i8,
        height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let tree_height = super::huge_brown_mushroom::mushroom_tree_height(random);

        let min_y = i32::from(min_y);
        let max_y = min_y + i32::from(height);
        if pos.0.y < min_y + 1 || pos.0.y + tree_height + 1 > max_y {
            return false;
        }

        let below_block = chunk.get_block(&pos.down());
        if !below_block.has_tag(&tag::Block::MINECRAFT_HUGE_RED_MUSHROOM_CAN_PLACE_ON) {
            return false;
        }

        for dy in 0..=tree_height {
            let radius = if (dy < tree_height && dy >= tree_height - 3) || dy == tree_height {
                Self::FOLIAGE_RADIUS
            } else {
                0
            };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let check_pos = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
                    let (block, state) = chunk.get_block_and_state(&check_pos);
                    if !state.is_air() && !block.has_tag(&tag::Block::MINECRAFT_LEAVES) {
                        return false;
                    }
                }
            }
        }

        let radius = Self::FOLIAGE_RADIUS;
        for i in (tree_height - 3)..=tree_height {
            let j = if i < tree_height { radius } else { radius - 1 };

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
                        west: l < 0,
                        east: l > 0,
                        north: m < 0,
                        south: m > 0,
                    };
                    let state_id = props.to_state_id(&Block::RED_MUSHROOM_BLOCK);
                    let cap_pos = BlockPos::new(pos.0.x + l, pos.0.y + i, pos.0.z + m);
                    place_mushroom_block(chunk, &cap_pos, BlockState::from_id(state_id));
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
            place_mushroom_block(chunk, &stem_pos, stem_state);
        }

        true
    }
}
