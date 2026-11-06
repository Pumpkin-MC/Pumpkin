use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::poi_type::PoiType;
use pumpkin_data::villager::VillagerProfession;

use super::super::super::BrainTick;
use super::super::super::memory::{GlobalPos, MemoryStatus, types};
use super::super::OneShot;
use crate::entity::EntityBase;
use crate::entity::living::LivingEntity;
use crate::entity::passive::villager::VillagerEntity;

#[must_use]
pub fn create() -> OneShot {
    OneShot::new(
        "PoiCompetitorScan",
        vec![
            (types::JOB_SITE.id(), MemoryStatus::ValuePresent),
            (
                types::NEAREST_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        move |tick| {
            if tick
                .mob
                .cast_any()
                .downcast_ref::<VillagerEntity>()
                .is_none()
            {
                return false;
            }
            let Some(pos) = tick.state.get(types::JOB_SITE).copied() else {
                return false;
            };
            let poi_type = {
                let mut pois = tick
                    .world
                    .portal_poi
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                pois.get_type(&pos.pos)
            };
            let Some(poi_type) = poi_type else {
                return true;
            };
            let competitors = {
                let Some(entities) = tick.state.get(types::NEAREST_LIVING_ENTITIES) else {
                    return true;
                };
                entities
                    .iter()
                    .filter(|entity| {
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
                        competes_for_same_jobsite(pos, poi_type, nearby)
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            };
            let Some(mut winner) = tick.world.get_entity_by_id(tick.mob.get_entity().entity_id)
            else {
                return true;
            };
            for competitor in competitors {
                winner = select_winner(winner, competitor, tick);
            }
            true
        },
    )
}

fn select_winner(
    first: Arc<dyn EntityBase>,
    second: Arc<dyn EntityBase>,
    tick: &mut BrainTick<'_>,
) -> Arc<dyn EntityBase> {
    let (winner, loser) = if villager_xp(first.as_ref()) > villager_xp(second.as_ref()) {
        (first, second)
    } else {
        (second, first)
    };
    erase_job_site(&loser, tick);
    winner
}

fn villager_xp(entity: &dyn EntityBase) -> i32 {
    entity
        .cast_any()
        .downcast_ref::<VillagerEntity>()
        .map_or(0, |villager| villager.xp.load(Ordering::Relaxed))
}

fn erase_job_site(entity: &Arc<dyn EntityBase>, tick: &mut BrainTick<'_>) {
    if entity.get_entity().entity_id == tick.mob.get_entity().entity_id {
        tick.state.erase(types::JOB_SITE.id());
    } else if let Some(mob) = entity.get_mob() {
        mob.get_mob_entity()
            .with_brain(mob, |brain| brain.erase(types::JOB_SITE.id()));
    }
}

fn competes_for_same_jobsite(pos: GlobalPos, poi_type: PoiType, nearby: &VillagerEntity) -> bool {
    let job_site = nearby
        .mob_entity
        .with_brain(nearby, |brain| brain.get(types::JOB_SITE).copied());
    let Some(job_site) = job_site else {
        return false;
    };
    let profession = nearby
        .villager_data
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .profession_enum();
    job_site == pos && has_matching_profession(poi_type, profession)
}

fn has_matching_profession(poi_type: PoiType, profession: VillagerProfession) -> bool {
    profession.held_job_site().contains(&poi_type)
}
