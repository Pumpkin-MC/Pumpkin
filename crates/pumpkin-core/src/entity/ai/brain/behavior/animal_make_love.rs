use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::super::{BrainState, BrainTick};
use super::timed::Behavior;
use super::utils;
use crate::entity::EntityBase;
use crate::entity::ai::goal::breed::BreedGoal;

const BREED_RANGE: f64 = 3.0;
const DEFAULT_SPEED_MODIFIER: f32 = 1.0;
const DEFAULT_CLOSE_ENOUGH_DISTANCE: i32 = 2;
const DURATION: i32 = 110;
const MIN_CHILD_DELAY: i32 = 60;
const CHILD_DELAY_SPREAD: i32 = 50;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 5] = [
    (
        types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
        MemoryStatus::ValuePresent,
    ),
    (types::BREED_TARGET.id(), MemoryStatus::ValueAbsent),
    (types::WALK_TARGET.id(), MemoryStatus::Registered),
    (types::LOOK_TARGET.id(), MemoryStatus::Registered),
    (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
];

pub struct AnimalMakeLove {
    partner_type: &'static EntityType,
    speed_modifier: f32,
    close_enough_distance: i32,
    spawn_child_at_time: i64,
}

impl AnimalMakeLove {
    #[must_use]
    pub const fn new(partner_type: &'static EntityType) -> Self {
        Self::with_speed_and_distance(
            partner_type,
            DEFAULT_SPEED_MODIFIER,
            DEFAULT_CLOSE_ENOUGH_DISTANCE,
        )
    }

    /// Vanilla's `AnimalMakeLove(partnerType, speedModifier, closeEnoughDistance)`.
    #[must_use]
    pub const fn with_speed_and_distance(
        partner_type: &'static EntityType,
        speed_modifier: f32,
        close_enough_distance: i32,
    ) -> Self {
        Self {
            partner_type,
            speed_modifier,
            close_enough_distance,
            spawn_child_at_time: 0,
        }
    }

    fn find_valid_breed_partner(&self, tick: &BrainTick<'_>) -> Option<Arc<dyn EntityBase>> {
        let animal = tick.mob.as_animal()?;
        tick.state
            .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)?
            .find_closest(&tick.visibility(), |entity| {
                entity.get_entity().entity_type == self.partner_type
                    && entity.get_mob().and_then(|mob| mob.as_animal()).is_some()
                    && animal.can_mate(entity.as_ref())
                    && !entity.is_panicking()
            })
    }

    fn breed_target(&self, state: &BrainState) -> Option<Arc<dyn EntityBase>> {
        state
            .get(types::BREED_TARGET)
            .filter(|target| target.get_entity().entity_type == self.partner_type)
            .cloned()
    }
}

impl Behavior for AnimalMakeLove {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        DURATION
    }

    fn max_duration(&self) -> i32 {
        DURATION
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.mob.get_mob_entity().is_in_love() && self.find_valid_breed_partner(tick).is_some()
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        let Some(partner) = self.find_valid_breed_partner(tick) else {
            return;
        };
        tick.state.set(types::BREED_TARGET, partner.clone());
        if let (Some(partner_mob), Some(body)) = (
            partner.get_mob(),
            tick.world.get_entity_by_id(tick.mob.get_entity().entity_id),
        ) {
            partner_mob
                .get_mob_entity()
                .with_brain(partner_mob, |brain| brain.set(types::BREED_TARGET, body));
        }
        utils::lock_gaze_and_walk_to_each_other(
            tick,
            &partner,
            self.speed_modifier,
            self.close_enough_distance,
        );
        let delay = MIN_CHILD_DELAY
            + rand::RngExt::random_range(&mut tick.mob.get_random(), 0..CHILD_DELAY_SPREAD);
        self.spawn_child_at_time = tick.time + i64::from(delay);
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let Some(partner) = self.breed_target(tick.state) else {
            return false;
        };
        partner.get_entity().is_alive()
            && tick
                .mob
                .as_animal()
                .is_some_and(|animal| animal.can_mate(partner.as_ref()))
            && utils::entity_is_visible(&tick.visibility(), partner.as_ref())
            && tick.time <= self.spawn_child_at_time
            && !tick.state.has_memory_value(types::IS_PANICKING.id())
            && !partner.is_panicking()
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(partner) = self.breed_target(tick.state) else {
            return;
        };
        utils::lock_gaze_and_walk_to_each_other(
            tick,
            &partner,
            self.speed_modifier,
            self.close_enough_distance,
        );
        let distance = tick
            .mob
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&partner.get_entity().pos.load());
        if distance < BREED_RANGE * BREED_RANGE && tick.time >= self.spawn_child_at_time {
            BreedGoal::breed(tick.mob, partner.as_ref(), Some(tick.state));
            tick.state.erase(types::BREED_TARGET.id());
            if let Some(partner_mob) = partner.get_mob() {
                partner_mob
                    .get_mob_entity()
                    .with_brain(partner_mob, |brain| brain.erase(types::BREED_TARGET.id()));
            }
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.erase(types::BREED_TARGET.id());
        tick.state.erase(types::WALK_TARGET.id());
        tick.state.erase(types::LOOK_TARGET.id());
        self.spawn_child_at_time = 0;
    }

    fn debug_name(&self) -> &'static str {
        "AnimalMakeLove"
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering::Relaxed;

    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::brain::behavior::{BehaviorControl, Timed};
    use crate::entity::ai::brain::memory::NearestVisibleLivingEntities;
    use crate::entity::ai::brain::{BrainState, BrainTick};
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    /// The speed and distance of the walk target of a cow in love that starts looking for a
    /// partner with `behavior`.
    fn walks_to_partner(
        mut behavior: Timed<AnimalMakeLove>,
    ) -> Result<Option<(f32, i32)>, Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cow = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::COW,
        ));
        cow.mob_entity.love_ticks.store(600, Relaxed);
        let partner: Arc<dyn EntityBase> = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(3.5, 64.0, 0.5),
            &EntityType::COW,
        ));
        let mut state = BrainState::default();
        for id in behavior.required_memories() {
            state.register_memory(*id);
        }
        state.set(
            types::NEAREST_VISIBLE_LIVING_ENTITIES,
            NearestVisibleLivingEntities::new(vec![partner]),
        );
        behavior.try_start(&mut BrainTick {
            state: &mut state,
            world: &world,
            mob: &*cow,
            time: 0,
        });
        Ok(state
            .get(types::WALK_TARGET)
            .map(|walk| (walk.speed_modifier(), walk.close_enough_dist())))
    }

    /// The speed and the distance are the ones given (axolotls court at 0.2 and stop 2 blocks
    /// short, hence the `new` defaults of 1.0 and 2 are not what they get).
    #[tokio::test]
    async fn the_courting_speed_and_distance_can_be_set() -> TestResult {
        let slow = Timed::new(AnimalMakeLove::with_speed_and_distance(
            &EntityType::COW,
            0.2,
            4,
        ));
        let default = Timed::new(AnimalMakeLove::new(&EntityType::COW));

        assert_eq!(walks_to_partner(slow)?, Some((0.2, 4)));
        assert_eq!(walks_to_partner(default)?, Some((1.0, 2)));
        Ok(())
    }
}
