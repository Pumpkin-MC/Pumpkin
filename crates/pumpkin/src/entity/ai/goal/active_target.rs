use super::{Controls, Goal, to_goal_ticks};

use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::mob::Mob;
use crate::entity::mob::neutral::{NeutralMob, find_by_uuid};
use crate::entity::{EntityBase, mob::MobEntity, player::Player};
use crate::world::World;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use rand::RngExt;
use std::sync::Arc;
use uuid::Uuid;

const DEFAULT_RECIPROCAL_CHANCE: i32 = 10;

/// Extra gate on top of the target predicate, for mobs that pick targets conditionally.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum TargetCondition {
    #[default]
    Always,
    /// Only what the mob holds a grudge against. Neutral mobs.
    AngryAt,
    /// Nothing at all in daylight. Spiders.
    NoDaylight,
}

impl TargetCondition {
    /// Checked once per search attempt and every tick the goal continues.
    fn allows_search(self, mob: &dyn Mob) -> bool {
        match self {
            Self::NoDaylight => !mob.get_mob_entity().is_in_daylight(),
            // A calm mob matches nobody: skip the search. Grudge check first, it is lock-free.
            Self::AngryAt => mob.as_neutral().is_some_and(|neutral| {
                neutral.get_persistent_anger_target().is_some() || neutral.is_angry()
            }),
            Self::Always => true,
        }
    }

    /// Fixed candidate the grudge points at. Replaces the area search.
    /// Universal anger has no such target and still searches.
    fn grudge_target(self, mob: &dyn Mob) -> Option<Uuid> {
        match self {
            Self::AngryAt => mob
                .as_neutral()
                .and_then(NeutralMob::get_persistent_anger_target),
            Self::Always | Self::NoDaylight => None,
        }
    }

    /// Checked per candidate and against the held target while the goal continues.
    fn allows_target(self, mob: &dyn Mob, target: &dyn EntityBase, world: &World) -> bool {
        match self {
            Self::AngryAt => mob
                .as_neutral()
                .is_some_and(|neutral| neutral.is_angry_at(target, world)),
            Self::Always | Self::NoDaylight => true,
        }
    }
}

pub struct ActiveTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    reciprocal_chance: i32,
    target_type: Option<&'static EntityType>,
    target_predicate: TargetPredicate,
    condition: TargetCondition,
}

impl ActiveTargetGoal {
    pub fn new<F>(
        mob: &MobEntity,
        target_type: &'static EntityType,
        reciprocal_chance: i32,
        check_visibility: bool,
        check_can_navigate: bool,
        predicate: Option<F>,
    ) -> Self
    where
        F: Fn(&LivingEntity, &World) -> bool + Send + Sync + 'static,
    {
        let track_target_goal = TrackTargetGoal::new(check_visibility, check_can_navigate);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        if let Some(predicate) = predicate {
            target_predicate.set_predicate(predicate);
        }

        Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(reciprocal_chance),
            target_type: Some(target_type),
            target_predicate,
            condition: TargetCondition::Always,
        }
    }

    /// Chains onto any of the constructors, including the boxed ones.
    #[must_use]
    pub fn when(mut self: Box<Self>, condition: TargetCondition) -> Box<Self> {
        self.condition = condition;
        self
    }

    #[must_use]
    pub fn with_default(
        mob: &MobEntity,
        target_type: &'static EntityType,
        check_visibility: bool,
    ) -> Box<Self> {
        let track_target_goal = TrackTargetGoal::with_default(check_visibility);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        Box::new(Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(DEFAULT_RECIPROCAL_CHANCE),
            target_type: Some(target_type),
            target_predicate,
            condition: TargetCondition::Always,
        })
    }

    /// Targets the closest entity of any type passing `predicate`, like vanilla's
    /// class-agnostic `NearestAttackableTargetGoal` (e.g. iron golems targeting
    /// every `Enemy` but creepers). All candidates are tested, so an invalid
    /// entity nearest to the mob cannot block a valid one farther away.
    pub fn predicated(
        mob: &MobEntity,
        reciprocal_chance: i32,
        check_visibility: bool,
        predicate: impl Fn(&LivingEntity, &World) -> bool + Send + Sync + 'static,
    ) -> Box<Self> {
        let track_target_goal = TrackTargetGoal::new(check_visibility, false);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);
        target_predicate.set_predicate(predicate);

        Box::new(Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(reciprocal_chance),
            target_type: None,
            target_predicate,
            condition: TargetCondition::Always,
        })
    }

    pub fn set_target(&mut self, target: Option<Arc<dyn EntityBase>>) {
        self.target = target;
    }

    fn find_closest_target(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let follow_range = mob_entity
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        // Vanilla updates the target conditions with the current follow distance on every search
        self.target_predicate.base_max_distance = follow_range;

        let world = mob_entity.living_entity.entity.world.load();

        // Vanilla searches using getEyeY(), so we offset the position by the eye height
        let mut search_pos = mob_entity.living_entity.entity.pos.load();
        search_pos.y += mob_entity
            .living_entity
            .entity
            .entity_dimension
            .load()
            .eye_height as f64;

        // Pick the nearest candidate that passes the conditions, not the nearest overall.
        let predicate = &self.target_predicate;
        let condition = self.condition;
        let found = if let Some(uuid) = condition.grudge_target(mob) {
            // Same range rule as the area search: follow range from the eye.
            find_by_uuid(&world, uuid).filter(|candidate| {
                let entity = candidate.get_entity();
                self.target_type
                    .is_none_or(|target_type| entity.entity_type == target_type)
                    && entity.pos.load().squared_distance_to_vec(&search_pos)
                        <= follow_range * follow_range
                    && predicate.test(&world, Some(mob), candidate.as_ref())
                    && condition.allows_target(mob, candidate.as_ref(), &world)
            })
        } else if self.target_type == Some(&EntityType::PLAYER) {
            world
                .get_nearest_player(search_pos, follow_range, |player| {
                    predicate.test(&world, Some(mob), player.as_ref())
                        && condition.allows_target(mob, player.as_ref(), &world)
                })
                .map(|p: Arc<Player>| p as Arc<dyn EntityBase>)
        } else {
            let entity_types = self.target_type.map(|t| [t]);
            world.get_nearest_entity(
                search_pos,
                follow_range,
                entity_types.as_ref().map(<[&EntityType; 1]>::as_slice),
                |entity| {
                    predicate.test(&world, Some(mob), entity.as_ref())
                        && condition.allows_target(mob, entity.as_ref(), &world)
                },
            )
        };

        self.target = found;
    }
}

impl Goal for ActiveTargetGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.reciprocal_chance > 0
            && mob.get_random().random_range(0..self.reciprocal_chance) != 0
        {
            return false;
        }
        if !self.condition.allows_search(mob) {
            return false;
        }
        self.find_closest_target(mob);
        self.target.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        // Condition holding at start must keep holding. daylight, grudge.
        if !self.condition.allows_search(mob) {
            return false;
        }
        if let Some(target) = mob.get_mob_entity().get_target() {
            let world = mob.get_mob_entity().living_entity.entity.world.load();
            if !self.condition.allows_target(mob, target.as_ref(), &world) {
                return false;
            }
        }
        self.track_target_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.set_mob_target(self.target.clone());
        self.track_target_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.track_target_goal.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}
