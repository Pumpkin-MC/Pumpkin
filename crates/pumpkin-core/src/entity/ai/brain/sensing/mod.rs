use rand::{Rng, RngExt};

use crate::entity::EntityBase;
use crate::entity::ai::target_predicate::TargetPredicate;

use super::memory::{MemoryModuleId, types};
use super::{BrainTick, VisibilityContext};

pub mod dummy;
pub mod hurt_by;
pub mod nearest_items;
pub mod nearest_living_entities;
pub mod nearest_players;

pub use dummy::DummySensor;
pub use hurt_by::HurtBySensor;
pub use nearest_living_entities::NearestLivingEntitySensor;
pub use nearest_players::PlayerSensor;

pub const DEFAULT_SCAN_RATE: i32 = 20;

pub trait Sensor: Send + Sync {
    fn requires(&self) -> &'static [MemoryModuleId];
    fn do_tick(&mut self, tick: &mut BrainTick<'_>);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SensorType {
    Dummy,
    NearestLivingEntities,
    NearestPlayers,
    HurtBy,
    NearestItems,
}

impl SensorType {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dummy => "minecraft:dummy",
            Self::NearestLivingEntities => "minecraft:nearest_living_entities",
            Self::NearestPlayers => "minecraft:nearest_players",
            Self::HurtBy => "minecraft:hurt_by",
            Self::NearestItems => "minecraft:nearest_items",
        }
    }

    #[must_use]
    pub const fn scan_rate(self) -> i32 {
        match self {
            Self::Dummy
            | Self::NearestLivingEntities
            | Self::NearestPlayers
            | Self::HurtBy
            | Self::NearestItems => DEFAULT_SCAN_RATE,
        }
    }

    fn create_sensor(self) -> Box<dyn Sensor> {
        match self {
            Self::Dummy => Box::new(DummySensor),
            Self::NearestLivingEntities => Box::new(NearestLivingEntitySensor),
            Self::NearestPlayers => Box::new(PlayerSensor),
            Self::HurtBy => Box::new(HurtBySensor),
            Self::NearestItems => Box::new(nearest_items::NearestItemSensor),
        }
    }

    pub fn create<R: Rng>(self, rng: &mut R) -> SensorEntry {
        SensorEntry::new(self, rng)
    }
}

pub struct SensorEntry {
    ty: SensorType,
    sensor: Box<dyn Sensor>,
    scan_rate: i32,
    time_to_tick: i64,
}

impl SensorEntry {
    pub fn new<R: Rng>(ty: SensorType, rng: &mut R) -> Self {
        let scan_rate = ty.scan_rate();
        Self {
            ty,
            sensor: ty.create_sensor(),
            scan_rate,
            time_to_tick: i64::from(rng.random_range(0..scan_rate)),
        }
    }

    #[must_use]
    pub const fn sensor_type(&self) -> SensorType {
        self.ty
    }

    #[must_use]
    pub fn requires(&self) -> &'static [MemoryModuleId] {
        self.sensor.requires()
    }

    pub fn tick(&mut self, tick: &mut BrainTick<'_>) {
        self.time_to_tick -= 1;
        if self.time_to_tick <= 0 {
            self.time_to_tick = i64::from(self.scan_rate);
            self.sensor.do_tick(tick);
        }
    }
}

fn is_living_alive(entity: &dyn EntityBase) -> bool {
    entity
        .get_living_entity()
        .is_some_and(|living| living.entity.is_alive() && !living.is_dead_or_dying())
}

fn is_current_attack_target(ctx: &VisibilityContext<'_>, target: &dyn EntityBase) -> bool {
    ctx.state
        .get(types::ATTACK_TARGET)
        .is_some_and(|current| current.get_entity().entity_id == target.get_entity().entity_id)
}

// Vanilla's `ignoreInvisibilityTesting` drops the visibility-percent range scaling, which is
// `use_distance_scaling_factor` here, not the line of sight check.
fn targeting_predicate(
    ctx: &VisibilityContext<'_>,
    target: &dyn EntityBase,
    attackable: bool,
) -> TargetPredicate {
    let predicate = if attackable {
        TargetPredicate::create_attackable()
    } else {
        TargetPredicate::create_non_attackable()
    }
    .set_base_max_distance(ctx.follow_range);
    if is_current_attack_target(ctx, target) {
        predicate.ignore_distance_scaling_factor()
    } else {
        predicate
    }
}

#[must_use]
pub fn is_entity_targetable(ctx: &VisibilityContext<'_>, target: &dyn EntityBase) -> bool {
    targeting_predicate(ctx, target, false).test(ctx.world, Some(ctx.mob), target)
}

