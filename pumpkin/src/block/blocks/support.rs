//! Shared support / survival helpers matching vanilla block-update patterns.
//!
//! Vanilla splits physics into:
//! - **updateShape** (immediate): return AIR or new state when neighbour changes
//! - **scheduleTick** (NTE): delay 1–2 gt then `tick` destroys if !canSurvive
//! - **randomTick**: growth/melt only — never attachment survival
//!
//! Apply results with [`BlockFlags::NOTIFY_ALL`] so clients and neighbours cascade.

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::is_air;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::world::World;

/// Flags for applying survival failures (break / set). Always notify clients + neighbours.
pub const SURVIVAL_BREAK_FLAGS: BlockFlags = BlockFlags::NOTIFY_ALL;

/// Vanilla vegetation/carpet style: if unsupported, `updateShape` returns AIR immediately.
#[inline]
pub fn air_if_unsupported(
    supported: bool,
    current_state_id: BlockStateId,
) -> BlockStateId {
    if supported {
        current_state_id
    } else {
        BlockStateId::AIR
    }
}

/// Vanilla cactus/sugar-cane style: if unsupported, schedule a 1-gt tick (do not break here).
pub fn schedule_break_if_unsupported(
    world: &World,
    block: &pumpkin_data::Block,
    pos: BlockPos,
    supported: bool,
) {
    if !supported && !world.is_block_tick_scheduled(&pos, block) {
        world.schedule_block_tick(block, pos, 1, TickPriority::Normal);
    }
}

/// True if the block below is not air (carpet / simple floor attachment).
#[inline]
pub fn has_floor_support(block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    !is_air(block_accessor.get_block_state_id(&pos.down()))
}
