use std::sync::atomic::Ordering;

use crate::entity::EntityBase;
use pumpkin_data::{
    attributes::Attributes,
    particle::Particle,
    sound::{Sound, SoundCategory},
};
use pumpkin_util::math::vector3::Vector3;

use crate::{
    entity::{Entity, player::Player},
    world::World,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackType {
    Knockback,
    Critical,
    Sweeping,
    Strong,
    Weak,
    MaceSmash,
}

#[expect(
    clippy::fn_params_excessive_bools,
    reason = "These flags directly mirror vanilla's critical-attack predicate"
)]
#[expect(
    clippy::too_many_arguments,
    reason = "Predicate mirrors vanilla state gates"
)]
fn can_critical_attack(
    on_ground: bool,
    fall_distance: f32,
    climbing: bool,
    in_water: bool,
    mobility_restricted: bool,
    mounted: bool,
    target_is_living: bool,
    sprinting: bool,
) -> bool {
    !on_ground
        && fall_distance > 0.0
        && !climbing
        && !in_water
        && !mobility_restricted
        && !mounted
        && target_is_living
        && !sprinting
}

fn can_sweep_attack(
    sword: bool,
    is_strong: bool,
    on_ground: bool,
    horizontal_speed_squared: f64,
    movement_speed: f64,
) -> bool {
    sword && is_strong && on_ground && horizontal_speed_squared < (movement_speed * 2.5).powi(2)
}

impl AttackType {
    pub async fn new(
        player: &Player,
        target: &dyn EntityBase,
        attack_cooldown_progress: f32,
    ) -> Self {
        let entity = &player.get_entity();

        let sprinting = entity.is_sprinting();
        let on_ground = entity.on_ground.load(Ordering::Relaxed);
        let fall_distance = player.living_entity.fall_distance.load();
        let held_item = player.inventory().held_item();
        let is_mace = {
            let stack = held_item.lock().await;
            stack.item.id == pumpkin_data::item::Item::MACE.id
        };

        if is_mace && !on_ground && fall_distance > 1.5 {
            return Self::MaceSmash;
        }

        let sword = {
            let stack = held_item.lock().await;
            stack.is_sword()
        };

        let in_water = player.living_entity.is_in_water();
        let mobility_restricted = player
            .living_entity
            .get_effect(&pumpkin_data::effect::StatusEffect::BLINDNESS)
            .await
            .is_some();
        let climbing = player.living_entity.climbing.load(Ordering::Relaxed);
        let mounted = entity.has_vehicle().await;
        let target_is_living = target.get_living_entity().is_some();
        let movement = entity.velocity.load();
        let movement_speed = player
            .living_entity
            .get_attribute_value(&Attributes::MOVEMENT_SPEED);

        let is_strong = attack_cooldown_progress > 0.9;
        if sprinting && is_strong {
            return Self::Knockback;
        }

        if is_strong
            && can_critical_attack(
                on_ground,
                fall_distance,
                climbing,
                in_water,
                mobility_restricted,
                mounted,
                target_is_living,
                sprinting,
            )
        {
            return Self::Critical;
        }

        if can_sweep_attack(
            sword,
            is_strong,
            on_ground,
            movement.horizontal_length_squared(),
            movement_speed,
        ) {
            return Self::Sweeping;
        }

        if is_strong { Self::Strong } else { Self::Weak }
    }
}

/// Scales a knockback `strength` by a living entity's knockback resistance,
/// mirroring vanilla `LivingEntity.knockback`: `strength *= 1.0 - resistance`.
/// A resistance of 1.0 (iron golem, warden, ...) cancels the knockback entirely.
pub fn knockback_after_resistance(strength: f64, resistance: f64) -> f64 {
    strength * (1.0 - resistance)
}

