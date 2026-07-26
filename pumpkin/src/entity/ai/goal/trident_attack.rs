//! Vanilla `DrownedTridentAttackGoal` — a trident-holding drowned throws it
//! from range instead of swinging.

use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::projectile::arrow::ArrowPickup;
use crate::entity::projectile::trident::TridentEntity;
use crate::entity::{Entity, EntityBase};

/// Vanilla RangedAttackGoal(1.0, 40, 10.0F) wrapped by DrownedTridentAttackGoal.
const ATTACK_RADIUS_SQ: f64 = 10.0 * 10.0;
const THROW_INTERVAL: i32 = 40;

pub struct TridentAttackGoal {
    attack_time: i32,
    speed: f64,
}

impl TridentAttackGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            attack_time: 0,
            speed,
        })
    }

    async fn holds_trident(mob: &dyn Mob) -> bool {
        let equipment = mob
            .get_mob_entity()
            .living_entity
            .entity_equipment
            .lock()
            .await;
        let hand = equipment.get(&EquipmentSlot::MAIN_HAND);
        let stack = hand.lock().await;
        !stack.is_empty() && stack.item.id == Item::TRIDENT.id
    }

    fn look_angles(from: Vector3<f64>, to: Vector3<f64>) -> (f32, f32) {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let dz = to.z - from.z;
        let horizontal = dx.hypot(dz);
        let yaw = (dz.atan2(dx).to_degrees() as f32) - 90.0;
        let pitch = -(dy.atan2(horizontal).to_degrees() as f32);
        (yaw, pitch)
    }

    /// Vanilla `Drowned.performRangedAttack`.
    async fn throw_trident(mob: &dyn Mob, target: &dyn EntityBase) {
        let shooter = mob.get_entity();
        let world = shooter.world.load();
        let eye = shooter.get_eye_pos();
        let target_eye = target.get_entity().get_eye_pos();

        let entity = Entity::new(world.clone(), eye, &EntityType::TRIDENT);
        let trident = TridentEntity::new_shot(
            entity,
            shooter,
            ItemStack::new(1, &Item::TRIDENT),
            ArrowPickup::Disallowed,
        );
        let (yaw, pitch) = Self::look_angles(eye, target_eye);
        // Vanilla: speed 1.6, divergence 14 - difficulty*4 (Normal → 6).
        trident.set_velocity_from_rotation(pitch, yaw, 0.0, 1.6, 6.0);

        world.spawn_entity(Arc::new(trident)).await;
        world.play_sound(
            Sound::EntityDrownedShoot,
            SoundCategory::Hostile,
            &shooter.pos.load(),
        );
        mob.get_mob_entity().living_entity.swing_hand().await;
    }
}

impl Goal for TridentAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !Self::holds_trident(mob).await {
                return false;
            }
            let target = mob.get_mob_entity().target.lock().await;
            target.as_ref().is_some_and(|t| t.get_entity().is_alive())
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !Self::holds_trident(mob).await {
                return false;
            }
            let target = mob.get_mob_entity().target.lock().await;
            target.as_ref().is_some_and(|t| t.get_entity().is_alive())
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.attack_time = 0;
            // Vanilla DrownedTridentAttackGoal.start: raise the throwing arm.
            mob.get_mob_entity().set_attacking(true);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().set_attacking(false);
            self.attack_time = 0;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let target = {
                let guard = mob.get_mob_entity().target.lock().await;
                guard.clone()
            };
            let Some(target) = target else {
                return;
            };

            let mob_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let distance_sq = mob_pos.squared_distance_to_vec(&target_pos);

            if distance_sq > ATTACK_RADIUS_SQ {
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(mob_pos, target_pos, self.speed));
            } else {
                mob.get_mob_entity().navigator.lock().unwrap().stop();
            }

            self.attack_time -= 1;
            if self.attack_time <= 0 {
                self.attack_time = to_goal_ticks(THROW_INTERVAL);
                Self::throw_trident(mob, target.as_ref()).await;
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        false
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
