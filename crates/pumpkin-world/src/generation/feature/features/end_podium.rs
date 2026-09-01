use pumpkin_data::{
    Block,
    block_properties::{HorizontalFacing, WallTorchLikeProperties},
};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::generation::proto_chunk::GenerationCache;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EndPodiumFeature {
    pub active: bool,
}

impl EndPodiumFeature {
    pub const PODIUM_RADIUS: i32 = 4;
    pub const PODIUM_PILLAR_HEIGHT: i32 = 4;
    pub const RIM_RADIUS: i32 = 1;
    pub const CORNER_ROUNDING: f32 = 0.5;
    pub const END_PODIUM_LOCATION: BlockPos = BlockPos::new(0, 0, 0);

    #[must_use]
    pub const fn new(active: bool) -> Self {
        Self { active }
    }

    #[must_use]
    pub const fn get_location(offset: BlockPos) -> BlockPos {
        BlockPos::new(
            Self::END_PODIUM_LOCATION.0.x + offset.0.x,
            Self::END_PODIUM_LOCATION.0.y + offset.0.y,
            Self::END_PODIUM_LOCATION.0.z + offset.0.z,
        )
    }

    pub fn generate<T: GenerationCache>(&self, chunk: &mut T, origin: BlockPos) -> bool {
        let chunk_x = chunk.get_center_chunk().x;
        let chunk_z = chunk.get_center_chunk().z;
        let min_x = chunk_x * 16;
        let max_x = min_x + 15;
        let min_z = chunk_z * 16;
        let max_z = min_z + 15;

        let ox = origin.0.x;
        let mut oy = origin.0.y;
        let oz = origin.0.z;

        // Podium spans [ox - 4, ox + 4] and [oz - 4, oz + 4]
        if ox + 4 < min_x || ox - 4 > max_x || oz + 4 < min_z || oz - 4 > max_z {
            return false;
        }

        if oy < 50 {
            oy = 65;
        }

        let in_chunk =
            |x: i32, z: i32| -> bool { x >= min_x && x <= max_x && z >= min_z && z <= max_z };

        for y in (oy - 1)..=(oy + 32) {
            for x in (ox - 4)..=(ox + 4) {
                for z in (oz - 4)..=(oz + 4) {
                    if !in_chunk(x, z) {
                        continue;
                    }

                    let dx = (x - ox) as f64;
                    let dy = (y - oy) as f64;
                    let dz = (z - oz) as f64;
                    let dist_sq = dx * dx + dy * dy + dz * dz;

                    let closer_than_2_5 = dist_sq < 2.5 * 2.5;
                    let closer_than_3_5 = dist_sq < 3.5 * 3.5;

                    if closer_than_2_5 || closer_than_3_5 {
                        let target = Vector3::new(x, y, z);
                        if y < oy {
                            if closer_than_2_5 {
                                chunk.set_block_state(&target, Block::BEDROCK.default_state);
                            } else if self.active {
                                chunk.set_block_state(&target, Block::END_STONE.default_state);
                            }
                        } else if y > oy {
                            chunk.set_block_state(&target, Block::AIR.default_state);
                        } else if !closer_than_2_5 {
                            chunk.set_block_state(&target, Block::BEDROCK.default_state);
                        } else if self.active {
                            chunk.set_block_state(&target, Block::END_PORTAL.default_state);
                        } else {
                            chunk.set_block_state(&target, Block::AIR.default_state);
                        }
                    }
                }
            }
        }

        if in_chunk(ox, oz) {
            for y in 0..4 {
                let target = Vector3::new(ox, oy + y, oz);
                chunk.set_block_state(&target, Block::BEDROCK.default_state);
            }
        }

        let center_of_pillar_y = oy + 2;
        for (dx, dz, facing) in [
            (0i32, -1i32, HorizontalFacing::North),
            (0, 1, HorizontalFacing::South),
            (-1, 0, HorizontalFacing::West),
            (1, 0, HorizontalFacing::East),
        ] {
            let tx = ox + dx;
            let tz = oz + dz;
            if in_chunk(tx, tz) {
                let props = WallTorchLikeProperties { facing };
                let state = props.to_state_id(&Block::WALL_TORCH);
                let target = Vector3::new(tx, center_of_pillar_y, tz);
                chunk.set_block_state(&target, pumpkin_data::BlockState::from_id(state));
            }
        }

        true
    }
}
