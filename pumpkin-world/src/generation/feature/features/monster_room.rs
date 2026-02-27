use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

/// The three mob types that can appear in a dungeon spawner.
///
/// Skeleton 25%, Zombie 50%, Spider 25%.
const DUNGEON_MOBS: [&str; 4] = [
    "minecraft:skeleton",
    "minecraft:zombie",
    "minecraft:zombie",
    "minecraft:spider",
];

pub struct DungeonFeature {}

impl DungeonFeature {
    /// Generate a monster room (dungeon) at `pos`.
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let xr = random.next_bounded_i32(2) + 2;
        let zr = random.next_bounded_i32(2) + 2;

        let min_x = -xr - 1;
        let max_x = xr + 1;
        let min_z = -zr - 1;
        let max_z = zr + 1;

        let mut hole_count = 0i32;
        for dx in min_x..=max_x {
            for dy in -1..=4i32 {
                for dz in min_z..=max_z {
                    let check_pos = pos.0.add(&Vector3::new(dx, dy, dz));
                    let solid = !chunk.is_air(&check_pos);

                    if dy == -1 && !solid {
                        return false; // Floor has a gap
                    }
                    if dy == 4 && !solid {
                        return false; // Ceiling has a gap
                    }

                    // Wall openings
                    let on_wall = dx == min_x || dx == max_x || dz == min_z || dz == max_z;
                    if on_wall
                        && dy == 0
                        && chunk.is_air(&check_pos)
                        && chunk.is_air(&check_pos.add(&Vector3::new(0, 1, 0)))
                    {
                        hole_count += 1;
                    }
                }
            }
        }

        if !(1..=5).contains(&hole_count) {
            return false;
        }

        for dx in min_x..=max_x {
            for dy in (-1..=3i32).rev() {
                for dz in min_z..=max_z {
                    let wall_pos = pos.0.add(&Vector3::new(dx, dy, dz));

                    let on_boundary = dx == min_x
                        || dy == -1
                        || dy == 3
                        || dz == min_z
                        || dx == max_x
                        || dy == 4
                        || dz == max_z;

                    if on_boundary {
                        let below_pos = wall_pos.add(&Vector3::new(0, -1, 0));
                        let below_solid = !chunk.is_air(&below_pos);
                        let cur_solid = !chunk.is_air(&wall_pos);
                        let is_chest = GenerationCache::get_block_state(chunk, &wall_pos)
                            .to_block()
                            == &Block::CHEST;

                        let world_min_y = chunk.bottom_y() as i32;
                        if wall_pos.y >= world_min_y && !below_solid {
                            chunk.set_block_state(&wall_pos, Block::CAVE_AIR.default_state);
                        } else if cur_solid && !is_chest {
                            if dy == -1 && random.next_bounded_i32(4) != 0 {
                                // 75% mossy cobblestone
                                chunk.set_block_state(
                                    &wall_pos,
                                    Block::MOSSY_COBBLESTONE.default_state,
                                );
                            } else {
                                // 25% cobblestone (or 100% if it's not the floor)
                                chunk.set_block_state(&wall_pos, Block::COBBLESTONE.default_state);
                            }
                        }
                    } else {
                        // Clear interior, but preserve chests and spawners
                        let state = GenerationCache::get_block_state(chunk, &wall_pos);
                        let is_chest = state.to_block() == &Block::CHEST;
                        let is_spawner = state.to_block() == &Block::SPAWNER;
                        if !is_chest && !is_spawner {
                            chunk.set_block_state(&wall_pos, Block::CAVE_AIR.default_state);
                        }
                    }
                }
            }
        }

        // Chest placement attempts (rooms have 1-2 chests)
        for _ in 0..2 {
            for _ in 0..3 {
                let cx = pos.0.x + random.next_bounded_i32(xr * 2 + 1) - xr;
                let cy = pos.0.y;
                let cz = pos.0.z + random.next_bounded_i32(zr * 2 + 1) - zr;
                let chest_pos = Vector3::new(cx, cy, cz);

                if !chunk.is_air(&chest_pos) {
                    continue;
                }

                // Count solid horizontal neighbours
                let wall_count = BlockDirection::horizontal()
                    .iter()
                    .filter(|d| !chunk.is_air(&chest_pos.add(&d.to_offset())))
                    .count();

                if wall_count == 1 {
                    // TODO: Set chest loot and facing
                    chunk.set_block_state(&chest_pos, Block::CHEST.default_state);
                    break;
                }
            }
        }

        // TODO: set spawner entity type
        let _ = DUNGEON_MOBS[random.next_bounded_i32(DUNGEON_MOBS.len() as i32) as usize];
        chunk.set_block_state(&pos.0, Block::SPAWNER.default_state);

        true
    }
}
