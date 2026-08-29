use pumpkin_data::block_properties::{BlockProperties, MovingPistonLikeProperties};
use pumpkin_data::{Block, BlockStateId, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::BlockBehaviour;
use crate::block::blocks::piston::piston::PistonBlock;
use crate::block::{BrokenArgs, OnNeighborUpdateArgs};

use super::piston::PistonProps;

pub(crate) type PistonHeadProperties = pumpkin_data::block_properties::PistonHeadLikeProperties;

#[pumpkin_block("minecraft:piston_head")]
pub struct PistonHeadBlock;

impl BlockBehaviour for PistonHeadBlock {
    fn broken(&self, args: BrokenArgs<'_>) {
        let props = PistonHeadProperties::from_state_id(args.state.id, &Block::PISTON_HEAD);
        let pos = args
            .position
            .offset(props.facing.opposite().to_block_direction().to_offset());
        let (new_block, new_state) = args.world.get_block_and_state_id(&pos);
        if PistonBlock::is_base(new_block) {
            let props = PistonProps::from_state_id(new_state, new_block);
            if props.extended {
                // TODO: use player
                args.world.break_block(&pos, None, BlockFlags::SKIP_DROPS);
            }
        }
    }
    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        // Vanilla `PistonHeadBlock.neighborChanged`: if `canSurvive`, forward to the
        // cell behind the head (base or retracting `MOVING_PISTON`). Not a facing-up
        // redstone-block shortcut.
        let head_state_id = args.world.get_block_state_id(args.position);
        let head_props = PistonHeadProperties::from_state_id(head_state_id, &Block::PISTON_HEAD);
        let base_pos = args.position.offset(
            head_props
                .facing
                .opposite()
                .to_block_direction()
                .to_offset(),
        );
        let (base_block, base_state_id) = args.world.get_block_and_state_id(&base_pos);
        if !head_can_survive(head_props, base_block, base_state_id) {
            return;
        }
        args.world.update_neighbor(&base_pos, args.source_block);
    }
}

/// Vanilla `PistonHeadBlock.canSurvive`: fitting extended base, or `MOVING_PISTON` with
/// the same facing.
fn head_can_survive(
    head: PistonHeadProperties,
    base_block: &Block,
    base_state_id: BlockStateId,
) -> bool {
    if PistonBlock::is_base(base_block) {
        let props = PistonProps::from_state_id(base_state_id, base_block);
        return props.extended
            && props.facing == head.facing
            && PistonBlock::piston_type(base_block) == head.r#type;
    }
    if base_block == &Block::MOVING_PISTON {
        let props = MovingPistonLikeProperties::from_state_id(base_state_id, base_block);
        return props.facing == head.facing;
    }
    false
}
