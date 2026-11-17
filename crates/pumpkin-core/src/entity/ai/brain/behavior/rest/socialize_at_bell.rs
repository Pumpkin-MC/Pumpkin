use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use super::super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::super::memory::walk_target::WalkTarget;
use super::super::super::memory::{MemoryStatus, types};
use super::super::OneShot;
use rand::RngExt;

const SPEED_MODIFIER: f32 = 0.3;

#[must_use]
pub fn create() -> OneShot {
    OneShot::new(
        "SocializeAtBell",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::Registered),
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::MEETING_POINT.id(), MemoryStatus::ValuePresent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
            (types::INTERACTION_TARGET.id(), MemoryStatus::ValueAbsent),
        ],
        |tick| {
            if tick.mob.get_random().random_range(0..100) != 0 {
                return false;
            }
            let Some(meeting_point) = tick.state.get(types::MEETING_POINT).copied() else {
                return false;
            };
            if tick.world.dimension.id != meeting_point.dimension.id {
                return false;
            }
            let body_pos = tick.mob.get_entity().pos.load();
            if meeting_point
                .pos
                .dist_to_center_sqr(body_pos.x, body_pos.y, body_pos.z)
                >= 4.0 * 4.0
            {
                return false;
            }
            let closest = {
                let ctx = tick.visibility();
                let Some(visible) = tick.state.get(types::NEAREST_VISIBLE_LIVING_ENTITIES) else {
                    return false;
                };
                if !visible.contains_any(&ctx, is_villager) {
                    return false;
                }
                visible.find_closest(&ctx, |entity| {
                    is_villager(entity)
                        && entity
                            .get_entity()
                            .pos
                            .load()
                            .squared_distance_to_vec(&body_pos)
                            <= 32.0
                })
            };
            if let Some(target) = closest {
                tick.state.set(types::INTERACTION_TARGET, target.clone());
                tick.state.set(
                    types::LOOK_TARGET,
                    Arc::new(EntityTracker::new(target.clone(), true)) as Arc<dyn PositionTracker>,
                );
                tick.state.set(
                    types::WALK_TARGET,
                    WalkTarget::new(
                        Arc::new(EntityTracker::new(target, false)),
                        SPEED_MODIFIER,
                        1,
                    ),
                );
            }
            true
        },
    )
}

fn is_villager(entity: &Arc<dyn crate::entity::EntityBase>) -> bool {
    entity.get_entity().entity_type == &EntityType::VILLAGER
}
