use pumpkin_data::entity::EntityType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::poi::{Occupancy, SectionPos};
use rand::RngExt;

use super::wander_around::WanderAroundGoal;
use crate::entity::EntityBase;
use crate::entity::ai::util::land_random_pos;
use crate::entity::mob::Mob;
use crate::entity::passive::villager::VillagerEntity;

const POI_SECTION_SCAN_RADIUS: i32 = 2;
const VILLAGER_SCAN_RADIUS: f64 = 32.0;
const RANDOM_POS_XY_DISTANCE: i32 = 10;
const RANDOM_POS_Y_DISTANCE: i32 = 7;
const POI_SEARCH_RADIUS: i32 = 8;
const INTERVAL: i32 = 240;

#[must_use]
pub const fn golem_random_stroll_in_village(speed: f64) -> WanderAroundGoal {
    WanderAroundGoal::with_interval(speed, INTERVAL)
        .with_check_no_action_time(false)
        .with_position(get_position, VILLAGER_SCAN_RADIUS)
}

fn get_position(mob: &dyn Mob) -> Option<Vector3<f64>> {
    let mut random = mob.get_random();
    if random.random::<f32>() < 0.3 {
        return get_position_towards_anywhere(mob);
    }
    let target = if random.random::<f32>() < 0.7 {
        get_position_towards_villager_who_wants_golem(mob).or_else(|| get_position_towards_poi(mob))
    } else {
        get_position_towards_poi(mob).or_else(|| get_position_towards_villager_who_wants_golem(mob))
    };
    target.or_else(|| get_position_towards_anywhere(mob))
}

fn get_position_towards_anywhere(mob: &dyn Mob) -> Option<Vector3<f64>> {
    land_random_pos::get_pos(mob, RANDOM_POS_XY_DISTANCE, RANDOM_POS_Y_DISTANCE)
}

fn get_position_towards_villager_who_wants_golem(mob: &dyn Mob) -> Option<Vector3<f64>> {
    let world = mob.get_entity().world.load();
    let game_time = world.get_world_age();
    let search_box = mob.get_entity().bounding_box.load().expand(
        VILLAGER_SCAN_RADIUS,
        VILLAGER_SCAN_RADIUS,
        VILLAGER_SCAN_RADIUS,
    );
    let villagers: Vec<_> = world
        .get_entities_at_box(&search_box)
        .into_iter()
        .filter(|entity| entity.get_entity().entity_type == &EntityType::VILLAGER)
        .filter(|entity| does_villager_want_golem(entity.as_ref(), game_time))
        .collect();
    if villagers.is_empty() {
        return None;
    }
    let villager = &villagers[mob.get_random().random_range(0..villagers.len())];
    land_random_pos::get_pos_towards(
        mob,
        RANDOM_POS_XY_DISTANCE,
        RANDOM_POS_Y_DISTANCE,
        villager.get_entity().pos.load(),
    )
}

fn does_villager_want_golem(entity: &dyn EntityBase, game_time: i64) -> bool {
    entity
        .cast_any()
        .downcast_ref::<VillagerEntity>()
        .is_some_and(|villager| {
            villager.mob_entity.with_brain(villager, |brain| {
                villager.wants_to_spawn_golem(brain, game_time)
            })
        })
}

fn get_position_towards_poi(mob: &dyn Mob) -> Option<Vector3<f64>> {
    let target_section = get_random_village_section(mob)?;
    let target_pos = get_random_poi_within_section(mob, target_section)?;
    land_random_pos::get_pos_towards(
        mob,
        RANDOM_POS_XY_DISTANCE,
        RANDOM_POS_Y_DISTANCE,
        target_pos.to_f64(),
    )
}

fn get_random_village_section(mob: &dyn Mob) -> Option<SectionPos> {
    let world = mob.get_entity().world.load();
    let center = SectionPos::of(&mob.get_entity().block_pos.load());
    let radius = POI_SECTION_SCAN_RADIUS;
    let village_sections: Vec<SectionPos> = (-radius..=radius)
        .flat_map(|dz| {
            (-radius..=radius).flat_map(move |dy| {
                (-radius..=radius)
                    .map(move |dx| SectionPos::new(center.x + dx, center.y + dy, center.z + dz))
            })
        })
        .filter(|section| world.sections_to_village(*section) == 0)
        .collect();
    if village_sections.is_empty() {
        return None;
    }
    Some(village_sections[mob.get_random().random_range(0..village_sections.len())])
}

fn get_random_poi_within_section(mob: &dyn Mob, section: SectionPos) -> Option<BlockPos> {
    let world = mob.get_entity().world.load();
    let center = BlockPos::new(
        (section.x << 4) + 8,
        (section.y << 4) + 8,
        (section.z << 4) + 8,
    );
    let pois = world
        .portal_poi
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_records_in_range(&|_| true, center, POI_SEARCH_RADIUS, Occupancy::IsOccupied);
    if pois.is_empty() {
        return None;
    }
    Some(pois[mob.get_random().random_range(0..pois.len())].pos)
}
