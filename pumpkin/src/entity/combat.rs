use pumpkin_data::{
    particle::Particle,
    sound::{Sound, SoundCategory},
};
use pumpkin_util::math::vector3::Vector3;

use crate::{
    entity::{Entity, LivingEntity, player::Player},
    world::World,
};

#[derive(Debug, Clone, Copy)]
pub enum AttackType {
    Knockback,
    Critical,
    Sweeping,
    Strong,
    Weak,
}

impl AttackType {
    pub async fn new(player: &Player, attack_cooldown_progress: f32) -> Self {
        let entity = &player.living_entity.entity;

        let sprinting = entity.sprinting.load(std::sync::atomic::Ordering::Relaxed);
        let on_ground = entity.on_ground.load(std::sync::atomic::Ordering::Relaxed);
        let fall_distance = player.living_entity.fall_distance.load();
        let sword = player.inventory().held_item().lock().await.is_sword();

        let is_strong = attack_cooldown_progress > 0.9;
        if sprinting && is_strong {
            return Self::Knockback;
        }

        // TODO: even more checks
        if is_strong && !on_ground && fall_distance > 0.0 {
            // !sprinting omitted
            return Self::Critical;
        }

        // TODO: movement speed check
        if sword && is_strong {
            // !is_crit, !is_knockback_hit, on_ground omitted
            return Self::Sweeping;
        }

        if is_strong { Self::Strong } else { Self::Weak }
    }
}

pub fn handle_knockback(attacker: &Entity, victim: &LivingEntity, strength: f64) {
    let angle = attacker.pos.load().sub(&victim.entity.pos.load());

    victim
        .entity
        .apply_knockback(strength * 0.5, angle.x, angle.z);
    let velocity = attacker.velocity.load();
    attacker.velocity.store(velocity.multiply(0.6, 1.0, 0.6));
}

pub async fn spawn_sweep_particle(attacker_entity: &Entity, world: &World, pos: &Vector3<f64>) {
    let yaw = attacker_entity.yaw.load();
    let d = -f64::from((yaw.to_radians()).sin());
    let e = f64::from((yaw.to_radians()).cos());

    let scale = 0.5;
    let body_y = pos.y + f64::from(attacker_entity.height()) * scale;

    world
        .spawn_particle(
            Vector3::new(pos.x + d, body_y, pos.z + e),
            Vector3::new(0.0, 0.0, 0.0),
            0.0,
            0,
            Particle::SweepAttack,
        )
        .await;
}

pub async fn player_attack_sound(pos: &Vector3<f64>, world: &World, attack_type: AttackType) {
    match attack_type {
        AttackType::Knockback => {
            world
                .play_sound(
                    Sound::EntityPlayerAttackKnockback,
                    SoundCategory::Players,
                    pos,
                )
                .await;
        }
        AttackType::Critical => {
            world
                .play_sound(Sound::EntityPlayerAttackCrit, SoundCategory::Players, pos)
                .await;
        }
        AttackType::Sweeping => {
            world
                .play_sound(Sound::EntityPlayerAttackSweep, SoundCategory::Players, pos)
                .await;
        }
        AttackType::Strong => {
            world
                .play_sound(Sound::EntityPlayerAttackStrong, SoundCategory::Players, pos)
                .await;
        }
        AttackType::Weak => {
            world
                .play_sound(Sound::EntityPlayerAttackWeak, SoundCategory::Players, pos)
                .await;
        }
    }
}
