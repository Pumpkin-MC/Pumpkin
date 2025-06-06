use itertools::Itertools;
use pumpkin_data::{
    Block, BlockDirection, BlockState, block_properties::get_block_by_state_id, tag::Tagable,
};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use serde::Deserialize;

use crate::{
    ProtoChunk, block::BlockStateCodec, generation::height_limit::HeightLimitView,
    world::BlockRegistryExt,
};

#[derive(Deserialize)]
pub struct EmptyTODOStruct {}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum BlockPredicate {
    #[serde(rename = "minecraft:matching_blocks")]
    MatchingBlocks(MatchingBlocksBlockPredicate),
    #[serde(rename = "minecraft:matching_block_tag")]
    MatchingBlockTag(MatchingBlockTagPredicate),
    #[serde(rename = "minecraft:matching_fluids")]
    MatchingFluids(EmptyTODOStruct),
    #[serde(rename = "minecraft:has_sturdy_face")]
    HasSturdyFace(HasSturdyFacePredicate),
    #[serde(rename = "minecraft:solid")]
    Solid(SolidBlockPredicate),
    #[serde(rename = "minecraft:replaceable")]
    Replaceable(ReplaceableBlockPredicate),
    #[serde(rename = "minecraft:would_survive")]
    WouldSurvive(WouldSurviveBlockPredicate),
    #[serde(rename = "minecraft:inside_world_bounds")]
    InsideWorldBounds(InsideWorldBoundsBlockPredicate),
    #[serde(rename = "minecraft:any_of")]
    AnyOf(AnyOfBlockPredicate),
    #[serde(rename = "minecraft:all_of")]
    AllOf(AllOfBlockPredicate),
    #[serde(rename = "minecraft:not")]
    Not(NotBlockPredicate),
    #[serde(rename = "minecraft:true")]
    AlwaysTrue,
    /// Not used
    #[serde(rename = "minecraft:unobstructed")]
    Unobstructed(EmptyTODOStruct),
}

impl BlockPredicate {
    pub async fn test(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &ProtoChunk<'_>,
        pos: &BlockPos,
    ) -> bool {
        match self {
            BlockPredicate::MatchingBlocks(predicate) => predicate.test(chunk, pos),
            BlockPredicate::MatchingBlockTag(predicate) => predicate.test(chunk, pos),
            BlockPredicate::MatchingFluids(_predicate) => false,
            BlockPredicate::HasSturdyFace(predicate) => predicate.test(chunk, pos),
            BlockPredicate::Solid(predicate) => predicate.test(chunk, pos),
            BlockPredicate::Replaceable(predicate) => predicate.test(chunk, pos),
            BlockPredicate::WouldSurvive(predicate) => {
                predicate.test(block_registry, chunk, pos).await
            }
            BlockPredicate::InsideWorldBounds(predicate) => predicate.test(chunk, pos),
            BlockPredicate::AnyOf(predicate) => predicate.test(block_registry, chunk, pos).await,
            BlockPredicate::AllOf(predicate) => predicate.test(block_registry, chunk, pos).await,
            BlockPredicate::Not(predicate) => predicate.test(block_registry, chunk, pos).await,
            BlockPredicate::AlwaysTrue => true,
            BlockPredicate::Unobstructed(_predicate) => false,
        }
    }
}

#[derive(Deserialize)]
pub struct MatchingBlocksBlockPredicate {
    #[serde(flatten)]
    offset: OffsetBlocksBlockPredicate,
    blocks: MatchingBlocksWrapper,
}

impl MatchingBlocksBlockPredicate {
    pub fn test(&self, chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        let block = self.offset.get_block(chunk, pos);
        match &self.blocks {
            MatchingBlocksWrapper::Single(single_block) => {
                single_block.replace("minecraft:", "") == block.name
            }
            MatchingBlocksWrapper::Multiple(blocks) => blocks
                .iter()
                .map(|s| s.replace("minecraft:", ""))
                .contains(block.name),
        }
    }
}

#[derive(Deserialize)]
pub struct InsideWorldBoundsBlockPredicate {
    offset: Vector3<i32>,
}

impl InsideWorldBoundsBlockPredicate {
    pub fn test(&self, chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        let pos = pos.offset(self.offset);
        !chunk.out_of_height(pos.0.y as i16)
    }
}

