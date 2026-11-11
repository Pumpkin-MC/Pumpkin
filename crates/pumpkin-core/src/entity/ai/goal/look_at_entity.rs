use super::{Controls, Goal, GoalReach};

use crate::entity::EntityBase;
use crate::entity::ai::target_match::{SearchArea, TargetMatch};
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::predicate::EntityPredicate;
use pumpkin_data::data_component_impl::MobVisibilityImpl;
use pumpkin_data::entity::EntityType;
use rand::RngExt;
use std::sync::{Arc, Mutex, PoisonError, Weak};

/// A hand on a [`LookAtEntityGoal`] that is already registered: the owner of the mob sets the
/// entity the goal looks at, as vanilla `PandaLookAtPlayerGoal.setTarget` does.
#[derive(Default)]
pub struct LookTargetSlot {
    pending: Mutex<PendingTarget>,
}

/// A `set_target` that the goal has not picked up yet; `Set(None)` is `setTarget(null)`.
#[derive(Default)]
enum PendingTarget {
    #[default]
    Idle,
    Set(Option<Arc<dyn EntityBase>>),
}

impl LookTargetSlot {
    pub fn set_target(&self, target: Option<Arc<dyn EntityBase>>) {
        *self.pending.lock().unwrap_or_else(PoisonError::into_inner) = PendingTarget::Set(target);
    }

    fn take(&self) -> PendingTarget {
        std::mem::take(&mut *self.pending.lock().unwrap_or_else(PoisonError::into_inner))
    }

    fn pending_id(&self) -> Option<i32> {
        match &*self.pending.lock().unwrap_or_else(PoisonError::into_inner) {
            PendingTarget::Set(Some(target)) => Some(target.get_entity().entity_id),
            _ => None,
        }
    }
}

pub struct LookAtEntityGoal {
    goal_control: Controls,
    target: Option<Arc<dyn EntityBase>>,
    range: f32,
    look_time: i32,
    chance: f32,
    look_forward: bool,
    target_match: TargetMatch,
    target_predicate: TargetPredicate,
    target_slot: Option<Arc<LookTargetSlot>>,
}

impl LookAtEntityGoal {
    #[must_use]
    pub fn new(
        mob_weak: Weak<dyn Mob>,
        target_type: &'static EntityType,
        range: f32,
        chance: f32,
        look_forward: bool,
    ) -> Self {
        Self::matching(
            mob_weak,
            TargetMatch::Type(target_type),
            range,
            chance,
            look_forward,
        )
    }

    #[must_use]
    pub fn matching(
        mob_weak: Weak<dyn Mob>,
        target_match: TargetMatch,
        range: f32,
        chance: f32,
        look_forward: bool,
    ) -> Self {
        let target_predicate = Self::create_target_predicate(mob_weak, target_match, range);
        Self {
            goal_control: Controls::LOOK,
            target: None,
            range,
            look_time: 0,
            chance,
            look_forward,
            target_match,
            target_predicate,
            target_slot: None,
        }
    }

    /// The target can be set through `slot`, and a target that is set is kept instead of being
    /// replaced by the nearest entity: vanilla `PandaLookAtPlayerGoal.canUse` only searches while
    /// it has no target, and it ignores the mob's attack target.
    #[must_use]
    pub fn with_target_slot(mut self, slot: Arc<LookTargetSlot>) -> Self {
        self.target_slot = Some(slot);
        self
    }

    fn take_slot_target(&mut self) {
        if let Some(PendingTarget::Set(target)) = self.target_slot.as_ref().map(|slot| slot.take())
        {
            self.target = target;
        }
    }

    #[must_use]
    pub fn with_default(
        mob_weak: Weak<dyn Mob>,
        target_type: &'static EntityType,
        range: f32,
    ) -> Box<Self> {
        Box::new(Self::new(mob_weak, target_type, range, 0.02, false))
    }

    fn create_target_predicate(
        mob_weak: Weak<dyn Mob>,
        target_match: TargetMatch,
        range: f32,
    ) -> TargetPredicate {
        let mut target_predicate = TargetPredicate::create_non_attackable();
        target_predicate.base_max_distance = f64::from(range);
        if target_match.is_player() {
            target_predicate.set_predicate(move |living_entity, _world| {
                mob_weak.upgrade().is_some_and(|mob_arc| {
                    let predicate = EntityPredicate::Rides(mob_arc.get_entity());
                    predicate.test(&living_entity.entity)
                })
            });
        }
        target_predicate
    }
}

