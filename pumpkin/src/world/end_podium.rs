//! Places the End exit podium structure (the beacon-like structure at (0, y, 0)).
//!
//! `active = false` = inactive podium: bedrock rim + air centre, placed before the dragon is killed.
//! `active = true` = active podium: bedrock rim + `END_PORTAL` tiles in the centre, placed after kill.

use std::sync::Arc;

use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, HorizontalFacing, WallTorchLikeProperties},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use super::World;

/// Place the podium structure centred on `origin` into `world`.
pub async fn place(world: &Arc<World>, origin: BlockPos, active: bool) {
    let ox = origin.0.x;
    let oy = origin.0.y;
    let oz = origin.0.z;

    for y in (oy - 4)..=(oy + 32) {
        for x in (ox - 4)..=(ox + 4) {
            for z in (oz - 4)..=(oz + 4) {
                let dx = (x - ox) as f64;
                let dy = (y - oy) as f64;
                let dz = (z - oz) as f64;
                let dist_sq = dx * dx + dy * dy + dz * dz;

                let inside_rim = dist_sq < 2.5 * 2.5; // inner 2-block disc
                let near = inside_rim || dist_sq < 3.5 * 3.5; // outer rim up to 3.5

                if !near {
                    continue;
                }

                let pos = BlockPos::new(x, y, z);
                let state_id = match y.cmp(&oy) {
                    std::cmp::Ordering::Less => {
                        if inside_rim {
                            Block::BEDROCK.default_state.id
                        } else {
                            Block::END_STONE.default_state.id
                        }
                    }
                    std::cmp::Ordering::Greater => Block::AIR.default_state.id,
                    std::cmp::Ordering::Equal => {
                        if !inside_rim {
                            Block::BEDROCK.default_state.id
                        } else if active {
                            Block::END_PORTAL.default_state.id
                        } else {
                            Block::AIR.default_state.id
                        }
                    }
                };

                world
                    .set_block_state(&pos, state_id, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        }
    }

    for y in oy..=(oy + 3) {
        world
            .set_block_state(
                &BlockPos::new(ox, y, oz),
                Block::BEDROCK.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;
    }

    // Wall torches on N/S/E/W faces at pillar height 2
    let torch_y = oy + 2;
    // Torch position relative to origin and facing direction: (dx, dz, facing)
    for (dx, dz, facing) in [
        (0i32, -1i32, HorizontalFacing::North),
        (0, 1, HorizontalFacing::South),
        (-1, 0, HorizontalFacing::West),
        (1, 0, HorizontalFacing::East),
    ] {
        let props = WallTorchLikeProperties { facing };
        let state_id = props.to_state_id(&Block::WALL_TORCH);
        world
            .set_block_state(
                &BlockPos::new(ox + dx, torch_y, oz + dz),
                state_id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;
    }
}
