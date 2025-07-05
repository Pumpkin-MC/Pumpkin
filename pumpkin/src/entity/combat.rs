use std::sync::Arc;
use std::sync::atomic::Ordering;
use async_trait::async_trait;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use rand::Rng;
use uuid::Uuid;
use pumpkin_config::advanced_config;
use pumpkin_data::{
    particle::Particle,
    sound::{Sound, SoundCategory},
};
use pumpkin_protocol::{codec::var_int::VarInt, java::client::play::CEntityVelocity};
use pumpkin_util::math::square;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    entity::{Entity, player::Player},
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

pub async fn handle_knockback(attacker: &Entity, world: &World, victim: &Entity, strength: f64) {
    let yaw = attacker.yaw.load();

    let saved_velo = victim.velocity.load();
    victim.knockback(
        strength * 0.5,
        f64::from((yaw.to_radians()).sin()),
        f64::from(-(yaw.to_radians()).cos()),
    );

    let entity_id = VarInt(victim.entity_id);
    let victim_velocity = victim.velocity.load();

    let packet = CEntityVelocity::new(entity_id, victim_velocity);
    let velocity = attacker.velocity.load();
    attacker.velocity.store(velocity.multiply(0.6, 1.0, 0.6));

    victim.velocity.store(saved_velo);
    world.broadcast_packet_all(&packet).await;
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

/// This will only be used by plugins who override per-player `CombatProfile`s
pub static COMBAT_PROFILES: Lazy<DashMap<Uuid, Arc<dyn CombatProfile>>> = Lazy::new(|| DashMap::new());

/// This is a global in-memory cache that is initialised once on pumpkin start. It holds the current knockback configuration.
pub static GLOBAL_COMBAT_PROFILE: Lazy<Arc<dyn CombatProfile + Send + Sync>> = Lazy::new(|| {
    let config = &advanced_config().pvp;

    match config.combat_type.to_lowercase().as_str() {
        "classic" => Arc::new(ClassicProfile {
            friction: config.friction,
            horizontal_kb: config.horizontal_kb,
            vertical_kb: config.vertical_kb,
            vertical_limit: config.vertical_limit,
            extra_horizontal_kb: config.extra_horizontal_kb,
            extra_vertical_kb: config.extra_vertical_kb,
        }),
        "modern" => Arc::new(ModernProfile {
            friction: config.friction,
            horizontal_kb: config.horizontal_kb,
            vertical_kb: config.vertical_kb,
            vertical_limit: config.vertical_limit,
            extra_horizontal_kb: config.extra_horizontal_kb,
            extra_vertical_kb: config.extra_vertical_kb,
        }),
        unknown => {
            log::warn!("Combat Profile '{}' does not exist! Loaded Modern Combat Profile instead.", unknown);
            Arc::new(ModernProfile {
                friction: config.friction,
                horizontal_kb: config.horizontal_kb,
                vertical_kb: config.vertical_kb,
                vertical_limit: config.vertical_limit,
                extra_horizontal_kb: config.extra_horizontal_kb,
                extra_vertical_kb: config.extra_vertical_kb,
            })
        }
    }
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatType {
    Legacy,
    Classic,
    Modern,
}

#[async_trait]
pub trait CombatProfile {
    async fn apply_attack_knockback(&self, attacker: Arc<Player>, target: Arc<Entity>, strength: f64);
    async fn receive_knockback(&self, entity: Arc<Entity>, knockback_x: f64, knockback_z: f64);
    async fn combat_type(&self) -> CombatType;
    async fn friction(&self) -> f64;
    async fn horizontal_kb(&self) -> f64;
    async fn vertical_kb(&self) -> f64;
    async fn vertical_limit(&self) -> f64;
    async fn extra_horizontal_kb(&self) -> f64;
    async fn extra_vertical_kb(&self) -> f64;
}

pub struct ClassicProfile {
    pub friction: f64,
    pub horizontal_kb: f64,
    pub vertical_kb: f64,
    pub vertical_limit: f64,
    pub extra_horizontal_kb: f64,
    pub extra_vertical_kb: f64,
}

impl CombatProfile for ClassicProfile {
    // TODO: send update packet, but maybe do that when this fn is called
    /// Getting called from an attacker, when attacking an entity
    async fn apply_attack_knockback(&self, attacker: Arc<Player>, target: Arc<Entity>, strength: f64) {
        // TODO: Velocity changed flag? + critical hit flag?
        let yaw: f64 = target.yaw.load() as f64;
        let yaw_rad = yaw.to_radians();

        // The `extra_horizontal_kb` is 0.5 and `extra_vertical_kb` 0.1 by default in java mc 1.8
        let knockback_x = -yaw_rad.sin() * strength * self.extra_horizontal_kb;
        let knockback_z =  yaw_rad.cos() * strength * self.extra_horizontal_kb;
        let knockback_y = self.extra_vertical_kb;

        let mut velocity = target.velocity.load();

        velocity.x += knockback_x;
        velocity.y += knockback_y;
        velocity.z += knockback_z;

        let mut attacker_velocity = attacker.living_entity.entity.velocity.load();
        attacker_velocity.x *= 0.6;
        attacker_velocity.z *= 0.6;

        // TODO: ADD not STORE the velocity ? Lune I'm confused
        target.velocity.store(velocity);

        attacker.living_entity.entity.sprinting.store(false, Ordering::Relaxed);
    }

    /// Getting called on a target, when being attacked
    // the `float p_70653_2_` from java is dead code, so I removed it
    async fn receive_knockback(&self, target: Arc<Entity>, knockback_x: f64, knockback_z: f64) {
        let mut rng = rand::rng();
        let knockback_resistance = self.get_attribute_value("knockback_resistance").await;

        if rng.random::<f64>() >= knockback_resistance {
            target.on_ground = false;

            let magnitude = (square(knockback_x) + square(knockback_z)).sqrt();
            let mut velocity = target.velocity.load();

            // `friction` is 2.0 by default in java mc 1.8
            velocity.x /= self.friction;
            velocity.y /= self.friction;
            velocity.z /= self.friction;

            // `horizontal_kb` and `vertical_kb` are 0.4 by default in java mc 1.8
            velocity.x -= knockback_x / magnitude * self.horizontal_kb;
            velocity.y += self.vertical_kb;
            velocity.z -= knockback_z / magnitude * self.horizontal_kb;

            // `vertical_limit` is 0.4000000059604645 by default in java mc 1.8
            if velocity.y > self.vertical_limit {
                velocity.y = self.vertical_limit;
            }
        }
    }

    async fn combat_type(&self) -> CombatType {
        CombatType::Classic
    }

    async fn friction(&self) -> f64 {
        self.friction
    }

    async fn horizontal_kb(&self) -> f64 {
        self.horizontal_kb
    }

    async fn vertical_kb(&self) -> f64 {
        self.vertical_kb
    }

    async fn vertical_limit(&self) -> f64 {
        self.vertical_limit
    }

    async fn extra_horizontal_kb(&self) -> f64 {
        self.extra_horizontal_kb
    }

    async fn extra_vertical_kb(&self) -> f64 {
        self.extra_vertical_kb
    }
}

// TODO: Hier sind andere Werte wichtig -> ändern
pub struct ModernProfile {
    pub friction: f64,
    pub horizontal_kb: f64,
    pub vertical_kb: f64,
    pub vertical_limit: f64,
    pub extra_horizontal_kb: f64,
    pub extra_vertical_kb: f64,
}

impl CombatProfile for ModernProfile {
    async fn apply_attack_knockback(&self, attacker: Arc<Player>, target: Arc<Entity>, strength: f64) {
        todo!()
    }

    async fn receive_knockback(&self, entity: Arc<Entity>, knockback_x: i32, knockback_z: i32) {
        todo!()
    }

    async fn combat_type(&self) -> CombatType {
        CombatType::Modern
    }

    async fn friction(&self) -> f64 {
        self.friction
    }

    async fn horizontal_kb(&self) -> f64 {
        self.horizontal_kb
    }

    async fn vertical_kb(&self) -> f64 {
        self.vertical_kb
    }

    async fn vertical_limit(&self) -> f64 {
        self.vertical_limit
    }

    async fn extra_horizontal_kb(&self) -> f64 {
        self.extra_horizontal_kb
    }

    async fn extra_vertical_kb(&self) -> f64 {
        self.extra_vertical_kb
    }
}
