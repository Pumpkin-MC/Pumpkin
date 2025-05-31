use pumpkin_data::BlockDirection;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;

use crate::{ProtoChunk, block::BlockStateCodec, world::BlockRegistryExt};

#[derive(Deserialize)]
pub struct SpringFeatureFeature {
    state: BlockStateCodec,
    requires_block_below: bool,
    rock_count: i32,
    hole_count: i32,
    valid_blocks: BlockWrapper,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
enum BlockWrapper {
    Single(String),
    Multi(Vec<String>),
}

impl SpringFeatureFeature {
    pub fn generate(
        &self,
        _block_registry: &dyn BlockRegistryExt,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        // i don't think this is the most efficient way, but it works
        let valid_blocks = match self.valid_blocks.clone() {
            BlockWrapper::Single(item) => vec![item],
            BlockWrapper::Multi(items) => items,
        };
        if !valid_blocks.contains(
            &chunk
                .get_block_state(&pos.up().0)
                .to_block()
                .name
                .to_string(),
        ) {
            return false;
        }
        if self.requires_block_below
            && !valid_blocks.contains(
                &chunk
                    .get_block_state(&pos.offset(BlockDirection::Down.to_offset()).0)
                    .to_block()
                    .name
                    .to_string(),
            )
        {
            return false;
        }
        let state = chunk.get_block_state(&pos.0);
        if !state.to_state().is_air() && !valid_blocks.contains(&state.to_block().name.to_string())
        {
            return false;
        }

        let mut valid = 0;
        if valid_blocks.contains(
            &chunk
                .get_block_state(&pos.offset(BlockDirection::West.to_offset()).0)
                .to_block()
                .name
                .to_string(),
        ) {
            valid += 1;
        }
        if valid_blocks.contains(
            &chunk
                .get_block_state(&pos.offset(BlockDirection::East.to_offset()).0)
                .to_block()
                .name
                .to_string(),
        ) {
            valid += 1;
        }
        if valid_blocks.contains(
            &chunk
                .get_block_state(&pos.offset(BlockDirection::North.to_offset()).0)
                .to_block()
                .name
                .to_string(),
        ) {
            valid += 1;
        }
        if valid_blocks.contains(
            &chunk
                .get_block_state(&pos.offset(BlockDirection::South.to_offset()).0)
                .to_block()
                .name
                .to_string(),
        ) {
            valid += 1;
        }
        if valid_blocks.contains(
            &chunk
                .get_block_state(&pos.offset(BlockDirection::Down.to_offset()).0)
                .to_block()
                .name
                .to_string(),
        ) {
            valid += 1;
        }
        let mut air = 0;
        if chunk.is_air(&pos.offset(BlockDirection::West.to_offset()).0) {
            air += 1;
        }
        if chunk.is_air(&pos.offset(BlockDirection::East.to_offset()).0) {
            air += 1;
        }
        if chunk.is_air(&pos.offset(BlockDirection::North.to_offset()).0) {
            air += 1;
        }
        if chunk.is_air(&pos.offset(BlockDirection::South.to_offset()).0) {
            air += 1;
        }
        if chunk.is_air(&pos.offset(BlockDirection::Down.to_offset()).0) {
            air += 1;
        }
        if valid == self.rock_count && air == self.hole_count {
            chunk.set_block_state(&pos.0, &self.state.get_state().unwrap());
            return true;
        }
        false
    }
}
