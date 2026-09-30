use itertools::Itertools;
use pumpkin_data::fluid::{Fluid, FluidState};
use pumpkin_data::tag::{self};
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::generation::proto_chunk::GenerationCache;
use crate::world::BlockAccessor;
use crate::{block::BlockStateCodec, world::WorldPortalExt};

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
    Unobstructed(UnobstructedBlockPredicate),
    AlwaysTrue,
}

impl BlockPredicate {
    pub fn test<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
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
            Self::Unobstructed(predicate) => predicate.test(chunk, pos),
            Self::AlwaysTrue => true,
        }
    }

    pub fn test_world(
        &self,
        world: &dyn BlockAccessor,
        block_registry: Option<&dyn WorldPortalExt>,
        pos: &BlockPos,
    ) -> bool {
        match self {
            Self::MatchingBlocks(predicate) => predicate.test_world(world, pos),
            Self::MatchingBlockTag(predicate) => predicate.test_world(world, pos),
            Self::MatchingFluids(predicate) => predicate.test_world(world, pos),
            Self::HasSturdyFace(predicate) => predicate.test_world(world, pos),
            Self::Solid(predicate) => predicate.test_world(world, pos),
            Self::Replaceable(predicate) => predicate.test_world(world, pos),
            Self::WouldSurvive(predicate) => predicate.test_world(block_registry, world, pos),
            Self::InsideWorldBounds(predicate) => predicate.test_world(world, pos),
            Self::AnyOf(predicate) => predicate.test_world(block_registry, world, pos),
            Self::AllOf(predicate) => predicate.test_world(block_registry, world, pos),
            Self::Not(predicate) => predicate.test_world(block_registry, world, pos),
            Self::Unobstructed(predicate) => predicate.test_world(world, pos),
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
                single_block
                    .strip_prefix("minecraft:")
                    .unwrap_or(single_block)
                    == block.name
            }
            MatchingBlocksWrapper::Multiple(blocks) => blocks
                .iter()
                .map(|s| s.strip_prefix("minecraft:").unwrap_or(s))
                .contains(block.name),
        }
    }

    pub fn test_world(&self, world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = self.offset.get_block_world(world, pos);
        match &self.blocks {
            MatchingBlocksWrapper::Single(single_block) => {
                single_block
                    .strip_prefix("minecraft:")
                    .unwrap_or(single_block)
                    == block.name
            }
            MatchingBlocksWrapper::Multiple(blocks) => blocks
                .iter()
                .map(|s| s.strip_prefix("minecraft:").unwrap_or(s))
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

    pub fn test_world(&self, _world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let pos = pos.offset(self.offset);
        pos.0.y >= -64 && pos.0.y < 320
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
                single_block
                    .strip_prefix("minecraft:")
                    .unwrap_or(single_block)
                    == fluid.name
            }
            MatchingBlocksWrapper::Multiple(blocks) => blocks
                .iter()
                .map(|s| s.strip_prefix("minecraft:").unwrap_or(s))
                .contains(fluid.name),
        }
    }

    pub fn test_world(&self, world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = self.offset.get_block_world(world, pos);
        let fluid_name = match block.name {
            "water" | "flowing_water" => "water",
            "lava" | "flowing_lava" => "lava",
            _ => "empty",
        };
        match &self.fluids {
            MatchingBlocksWrapper::Single(single_block) => {
                single_block
                    .strip_prefix("minecraft:")
                    .unwrap_or(single_block)
                    == fluid_name
            }
            MatchingBlocksWrapper::Multiple(blocks) => blocks
                .iter()
                .map(|s| s.strip_prefix("minecraft:").unwrap_or(s))
                .contains(fluid_name),
        }
    }
}

pub struct MatchingBlockTagPredicate {
    pub offset: OffsetBlocksBlockPredicate,
    pub tag: tag::Tag,
}

impl MatchingBlockTagPredicate {
    pub fn test<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> bool {
        let state = self.offset.get_id(chunk, pos);
        state.to_block_id().has_tag(self.tag)
    }

