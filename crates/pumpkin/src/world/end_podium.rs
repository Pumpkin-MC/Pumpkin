//! Places the End exit podium structure (the beacon-like structure at (0, y, 0)).
//!
//! `active = false` = inactive podium: bedrock rim + air centre, placed before the dragon is killed.
//! `active = true` = active podium: bedrock rim + `END_PORTAL` tiles in the centre, placed after kill.

use std::sync::Arc;

use pumpkin_data::{
    Block,
    block_properties::{HorizontalFacing, WallTorchLikeProperties},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use super::World;

pub const PODIUM_RADIUS: i32 = 4;
pub const PODIUM_PILLAR_HEIGHT: i32 = 4;
pub const RIM_RADIUS: i32 = 1;
pub const CORNER_ROUNDING: f32 = 0.5;
pub const END_PODIUM_LOCATION: BlockPos = BlockPos::new(0, 0, 0);

#[must_use]
pub const fn get_location(offset: BlockPos) -> BlockPos {
    BlockPos::new(
        END_PODIUM_LOCATION.0.x + offset.0.x,
        END_PODIUM_LOCATION.0.y + offset.0.y,
        END_PODIUM_LOCATION.0.z + offset.0.z,
    )
}

fn set_block(world: &Arc<World>, pos: &BlockPos, block: &Block) {
    world.set_block_state(pos, block.default_state.id, BlockFlags::NOTIFY_LISTENERS);
}

fn drop_previous_and_set_block(world: &Arc<World>, pos: &BlockPos, block: &Block) {
    if world.get_block(pos) != block {
        set_block(world, pos, block);
    }
}

/// Place the podium structure centred on `origin` into `world`.
pub fn place(world: &Arc<World>, origin: BlockPos, active: bool) {
    let ox = origin.0.x;
    let oy = if origin.0.y < 50 { 65 } else { origin.0.y };
    let oz = origin.0.z;

    for y in (oy - 1)..=(oy + 32) {
        for x in (ox - 4)..=(ox + 4) {
            for z in (oz - 4)..=(oz + 4) {
                let dx = (x - ox) as f64;
                let dy = (y - oy) as f64;
                let dz = (z - oz) as f64;
                let dist_sq = dx * dx + dy * dy + dz * dz;

                let closer_than_2_5 = dist_sq < 2.5 * 2.5;
                let closer_than_3_5 = dist_sq < 3.5 * 3.5;

                if closer_than_2_5 || closer_than_3_5 {
                    let pos = BlockPos::new(x, y, z);
                    if y < oy {
                        if closer_than_2_5 {
                            set_block(world, &pos, &Block::BEDROCK);
                        } else if active {
                            drop_previous_and_set_block(world, &pos, &Block::END_STONE);
                        } else {
                            set_block(world, &pos, &Block::END_STONE);
                        }
                    } else if y > oy {
                        if active {
                            drop_previous_and_set_block(world, &pos, &Block::AIR);
                        } else {
                            set_block(world, &pos, &Block::AIR);
                        }
                    } else if !closer_than_2_5 {
                        set_block(world, &pos, &Block::BEDROCK);
                    } else if active {
                        drop_previous_and_set_block(world, &pos, &Block::END_PORTAL);
                    } else {
                        set_block(world, &pos, &Block::AIR);
                    }
                }
            }
        }
    }

    for y in 0..4 {
        set_block(world, &BlockPos::new(ox, oy + y, oz), &Block::BEDROCK);
    }

    let center_of_pillar_y = oy + 2;
    for (dx, dz, facing) in [
        (0i32, -1i32, HorizontalFacing::North),
        (0, 1, HorizontalFacing::South),
        (-1, 0, HorizontalFacing::West),
        (1, 0, HorizontalFacing::East),
    ] {
        let props = WallTorchLikeProperties { facing };
        let state_id = props.to_state_id(&Block::WALL_TORCH);
        world.set_block_state(
            &BlockPos::new(ox + dx, center_of_pillar_y, oz + dz),
            state_id,
            BlockFlags::NOTIFY_LISTENERS,
        );
    }
}
