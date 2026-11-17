use pumpkin_data::Block;
use rand::RngExt;

use super::super::super::memory::{MemoryStatus, types};
use super::super::OneShot;
use crate::block::blocks::redstone::bell::ring_bell;

const BELL_RING_CHANCE: f32 = 0.95;
const RING_BELL_FROM_DISTANCE: i32 = 3;

#[must_use]
pub fn create() -> OneShot {
    OneShot::new(
        "RingBell",
        vec![(types::MEETING_POINT.id(), MemoryStatus::ValuePresent)],
        |tick| {
            if tick.mob.get_random().random::<f32>() <= BELL_RING_CHANCE {
                return false;
            }
            let Some(meeting_point) = tick.state.get(types::MEETING_POINT).copied() else {
                return false;
            };
            let pos = meeting_point.pos;
            if pos.squared_distance(&tick.mob.get_entity().block_pos.load())
                < RING_BELL_FROM_DISTANCE * RING_BELL_FROM_DISTANCE
                && tick.world.get_block(&pos) == &Block::BELL
            {
                let body = tick.world.get_entity_by_id(tick.mob.get_entity().entity_id);
                ring_bell(pos, tick.world, None, body);
            }
            true
        },
    )
}
