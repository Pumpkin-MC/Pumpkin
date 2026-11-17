use pumpkin_data::{Block, BlockDirection};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    HeightMap,
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct BonusChestFeature;

impl BonusChestFeature {
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let chunk_min_x = (pos.0.x >> 4) << 4;
        let chunk_min_z = (pos.0.z >> 4) << 4;

        let mut x_poses: Vec<i32> = (chunk_min_x..=chunk_min_x + 15).collect();
        let mut z_poses: Vec<i32> = (chunk_min_z..=chunk_min_z + 15).collect();

        for i in (1..x_poses.len()).rev() {
            let j = random.next_bounded_i32(i as i32 + 1) as usize;
            x_poses.swap(i, j);
        }
        for i in (1..z_poses.len()).rev() {
            let j = random.next_bounded_i32(i as i32 + 1) as usize;
            z_poses.swap(i, j);
        }

        for &x in &x_poses {
            for &z in &z_poses {
                let y = chunk.get_top_y(&HeightMap::MotionBlockingNoLeaves, x, z);
                let chest_pos = Vector3::new(x, y, z);
                if chunk.is_air(&chest_pos) {
                    chunk.set_block_state(&chest_pos, Block::CHEST.default_state);

                    let loot_seed = random.next_i64();
                    let mut chest_nbt = NbtCompound::new();
                    chest_nbt.put_string("id", "minecraft:chest".to_string());
                    chest_nbt.put_int("x", chest_pos.x);
                    chest_nbt.put_int("y", chest_pos.y);
                    chest_nbt.put_int("z", chest_pos.z);
                    chest_nbt.put_string(
                        "LootTable",
                        "minecraft:chests/spawn_bonus_chest".to_string(),
                    );
                    chest_nbt.put_long("LootTableSeed", loot_seed);
                    chunk.add_block_entity(&chest_pos, chest_nbt);

                    for dir in BlockDirection::horizontal() {
                        let torch_pos = chest_pos.add(&dir.to_offset());
                        let below_torch = torch_pos.add(&Vector3::new(0, -1, 0));
                        if !chunk.is_air(&below_torch) && chunk.is_air(&torch_pos) {
                            chunk.set_block_state(&torch_pos, Block::TORCH.default_state);
                        }
                    }

                    return true;
                }
            }
        }

        false
    }
}