impl Goal for LookAtEntityGoal {
    fn tick_reach(&self, _mob: &dyn Mob, _running: bool) -> Option<GoalReach> {
        let range = f64::from(self.range);
        let looks_at_players = self.target_match.is_player();
        Some(GoalReach {
            entities: if looks_at_players { 0.0 } else { range },
            // The player search is unbounded, only the visibility-scaled range limits it.
            players: if looks_at_players {
                range * f64::from(MobVisibilityImpl::MAX_VISIBILITY)
            } else {
                0.0
            },
            links: self
                .target
                .iter()
                .map(|target| target.get_entity().entity_id)
                .chain(self.target_slot.as_ref().and_then(|slot| slot.pending_id()))
                .collect(),
            ..GoalReach::default()
        })
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.take_slot_target();
        if mob.get_random().random::<f32>() >= self.chance {
            return false;
        }
        if self.target_slot.is_some() && self.target.is_some() {
            return true;
        }

        let mob_entity = mob.get_mob_entity();

        {
            let mob_target = mob_entity
                .target
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if mob_target.is_some() {
                self.target.clone_from(&mob_target);
            }
        }

        let entity = &mob_entity.living_entity.entity;
        let world = entity.world.load();
        // Players anywhere, other mobs within 3 blocks of height; the conditions carry the range.
        let area = SearchArea::from_eyes(entity, self.range.into(), 3.0).players_anywhere();

        // The conditions filter the candidates instead of being applied to the nearest one.
        let predicate = &self.target_predicate;
        self.target = self.target_match.find_nearest(&world, &area, |candidate| {
            predicate.test(&world, Some(mob), candidate)
        });

        self.target.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.take_slot_target();
        let mob_entity = mob.get_mob_entity();
        if let Some(target) = &self.target {
            if !target.get_entity().is_alive() {
                return false;
            }
            let mob_pos = mob_entity.living_entity.entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            if mob_pos.squared_distance_to_vec(&target_pos) as f32 > (self.range * self.range) {
                return false;
            }
            return self.look_time > 0;
        }
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.look_time = self.get_tick_count(40 + mob.get_random().random_range(0..40));
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target = None;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.take_slot_target();
        let mob_entity = mob.get_mob_entity();
        if let Some(target) = &self.target
            && target.get_entity().is_alive()
        {
            let target_entity = target.get_entity();
            let target_pos = target_entity.pos.load();
            let look_y = if self.look_forward {
                mob_entity.living_entity.entity.get_eye_y()
            } else {
                target_entity.get_eye_y()
            };
            mob_entity
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .look_at(mob, target_pos.x, look_y, target_pos.z);
            self.look_time -= 1;
        }
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;

    #[tokio::test]
    async fn a_target_set_through_the_slot_is_looked_at_and_cleared_again()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let spawn = |x: f64| {
            CowEntity::new(Entity::new(
                world.clone(),
                Vector3::new(x, 64.0, 0.5),
                &EntityType::COW,
            ))
        };
        let looker = spawn(0.5);
        let target = spawn(4.5);
        let mob: Arc<dyn Mob> = looker.clone();
        let slot = Arc::new(LookTargetSlot::default());
        let mut goal =
            LookAtEntityGoal::new(Arc::downgrade(&mob), &EntityType::PLAYER, 6.0, 1.0, false)
                .with_target_slot(slot.clone());

        assert!(
            !goal.can_start(looker.as_ref()),
            "no player and nothing set"
        );

        slot.set_target(Some(target));
        assert!(
            goal.can_start(looker.as_ref()),
            "the slot target was not used"
        );
        goal.start(looker.as_ref());
        goal.tick(looker.as_ref());
        let looked_at = looker
            .get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .wanted_position();
        assert!(looked_at.is_some_and(|pos| (pos.x - 4.5).abs() < 1e-9));

        slot.set_target(None);
        assert!(
            !goal.should_continue(looker.as_ref()),
            "the cleared target was kept"
        );
        world.level.shutdown().await;
        Ok(())
    }
}
