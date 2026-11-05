use pumpkin_data::poi_type::PoiType;
use pumpkin_data::tag::{PointOfInterestType, Taggable};
use pumpkin_data::villager::VillagerProfession;

use super::super::utils::world_for_dimension;
use crate::entity::ai::brain::BrainState;
use crate::entity::ai::brain::memory::{GlobalPos, MemoryModuleId, MemoryModuleType, types};
use crate::entity::mob::Mob;
use crate::entity::passive::villager::VillagerEntity;

pub fn release_poi(
    state: &mut BrainState,
    mob: &dyn Mob,
    memory_type: MemoryModuleType<GlobalPos>,
) {
    let Some(memory) = state.get(memory_type).copied() else {
        return;
    };
    let profession = mob.cast_any().downcast_ref::<VillagerEntity>().map_or(
        VillagerProfession::None,
        |villager| {
            villager
                .villager_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .profession_enum()
        },
    );
    let current_world = mob.get_entity().world.load_full();
    let Some(world) = world_for_dimension(&current_world, memory.dimension) else {
        return;
    };
    let mut poi_store = world
        .portal_poi
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(poi_type) = poi_store.get_type(&memory.pos) else {
        return;
    };
    if matches_poi_memory(memory_type.id(), poi_type, profession) {
        poi_store.release(&memory.pos);
    }
}

fn matches_poi_memory(
    memory_id: MemoryModuleId,
    poi_type: PoiType,
    profession: VillagerProfession,
) -> bool {
    if memory_id == types::HOME.id() {
        poi_type == PoiType::Home
    } else if memory_id == types::JOB_SITE.id() {
        profession.held_job_site().contains(&poi_type)
    } else if memory_id == types::POTENTIAL_JOB_SITE.id() {
        poi_type.has_tag(&PointOfInterestType::MINECRAFT_ACQUIRABLE_JOB_SITE)
    } else if memory_id == types::MEETING_POINT.id() {
        poi_type == PoiType::Meeting
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poi_memory_predicates_match_vanilla() {
        assert!(matches_poi_memory(
            types::HOME.id(),
            PoiType::Home,
            VillagerProfession::None
        ));
        assert!(!matches_poi_memory(
            types::HOME.id(),
            PoiType::Meeting,
            VillagerProfession::None
        ));
        assert!(matches_poi_memory(
            types::JOB_SITE.id(),
            PoiType::Farmer,
            VillagerProfession::Farmer
        ));
        assert!(!matches_poi_memory(
            types::JOB_SITE.id(),
            PoiType::Farmer,
            VillagerProfession::Fisherman
        ));
        assert!(matches_poi_memory(
            types::POTENTIAL_JOB_SITE.id(),
            PoiType::Farmer,
            VillagerProfession::None
        ));
        assert!(!matches_poi_memory(
            types::POTENTIAL_JOB_SITE.id(),
            PoiType::Home,
            VillagerProfession::None
        ));
        assert!(matches_poi_memory(
            types::MEETING_POINT.id(),
            PoiType::Meeting,
            VillagerProfession::None
        ));
        assert!(!matches_poi_memory(
            types::WALK_TARGET.id(),
            PoiType::Home,
            VillagerProfession::None
        ));
    }
}
