use pumpkin_data::BlockState;
use serde::Deserialize;

use super::{MaterialCondition, MaterialRuleContext};
use crate::{
    ProtoChunk, block::BlockStateCodec,
    generation::noise::router::surface_height_sampler::SurfaceHeightEstimateSampler,
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
        surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
    ) -> Option<&'static BlockState> {
        match self {
            MaterialRule::Badlands(badlands) => badlands.try_apply(context),
            MaterialRule::Block(block) => Some(block.try_apply()),
            MaterialRule::Sequence(sequence) => {
                sequence.try_apply(chunk, context, surface_height_estimate_sampler)
            }
            MaterialRule::Condition(condition) => {
                condition.try_apply(chunk, context, surface_height_estimate_sampler)
            }
        }
    }
}

#[derive(Deserialize)]
pub struct BadLandsMaterialRule;

impl BadLandsMaterialRule {
    pub fn try_apply(&self, context: &mut MaterialRuleContext) -> Option<&'static BlockState> {
        Some(context.terrain_builder.get_terracotta_block(
            context.block_pos_x,
            context.block_pos_y,
            context.block_pos_z,
        ))
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
        surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
    ) -> Option<&'static BlockState> {
        for seq in &self.sequence {
            if let Some(state) = seq.try_apply(chunk, context, surface_height_estimate_sampler) {
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
        surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
    ) -> Option<&'static BlockState> {
        if self
            .if_true
            .test(chunk, context, surface_height_estimate_sampler)
        {
            return self
                .then_run
                .try_apply(chunk, context, surface_height_estimate_sampler);
        }
        None
    }
}