#[derive(Deserialize)]
pub struct MatchingBlockTagPredicate {
    #[serde(flatten)]
    offset: OffsetBlocksBlockPredicate,
    tag: String,
}

impl MatchingBlockTagPredicate {
    pub fn test(&self, chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        let block = self.offset.get_block(chunk, pos);
        block.is_tagged_with(&self.tag).unwrap()
    }
}

#[derive(Deserialize)]
pub struct HasSturdyFacePredicate {
    #[serde(flatten)]
    offset: OffsetBlocksBlockPredicate,
    direction: BlockDirection,
}

impl HasSturdyFacePredicate {
    pub fn test(&self, chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        let state = self.offset.get_state(chunk, pos);
        state.is_side_solid(self.direction)
    }
}

#[derive(Deserialize)]
pub struct AnyOfBlockPredicate {
    predicates: Vec<BlockPredicate>,
}

impl AnyOfBlockPredicate {
    pub async fn test(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &ProtoChunk<'_>,
        pos: &BlockPos,
    ) -> bool {
        for predicate in &self.predicates {
            if !Box::pin(predicate.test(block_registry, chunk, pos)).await {
                continue;
            }
            return true;
        }
        false
    }
}

#[derive(Deserialize)]
pub struct AllOfBlockPredicate {
    predicates: Vec<BlockPredicate>,
}

impl AllOfBlockPredicate {
    pub async fn test(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &ProtoChunk<'_>,
        pos: &BlockPos,
    ) -> bool {
        for predicate in &self.predicates {
            if Box::pin(predicate.test(block_registry, chunk, pos)).await {
                continue;
            }
            return false;
        }
        true
    }
}

#[derive(Deserialize)]
pub struct NotBlockPredicate {
    predicate: Box<BlockPredicate>,
}

impl NotBlockPredicate {
    pub async fn test(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &ProtoChunk<'_>,
        pos: &BlockPos,
    ) -> bool {
        !Box::pin(self.predicate.test(block_registry, chunk, pos)).await
    }
}

#[derive(Deserialize)]
pub struct SolidBlockPredicate {
    #[serde(flatten)]
    offset: OffsetBlocksBlockPredicate,
}

impl SolidBlockPredicate {
    pub fn test(&self, chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        let state = self.offset.get_state(chunk, pos);
        state.is_solid()
    }
}

#[derive(Deserialize)]
pub struct WouldSurviveBlockPredicate {
    #[serde(flatten)]
    offset: OffsetBlocksBlockPredicate,
    state: BlockStateCodec,
}

impl WouldSurviveBlockPredicate {
    pub async fn test(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &ProtoChunk<'_>,
        pos: &BlockPos,
    ) -> bool {
        let state = self.state.get_state().unwrap();
        let pos = self.offset.get(pos);
        return block_registry
            .can_place_at(
                &get_block_by_state_id(state.id).unwrap(),
                chunk,
                &pos,
                BlockDirection::Up,
            )
            .await;
    }
}

#[derive(Deserialize)]
pub struct ReplaceableBlockPredicate {
    #[serde(flatten)]
    offset: OffsetBlocksBlockPredicate,
}

impl ReplaceableBlockPredicate {
    pub fn test(&self, chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        let state = self.offset.get_state(chunk, pos);
        state.replaceable()
    }
}

#[derive(Deserialize)]
pub struct OffsetBlocksBlockPredicate {
    offset: Option<Vector3<i32>>,
}

impl OffsetBlocksBlockPredicate {
    pub fn get(&self, pos: &BlockPos) -> BlockPos {
        if let Some(offset) = self.offset {
            return pos.offset(offset);
        }
        *pos
    }
    pub fn get_block(&self, chunk: &ProtoChunk, pos: &BlockPos) -> Block {
        let pos = self.get(pos);
        chunk.get_block_state(&pos.0).to_block()
    }
    pub fn get_state(&self, chunk: &ProtoChunk, pos: &BlockPos) -> BlockState {
        let pos = self.get(pos);
        chunk.get_block_state(&pos.0).to_state()
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MatchingBlocksWrapper {
    Single(String),
    Multiple(Vec<String>),
}