#[must_use]
pub fn is_entity_attackable(ctx: &VisibilityContext<'_>, target: &dyn EntityBase) -> bool {
    targeting_predicate(ctx, target, true).test_with_alliance(
        ctx.world,
        Some(ctx.mob),
        target,
        |tester, target| is_allied(ctx, tester, target),
    )
}

fn is_allied(
    ctx: &VisibilityContext<'_>,
    tester: &dyn EntityBase,
    target: &dyn EntityBase,
) -> bool {
    tester.get_entity().entity_id == target.get_entity().entity_id
        || ctx
            .mob
            .mob_considers_entity_as_ally(target, Some(ctx.state))
            .unwrap_or_else(|| tester.considers_entity_as_ally(target))
        || target.considers_entity_as_ally(tester)
}

#[must_use]
pub fn is_entity_attackable_ignoring_line_of_sight(
    ctx: &VisibilityContext<'_>,
    target: &dyn EntityBase,
) -> bool {
    targeting_predicate(ctx, target, true)
        .ignore_visibility()
        .test_with_alliance(ctx.world, Some(ctx.mob), target, |tester, target| {
            is_allied(ctx, tester, target)
        })
}

pub fn remember_positives<T>(
    invocations: i32,
    predicate: impl Fn(&T) -> bool,
) -> impl FnMut(&T) -> bool {
    let mut positives_left = 0;
    move |value| {
        if predicate(value) {
            positives_left = invocations;
            true
        } else {
            positives_left -= 1;
            positives_left >= 0
        }
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::damage::DamageType;
    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::brain::BrainState;
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;
    use crate::world::World;
    use std::sync::Arc;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn cow(world: &Arc<World>, x: f64) -> Arc<CowEntity> {
        CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, 0.0),
            &EntityType::COW,
        ))
    }

    fn sense(sensor: &mut dyn Sensor, world: &Arc<World>, body: &CowEntity) -> BrainState {
        let mut state = BrainState::default();
        for id in sensor.requires() {
            state.register_memory(*id);
        }
        sensor.do_tick(&mut BrainTick {
            state: &mut state,
            world,
            mob: body,
            time: 0,
        });
        state
    }

    fn ids(entities: &[Arc<dyn EntityBase>]) -> Vec<i32> {
        entities
            .iter()
            .map(|entity| entity.get_entity().entity_id)
            .collect()
    }

    #[tokio::test]
    async fn nearest_living_entities_are_alive_in_range_and_nearest_first() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = cow(&world, 0.0);
        let far = cow(&world, 6.0);
        let near = cow(&world, 2.0);
        let dead = cow(&world, 1.0);
        let out_of_range = cow(&world, 40.0);
        dead.mob_entity.living_entity.health.store(0.0);
        world.entities.store(Arc::new(vec![
            body.clone(),
            far.clone(),
            near.clone(),
            dead,
            out_of_range,
        ]));

        let state = sense(&mut NearestLivingEntitySensor, &world, &body);
        world.level.shutdown().await;

        let nearby = state
            .get(types::NEAREST_LIVING_ENTITIES)
            .map(|entities| ids(entities));
        assert_eq!(
            nearby,
            Some(vec![
                near.get_entity().entity_id,
                far.get_entity().entity_id
            ])
        );
        assert!(state.has_memory_value(types::NEAREST_VISIBLE_LIVING_ENTITIES.id()));
        Ok(())
    }

    #[tokio::test]
    async fn hurt_by_remembers_the_attacker_until_it_is_gone() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = cow(&world, 0.0);
        let attacker = cow(&world, 1.0);
        world
            .entities
            .store(Arc::new(vec![body.clone(), attacker.clone()]));
        body.mob_entity.living_entity.damage_with_context(
            &*body,
            1.0,
            DamageType::MOB_ATTACK,
            None,
            Some(&*attacker),
            Some(&*attacker),
        );

        let mut sensor = HurtBySensor;
        let hurt = sense(&mut sensor, &world, &body);
        let remembered = hurt
            .get(types::HURT_BY_ENTITY)
            .map(|entity| entity.get_entity().entity_id);
        let mut state = hurt;
        attacker.get_entity().remove();
        sensor.do_tick(&mut BrainTick {
            state: &mut state,
            world: &world,
            mob: &*body,
            time: 1,
        });
        world.level.shutdown().await;

        assert!(state.has_memory_value(types::HURT_BY.id()));
        assert_eq!(remembered, Some(attacker.get_entity().entity_id));
        assert!(!state.has_memory_value(types::HURT_BY_ENTITY.id()));
        Ok(())
    }
}
