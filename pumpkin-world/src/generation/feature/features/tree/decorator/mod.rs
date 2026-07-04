use std::borrow::Cow;

use crate::{generation::proto_chunk::GenerationCache, world::WorldPortalExt};
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

use trunk_vine::TrunkVineTreeDecorator;

pub mod alter_ground;
pub mod attached_to_leaves;
pub mod attached_to_logs;
pub mod beehive;
pub mod cocoa;
pub mod creaking_heart;
pub mod leave_vine;
pub mod pale_moss;
pub mod place_on_ground;
pub mod trunk_vine;

pub enum TreeDecorator {
    TrunkVine(TrunkVineTreeDecorator),
    LeaveVine(LeavesVineTreeDecorator),
    PaleMoss(PaleMossTreeDecorator),
    CreakingHeart(CreakingHeartTreeDecorator),
    Cocoa(CocoaTreeDecorator),
    Beehive(BeehiveTreeDecorator),
    AlterGround(AlterGroundTreeDecorator),
    AttachedToLeaves(AttachedToLeavesTreeDecorator),
    PlaceOnGround(PlaceOnGroundTreeDecorator),
    AttachedToLogs(AttachedToLogsTreeDecorator),
}

impl TreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        root_positions: &[BlockPos],
        log_positions: &[BlockPos],
        foliage_positions: &[BlockPos],
    ) {
        match self {
            Self::TrunkVine(_decorator) => {
                TrunkVineTreeDecorator::generate(chunk, random, log_positions);
            }
            Self::LeaveVine(decorator) => decorator.generate(chunk, random, foliage_positions),
            Self::PaleMoss(_decorator) => {}
            Self::CreakingHeart(_decorator) => {}
            Self::Cocoa(_decorator) => {}
            Self::Beehive(_decorator) => {}
            Self::AlterGround(_decorator) => {}
            Self::PlaceOnGround(decorator) => {
                decorator.generate(chunk, block_registry, random, root_positions, log_positions);
            }
            Self::AttachedToLeaves(_decorator) => {}
            Self::AttachedToLogs(decorator) => {
                decorator.generate(chunk, block_registry, random, log_positions);
            }
        }
    }

    pub(super) fn get_leaf_litter_positions<'a>(
        root_positions: &'a [BlockPos],
        log_positions: &'a [BlockPos],
    ) -> Cow<'a, [BlockPos]> {
        if root_positions.is_empty() {
            return Cow::Borrowed(log_positions);
        }

        if log_positions
            .first()
            .is_some_and(|log| root_positions[0].0.y == log.0.y)
        {
            let mut list = Vec::with_capacity(root_positions.len() + log_positions.len());
            list.extend_from_slice(log_positions);
            list.extend_from_slice(root_positions);
            return Cow::Owned(list);
        }

        Cow::Borrowed(root_positions)
    }
}
