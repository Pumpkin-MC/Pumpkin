use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Weak};

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        axolotl_play_dead::AxolotlPlayDeadGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
        non_tame_random_target::NonTameRandomTargetGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    living::LivingEntity,
    mob::{Mob, MobEntity},
    player::Player,
};
use crate::world::World;

/// Vanilla `data/minecraft/tags/entity_type/axolotl_hunt_targets.json` +
/// `axolotl_always_hostiles.json`, the type list consulted by `AxolotlAttackablesSensor`.
const AXOLOTL_TARGET_TYPES: &[&EntityType] = &[
    &EntityType::DROWNED,
    &EntityType::GUARDIAN,
    &EntityType::ELDER_GUARDIAN,
    &EntityType::TROPICAL_FISH,
    &EntityType::PUFFERFISH,
    &EntityType::SALMON,
    &EntityType::COD,
    &EntityType::SQUID,
    &EntityType::GLOW_SQUID,
    &EntityType::TADPOLE,
];

/// Vanilla `AxolotlAttackablesSensor.isMatchingEntity`: `mob.isInWater()`.
///
/// The sensor's `HAS_HUNTING_COOLDOWN` gate (a 2400-tick cooldown on re-targeting hunt-tag prey
/// after leaving the FIGHT activity) is not ported -- axolotls here are always willing to retarget
/// prey, so they hunt slightly more eagerly than vanilla after a fight ends.
async fn axolotl_attackable(target: Arc<LivingEntity>, _world: Arc<World>) -> bool {
    target.entity.touching_water.load(Relaxed)
}

/// Represents an Axolotl, a passive aquatic mob that can play dead to regenerate health.
///
/// Wiki: <https://minecraft.wiki/w/Axolotl>
pub struct AxolotlEntity {
    pub mob_entity: MobEntity,
}

impl AxolotlEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let axolotl = Self { mob_entity };
        let mob_arc = Arc::new(axolotl);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(AxolotlPlayDeadGoal::new()));
            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            // Vanilla `MeleeAttack.create(20)`: 20-tick attack cooldown, matched by
            // `MeleeAttackGoal`'s fixed `attack_interval_ticks`.
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(3, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                4,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(5, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(
                1,
                NonTameRandomTargetGoal::new(
                    &mob_arc.mob_entity,
                    AXOLOTL_TARGET_TYPES,
                    false,
                    Some(axolotl_attackable),
                ),
            );
        };

        mob_arc
    }

    /// Vanilla `Axolotl.applySupportingEffects`: grants the player Regeneration (topping the
    /// duration up to 2400 ticks) and clears Mining Fatigue.
    async fn apply_supporting_effects(player: &Player) {
        let living = &player.living_entity;
        let existing = living.get_effect(&StatusEffect::REGENERATION).await;
        // Vanilla: `regenEffect == null || regenEffect.endsWithin(2399)`.
        let should_apply = existing
            .as_ref()
            .is_none_or(|effect| effect.duration <= 2399);
        if should_apply {
            let previous_duration = existing.map_or(0, |effect| effect.duration);
            // Vanilla: `Math.min(2400, 100 + previousDuration)`.
            let regen_duration = (100 + previous_duration).min(2400);
            living
                .add_effect(Effect {
                    effect_type: &StatusEffect::REGENERATION,
                    duration: regen_duration,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                })
                .await;
        }
        living.remove_effect(&StatusEffect::MINING_FATIGUE).await;
    }
}

impl NBTStorage for AxolotlEntity {}

impl Mob for AxolotlEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla `Axolotl.onStopAttacking`: when a hit kills the target and the target's last
    /// damage source was a player within 20 blocks of this axolotl, that player gets a combat
    /// support buff. Simplification: vanilla checks `body.getBoundingBox().inflate(20.0)`
    /// (an AABB); this uses a plain 20-block spherical distance check instead.
    fn on_successful_attack<'a>(&'a self, target: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let Some(target_living) = target.get_living_entity() else {
                return;
            };
            if target_living.entity.is_alive() {
                return;
            }

            let attacker_id = target_living.last_attacker_id.load(Relaxed);
            let world = self.mob_entity.living_entity.entity.world.load();
            let Some(attacker) = world.get_entity_by_id(attacker_id) else {
                return;
            };
            let Some(player) = attacker.get_player() else {
                return;
            };

            let axolotl_pos = self.mob_entity.living_entity.entity.pos.load();
            let player_pos = player.get_entity().pos.load();
            if axolotl_pos.squared_distance_to_vec(&player_pos) > 20.0 * 20.0 {
                return;
            }

            Self::apply_supporting_effects(player).await;
        })
    }
}
