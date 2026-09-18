use std::sync::Arc;

use rand::{Rng, RngExt};

use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::{Entity, EntityBase};
use crate::world::World;

use super::memory::{MemoryModuleId, types};
use super::{BrainTick, VisibilityContext};

pub mod dummy;
pub mod hurt_by;
pub mod nearest_items;
pub mod nearest_living_entities;
pub mod nearest_players;
pub mod piglin_brute_specific;
pub mod piglin_specific;

pub use dummy::DummySensor;
pub use hurt_by::HurtBySensor;
pub use nearest_items::NearestItemSensor;
pub use nearest_living_entities::NearestLivingEntitySensor;
pub use nearest_players::PlayerSensor;
pub use piglin_brute_specific::PiglinBruteSpecificSensor;
pub use piglin_specific::PiglinSpecificSensor;

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
    NearestItems,
    HurtBy,
    PiglinSpecific,
    PiglinBruteSpecific,
}

impl SensorType {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dummy => "minecraft:dummy",
            Self::NearestLivingEntities => "minecraft:nearest_living_entities",
            Self::NearestPlayers => "minecraft:nearest_players",
            Self::NearestItems => "minecraft:nearest_items",
            Self::HurtBy => "minecraft:hurt_by",
            Self::PiglinSpecific => "minecraft:piglin_specific_sensor",
            Self::PiglinBruteSpecific => "minecraft:piglin_brute_specific_sensor",
        }
    }

    #[must_use]
    pub const fn scan_rate(self) -> i32 {
        match self {
            Self::Dummy
            | Self::NearestLivingEntities
            | Self::NearestPlayers
            | Self::NearestItems
            | Self::HurtBy
            | Self::PiglinSpecific
            | Self::PiglinBruteSpecific => DEFAULT_SCAN_RATE,
        }
    }

    fn create_sensor(self) -> Box<dyn Sensor> {
        match self {
            Self::Dummy => Box::new(DummySensor),
            Self::NearestLivingEntities => Box::new(NearestLivingEntitySensor),
            Self::NearestPlayers => Box::new(PlayerSensor),
            Self::NearestItems => Box::new(NearestItemSensor),
            Self::HurtBy => Box::new(HurtBySensor),
            Self::PiglinSpecific => Box::new(PiglinSpecificSensor),
            Self::PiglinBruteSpecific => Box::new(PiglinBruteSpecificSensor),
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

/// Entities meeting `body`'s box inflated by the given extents, nearest first.
fn entities_in_inflated_box(
    world: &Arc<World>,
    body: &Entity,
    x: f64,
    y: f64,
    z: f64,
    filter: impl Fn(&Arc<dyn EntityBase>) -> bool,
) -> Vec<Arc<dyn EntityBase>> {
    let bounds = body.bounding_box.load().expand(x, y, z);
    let body_pos = body.pos.load();
    let body_id = body.entity_id;

    let entities = world.entities.load();
    let players = world.players.load();
    // Players are not in world.entities
    let mut found: Vec<(f64, Arc<dyn EntityBase>)> = entities
        .iter()
        .map(Arc::clone)
        .chain(
            players
                .iter()
                .map(|player| Arc::clone(player) as Arc<dyn EntityBase>),
        )
        .filter(|entity| {
            entity.get_entity().entity_id != body_id
                && entity.get_entity().bounding_box.load().intersects(&bounds)
                && filter(entity)
        })
        .map(|entity| {
            (
                entity
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&body_pos),
                entity,
            )
        })
        .collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().map(|(_, entity)| entity).collect()
}

fn is_current_attack_target(ctx: &VisibilityContext<'_>, target: &dyn EntityBase) -> bool {
    ctx.brain
        .get(types::ATTACK_TARGET)
        .is_some_and(|current| current.get_entity().entity_id == target.get_entity().entity_id)
}

// ignoreInvisibilityTesting drops the range scaling, not line of sight
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
    targeting_predicate(ctx, target, true).test(ctx.world, Some(ctx.mob), target)
}

#[must_use]
pub fn is_entity_attackable_ignoring_line_of_sight(
    ctx: &VisibilityContext<'_>,
    target: &dyn EntityBase,
) -> bool {
    targeting_predicate(ctx, target, true)
        .ignore_visibility()
        .test(ctx.world, Some(ctx.mob), target)
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
