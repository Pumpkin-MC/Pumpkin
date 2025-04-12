use itertools::Itertools;
use pumpkin_data::block::{Block, BlockState, get_state_by_state_id};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use serde::Deserialize;

use crate::ProtoChunk;
#[derive(Deserialize)]
pub struct EmptyTODOStruct {}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum BlockPredicate {
    #[serde(rename = "minecraft:matching_blocks")]
    MatchingBlocksBlockPredicate(MatchingBlocksBlockPredicate),
    #[serde(rename = "minecraft:matching_block_tag")]
    MatchingBlockTagPredicate(EmptyTODOStruct),
    #[serde(rename = "minecraft:matching_fluids")]
    MatchingFluidsBlockPredicate(EmptyTODOStruct),
    #[serde(rename = "minecraft:has_sturdy_face")]
    HasSturdyFacePredicate(EmptyTODOStruct),
    #[serde(rename = "minecraft:solid")]
    SolidBlockPredicate(SolidBlockPredicate),
    #[serde(rename = "minecraft:replaceable")]
    ReplaceableBlockPredicate(ReplaceableBlockPredicate),
    #[serde(rename = "minecraft:would_survive")]
    WouldSurviveBlockPredicate(EmptyTODOStruct),
    #[serde(rename = "minecraft:inside_world_bounds")]
    InsideWorldBoundsBlockPredicate(EmptyTODOStruct),
    #[serde(rename = "minecraft:any_of")]
    AnyOfBlockPredicate(AnyOfBlockPredicate),
    #[serde(rename = "minecraft:all_of")]
    AllOfBlockPredicate(AllOfBlockPredicate),
    #[serde(rename = "minecraft:not")]
    NotBlockPredicate(NotBlockPredicate),
    #[serde(rename = "minecraft:true")]
    AlwaysTrueBlockPredicate,
    #[serde(rename = "minecraft:unobstructed")]
    UnobstructedBlockPredicate(EmptyTODOStruct),
}

impl BlockPredicate {
    pub fn test(&self, chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        match self {
            Self::MatchingBlocksBlockPredicate(predicate) => predicate.test(chunk, pos),
            Self::ReplaceableBlockPredicate(predicate) => predicate.test(chunk, pos),
            Self::SolidBlockPredicate(predicate) => predicate.test(chunk, pos),
            Self::AlwaysTrueBlockPredicate => true,
            Self::NotBlockPredicate(predicate) => predicate.test(chunk, pos),
            Self::AnyOfBlockPredicate(predicate) => predicate.test(chunk, pos),
            Self::AllOfBlockPredicate(predicate) => predicate.test(chunk, pos),
            Self::WouldSurviveBlockPredicate(_) => true, // TODO
            _ => false,
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
pub struct AnyOfBlockPredicate {
    predicates: Vec<BlockPredicate>,
}

impl AnyOfBlockPredicate {
    pub fn test(&self, chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        for predicate in &self.predicates {
            if !predicate.test(chunk, pos) {
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
    pub fn test(&self, chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        for predicate in &self.predicates {
            if predicate.test(chunk, pos) {
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
    pub fn test(&self, chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        !self.predicate.test(chunk, pos)
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
        state.is_solid
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
        state.replaceable
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
        Block::from_id(chunk.get_block_state(&pos.0).block_id).unwrap()
    }
    pub fn get_state(&self, chunk: &ProtoChunk, pos: &BlockPos) -> BlockState {
        let pos = self.get(pos);
        let block = &Block::from_id(chunk.get_block_state(&pos.0).block_id).unwrap();
        get_state_by_state_id(block.default_state_id).unwrap()
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MatchingBlocksWrapper {
    Single(String),
    Multiple(Vec<String>),
}
