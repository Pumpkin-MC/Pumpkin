use pumpkin_data::dimension::Dimension;
use pumpkin_data::poi_type::PoiType;
use pumpkin_data::villager::VillagerProfession;
use pumpkin_util::math::position::BlockPos;

use super::super::super::memory::{GlobalPos, MemoryStatus, types};
use super::super::{OneShot, utils};
use crate::entity::living::LivingEntity;
use crate::entity::passive::villager::VillagerEntity;

#[must_use]
pub fn create(speed_modifier: f32) -> OneShot {
    OneShot::new(
        "YieldJobSite",
        vec![
            (types::POTENTIAL_JOB_SITE.id(), MemoryStatus::ValuePresent),
            (types::JOB_SITE.id(), MemoryStatus::ValueAbsent),
            (
                types::NEAREST_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
            (types::WALK_TARGET.id(), MemoryStatus::Registered),
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
        ],
        move |tick| {
            let Some(villager) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() else {
                return false;
            };
            if tick.mob.get_mob_entity().living_entity.is_baby() {
                return false;
            }
            let profession = villager
                .villager_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .profession_enum();
            if profession != VillagerProfession::None {
                return false;
            }
            let Some(potential_job_site) = tick.state.get(types::POTENTIAL_JOB_SITE).copied()
            else {
                return false;
            };
            let poi_type = {
                let mut pois = tick
                    .world
                    .portal_poi
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                pois.get_type(&potential_job_site.pos)
            };
            let Some(poi_type) = poi_type else {
                return true;
            };
            let Some(dimension) = Dimension::from_name(tick.world.dimension.minecraft_name) else {
                return true;
            };
            let nearby_entity = {
                let Some(entities) = tick.state.get(types::NEAREST_LIVING_ENTITIES) else {
                    return true;
                };
                entities
                    .iter()
                    .find(|entity| {
                        if entity.get_entity().entity_id == tick.mob.get_entity().entity_id {
                            return false;
                        }
                        if !entity
                            .get_living_entity()
                            .is_some_and(LivingEntity::is_alive)
                        {
                            return false;
                        }
                        let Some(nearby) = entity.cast_any().downcast_ref::<VillagerEntity>()
                        else {
                            return false;
                        };
                        nearby_wants_jobsite(poi_type, nearby, potential_job_site.pos)
                    })
                    .cloned()
            };
            let Some(nearby_entity) = nearby_entity else {
                return true;
            };
            let Some(nearby) = nearby_entity.cast_any().downcast_ref::<VillagerEntity>() else {
                return true;
            };
            tick.state.erase(types::WALK_TARGET.id());
            tick.state.erase(types::LOOK_TARGET.id());
            tick.state.erase(types::POTENTIAL_JOB_SITE.id());
            let nearby_has_no_job_site = nearby
                .mob_entity
                .with_brain(nearby, |brain| brain.get(types::JOB_SITE).is_none());
            if nearby_has_no_job_site {
                nearby.mob_entity.with_brain(nearby, |brain| {
                    utils::set_walk_and_look_target_memories_to_block(
                        brain,
                        potential_job_site.pos,
                        speed_modifier,
                        1,
                    );
                    brain.set(
                        types::POTENTIAL_JOB_SITE,
                        GlobalPos::new(dimension, potential_job_site.pos),
                    );
                });
            }
            true
        },
    )
}

fn nearby_wants_jobsite(poi_type: PoiType, nearby: &VillagerEntity, poi_pos: BlockPos) -> bool {
    let (has_potential_job_site, job_site) = nearby.mob_entity.with_brain(nearby, |brain| {
        (
            brain.has_memory_value(types::POTENTIAL_JOB_SITE.id()),
            brain.get(types::JOB_SITE).copied(),
        )
    });
    if has_potential_job_site {
        return false;
    }
    let profession = nearby
        .villager_data
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .profession_enum();
    if !profession.held_job_site().contains(&poi_type) {
        return false;
    }
    job_site.map_or_else(
        || can_reach_pos(nearby, poi_pos, poi_type),
        |job_site| job_site.pos == poi_pos,
    )
}

fn can_reach_pos(nearby: &VillagerEntity, poi_pos: BlockPos, poi_type: PoiType) -> bool {
    let mob_entity = &nearby.mob_entity;
    let living = &mob_entity.living_entity;
    let mut navigator = mob_entity
        .navigator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    navigator
        .create_path(living, poi_pos.0.to_f64(), poi_type.valid_range())
        .is_some_and(|path| path.can_reach())
}
