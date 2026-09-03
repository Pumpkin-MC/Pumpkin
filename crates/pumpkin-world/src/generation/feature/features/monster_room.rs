use pumpkin_data::{Block, BlockDirection};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

const DUNGEON_MOBS: [&str; 4] = [
    "minecraft:skeleton",
    "minecraft:zombie",
    "minecraft:zombie",
    "minecraft:spider",
];

pub struct DungeonFeature;

impl DungeonFeature {
    #[expect(clippy::too_many_lines)]
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
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
                        return false;
                    }
                    if dy == 4 && !solid {
                        return false;
                    }

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
                                chunk.set_block_state(
                                    &wall_pos,
                                    Block::MOSSY_COBBLESTONE.default_state,
                                );
                            } else {
                                chunk.set_block_state(&wall_pos, Block::COBBLESTONE.default_state);
                            }
                        }
                    } else {
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

        for _ in 0..2 {
            for _ in 0..3 {
                let cx = pos.0.x + random.next_bounded_i32(xr * 2 + 1) - xr;
                let cy = pos.0.y;
                let cz = pos.0.z + random.next_bounded_i32(zr * 2 + 1) - zr;
                let chest_pos = Vector3::new(cx, cy, cz);

                if !chunk.is_air(&chest_pos) {
                    continue;
                }

                let wall_count = BlockDirection::horizontal()
                    .iter()
                    .filter(|d| !chunk.is_air(&chest_pos.add(&d.to_offset())))
                    .count();

                if wall_count == 1 {
                    let facing_dir = BlockDirection::horizontal()
                        .iter()
                        .find(|d| !chunk.is_air(&chest_pos.add(&d.to_offset())))
                        .copied();

                    let chest_state = facing_dir.map_or(Block::CHEST.default_state, |dir| {
                        use pumpkin_data::block_properties::ChestLikeProperties;
                        let mut props = ChestLikeProperties::default(&Block::CHEST);
                        props.facing = dir.opposite();
                        let state_id = props.to_state_id(&Block::CHEST);
                        pumpkin_data::BlockState::from_id(state_id)
                    });

                    chunk.set_block_state(&chest_pos, chest_state);

                    let loot_seed = random.next_i64();
                    let mut chest_nbt = NbtCompound::new();
                    chest_nbt.put_string("id", "minecraft:chest".to_string());
                    chest_nbt.put_int("x", chest_pos.x);
                    chest_nbt.put_int("y", chest_pos.y);
                    chest_nbt.put_int("z", chest_pos.z);
                    chest_nbt
                        .put_string("LootTable", "minecraft:chests/simple_dungeon".to_string());
                    chest_nbt.put_long("LootTableSeed", loot_seed);
                    chunk.add_block_entity(&chest_pos, chest_nbt);

                    break;
                }
            }
        }

        let mob = DUNGEON_MOBS[random.next_bounded_i32(DUNGEON_MOBS.len() as i32) as usize];
        chunk.set_block_state(&pos.0, Block::SPAWNER.default_state);

        let mut entity_nbt = NbtCompound::new();
        entity_nbt.put_string("id", "minecraft:mob_spawner".to_string());
        entity_nbt.put_int("x", pos.0.x);
        entity_nbt.put_int("y", pos.0.y);
        entity_nbt.put_int("z", pos.0.z);

        let mut spawn_entry = NbtCompound::new();
        let mut entity_nbt_inner = NbtCompound::new();
        entity_nbt_inner.put_string("id", mob.to_string());
        spawn_entry.put_compound("entity", entity_nbt_inner);
        entity_nbt.put_compound("SpawnData", spawn_entry);

        chunk.add_block_entity(&pos.0, entity_nbt);

        true
    }
}