pub fn handle_knockback(attacker: &Entity, victim: &dyn EntityBase, strength: f64) {
    let resistance = victim.get_living_entity().map_or(0.0, |living| {
        living.get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE)
    });
    let strength = knockback_after_resistance(strength * 0.5, resistance);

    if strength > 0.0 {
        let yaw = attacker.yaw.load();
        victim.get_entity().knockback(
            strength,
            f64::from((yaw.to_radians()).sin()),
            f64::from(-(yaw.to_radians()).cos()),
        );
    }

    let velocity = attacker.velocity.load();
    attacker.velocity.store(velocity.multiply(0.6, 1.0, 0.6));
}

pub fn spawn_sweep_particle(attacker_entity: &Entity, world: &World, pos: &Vector3<f64>) {
    let yaw = attacker_entity.yaw.load();
    let d = -f64::from((yaw.to_radians()).sin());
    let e = f64::from((yaw.to_radians()).cos());

    let scale = 0.5;
    let body_y = f64::from(attacker_entity.height()).mul_add(scale, pos.y);

    world.spawn_particle(
        Vector3::new(pos.x + d, body_y, pos.z + e),
        Vector3::new(0.0, 0.0, 0.0),
        0.0,
        0,
        Particle::SweepAttack,
    );
}

pub async fn player_attack_sound(pos: &Vector3<f64>, world: &World, attack_type: AttackType) {
    match attack_type {
        AttackType::Knockback => {
            world.play_sound(
                Sound::EntityPlayerAttackKnockback,
                SoundCategory::Players,
                pos,
            );
        }
        AttackType::Critical => {
            world.play_sound(Sound::EntityPlayerAttackCrit, SoundCategory::Players, pos);
        }
        AttackType::Sweeping => {
            world.play_sound(Sound::EntityPlayerAttackSweep, SoundCategory::Players, pos);
        }
        AttackType::Strong => {
            world.play_sound(Sound::EntityPlayerAttackStrong, SoundCategory::Players, pos);
        }
        AttackType::Weak => {
            world.play_sound(Sound::EntityPlayerAttackWeak, SoundCategory::Players, pos);
        }
        AttackType::MaceSmash => {
            world.play_sound(Sound::ItemMaceSmashAir, SoundCategory::Players, pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{can_critical_attack, can_sweep_attack, knockback_after_resistance};

    #[test]
    fn critical_attacks_require_vanilla_movement_conditions() {
        assert!(can_critical_attack(
            false, 1.0, false, false, false, false, true, false
        ));
        assert!(!can_critical_attack(
            false, 1.0, true, false, false, false, true, false
        ));
        assert!(!can_critical_attack(
            false, 1.0, false, true, false, false, true, false
        ));
        assert!(!can_critical_attack(
            false, 1.0, false, false, true, false, true, false
        ));
    }

    #[test]
    fn sweep_attacks_stop_above_vanilla_speed_limit() {
        assert!(can_sweep_attack(true, true, true, 1.0, 1.0));
        assert!(!can_sweep_attack(true, true, true, 39.0625, 1.0));
        assert!(!can_sweep_attack(true, false, true, 1.0, 1.0));
    }

    #[test]
    fn zero_resistance_keeps_full_strength() {
        assert_eq!(knockback_after_resistance(0.4, 0.0), 0.4);
    }

    #[test]
    fn full_resistance_cancels_knockback() {
        // Iron golem / warden have KNOCKBACK_RESISTANCE == 1.0.
        assert_eq!(knockback_after_resistance(0.4, 1.0), 0.0);
    }

    #[test]
    fn partial_resistance_scales_strength() {
        // Ravager has KNOCKBACK_RESISTANCE == 0.75.
        assert!((knockback_after_resistance(0.4, 0.75) - 0.1).abs() < 1e-9);
    }

    #[test]
    fn over_full_resistance_is_negative_so_callers_skip_it() {
        // Stacked armour modifiers can push resistance above 1.0; the result is
        // negative and callers guard on `strength > 0.0`.
        assert!(knockback_after_resistance(0.4, 1.2) < 0.0);
    }
}
