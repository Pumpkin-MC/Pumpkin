use pumpkin_data::{Block, BlockDirection};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::{noise::WATER_BLOCK, proto_chunk::GenerationCache};

pub struct DesertWellFeature;

impl DesertWellFeature {
    const CAN_GENERATE: Block = Block::SAND;
    const SAND: Block = Block::SAND;
    const SLAB: Block = Block::SANDSTONE_SLAB;
    const WALL: Block = Block::SANDSTONE;

    #[expect(clippy::too_many_lines)]
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        const CAN_GENERATE: Block = Block::SAND;

        let mut block_pos = pos.up();
        while chunk.is_air(&block_pos.0) && block_pos.0.y > chunk.bottom_y() as i32 + 2 {
            block_pos = block_pos.down();
        }
        let block = GenerationCache::get_block_state(chunk, &block_pos.0).to_block_id();
        if CAN_GENERATE.id != block {
            return false;
        }

        for i in -2..=2 {
            for j2 in -2..=2 {
                if !chunk.is_air(&block_pos.0.add(&Vector3::new(i, -1, j2)))
                    || !chunk.is_air(&block_pos.0.add(&Vector3::new(i, -2, j2)))
                {
                    continue;
                }
                return false;
            }
        }

        for i in -2..=0 {
            for j2 in -2..=2 {
                for k in -2..=2 {
                    chunk.set_block_state(
                        &block_pos.0.add(&Vector3::new(j2, i, k)),
                        Self::WALL.default_state,
                    );
                }
            }
        }

        chunk.set_block_state(&block_pos.0, WATER_BLOCK.default_state);

        for direction in &BlockDirection::horizontal() {
            chunk.set_block_state(
                &block_pos.0.add(&direction.to_offset()),
                WATER_BLOCK.default_state,
            );
        }

        let block_pos2 = &block_pos.0.add(&Vector3::new(0, -1, 0));
        chunk.set_block_state(block_pos2, Self::SAND.default_state);

        for direction2 in &BlockDirection::horizontal() {
            chunk.set_block_state(
                &block_pos2.add(&direction2.to_offset()),
                Self::SAND.default_state,
            );
        }

        for j in -2..=2 {
            for k in -2..=2 {
                if j != -2 && j != 2 && k != -2 && k != 2 {
                    continue;
                }
                chunk.set_block_state(
                    &block_pos.0.add(&Vector3::new(j, 1, k)),
                    Self::WALL.default_state,
                );
            }
        }

        chunk.set_block_state(
            &block_pos.0.add(&Vector3::new(2, 1, 0)),
            Self::SLAB.default_state,
        );
        chunk.set_block_state(
            &block_pos.0.add(&Vector3::new(-2, 1, 0)),
            Self::SLAB.default_state,
        );
        chunk.set_block_state(
            &block_pos.0.add(&Vector3::new(0, 1, 2)),
            Self::SLAB.default_state,
        );
        chunk.set_block_state(
            &block_pos.0.add(&Vector3::new(0, 1, -2)),
            Self::SLAB.default_state,
        );

        for j in -1..=1 {
            for k in -1..=1 {
                if j == 0 && k == 0 {
                    chunk.set_block_state(
                        &block_pos.0.add(&Vector3::new(j, 4, k)),
                        Self::WALL.default_state,
                    );
                    continue;
                }
                chunk.set_block_state(
                    &block_pos.0.add(&Vector3::new(j, 4, k)),
                    Self::SLAB.default_state,
                );
            }
        }

        for j in 1..=3 {
            chunk.set_block_state(
                &block_pos.0.add(&Vector3::new(-1, j, -1)),
                Self::WALL.default_state,
            );
            chunk.set_block_state(
                &block_pos.0.add(&Vector3::new(-1, j, 1)),
                Self::WALL.default_state,
            );
            chunk.set_block_state(
                &block_pos.0.add(&Vector3::new(1, j, -1)),
                Self::WALL.default_state,
            );
            chunk.set_block_state(
                &block_pos.0.add(&Vector3::new(1, j, 1)),
                Self::WALL.default_state,
            );
        }

        let water_center = block_pos.0;
        let water_positions = [
            water_center,
            water_center.add(&Vector3::new(1, 0, 0)),
            water_center.add(&Vector3::new(0, 0, 1)),
            water_center.add(&Vector3::new(-1, 0, 0)),
            water_center.add(&Vector3::new(0, 0, -1)),
        ];
        let pos1 =
            water_positions[random.next_bounded_i32(5) as usize].add(&Vector3::new(0, -1, 0));
        let pos2 =
            water_positions[random.next_bounded_i32(5) as usize].add(&Vector3::new(0, -2, 0));
        Self::place_sus_sand(chunk, &pos1);
        Self::place_sus_sand(chunk, &pos2);

        true
    }

    fn place_sus_sand<T: GenerationCache>(chunk: &mut T, pos: &Vector3<i32>) {
        chunk.set_block_state(pos, Block::SUSPICIOUS_SAND.default_state);
        let mut nbt = NbtCompound::new();
        nbt.put_string("id", "minecraft:brushable_block".to_string());
        nbt.put_int("x", pos.x);
        nbt.put_int("y", pos.y);
        nbt.put_int("z", pos.z);
        nbt.put_string("LootTable", "minecraft:archaeology/desert_well".to_string());
        nbt.put_long("LootTableSeed", BlockPos(*pos).as_long());
        chunk.add_block_entity(pos, nbt);
    }
}
