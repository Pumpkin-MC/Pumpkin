use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use serde::Deserialize;

use crate::{ProtoChunk, generation::feature::features::tree::TreeNode, level::Level};

use super::{FoliagePlacer, LeaveValidator};

#[derive(Deserialize)]
pub struct DarkOakFoliagePlacer;

impl DarkOakFoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub async fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        random: &mut RandomGenerator,
        node: &TreeNode,
        _foliage_height: i32,
        radius: i32,
        offset: i32,
        foliage_provider: &BlockState,
    ) {
        let pos = node.center.up_height(offset);
        let is_giant = node.giant_trunk;
        if is_giant {
            FoliagePlacer::generate_square(
                self,
                chunk,
                level,
                random,
                pos,
                radius + 2,
                -1,
                node.giant_trunk,
                foliage_provider,
            )
            .await;
            FoliagePlacer::generate_square(
                self,
                chunk,
                level,
                random,
                pos,
                radius + 3,
                0,
                node.giant_trunk,
                foliage_provider,
            )
            .await;
            FoliagePlacer::generate_square(
                self,
                chunk,
                level,
                random,
                pos,
                radius + 2,
                1,
                node.giant_trunk,
                foliage_provider,
            )
            .await;
            if random.next_bool() {
                FoliagePlacer::generate_square(
                    self,
                    chunk,
                    level,
                    random,
                    pos,
                    radius,
                    2,
                    node.giant_trunk,
                    foliage_provider,
                )
                .await;
            }
        } else {
            FoliagePlacer::generate_square(
                self,
                chunk,
                level,
                random,
                pos,
                radius + 2,
                -1,
                node.giant_trunk,
                foliage_provider,
            )
            .await;
            FoliagePlacer::generate_square(
                self,
                chunk,
                level,
                random,
                pos,
                radius + 1,
                0,
                node.giant_trunk,
                foliage_provider,
            )
            .await;
        }
    }

    pub fn get_random_height(&self, _random: &mut RandomGenerator) -> i32 {
        4
    }
}

impl LeaveValidator for DarkOakFoliagePlacer {
    fn is_position_invalid(
        &self,
        random: &mut RandomGenerator,
        dx: i32,
        y: i32,
        dz: i32,
        radius: i32,
        giant_trunk: bool,
    ) -> bool {
        if !(y != 0 || !giant_trunk || dx != -radius && dx < radius || dz != -radius && dz < radius)
        {
            return true;
        }
        // This is default
        let x = if giant_trunk {
            dx.abs().min((dx - 1).abs())
        } else {
            dx.abs()
        };
        let z = if giant_trunk {
            dz.abs().min((dz - 1).abs())
        } else {
            dz.abs()
        };
        self.is_invalid_for_leaves(random, x, y, z, radius, giant_trunk)
    }

    fn is_invalid_for_leaves(
        &self,
        _random: &mut pumpkin_util::random::RandomGenerator,
        dx: i32,
        y: i32,
        dz: i32,
        radius: i32,
        giant_trunk: bool,
    ) -> bool {
        if y == -1 && !giant_trunk {
            return dx == radius && dz == radius;
        }
        if y == 1 {
            return dx + dz > radius * 2 - 2;
        }
        false
    }
}
