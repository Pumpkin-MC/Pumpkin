use pumpkin_data::entity::EntityStatus;
use pumpkin_data::villager::VillagerProfession;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;

use super::super::super::memory::{MemoryStatus, types};
use super::super::OneShot;
use super::super::utils::world_for_dimension;
use super::validate_nearby_poi::closer_to_center_than;
use crate::entity::passive::villager::{VillagerData, VillagerEntity};

#[must_use]
pub fn create() -> OneShot {
    OneShot::new(
        "AssignProfessionFromJobSite",
        vec![
            (types::POTENTIAL_JOB_SITE.id(), MemoryStatus::ValuePresent),
            (types::JOB_SITE.id(), MemoryStatus::Registered),
        ],
        move |tick| {
            let Some(villager) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() else {
                return false;
            };
            let Some(global_pos) = tick.state.get(types::POTENTIAL_JOB_SITE).copied() else {
                return false;
            };
            if !closer_to_center_than(global_pos.pos, tick.mob.get_entity().pos.load(), 2.0) {
                return false;
            }
            tick.state.erase(types::POTENTIAL_JOB_SITE.id());
            tick.state.set(types::JOB_SITE, global_pos);
            tick.world.send_entity_status(
                tick.mob.get_entity(),
                EntityStatus::VillagerHappy,
                ActorEventID::VillagerHappy,
            );
            let data = *villager
                .villager_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if data.profession_enum() == VillagerProfession::None
                && let Some(poi_level) = world_for_dimension(tick.world, global_pos.dimension)
            {
                let poi_type = {
                    let mut pois = poi_level
                        .portal_poi
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    pois.get_type(&global_pos.pos)
                };
                if let Some(poi_type) = poi_type
                    && let Some(profession) = VillagerProfession::from_held_job_site(poi_type)
                {
                    villager.set_villager_data(VillagerData::new(
                        data.type_enum(),
                        profession,
                        data.level.0,
                    ));
                    tick.mob.get_mob_entity().request_refresh_brain();
                }
            }
            true
        },
    )
}
