use std::sync::OnceLock;

use serde::Deserialize;

use super::{MaterialCondition, MaterialRuleContext};
use crate::{
    ProtoChunk,
    block::{BlockStateCodec, ChunkBlockState},
};

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum MaterialRule {
    #[serde(rename = "minecraft:bandlands")]
    Badlands(BadLandsMaterialRule),
    #[serde(rename = "minecraft:block")]
    Block(BlockMaterialRule),
    #[serde(rename = "minecraft:sequence")]
    Sequence(SequenceMaterialRule),
    #[serde(rename = "minecraft:condition")]
    Condition(ConditionMaterialRule),
}

impl MaterialRule {
    pub fn try_apply(
        &self,
        chunk: &mut ProtoChunk,
        context: &mut MaterialRuleContext,
    ) -> Option<ChunkBlockState> {
        match self {
            MaterialRule::Badlands(badlands) => badlands.try_apply(context),
            MaterialRule::Block(block) => block.try_apply(),
            MaterialRule::Sequence(sequence) => sequence.try_apply(chunk, context),
            MaterialRule::Condition(condition) => condition.try_apply(chunk, context),
        }
    }
}

#[derive(Deserialize)]
pub struct BadLandsMaterialRule;

impl BadLandsMaterialRule {
    pub fn try_apply(&self, context: &mut MaterialRuleContext) -> Option<ChunkBlockState> {
        Some(
            context
                .terrain_builder
                .get_terracotta_block(&context.block_pos),
        )
    }
}

#[derive(Deserialize)]
pub struct BlockMaterialRule {
    result_state: BlockStateCodec,
    #[serde(skip)]
    block_state: OnceLock<Option<ChunkBlockState>>,
}

impl BlockMaterialRule {
    pub fn try_apply(&self) -> Option<ChunkBlockState> {
        *self
            .block_state
            .get_or_init(|| ChunkBlockState::new(&self.result_state.name))
    }
}

#[derive(Deserialize)]
pub struct SequenceMaterialRule {
    sequence: Vec<MaterialRule>,
}

impl SequenceMaterialRule {
    pub fn try_apply(
        &self,
        chunk: &mut ProtoChunk,
        context: &mut MaterialRuleContext,
    ) -> Option<ChunkBlockState> {
        for seq in &self.sequence {
            if let Some(state) = seq.try_apply(chunk, context) {
                return Some(state);
            }
        }
        None
    }
}

#[derive(Deserialize)]
pub struct ConditionMaterialRule {
    if_true: MaterialCondition,
    then_run: Box<MaterialRule>,
}

impl ConditionMaterialRule {
    pub fn try_apply(
        &self,
        chunk: &mut ProtoChunk,
        context: &mut MaterialRuleContext,
    ) -> Option<ChunkBlockState> {
        if self.if_true.test(chunk, context) {
            return self.then_run.try_apply(chunk, context);
        }
        None
    }
}
