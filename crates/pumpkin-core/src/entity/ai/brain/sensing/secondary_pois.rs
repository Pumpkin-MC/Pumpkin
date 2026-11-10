use pumpkin_data::dimension::Dimension;

use crate::entity::passive::villager::VillagerEntity;

use super::super::BrainTick;
use super::super::memory::{GlobalPos, MemoryModuleId, types};
use super::Sensor;

const REQUIRES: [MemoryModuleId; 1] = [types::SECONDARY_JOB_SITE.id()];

pub struct SecondaryPoiSensor;

impl Sensor for SecondaryPoiSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(villager) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() else {
            return;
        };
        let profession = villager
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .profession_enum();
        let secondary_poi = profession.secondary_poi();
        let Some(dimension) = Dimension::from_name(tick.world.dimension.minecraft_name) else {
            return;
        };
        let center = tick.mob.get_entity().block_pos.load();
        let mut job_sites = Vec::new();
        for x in -4..=4 {
            for y in -2..=2 {
                for z in -4..=4 {
                    let pos = center.add(x, y, z);
                    let block = tick.world.get_block(&pos);
                    if secondary_poi.contains(&block) {
                        job_sites.push(GlobalPos::new(dimension, pos));
                    }
                }
            }
        }
        if job_sites.is_empty() {
            tick.state.erase(types::SECONDARY_JOB_SITE.id());
        } else {
            tick.state.set(types::SECONDARY_JOB_SITE, job_sites);
        }
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::Block;
    use pumpkin_data::villager::VillagerProfession;

    #[test]
    fn professions_expose_their_secondary_poi_blocks() {
        assert!(
            VillagerProfession::Farmer
                .secondary_poi()
                .contains(&&Block::FARMLAND)
        );
        assert!(VillagerProfession::Armorer.secondary_poi().is_empty());
    }
}
