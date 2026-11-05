use std::f64::consts::FRAC_PI_2;

use pumpkin_util::math::position::BlockPos;
use pumpkin_world::poi::SectionPos;

use super::super::OneShot;
use crate::entity::ai::brain::memory::{MemoryStatus, types, walk_target::WalkTarget};
use crate::entity::ai::util::{default_random_pos, land_random_pos};
use crate::world::World;

const MAX_XZ_DIST: i32 = 10;
const MAX_Y_DIST: i32 = 7;

#[must_use]
pub fn create(speed_modifier: f32) -> OneShot {
    create_with_distances(speed_modifier, MAX_XZ_DIST, MAX_Y_DIST)
}

#[must_use]
pub fn create_with_distances(speed_modifier: f32, max_xz_dist: i32, max_y_dist: i32) -> OneShot {
    OneShot::new(
        "VillageBoundRandomStroll",
        vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)],
        move |tick| {
            let body_pos = tick.mob.get_entity().block_pos.load();
            let land_pos = if tick.world.is_village(&body_pos) {
                land_random_pos::get_pos(tick.mob, max_xz_dist, max_y_dist)
            } else {
                let section_pos = SectionPos::of(&body_pos);
                let optimal_section_pos =
                    find_section_closest_to_village(tick.world, section_pos, 2);
                if optimal_section_pos == section_pos {
                    land_random_pos::get_pos(tick.mob, max_xz_dist, max_y_dist)
                } else {
                    let center = section_center(optimal_section_pos);
                    default_random_pos::get_pos_towards(
                        tick.mob,
                        max_xz_dist,
                        max_y_dist,
                        center.to_f64(),
                        FRAC_PI_2,
                    )
                }
            };
            tick.state.set_optional(
                types::WALK_TARGET,
                land_pos.map(|pos| WalkTarget::from_vec(pos, speed_modifier, 0)),
            );
            true
        },
    )
}

#[must_use]
pub fn find_section_closest_to_village(
    world: &World,
    center: SectionPos,
    radius: i32,
) -> SectionPos {
    let center_distance = world.sections_to_village(center);
    let candidates = (-radius..=radius).flat_map(|dz| {
        (-radius..=radius).flat_map(move |dy| {
            (-radius..=radius).map(move |dx| {
                let section = SectionPos::new(center.x + dx, center.y + dy, center.z + dz);
                (section, world.sections_to_village(section))
            })
        })
    });
    select_closest_section(center, center_distance, candidates)
}

fn select_closest_section(
    center: SectionPos,
    center_distance: i32,
    candidates: impl Iterator<Item = (SectionPos, i32)>,
) -> SectionPos {
    let mut best = None;
    for (section, distance) in candidates {
        if distance < center_distance
            && best.is_none_or(|(_, best_distance)| distance < best_distance)
        {
            best = Some((section, distance));
        }
    }
    best.map_or(center, |(section, _)| section)
}

pub(crate) const fn section_center(section: SectionPos) -> BlockPos {
    BlockPos::new(
        (section.x << 4) + 8,
        (section.y << 4) + 8,
        (section.z << 4) + 8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_closest_section_keeps_the_first_on_ties() {
        let center = SectionPos::new(0, 0, 0);
        let first = SectionPos::new(1, 0, 0);
        let second = SectionPos::new(2, 0, 0);
        let candidates = [(first, 2), (second, 2)];
        assert_eq!(
            select_closest_section(center, 3, candidates.into_iter()),
            first
        );
    }

    #[test]
    fn select_closest_section_falls_back_to_the_center_when_none_is_closer() {
        let center = SectionPos::new(0, 0, 0);
        let first = SectionPos::new(1, 0, 0);
        let second = SectionPos::new(2, 0, 0);
        let candidates = [(first, 3), (second, 4)];
        assert_eq!(
            select_closest_section(center, 3, candidates.into_iter()),
            center
        );
    }
}
