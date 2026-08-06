use std::sync::{Arc, Weak};

use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{DataComponentImpl, PotionContentsImpl};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Potion;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::mob::witch::WitchEntity;
use crate::entity::projectile::splash_potion::SplashPotionEntity;
use crate::entity::{Entity, EntityBase};

pub struct WitchAttackGoal {
    witch: Weak<WitchEntity>,
    cooldown: i32,
    interval: i32,
    range: f64,
}

impl WitchAttackGoal {
    #[must_use]
    pub const fn new(witch: Weak<WitchEntity>, interval: i32, range: f64) -> Self {
        Self {
            witch,
            cooldown: 0,
            interval,
            range,
        }
    }

    #[must_use]
    pub const fn choose_potion(
        distance: f64,
        health: f32,
        has_slowness: bool,
        has_poison: bool,
        has_weakness: bool,
        weakness_roll: f32,
    ) -> &'static Potion {
        if distance >= 8.0 && !has_slowness {
            &Potion::SLOWNESS
        } else if health >= 8.0 && !has_poison {
            &Potion::POISON
        } else if distance <= 3.0 && !has_weakness && weakness_roll < 0.25 {
            &Potion::WEAKNESS
        } else {
            &Potion::HARMING
        }
    }

    /// Vanilla `Witch.performRangedAttack`'s `target instanceof Raider` branch: heals rather than
    /// harms hurt raid-mates.
    #[must_use]
    const fn choose_raider_heal_potion(health: f32) -> &'static Potion {
        if health <= 4.0 {
            &Potion::HEALING
        } else {
            &Potion::REGENERATION
        }
    }

    async fn shoot(&self, mob: &dyn Mob, target: &dyn EntityBase) {
        let shooter = mob.get_entity();
        let world = shooter.world.load_full();
        let shooter_pos = shooter.pos.load();
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();
        let target_velocity = target_entity.velocity.load();
        let x = target_pos.x + target_velocity.x - shooter_pos.x;
        let y = target_pos.y + target_entity.get_eye_height() - 1.1 - shooter_pos.y;
        let z = target_pos.z + target_velocity.z - shooter_pos.z;
        let distance = x.hypot(z);

        let is_raider = target_entity
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_RAIDERS);
        let (health, has_slowness, has_poison, has_weakness) =
            if let Some(living) = target.get_living_entity() {
                (
                    living.health.load(),
                    living
                        .get_effect(&pumpkin_data::effect::StatusEffect::SLOWNESS)
                        .await
                        .is_some(),
                    living
                        .get_effect(&pumpkin_data::effect::StatusEffect::POISON)
                        .await
                        .is_some(),
                    living
                        .get_effect(&pumpkin_data::effect::StatusEffect::WEAKNESS)
                        .await
                        .is_some(),
                )
            } else {
                (0.0, false, false, false)
            };

        let potion = if is_raider {
            Self::choose_raider_heal_potion(health)
        } else {
            Self::choose_potion(
                distance,
                health,
                has_slowness,
                has_poison,
                has_weakness,
                rand::random(),
            )
        };

        let stack = ItemStack::new_with_component(
            1,
            &Item::SPLASH_POTION,
            vec![(
                DataComponent::PotionContents,
                Some(
                    PotionContentsImpl {
                        potion_id: Some(potion.id as i32),
                        custom_color: None,
                        custom_effects: Vec::new(),
                        custom_name: None,
                    }
                    .to_dyn(),
                ),
            )],
        );
        let projectile_entity = Entity::new(world.clone(), shooter_pos, &EntityType::SPLASH_POTION);
        let projectile = SplashPotionEntity::new_shot(projectile_entity, shooter);
        projectile.set_item_stack(stack).await;
        projectile.thrown.set_velocity(
            x,
            y + distance * 0.2,
            z,
            if distance <= 2.0 { 0.45 } else { 0.75 },
            8.0,
        );
        world.spawn_entity(Arc::new(projectile)).await;
        world.play_sound(
            Sound::EntityWitchThrow,
            SoundCategory::Hostile,
            &shooter_pos,
        );

        // Vanilla: a witch that decides to help a hurt raid-mate immediately clears its own
        // target, rather than continuing to treat it as an attack target.
        if is_raider {
            mob.set_mob_target(None).await;
        }
    }
}

impl Goal for WitchAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self
                .witch
                .upgrade()
                .is_some_and(|witch| witch.is_drinking_potion())
            {
                return false;
            }
            mob.get_mob_entity()
                .target
                .lock()
                .await
                .as_ref()
                .is_some_and(|target| target.get_entity().is_alive())
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self
                .witch
                .upgrade()
                .is_some_and(|witch| witch.is_drinking_potion())
            {
                return false;
            }
            mob.get_mob_entity()
                .target
                .lock()
                .await
                .as_ref()
                .is_some_and(|target| target.get_entity().is_alive())
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.cooldown = 0;
            mob.get_mob_entity().set_attacking(true);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            mob.get_mob_entity().set_attacking(false);
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if self
                .witch
                .upgrade()
                .is_some_and(|witch| witch.is_drinking_potion())
            {
                return;
            }
            let Some(target) = mob.get_mob_entity().target.lock().await.clone() else {
                return;
            };
            let shooter = mob.get_entity();
            let shooter_pos = shooter.pos.load();
            let target_pos = target.get_entity().pos.load();
            let distance_squared = shooter_pos.squared_distance_to_vec(&target_pos);
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);
            self.cooldown = (self.cooldown - 1).max(0);
            if distance_squared > self.range * self.range {
                mob.get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap()
                    .set_progress(NavigatorGoal {
                        current_progress: shooter_pos,
                        destination: target_pos,
                        speed: 1.0,
                    });
            } else {
                mob.get_mob_entity().navigator.lock().unwrap().stop();
                if self.cooldown == 0 {
                    self.shoot(mob, target.as_ref()).await;
                    self.cooldown = self.interval;
                }
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

#[cfg(test)]
mod tests {
    use super::WitchAttackGoal;
    use pumpkin_data::potion::Potion;

    #[test]
    fn witch_potion_selection_matches_vanilla_priority() {
        assert_eq!(
            WitchAttackGoal::choose_potion(10.0, 20.0, false, false, false, 1.0).id,
            Potion::SLOWNESS.id
        );
        assert_eq!(
            WitchAttackGoal::choose_potion(6.0, 20.0, true, false, false, 1.0).id,
            Potion::POISON.id
        );
        assert_eq!(
            WitchAttackGoal::choose_potion(2.0, 6.0, true, true, false, 0.1).id,
            Potion::WEAKNESS.id
        );
        assert_eq!(
            WitchAttackGoal::choose_potion(4.0, 6.0, true, true, true, 1.0).id,
            Potion::HARMING.id
        );
    }

    #[test]
    fn raider_heal_potion_matches_vanilla_health_threshold() {
        assert_eq!(
            WitchAttackGoal::choose_raider_heal_potion(4.0).id,
            Potion::HEALING.id
        );
        assert_eq!(
            WitchAttackGoal::choose_raider_heal_potion(4.01).id,
            Potion::REGENERATION.id
        );
    }
}
