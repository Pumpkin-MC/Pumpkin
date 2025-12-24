use pumpkin_data::BlockDirection;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;

use crate::generation::proto_chunk::GenerationCache;
use crate::{block::BlockStateCodec, world::BlockRegistryExt};

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

impl BlockWrapper {
    fn contains(&self, name: &str) -> bool {
        match self {
            Self::Single(s) => s == name,
            Self::Multi(h) => h.contains(&name.to_string()),
        }
    }
}

impl SpringFeatureFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        _block_registry: &dyn BlockRegistryExt,
        chunk: &mut T,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let valid_blocks = &self.valid_blocks;

        if !valid_blocks.contains(
            GenerationCache::get_block_state(chunk, &pos.up().0)
                .to_block()
                .name,
        ) {
            return false;
        }
        if self.requires_block_below
            && !valid_blocks.contains(
                GenerationCache::get_block_state(
                    chunk,
                    &pos.offset(BlockDirection::Down.to_offset()).0,
                )
                .to_block()
                .name,
            )
        {
            return false;
        }
        let state = GenerationCache::get_block_state(chunk, &pos.0);
        if !state.to_state().is_air() && !valid_blocks.contains(state.to_block().name) {
            return false;
        }

        let mut valid = 0;
        if valid_blocks.contains(
            GenerationCache::get_block_state(
                chunk,
                &pos.offset(BlockDirection::West.to_offset()).0,
            )
            .to_block()
            .name,
        ) {
            valid += 1;
        }
        if valid_blocks.contains(
            GenerationCache::get_block_state(
                chunk,
                &pos.offset(BlockDirection::East.to_offset()).0,
            )
            .to_block()
            .name,
        ) {
            valid += 1;
        }
        if valid_blocks.contains(
            GenerationCache::get_block_state(
                chunk,
                &pos.offset(BlockDirection::North.to_offset()).0,
            )
            .to_block()
            .name,
        ) {
            valid += 1;
        }
        if valid_blocks.contains(
            GenerationCache::get_block_state(
                chunk,
                &pos.offset(BlockDirection::South.to_offset()).0,
            )
            .to_block()
            .name,
        ) {
            valid += 1;
        }
        if valid_blocks.contains(
            GenerationCache::get_block_state(
                chunk,
                &pos.offset(BlockDirection::Down.to_offset()).0,
            )
            .to_block()
            .name,
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
            chunk.set_block_state(&pos.0, self.state.get_state());
            return true;
        }
        false
    }
}
