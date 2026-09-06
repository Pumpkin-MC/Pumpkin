use pumpkin_data::{Block, BlockDirection, BlockState, tag::Taggable};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::RandomGenerator,
};

use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;

pub struct SpringFeatureFeature {
    pub state: &'static BlockState,
    pub requires_block_below: bool,
    pub rock_count: i32,
    pub hole_count: i32,
    pub valid_blocks: BlockWrapper,
}

pub enum BlockWrapper {
    Single(String),
    Multi(Vec<String>),
}

impl BlockWrapper {
    /// The raw entries of the vanilla `HolderSet`: either plain block ids
    /// (`"minecraft:deepslate"`) or tag references (`"#minecraft:geode_invalid_blocks"`).
    pub fn entries(&self) -> impl Iterator<Item = &str> {
        match self {
            Self::Single(s) => std::slice::from_ref(s),
            Self::Multi(v) => v.as_slice(),
        }
        .iter()
        .map(String::as_str)
    }

    /// Vanilla compares with `BlockState.is(HolderSet<Block>)`, i.e. against *resolved*
    /// registry entries. The datapack JSON spells them with their namespace
    /// (`"minecraft:deepslate"`) while `Block::name` is the bare path (`"deepslate"`), so the
    /// entries have to go through `Block::from_name` (which strips the namespace) instead of
    /// being compared as strings.
    #[must_use]
    pub fn contains_block(&self, block: &Block) -> bool {
        self.entries().any(|entry| {
            entry.strip_prefix('#').map_or_else(
                || Block::from_name(entry).is_some_and(|b| b.id == block.id),
                |tag| {
                    Block::get_tag_values(tag)
                        .into_iter()
                        .flatten()
                        .any(|name| Block::from_name(name).is_some_and(|b| b.id == block.id))
                },
            )
        })
    }
}

/// `SpringFeature.place` inspects west, east, north, south and below (never above).
const NEIGHBOURS: [BlockDirection; 5] = [
    BlockDirection::West,
    BlockDirection::East,
    BlockDirection::North,
    BlockDirection::South,
    BlockDirection::Down,
];

impl SpringFeatureFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        _block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let valid_blocks = &self.valid_blocks;
        let block_at = |chunk: &mut T, offset: Vector3<i32>| {
            GenerationCache::get_block_state(chunk, &(pos.0 + offset)).to_block()
        };

        if !valid_blocks.contains_block(block_at(chunk, Vector3::new(0, 1, 0))) {
            return false;
        }
        if self.requires_block_below
            && !valid_blocks.contains_block(block_at(chunk, BlockDirection::Down.to_offset()))
        {
            return false;
        }
        let state = GenerationCache::get_block_state(chunk, &pos.0);
        if !state.to_state().is_air() && !valid_blocks.contains_block(state.to_block()) {
            return false;
        }

        let mut valid = 0;
        for direction in NEIGHBOURS {
            if valid_blocks.contains_block(block_at(chunk, direction.to_offset())) {
                valid += 1;
            }
        }
        let mut air = 0;
        for direction in NEIGHBOURS {
            if chunk.is_air(&(pos.0 + direction.to_offset())) {
                air += 1;
            }
        }

        if valid == self.rock_count && air == self.hole_count {
            chunk.set_block_state(&pos.0, self.state);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::Block;

    use super::BlockWrapper;

    /// `spring_lava_overworld.json` lists its `valid_blocks` with the `minecraft:` namespace,
    /// and vanilla resolves them into a `HolderSet<Block>` before
    /// `SpringFeature.place` asks `level.getBlockState(origin.above()).is(config.validBlocks)`.
    /// Comparing the namespaced id against the bare `Block::name` never matched, so no
    /// overworld spring was ever placed.
    #[test]
    fn namespaced_ids_resolve_to_blocks() {
        let valid = BlockWrapper::Multi(
            [
                "minecraft:stone",
                "minecraft:granite",
                "minecraft:diorite",
                "minecraft:andesite",
                "minecraft:deepslate",
                "minecraft:tuff",
                "minecraft:calcite",
                "minecraft:dirt",
            ]
            .map(str::to_string)
            .to_vec(),
        );

        assert!(valid.contains_block(&Block::DEEPSLATE));
        assert!(valid.contains_block(&Block::TUFF));
        assert!(valid.contains_block(&Block::DIRT));
        assert!(!valid.contains_block(&Block::AIR));
        assert!(!valid.contains_block(&Block::GRAVEL));

        // A bare id keeps working, and so does a `#tag` reference.
        assert!(
            BlockWrapper::Single("minecraft:netherrack".to_string())
                .contains_block(&Block::NETHERRACK)
        );
        assert!(
            BlockWrapper::Single("#minecraft:base_stone_overworld".to_string())
                .contains_block(&Block::DEEPSLATE)
        );
    }
}
