use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering::Relaxed},
};

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_protocol::java::client::play::{CGameEvent, GameEvent};
use pumpkin_util::GameMode;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, guardian_attack::GuardianAttackGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        move_towards_restriction::MoveTowardsRestrictionGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// `ElderGuardian.EFFECT_INTERVAL` / `EFFECT_RADIUS` / `EFFECT_DURATION` /
/// `EFFECT_AMPLIFIER` / `EFFECT_DISPLAY_LIMIT`.
const EFFECT_INTERVAL: i32 = 1200;
const EFFECT_RADIUS: f64 = 50.0;
const EFFECT_DURATION: i32 = 6000;
const EFFECT_AMPLIFIER: u8 = 2;
const EFFECT_DISPLAY_LIMIT: i32 = 1200;

pub struct ElderGuardianEntity {
    pub mob_entity: MobEntity,
    /// Stands in for vanilla's `Entity.tickCount`. `Entity::age` cannot be used here:
    /// in Pumpkin it is the breeding age (negative means baby) and is never advanced
    /// per tick.
    tick_count: AtomicI32,
}

impl ElderGuardianEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let guardian = Self {
            mob_entity,
            tick_count: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(guardian);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // Priorities follow Guardian#registerGoals, which ElderGuardian inherits. No
            // float/swim goal: vanilla doesn't register one.
            goal_selector.add_goal(4, Box::new(GuardianAttackGoal::new()));
            goal_selector.add_goal(5, MoveTowardsRestrictionGoal::new(1.0));
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new_with_interval(1.0, 80)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::GUARDIAN, 12.0),
            );
            goal_selector.add_goal(9, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    /// `MobEffectUtil#addEffectToPlayersAround` predicate.
    const fn should_receive_effect(existing: Option<&Effect>) -> bool {
        match existing {
            None => true,
            Some(effect) => {
                effect.amplifier < EFFECT_AMPLIFIER
                    || (effect.duration != -1 && effect.duration < EFFECT_DISPLAY_LIMIT)
            }
        }
    }

    async fn apply_mining_fatigue(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let origin = entity.pos.load();

        let players: Vec<_> = world.players.load().iter().cloned().collect();
        for player in players {
            if !matches!(
                player.gamemode.load(),
                GameMode::Survival | GameMode::Adventure
            ) {
                continue;
            }

            let player_pos = player.living_entity.entity.pos.load();
            if origin.squared_distance_to(player_pos.x, player_pos.y, player_pos.z)
                > EFFECT_RADIUS * EFFECT_RADIUS
            {
                continue;
            }

            let should_apply = {
                let effects = player.living_entity.active_effects.lock().await;
                Self::should_receive_effect(effects.get(&StatusEffect::MINING_FATIGUE))
            };
            if !should_apply {
                continue;
            }

            player
                .add_effect(Effect {
                    effect_type: &StatusEffect::MINING_FATIGUE,
                    duration: EFFECT_DURATION,
                    amplifier: EFFECT_AMPLIFIER,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                })
                .await;

            player.client.try_enqueue_packet(&CGameEvent::new(
                GameEvent::PlayElderGuardianMobAppearance,
                1.0,
            ));
        }
    }
}

impl NBTStorage for ElderGuardianEntity {}

impl Mob for ElderGuardianEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;
            if !entity.is_alive() {
                return;
            }

            // Vanilla staggers the aura per entity so guardians in one monument do not
            // all fire on the same tick.
            let ticks = self.tick_count.fetch_add(1, Relaxed);
            if ticks
                .wrapping_add(entity.entity_id)
                .rem_euclid(EFFECT_INTERVAL)
                == 0
            {
                self.apply_mining_fatigue().await;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{EFFECT_AMPLIFIER, EFFECT_DISPLAY_LIMIT, ElderGuardianEntity};
    use pumpkin_data::effect::StatusEffect;
    use pumpkin_data::potion::Effect;

    fn mining_fatigue(duration: i32, amplifier: u8) -> Effect {
        Effect {
            effect_type: &StatusEffect::MINING_FATIGUE,
            duration,
            amplifier,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }
    }

    #[test]
    fn refreshes_only_weaker_or_expiring_effects() {
        assert!(ElderGuardianEntity::should_receive_effect(None));
        assert!(ElderGuardianEntity::should_receive_effect(Some(
            &mining_fatigue(6000, EFFECT_AMPLIFIER - 1)
        )));
        assert!(ElderGuardianEntity::should_receive_effect(Some(
            &mining_fatigue(EFFECT_DISPLAY_LIMIT - 1, EFFECT_AMPLIFIER)
        )));
        assert!(!ElderGuardianEntity::should_receive_effect(Some(
            &mining_fatigue(EFFECT_DISPLAY_LIMIT, EFFECT_AMPLIFIER)
        )));
        assert!(!ElderGuardianEntity::should_receive_effect(Some(
            &mining_fatigue(-1, EFFECT_AMPLIFIER)
        )));
    }
}
