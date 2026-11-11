use pumpkin_data::tag::{self, Taggable};

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::{Sensor, is_entity_attackable};
use crate::entity::EntityBase;

const REQUIRES: [MemoryModuleId; 3] = [
    types::NEAREST_ATTACKABLE.id(),
    types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
    types::HAS_HUNTING_COOLDOWN.id(),
];

const TARGET_DETECTION_DISTANCE_SQR: f64 = 8.0 * 8.0;

pub struct AxolotlAttackablesSensor;

impl Sensor for AxolotlAttackablesSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let position = tick.mob.get_entity().pos.load();
        let hunting = !tick
            .state
            .has_memory_value(types::HAS_HUNTING_COOLDOWN.id());
        let ctx = tick.visibility();
        let nearest = tick
            .state
            .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
            .and_then(|visible| {
                visible.find_closest(&ctx, |entity| {
                    let squared_distance =
                        position.squared_distance_to_vec(&entity.get_entity().pos.load());
                    is_matching_entity(squared_distance, entity.as_ref(), hunting)
                        && is_entity_attackable(&ctx, entity.as_ref())
                })
            });
        tick.state.set_optional(types::NEAREST_ATTACKABLE, nearest);
    }
}

fn is_matching_entity(squared_distance: f64, entity: &dyn EntityBase, hunting: bool) -> bool {
    let entity = entity.get_entity();
    squared_distance <= TARGET_DETECTION_DISTANCE_SQR
        && entity.is_in_water()
        && (entity
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_AXOLOTL_ALWAYS_HOSTILES)
            || (hunting
                && entity
                    .entity_type
                    .has_tag(&tag::EntityType::MINECRAFT_AXOLOTL_HUNT_TARGETS)))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::ai::brain::BrainState;
    use crate::entity::ai::brain::memory::NearestVisibleLivingEntities;
    use crate::entity::ai::test_support::world;
    use crate::entity::mob::zombie::drowned::DrownedEntity;
    use crate::entity::passive::cod::CodEntity;
    use crate::entity::passive::cow::CowEntity;
    use crate::entity::{Entity, EntityBase};
    use crate::world::World;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn entity(world: &Arc<World>, x: f64, entity_type: &'static EntityType, wet: bool) -> Entity {
        let entity = Entity::new(world.clone(), Vector3::new(x, 64.0, 0.0), entity_type);
        entity.touching_water.store(wet, Ordering::Relaxed);
        entity
    }

    fn cod(world: &Arc<World>, x: f64, wet: bool) -> Arc<dyn EntityBase> {
        CodEntity::new(entity(world, x, &EntityType::COD, wet))
    }

    fn drowned(world: &Arc<World>, x: f64) -> Arc<dyn EntityBase> {
        DrownedEntity::new(entity(world, x, &EntityType::DROWNED, true))
    }

    fn cow(world: &Arc<World>, x: f64) -> Arc<dyn EntityBase> {
        CowEntity::new(entity(world, x, &EntityType::COW, true))
    }

    fn id(entity: &Arc<dyn EntityBase>) -> i32 {
        entity.get_entity().entity_id
    }

    fn sense(
        world: &Arc<World>,
        visible: Vec<Arc<dyn EntityBase>>,
        cooldown: bool,
        previous: Option<Arc<dyn EntityBase>>,
    ) -> Option<i32> {
        let body = CowEntity::new(entity(world, 0.0, &EntityType::AXOLOTL, true));
        let mut sensor = AxolotlAttackablesSensor;
        let mut state = BrainState::default();
        for id in sensor.requires() {
            state.register_memory(*id);
        }
        state.set(
            types::NEAREST_VISIBLE_LIVING_ENTITIES,
            NearestVisibleLivingEntities::new(visible),
        );
        if cooldown {
            state.set(types::HAS_HUNTING_COOLDOWN, true);
        }
        if let Some(previous) = previous {
            state.set(types::NEAREST_ATTACKABLE, previous);
        }
        sensor.do_tick(&mut BrainTick {
            state: &mut state,
            world,
            mob: &*body,
            time: 0,
        });
        state.get(types::NEAREST_ATTACKABLE).map(id)
    }

    #[tokio::test]
    async fn picks_water_bound_prey_within_eight_blocks() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let fish = cod(&world, 3.0, true);
        let drowned = drowned(&world, 3.0);
        let at_the_limit = cod(&world, 8.0, true);
        let too_far = cod(&world, 8.01, true);
        let stranded = cod(&world, 3.0, false);
        let wet_cow = cow(&world, 3.0);

        let picked = [
            sense(&world, vec![fish.clone()], false, None),
            sense(&world, vec![drowned.clone()], false, None),
            sense(&world, vec![at_the_limit.clone()], false, None),
            sense(&world, vec![too_far], false, None),
            sense(&world, vec![stranded], false, None),
            sense(&world, vec![wet_cow], false, None),
        ];
        world.level.shutdown().await;

        assert_eq!(
            picked,
            [
                Some(id(&fish)),
                Some(id(&drowned)),
                Some(id(&at_the_limit)),
                None,
                None,
                None
            ]
        );
        Ok(())
    }

    #[tokio::test]
    async fn a_hunting_cooldown_spares_fish_but_not_the_always_hostile() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let near_fish = cod(&world, 2.0, true);
        let drowned = drowned(&world, 4.0);
        let far_fish = cod(&world, 6.0, true);
        let list = || vec![near_fish.clone(), drowned.clone(), far_fish.clone()];

        let on_cooldown = sense(&world, list(), true, None);
        let off_cooldown = sense(&world, list(), false, None);
        world.level.shutdown().await;

        assert_eq!(on_cooldown, Some(id(&drowned)));
        assert_eq!(off_cooldown, Some(id(&near_fish)));
        Ok(())
    }
}
