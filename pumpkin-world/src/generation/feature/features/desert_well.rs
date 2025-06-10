use pumpkin_data::BlockDirection;
use pumpkin_macros::default_block_state;
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::RandomGenerator,
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    block::RawBlockState,
    generation::{chunk_noise::WATER_BLOCK, height_limit::HeightLimitView},
};

// TODO: remove .to_state()

#[derive(Deserialize)]
pub struct DesertWellFeature;

impl DesertWellFeature {
    const CAN_GENERATE: RawBlockState = default_block_state!("sand");
    const SAND: RawBlockState = default_block_state!("sand");
    const SLAB: RawBlockState = default_block_state!("sandstone_slab");
    const WALL: RawBlockState = default_block_state!("sandstone");

    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        _min_y: i8,
        _height: u16,
        _feature: &str, // This placed feature
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut block_pos = pos.up();
        while chunk.is_air(&block_pos.0) && block_pos.0.y > chunk.bottom_y() as i32 + 2 {
            block_pos = block_pos.down();
        }
        let block = chunk.get_block_state(&pos.0).to_block();
        const CAN_GENERATE: RawBlockState = default_block_state!("sand");
        if CAN_GENERATE.to_block().id != block.id {
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
                        &Self::WALL.to_state(),
                    );
                }
            }
        }

        chunk.set_block_state(&block_pos.0, &WATER_BLOCK.to_state());

        for direction in BlockDirection::horizontal().iter() {
            chunk.set_block_state(
                &block_pos.0.add(&direction.to_offset()),
                &WATER_BLOCK.to_state(),
            );
        }

        let block_pos2 = &block_pos.0.add(&Vector3::new(0, -1, 0));
        chunk.set_block_state(block_pos2, &Self::SAND.to_state());

        for direction2 in BlockDirection::horizontal().iter() {
            chunk.set_block_state(
                &block_pos2.add(&direction2.to_offset()),
                &Self::SAND.to_state(),
            );
        }

        for j in -2..=2 {
            for k in -2..=2 {
                if j != -2 && j != 2 && k != -2 && k != 2 {
                    continue;
                }
                chunk.set_block_state(
                    &block_pos.0.add(&Vector3::new(j, 1, k)),
                    &Self::WALL.to_state(),
                );
            }
        }

        chunk.set_block_state(
            &block_pos.0.add(&Vector3::new(2, 1, 0)),
            &Self::SLAB.to_state(),
        );
        chunk.set_block_state(
            &block_pos.0.add(&Vector3::new(-2, 1, 0)),
            &Self::SLAB.to_state(),
        );
        chunk.set_block_state(
            &block_pos.0.add(&Vector3::new(0, 1, 2)),
            &Self::SLAB.to_state(),
        );
        chunk.set_block_state(
            &block_pos.0.add(&Vector3::new(0, 1, -2)),
            &Self::SLAB.to_state(),
        );

        for j in -1..=1 {
            for k in -1..=1 {
                if j == 0 && k == 0 {
                    chunk.set_block_state(
                        &block_pos.0.add(&Vector3::new(j, 4, k)),
                        &Self::WALL.to_state(),
                    );
                    continue;
                }
                chunk.set_block_state(
                    &block_pos.0.add(&Vector3::new(j, 4, k)),
                    &Self::SLAB.to_state(),
                );
            }
        }

        for j in 1..=3 {
            chunk.set_block_state(
                &block_pos.0.add(&Vector3::new(-1, j, -1)),
                &Self::WALL.to_state(),
            );
            chunk.set_block_state(
                &block_pos.0.add(&Vector3::new(-1, j, 1)),
                &Self::WALL.to_state(),
            );
            chunk.set_block_state(
                &block_pos.0.add(&Vector3::new(1, j, -1)),
                &Self::WALL.to_state(),
            );
            chunk.set_block_state(
                &block_pos.0.add(&Vector3::new(1, j, 1)),
                &Self::WALL.to_state(),
            );
        }

        true
    }
}
