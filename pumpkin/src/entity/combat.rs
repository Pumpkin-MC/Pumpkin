use std::sync::atomic::Ordering;

use crate::entity::EntityBase;
use pumpkin_data::{
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

impl AttackType {
    pub async fn new(player: &Player, attack_cooldown_progress: f32) -> Self {
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

        let is_strong = attack_cooldown_progress > 0.9;
        if sprinting && is_strong {
            return Self::Knockback;
        }

        if is_strong && !on_ground && fall_distance > 0.0 {
            return Self::Critical;
        }

        // Vanilla: sweep only for grounded non-sprint full-charge sword hits.
        // Previously every full sword hit used Sweeping SFX (very loud whoosh).
        if sword && is_strong && on_ground && !sprinting {
            return Self::Sweeping;
        }

        if is_strong { Self::Strong } else { Self::Weak }
    }
}

/// Vanilla player-attack knockback. Caller must already scale `strength` by
/// `(1 - knockbackResistance)` (see `Player::attack`). Iron golems have
/// resistance `1.0` so they take zero horizontal KB.
pub fn handle_knockback(attacker: &Entity, victim: &Entity, strength: f64) {
    if strength <= 0.0 {
        return;
    }

    let yaw = attacker.yaw.load();
    victim.knockback(
        strength * 0.5,
        f64::from((yaw.to_radians()).sin()),
        f64::from(-(yaw.to_radians()).cos()),
    );
    victim.send_velocity();

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
    use rand::RngExt;

    // Vanilla plays these at volume 1.0 / pitch 1.0. Slight pitch jitter matches
    // other entity combat SFX and avoids every hit sounding identical/harsh.
    // Weak/no-full-charge attacks use a lower volume so spam hits are quieter.
    let mut rng = rand::rng();
    let pitch = rng.random_range(0.9f32..1.1);
    let (sound, volume) = match attack_type {
        AttackType::Knockback => (Sound::EntityPlayerAttackKnockback, 1.0),
        AttackType::Critical => (Sound::EntityPlayerAttackCrit, 1.0),
        AttackType::Sweeping => (Sound::EntityPlayerAttackSweep, 1.0),
        AttackType::Strong => (Sound::EntityPlayerAttackStrong, 1.0),
        AttackType::Weak => (Sound::EntityPlayerAttackWeak, 0.5),
        AttackType::MaceSmash => (Sound::ItemMaceSmashAir, 1.0),
    };
    world.play_sound_fine(sound, SoundCategory::Players, pos, volume, pitch);
}
