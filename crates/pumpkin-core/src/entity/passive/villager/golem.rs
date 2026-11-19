use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use crate::entity::EntityBase;
use crate::entity::ai::brain::BrainState;
use crate::entity::ai::brain::memory::types;
use crate::entity::ai::brain::sensing::golem::golem_detected;
use crate::entity::passive::iron_golem::IronGolemEntity;
use crate::entity::spawn_util::{SpawnStrategy, try_spawn_mob};
use crate::world::World;

use super::VillagerEntity;

const GOLEM_SPAWN_ATTEMPTS: i32 = 10;
const GOLEM_SPAWN_RANGE_XZ: i32 = 8;
const GOLEM_SPAWN_RANGE_Y: i32 = 6;
const VILLAGER_SEARCH_INFLATE: f64 = 10.0;
const VILLAGER_AGREEMENT_LIMIT: usize = 5;
const LAST_SLEPT_SPAWN_WINDOW: i64 = 24_000;

impl VillagerEntity {
    #[must_use]
    pub fn wants_to_spawn_golem(&self, state: &BrainState, game_time: i64) -> bool {
        let slept_recently = state
            .get(types::LAST_SLEPT)
            .is_some_and(|last_slept| game_time - *last_slept < LAST_SLEPT_SPAWN_WINDOW);
        slept_recently && !state.has_memory_value(types::GOLEM_DETECTED_RECENTLY.id())
    }

    pub fn spawn_golem_if_needed(
        &self,
        state: &mut BrainState,
        world: &Arc<World>,
        timestamp: i64,
        villagers_needed_to_agree: i32,
    ) {
        if !self.wants_to_spawn_golem(state, timestamp) {
            return;
        }
        let search_box = self.get_entity().bounding_box.load().expand(
            VILLAGER_SEARCH_INFLATE,
            VILLAGER_SEARCH_INFLATE,
            VILLAGER_SEARCH_INFLATE,
        );
        let nearby_villagers: Vec<_> = world
            .get_entities_at_box(&search_box)
            .into_iter()
            .filter(|entity| entity.get_entity().entity_type == &EntityType::VILLAGER)
            .collect();
        let self_id = self.get_entity().entity_id;
        let mut agreeing = 0;
        for entity in &nearby_villagers {
            if agreeing >= VILLAGER_AGREEMENT_LIMIT {
                break;
            }
            let wants_golem = if entity.get_entity().entity_id == self_id {
                self.wants_to_spawn_golem(state, timestamp)
            } else if let Some(villager) = entity.cast_any().downcast_ref::<Self>() {
                villager.mob_entity.with_brain(villager, |brain| {
                    villager.wants_to_spawn_golem(brain, timestamp)
                })
            } else {
                false
            };
            if wants_golem {
                agreeing += 1;
            }
        }
        if agreeing < villagers_needed_to_agree as usize {
            return;
        }
        let block_position = self.get_entity().block_pos.load();
        if try_spawn_mob(
            &EntityType::IRON_GOLEM,
            IronGolemEntity::new,
            world,
            &block_position,
            GOLEM_SPAWN_ATTEMPTS,
            GOLEM_SPAWN_RANGE_XZ,
            GOLEM_SPAWN_RANGE_Y,
            SpawnStrategy::LegacyIronGolem,
            false,
        )
        .is_none()
        {
            return;
        }
        for entity in &nearby_villagers {
            if entity.get_entity().entity_id == self_id {
                golem_detected(state);
            } else if let Some(villager) = entity.cast_any().downcast_ref::<Self>() {
                villager
                    .mob_entity
                    .with_brain(villager, |brain| golem_detected(brain));
            }
        }
    }
}
