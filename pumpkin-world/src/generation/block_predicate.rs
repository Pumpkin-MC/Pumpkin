use itertools::Itertools;
use pumpkin_data::fluid::{Fluid, FluidState};
use pumpkin_data::tag::{RegistryKey, get_tag_ids};
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::block::RawBlockState;
use crate::generation::proto_chunk::GenerationCache;
use crate::{block::BlockStateCodec, world::BlockRegistryExt};

pub enum BlockPredicate {
    MatchingBlocks(MatchingBlocksBlockPredicate),
    MatchingBlockTag(MatchingBlockTagPredicate),
    MatchingFluids(MatchingFluidsBlockPredicate),
    HasSturdyFace(HasSturdyFacePredicate),
    Solid(SolidBlockPredicate),
    Replaceable(ReplaceableBlockPredicate),
    WouldSurvive(WouldSurviveBlockPredicate),
    InsideWorldBounds(InsideWorldBoundsBlockPredicate),
    AnyOf(AnyOfBlockPredicate),
    AllOf(AllOfBlockPredicate),
    Not(NotBlockPredicate),
    AlwaysTrue,
    // Not used
    //     // Unobstructed(EmptyTODOStruct),
}

impl BlockPredicate {
    pub fn test<T: GenerationCache>(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &T,
        pos: &BlockPos,
    ) -> bool {
        match self {
            Self::MatchingBlocks(predicate) => predicate.test(chunk, pos),
            Self::MatchingBlockTag(predicate) => predicate.test(chunk, pos),
            Self::MatchingFluids(predicate) => predicate.test(chunk, pos),
            Self::HasSturdyFace(predicate) => predicate.test(chunk, pos),
            Self::Solid(predicate) => predicate.test(chunk, pos),
            Self::Replaceable(predicate) => predicate.test(chunk, pos),
            Self::WouldSurvive(predicate) => predicate.test(block_registry, chunk, pos),
            Self::InsideWorldBounds(predicate) => predicate.test(chunk, pos),
            Self::AnyOf(predicate) => predicate.test(block_registry, chunk, pos),
            Self::AllOf(predicate) => predicate.test(block_registry, chunk, pos),
            Self::Not(predicate) => predicate.test(block_registry, chunk, pos),
            Self::AlwaysTrue => true,
        }
    }
}

pub struct MatchingBlocksBlockPredicate {
    pub offset: OffsetBlocksBlockPredicate,
    pub blocks: MatchingBlocksWrapper,
}

impl MatchingBlocksBlockPredicate {
    pub fn test<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> bool {
        let block = self.offset.get_block(chunk, pos);
        match &self.blocks {
            MatchingBlocksWrapper::Single(single_block) => {
                single_block.strip_prefix("minecraft:").unwrap() == block.name
            }
            MatchingBlocksWrapper::Multiple(blocks) => blocks
                .iter()
                .map(|s| s.strip_prefix("minecraft:").unwrap())
                .contains(block.name),
        }
    }
}

pub struct InsideWorldBoundsBlockPredicate {
    pub offset: Vector3<i32>,
}

impl InsideWorldBoundsBlockPredicate {
    pub fn test<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> bool {
        let pos = pos.offset(self.offset);
        !chunk.out_of_height(pos.0.y as i16)
    }
}

pub struct MatchingFluidsBlockPredicate {
    pub offset: OffsetBlocksBlockPredicate,
    pub fluids: MatchingBlocksWrapper,
}

impl MatchingFluidsBlockPredicate {
    pub fn test<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> bool {
        let (fluid, _) = self.offset.get_fluid_and_fluid_state(chunk, pos);
        match &self.fluids {
            MatchingBlocksWrapper::Single(single_block) => {
                single_block.strip_prefix("minecraft:").unwrap() == fluid.name
            }
            MatchingBlocksWrapper::Multiple(blocks) => blocks
                .iter()
                .map(|s| s.strip_prefix("minecraft:").unwrap())
                .contains(fluid.name),
        }
    }
}

