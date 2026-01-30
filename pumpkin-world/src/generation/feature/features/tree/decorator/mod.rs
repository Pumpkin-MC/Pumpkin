use crate::generation::proto_chunk::GenerationCache;
use alter_ground::AlterGroundTreeDecorator;
use attached_to_leaves::AttachedToLeavesTreeDecorator;
use attached_to_logs::AttachedToLogsTreeDecorator;
use beehive::BeehiveTreeDecorator;
use cocoa::CocoaTreeDecorator;
use creaking_heart::CreakingHeartTreeDecorator;
use leave_vine::LeavesVineTreeDecorator;
use pale_moss::PaleMossTreeDecorator;
use place_on_ground::PlaceOnGroundTreeDecorator;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;
use trunk_vine::TrunkVineTreeDecorator;

mod alter_ground;
mod attached_to_leaves;
mod attached_to_logs;
mod beehive;
mod cocoa;
mod creaking_heart;
mod leave_vine;
mod pale_moss;
mod place_on_ground;
mod trunk_vine;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum TreeDecorator {
    #[serde(rename = "minecraft:trunk_vine")]
    TrunkVine(TrunkVineTreeDecorator),
    #[serde(rename = "minecraft:leave_vine")]
    LeaveVine(LeavesVineTreeDecorator),
    #[serde(rename = "minecraft:pale_moss")]
    PaleMoss(PaleMossTreeDecorator),
    #[serde(rename = "minecraft:creaking_heart")]
    CreakingHeart(CreakingHeartTreeDecorator),
    #[serde(rename = "minecraft:cocoa")]
    Cocoa(CocoaTreeDecorator),
    #[serde(rename = "minecraft:beehive")]
    Beehive(BeehiveTreeDecorator),
    #[serde(rename = "minecraft:alter_ground")]
    AlterGround(AlterGroundTreeDecorator),
    #[serde(rename = "minecraft:attached_to_leaves")]
    AttachedToLeaves(AttachedToLeavesTreeDecorator),
    #[serde(rename = "minecraft:place_on_ground")]
    PlaceOnGround(PlaceOnGroundTreeDecorator),
    #[serde(rename = "minecraft:attached_to_logs")]
    AttachedToLogs(AttachedToLogsTreeDecorator),
}

impl TreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        root_positions: &[BlockPos],
        log_positions: &[BlockPos],
    ) {
        match self {
            Self::TrunkVine(decorator) => decorator.generate(chunk, random, log_positions),
            Self::LeaveVine(_decorator) => {}
            Self::PaleMoss(_decorator) => {}
            Self::CreakingHeart(_decorator) => {}
            Self::Cocoa(_decorator) => {}
            Self::Beehive(_decorator) => {}
            Self::AlterGround(_decorator) => {}
            Self::PlaceOnGround(decorator) => {
                decorator.generate(chunk, random, root_positions, log_positions);
            }
            Self::AttachedToLeaves(_decorator) => {}
            Self::AttachedToLogs(decorator) => {
                decorator.generate(chunk, random, log_positions);
            }
        }
    }

    pub(super) fn get_leaf_litter_positions(
        root_positions: &[BlockPos],
        log_positions: &[BlockPos],
    ) -> Vec<BlockPos> {
        if root_positions.is_empty() {
            return log_positions.to_vec();
        }

        if let (Some(root), Some(log)) = (root_positions.first(), log_positions.first())
            && root.0.y == log.0.y
        {
            let mut list = Vec::with_capacity(root_positions.len() + log_positions.len());
            list.extend_from_slice(log_positions);
            list.extend_from_slice(root_positions);
            return list;
        }
        root_positions.to_vec()
    }
}
