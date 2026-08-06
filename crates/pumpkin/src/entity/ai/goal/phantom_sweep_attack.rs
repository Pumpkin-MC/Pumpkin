//! Port of `Phantom.PhantomSweepAttackGoal` (`Phantom.java:443-505`).

use std::sync::Weak;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::entity::EntityType;
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::EntityBase;
use crate::entity::ai::goal::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::entity::mob::phantom::{AttackPhase, PhantomEntity};

/// Vanilla: `PhantomSweepAttackGoal.CAT_SEARCH_TICK_DELAY`.
const CAT_SEARCH_TICK_DELAY: i32 = 20;
/// Vanilla: cats scare phantoms within this radius (`getBoundingBox().inflate(16.0)`).
const CAT_SEARCH_RADIUS: f64 = 16.0;

pub struct PhantomSweepAttackGoal {
    phantom: Weak<PhantomEntity>,
    is_scared_of_cat: bool,
    next_cat_search_tick: i32,
}

impl PhantomSweepAttackGoal {
    #[must_use]
    pub const fn new(phantom: Weak<PhantomEntity>) -> Self {
        Self {
            phantom,
            is_scared_of_cat: false,
            next_cat_search_tick: 0,
        }
    }
}

impl Goal for PhantomSweepAttackGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(phantom) = self.phantom.upgrade() else {
                return false;
            };
            phantom.mob_entity.target.lock().await.is_some()
                && phantom.attack_phase() == AttackPhase::Swoop
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(phantom) = self.phantom.upgrade() else {
                return false;
            };
            let target = phantom.mob_entity.target.lock().await.clone();
            let Some(target) = target else {
                return false;
            };
            if !target.get_entity().is_alive() {
                return false;
            }
            if let Some(player) = target.get_player()
                && (player.is_spectator() || player.is_creative())
            {
                return false;
            }

            if phantom.mob_entity.target.lock().await.is_none()
                || phantom.attack_phase() != AttackPhase::Swoop
            {
                return false;
            }

            let tick_count = phantom.mob_entity.living_entity.entity.age.load(Relaxed);
            if tick_count > self.next_cat_search_tick {
                self.next_cat_search_tick = tick_count + CAT_SEARCH_TICK_DELAY;
                let entity = &phantom.mob_entity.living_entity.entity;
                let world = entity.world.load_full();
                let pos = entity.pos.load();
                let bb = entity.bounding_box.load().expand_all(CAT_SEARCH_RADIUS);
                let cats_nearby = world
                    .get_nearby_entities(pos, CAT_SEARCH_RADIUS + bb.get_average_side_length())
                    .into_values()
                    .filter(|candidate| {
                        candidate.get_entity().entity_type == &EntityType::CAT
                            && candidate.get_entity().is_alive()
                            && bb.intersects(&candidate.get_entity().bounding_box.load())
                    })
                    .count();
                // Vanilla also calls `cat.hiss()` on every found cat; Pumpkin's `CatEntity` has
                // no equivalent hook, so only the scare flag (which gates the dive) is ported.
                self.is_scared_of_cat = cats_nearby > 0;
            }

            !self.is_scared_of_cat
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(phantom) = self.phantom.upgrade() else {
                return;
            };
            let _ = mob;
            phantom.set_mob_target(None).await;
            phantom.set_attack_phase(AttackPhase::Circle);
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(phantom) = self.phantom.upgrade() else {
                return;
            };
            let target = phantom.mob_entity.target.lock().await.clone();
            let Some(target) = target else {
                return;
            };

            let target_entity = target.get_entity();
            let target_pos = target_entity.pos.load();
            let target_height = f64::from(target_entity.entity_dimension.load().height);
            phantom.set_move_target_point(Vector3::new(
                target_pos.x,
                target_pos.y + target_height * 0.5,
                target_pos.z,
            ));

            let entity = &phantom.mob_entity.living_entity.entity;
            let own_bb = entity.bounding_box.load().expand_all(0.2);
            if own_bb.intersects(&target_entity.bounding_box.load()) {
                mob.try_attack(target.as_ref()).await;
                phantom.set_attack_phase(AttackPhase::Circle);
                let pos = entity.block_pos.load();
                let world = entity.world.load_full();
                // Vanilla guards this on `!isSilent()`; Pumpkin's `Entity` has no silent flag,
                // so the bite event is always sent (matches the common, non-silenced case).
                world.sync_world_event(WorldEvent::SoundPhantomBite, pos, 0);
            } else if entity.horizontal_collision.load(Relaxed)
                || phantom.mob_entity.living_entity.hurt_cooldown.load(Relaxed) > 0
            {
                phantom.set_attack_phase(AttackPhase::Circle);
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