pub struct MatchingBlockTagPredicate {
    pub offset: OffsetBlocksBlockPredicate,
    pub tag: String,
}

impl MatchingBlockTagPredicate {
    pub fn test<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> bool {
        let block = self.offset.get_raw(chunk, pos);
        get_tag_ids(RegistryKey::Block, &self.tag)
            .unwrap()
            .contains(&block.to_block_id())
    }
}

pub struct HasSturdyFacePredicate {
    pub offset: OffsetBlocksBlockPredicate,
    pub direction: BlockDirection,
}

impl HasSturdyFacePredicate {
    pub fn test<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> bool {
        let state = self.offset.get_state(chunk, pos);
        state.is_side_solid(self.direction)
    }
}

pub struct AnyOfBlockPredicate {
    pub predicates: Vec<BlockPredicate>,
}

impl AnyOfBlockPredicate {
    pub fn test<T: GenerationCache>(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &T,
        pos: &BlockPos,
    ) -> bool {
        for predicate in &self.predicates {
            if !predicate.test(block_registry, chunk, pos) {
                continue;
            }
            return true;
        }
        false
    }
}

pub struct AllOfBlockPredicate {
    pub predicates: Vec<BlockPredicate>,
}

impl AllOfBlockPredicate {
    pub fn test<T: GenerationCache>(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &T,
        pos: &BlockPos,
    ) -> bool {
        for predicate in &self.predicates {
            if predicate.test(block_registry, chunk, pos) {
                continue;
            }
            return false;
        }
        true
    }
}

pub struct NotBlockPredicate {
    pub predicate: Box<BlockPredicate>,
}

impl NotBlockPredicate {
    pub fn test<T: GenerationCache>(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &T,
        pos: &BlockPos,
    ) -> bool {
        !self.predicate.test(block_registry, chunk, pos)
    }
}

pub struct SolidBlockPredicate {
    pub offset: OffsetBlocksBlockPredicate,
}

impl SolidBlockPredicate {
    pub fn test<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> bool {
        let state = self.offset.get_state(chunk, pos);
        state.is_solid()
    }
}

pub struct WouldSurviveBlockPredicate {
    pub offset: OffsetBlocksBlockPredicate,
    pub state: BlockStateCodec,
}

impl WouldSurviveBlockPredicate {
    pub fn test<T: GenerationCache>(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &T,
        pos: &BlockPos,
    ) -> bool {
        let block = self.state.get_block();
        let state = self.state.get_state();

        let pos = self.offset.get(pos);
        block_registry.can_place_at(block, state, chunk, &pos)
    }
}

pub struct ReplaceableBlockPredicate {
    pub offset: OffsetBlocksBlockPredicate,
}

impl ReplaceableBlockPredicate {
    pub fn test<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> bool {
        let state = self.offset.get_state(chunk, pos);
        state.replaceable()
    }
}

pub struct OffsetBlocksBlockPredicate {
    pub offset: Option<Vector3<i32>>,
}

impl OffsetBlocksBlockPredicate {
    pub fn get(&self, pos: &BlockPos) -> BlockPos {
        if let Some(offset) = self.offset {
            return pos.offset(offset);
        }
        *pos
    }
    pub fn get_raw<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> RawBlockState {
        let pos = self.get(pos);
        GenerationCache::get_block_state(chunk, &pos.0)
    }

    pub fn get_block<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> &'static Block {
        let pos = self.get(pos);
        GenerationCache::get_block_state(chunk, &pos.0).to_block()
    }

    pub fn get_fluid_and_fluid_state<T: GenerationCache>(
        &self,
        chunk: &T,
        pos: &BlockPos,
    ) -> (Fluid, FluidState) {
        let pos = self.get(pos);
        GenerationCache::get_fluid_and_fluid_state(chunk, &pos.0)
    }

    pub fn get_state<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> &'static BlockState {
        let pos = self.get(pos);
        GenerationCache::get_block_state(chunk, &pos.0).to_state()
    }
}

pub enum MatchingBlocksWrapper {
    Single(String),
    Multiple(Vec<String>),
}
