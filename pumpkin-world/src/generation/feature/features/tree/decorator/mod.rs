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

use crate::ProtoChunk;

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
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        root_positions: Vec<BlockPos>,
        log_positions: Vec<BlockPos>,
    ) {
        match self {
            TreeDecorator::TrunkVine(decorator) => decorator.generate(chunk, random, log_positions),
            TreeDecorator::LeaveVine(_decorator) => {}
            TreeDecorator::PaleMoss(_decorator) => {}
            TreeDecorator::CreakingHeart(_decorator) => {}
            TreeDecorator::Cocoa(_decorator) => {}
            TreeDecorator::Beehive(_decorator) => {}
            TreeDecorator::AlterGround(_decorator) => {}
            TreeDecorator::PlaceOnGround(decorator) => {
                decorator.generate(chunk, random, root_positions, log_positions)
            }
            TreeDecorator::AttachedToLeaves(_decorator) => {}
            TreeDecorator::AttachedToLogs(decorator) => {
                decorator.generate(chunk, random, root_positions, log_positions)
            }
        }
    }

    pub(super) fn get_leaf_litter_positions(
        root_positions: Vec<BlockPos>,
        log_positions: Vec<BlockPos>,
    ) -> Vec<BlockPos> {
        let mut list = Vec::new();
        if root_positions.is_empty() {
            list.extend_from_slice(&log_positions);
        } else if !log_positions.is_empty()
            && root_positions.first().unwrap().0.y == log_positions.first().unwrap().0.y
        {
            list.extend_from_slice(&log_positions);
            list.extend_from_slice(&root_positions);
        } else {
            list.extend_from_slice(&root_positions);
        }

        list
    }
}
