use pumpkin_util::math::position::BlockPos;
use pumpkin_world::poi::SectionPos;

use super::super::OneShot;
use crate::entity::ai::brain::memory::{MemoryStatus, types, walk_target::WalkTarget};
use crate::entity::ai::util::land_random_pos;

const MAX_TRIES: i32 = 5;

#[must_use]
pub fn create(speed_modifier: f32, close_enough_distance: i32) -> OneShot {
    OneShot::new(
        "GoToClosestVillage",
        vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)],
        move |tick| {
            let body_pos = tick.mob.get_entity().block_pos.load();
            if tick.world.is_village(&body_pos) {
                return false;
            }
            let sections_to_village = tick.world.sections_to_village(SectionPos::of(&body_pos));
            let mut target_pos = None;
            for _ in 0..MAX_TRIES {
                let land_pos = land_random_pos::get_pos_weighted(tick.mob, 15, 7, |pos| {
                    -f64::from(tick.world.sections_to_village(SectionPos::of(&pos)))
                });
                let Some(land_pos) = land_pos else {
                    continue;
                };
                let land_pos_sections_to_village = tick
                    .world
                    .sections_to_village(SectionPos::of(&BlockPos::floored_v(land_pos)));
                if land_pos_sections_to_village < sections_to_village {
                    target_pos = Some(land_pos);
                    break;
                }
                if land_pos_sections_to_village == sections_to_village {
                    target_pos = Some(land_pos);
                }
            }
            if let Some(target_pos) = target_pos {
                tick.state.set(
                    types::WALK_TARGET,
                    WalkTarget::from_vec(target_pos, speed_modifier, close_enough_distance),
                );
            }
            true
        },
    )
}
