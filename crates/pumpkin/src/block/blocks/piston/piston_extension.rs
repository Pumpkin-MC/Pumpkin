use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::{Block, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_world::world::BlockFlags;

use crate::block::{BlockBehaviour, BrokenArgs, GetCollisionShapesArgs};

use super::piston::{PistonBlock, PistonProps};

pub(crate) type MovingPistonProps = pumpkin_data::block_properties::MovingPistonLikeProperties;

#[pumpkin_block("minecraft:moving_piston")]
pub struct PistonExtensionBlock;

impl BlockBehaviour for PistonExtensionBlock {
    fn broken(&self, args: BrokenArgs<'_>) {
        // Vanilla `MovingPistonBlock.destroy`: extended base behind this cell, no facing match.
        let props = MovingPistonProps::from_state_id(args.state.id, &Block::MOVING_PISTON);
        let pos = args
            .position
            .offset(props.facing.opposite().to_block_direction().to_offset());
        let (new_block, new_state) = args.world.get_block_and_state_id(&pos);
        if PistonBlock::is_base(new_block) {
            let props = PistonProps::from_state_id(new_state, new_block);
            if props.extended {
                args.world.break_block(&pos, None, BlockFlags::SKIP_DROPS);
            }
        }
    }

    /// Vanilla `MovingPistonBlock.newBlockEntity()` is null. The real BE is attached with
    /// `World::add_block_entity` after `setBlock` (`movedState` is the pushed block).
    fn creates_block_entity_on_place(&self) -> bool {
        false
    }

    fn has_dynamic_collision_shape(&self) -> bool {
        true
    }

    fn collision_reaches_edge_cells(&self) -> bool {
        true
    }

    /// Vanilla `MovingPistonBlock.getCollisionShape`: the BE, not the placeholder voxel.
    fn get_collision_shapes(&self, args: GetCollisionShapesArgs<'_>) -> Option<Vec<BoundingBox>> {
        let noclip = args.entity.get_entity().piston_noclip.load();
        let shapes = args
            .world
            .get_live_block_entity(args.position)
            .map(|block_entity| block_entity.collision_shapes(noclip))
            .unwrap_or_default();
        Some(shapes)
    }
}
