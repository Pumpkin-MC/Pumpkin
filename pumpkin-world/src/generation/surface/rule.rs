use pumpkin_data::{
    BlockState,
    chunk_gen_settings::{
        BlockBlueprint, ConditionMaterialRule, MaterialRule, SequenceMaterialRule,
    },
};

use super::MaterialRuleContext;
use crate::{
    ProtoChunk,
    block::to_state_from_blueprint,
    generation::{
        noise::router::surface_height_sampler::SurfaceHeightEstimateSampler,
        surface::test_condition,
    },
};

pub fn try_apply_material_rule(
    rule: &MaterialRule,
    chunk: &mut ProtoChunk,
    context: &mut MaterialRuleContext,
    surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
) -> Option<&'static BlockState> {
    match rule {
        MaterialRule::Badlands(_badlands) => Some(BadLandsMaterialRule::try_apply(context)),
        MaterialRule::Block(block) => Some(BlockMaterialRule::try_apply(&block.result_state)),
        MaterialRule::Sequence(sequence) => {
            try_apply_sequence(sequence, chunk, context, surface_height_estimate_sampler)
        }
        MaterialRule::Condition(condition) => {
            try_apply_condition(condition, chunk, context, surface_height_estimate_sampler)
        }
        MaterialRule::Unsupported => todo!(),
    }
}

pub struct BadLandsMaterialRule;

impl BadLandsMaterialRule {
    pub fn try_apply(context: &mut MaterialRuleContext) -> &'static BlockState {
        context.terrain_builder.get_terracotta_block(
            context.block_pos_x,
            context.block_pos_y,
            context.block_pos_z,
        )
    }
}

pub struct BlockMaterialRule;

impl BlockMaterialRule {
    pub fn try_apply(blueprint: &BlockBlueprint) -> &'static BlockState {
        to_state_from_blueprint(blueprint)
    }
}

pub fn try_apply_sequence(
    rule: &SequenceMaterialRule,
    chunk: &mut ProtoChunk,
    context: &mut MaterialRuleContext,
    surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
) -> Option<&'static BlockState> {
    for seq in rule.sequence {
        if let Some(state) =
            try_apply_material_rule(seq, chunk, context, surface_height_estimate_sampler)
        {
            return Some(state);
        }
    }
    None
}

pub fn try_apply_condition(
    rule: &ConditionMaterialRule,
    chunk: &mut ProtoChunk,
    context: &mut MaterialRuleContext,
    surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
) -> Option<&'static BlockState> {
    if test_condition(
        &rule.if_true,
        chunk,
        context,
        surface_height_estimate_sampler,
    ) {
        return try_apply_material_rule(
            rule.then_run,
            chunk,
            context,
            surface_height_estimate_sampler,
        );
    }
    None
}
