use pumpkin_data::BlockState;
use serde::Deserialize;

use super::{MaterialCondition, MaterialRuleContext};
use crate::{ProtoChunk, block::BlockStateCodec};

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
    Condition(Box<ConditionMaterialRule>),
}

impl MaterialRule {
    pub fn try_apply(
        &self,
        chunk: &mut ProtoChunk,
        context: &mut MaterialRuleContext,
    ) -> Option<&'static BlockState> {
        match self {
            MaterialRule::Badlands(badlands) => badlands.try_apply(context),
            MaterialRule::Block(block) => Some(block.try_apply()),
            MaterialRule::Sequence(sequence) => sequence.try_apply(chunk, context),
            MaterialRule::Condition(condition) => condition.try_apply(chunk, context),
        }
    }
}

#[derive(Deserialize)]
pub struct BadLandsMaterialRule;

impl BadLandsMaterialRule {
    pub fn try_apply(&self, context: &mut MaterialRuleContext) -> Option<&'static BlockState> {
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
}

impl BlockMaterialRule {
    pub fn try_apply(&self) -> &'static BlockState {
        self.result_state.get_state()
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
    ) -> Option<&'static BlockState> {
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
    ) -> Option<&'static BlockState> {
        if self.if_true.test(chunk, context) {
            return self.then_run.try_apply(chunk, context);
        }
        None
    }
}