    pub fn test_world(&self, world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let state = self.offset.get_id_world(world, pos);
        state.to_block_id().has_tag(self.tag)
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

    pub fn test_world(&self, world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let state = self.offset.get_state_world(world, pos);
        state.is_side_solid(self.direction)
    }
}

pub struct AnyOfBlockPredicate {
    pub predicates: Vec<BlockPredicate>,
}

impl AnyOfBlockPredicate {
    pub fn test<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
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

    pub fn test_world(
        &self,
        block_registry: Option<&dyn WorldPortalExt>,
        world: &dyn BlockAccessor,
        pos: &BlockPos,
    ) -> bool {
        for predicate in &self.predicates {
            if predicate.test_world(world, block_registry, pos) {
                return true;
            }
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
        block_registry: &dyn WorldPortalExt,
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

    pub fn test_world(
        &self,
        block_registry: Option<&dyn WorldPortalExt>,
        world: &dyn BlockAccessor,
        pos: &BlockPos,
    ) -> bool {
        for predicate in &self.predicates {
            if !predicate.test_world(world, block_registry, pos) {
                return false;
            }
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
        block_registry: &dyn WorldPortalExt,
        chunk: &T,
        pos: &BlockPos,
    ) -> bool {
        !self.predicate.test(block_registry, chunk, pos)
    }

    pub fn test_world(
        &self,
        block_registry: Option<&dyn WorldPortalExt>,
        world: &dyn BlockAccessor,
        pos: &BlockPos,
    ) -> bool {
        !self.predicate.test_world(world, block_registry, pos)
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

    pub fn test_world(&self, world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let state = self.offset.get_state_world(world, pos);
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
        block_registry: &dyn WorldPortalExt,
        chunk: &T,
        pos: &BlockPos,
    ) -> bool {
        let block = self.state.get_block();
        let state = self.state.get_state();

        let pos = self.offset.get(pos);
        block_registry.can_place_at(block, state, chunk, &pos)
    }

    pub fn test_world(
        &self,
        block_registry: Option<&dyn WorldPortalExt>,
        world: &dyn BlockAccessor,
        pos: &BlockPos,
    ) -> bool {
        let block = self.state.get_block();
        let state = self.state.get_state();
        let pos = self.offset.get(pos);
        block_registry.is_none_or(|registry| registry.can_place_at(block, state, world, &pos))
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

    pub fn test_world(&self, world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let state = self.offset.get_state_world(world, pos);
        state.replaceable()
    }
}

pub struct UnobstructedBlockPredicate {
    pub offset: Option<Vector3<i32>>,
}

impl UnobstructedBlockPredicate {
    pub const fn test<T: GenerationCache>(&self, _chunk: &T, _pos: &BlockPos) -> bool {
        true
    }

    pub const fn test_world(&self, _world: &dyn BlockAccessor, _pos: &BlockPos) -> bool {
        true
    }
}

pub struct OffsetBlocksBlockPredicate {
    pub offset: Option<Vector3<i32>>,
}

impl OffsetBlocksBlockPredicate {
    #[must_use]
    pub fn get(&self, pos: &BlockPos) -> BlockPos {
        if let Some(offset) = self.offset {
            return pos.offset(offset);
        }
        *pos
    }
    pub fn get_id<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> BlockStateId {
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

    pub fn get_id_world(&self, world: &dyn BlockAccessor, pos: &BlockPos) -> BlockStateId {
        let pos = self.get(pos);
        world.get_block_state_id(&pos)
    }

    pub fn get_block_world(&self, world: &dyn BlockAccessor, pos: &BlockPos) -> &'static Block {
        let pos = self.get(pos);
        world.get_block(&pos)
    }

    pub fn get_state_world(
        &self,
        world: &dyn BlockAccessor,
        pos: &BlockPos,
    ) -> &'static BlockState {
        let pos = self.get(pos);
        world.get_block_state(&pos)
    }
}

pub enum MatchingBlocksWrapper {
    Single(String),
    Multiple(Vec<String>),
}
