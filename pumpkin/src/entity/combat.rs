use crate::entity::EntityBase;
use crate::{
    entity::{Entity, player::Player},
    world::World,
};
use dashmap::DashMap;
use pumpkin_config::advanced_config;

use pumpkin_data::{
    particle::Particle,
    sound::{Sound, SoundCategory},
};
use pumpkin_protocol::java::client::play::{CPlayDisconnect, CPlayerPosition, CUpdateEntityPos};
use pumpkin_protocol::java::server::play::SPlayerPosition;
use pumpkin_protocol::{codec::var_int::VarInt, java::client::play::CEntityVelocity};
use pumpkin_util::math::square;
use pumpkin_util::math::vector3::Vector3;
use rand::Rng;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::sync::{Arc, LazyLock};
use uuid::Uuid;

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
        let combat_profile = GLOBAL_COMBAT_PROFILE.clone();
        let entity = &player.living_entity.entity;

        let sprinting = entity.sprinting.load(Acquire);
        let on_ground = entity.on_ground.load(Acquire);

        let fall_distance = player.living_entity.fall_distance.load();
        let sword = player.inventory().held_item().lock().await.is_sword();

        let combat_modern = combat_profile.combat_type() == CombatType::Modern;

        let is_strong = if combat_modern {
            attack_cooldown_progress > 0.9
        } else {
            // TODO: probably this is done differently, depending on current velocity in classic
            true
        };

        if sprinting && is_strong {
            return Self::Knockback;
        }

        // TODO: even more checks
        if is_strong && !on_ground && fall_distance > 0.0 {
            // !sprinting omitted
            return Self::Critical;
        }

        // TODO: movement speed check
        if sword && is_strong && combat_modern {
            // !is_crit, !is_knockback_hit, on_ground omitted
            return Self::Sweeping;
        }

        if is_strong { Self::Strong } else { Self::Weak }
    }
}

pub async fn handle_knockback(attacker: &Entity, world: &World, victim: &Entity, strength: f64) {
    let combat_profile = GLOBAL_COMBAT_PROFILE.clone();

    let saved_velo = victim.velocity.load();
    combat_profile.apply_attack_knockback(attacker, victim, strength);

    // TODO: Call this on every hit, even if no knockback is applied
    if combat_profile.combat_type() == CombatType::Classic {
        if let Some(attacker) = attacker.get_living_entity() {
            let current_pos = attacker.entity.pos.load();
            let correction_packet = CUpdateEntityPos::new(
                attacker.entity_id().into(),
                Vector3::new(
                    current_pos.x as i16,
                    current_pos.y as i16,
                    current_pos.z as i16,
                ),
                true,
            );
            world.broadcast_packet_all(&correction_packet).await;
        }
    }

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
    let d = -f64::from(yaw.to_radians().sin());
    let e = f64::from(yaw.to_radians().cos());

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
    let combat_type = GLOBAL_COMBAT_PROFILE.combat_type();

    let sound = match attack_type {
        AttackType::Knockback => {
            if combat_type == CombatType::Modern {
                Sound::EntityPlayerAttackKnockback
            } else {
                Sound::EntityPlayerHurt
            }
        }
        AttackType::Critical => {
            if combat_type == CombatType::Modern {
                Sound::EntityPlayerAttackCrit
            } else {
                Sound::EntityPlayerHurt
            }
        }
        AttackType::Sweeping => Sound::EntityPlayerAttackSweep,
        AttackType::Strong => {
            if combat_type == CombatType::Modern {
                Sound::EntityPlayerAttackStrong
            } else {
                Sound::EntityPlayerHurt
            }
        }
        AttackType::Weak => {
            if combat_type == CombatType::Modern {
                Sound::EntityPlayerAttackWeak
            } else {
                Sound::EntityPlayerHurt
            }
        }
    };

    world.play_sound(sound, SoundCategory::Players, pos).await;
}

/// This map represents per-player `CombatProfile`s, in-case plugins need to overwrite the settings inside `features.toml` for some players, but not all.
pub static COMBAT_PROFILES: LazyLock<DashMap<Uuid, Arc<dyn CombatProfile + Send + Sync>>> =
    LazyLock::new(DashMap::new);

/// This is a global in-memory cache that holds the current knockback configuration.
// TODO: Better error handling
pub static GLOBAL_COMBAT_PROFILE: LazyLock<Arc<dyn CombatProfile + Send + Sync>> = LazyLock::new(
    || {
        let config = &advanced_config().pvp;

        match config.combat_type.to_lowercase().as_str() {
            "classic" => {
                log::info!("Loaded Classic Combat Profile");
                Arc::new(ClassicProfile {
                    friction: config.friction,
                    horizontal_kb: config.horizontal_kb,
                    vertical_kb: config.vertical_kb,
                    vertical_limit: config.vertical_limit,
                    extra_horizontal_kb: config.extra_horizontal_kb,
                    extra_vertical_kb: config.extra_vertical_kb,
                })
            }
            "modern" => {
                log::info!("Loaded Modern Combat Profile");
                Arc::new(ModernProfile {
                    friction: config.friction,
                    horizontal_kb: config.horizontal_kb,
                    vertical_kb: config.vertical_kb,
                    vertical_limit: config.vertical_limit,
                    extra_horizontal_kb: config.extra_horizontal_kb,
                    extra_vertical_kb: config.extra_vertical_kb,
                })
            }
            unknown => {
                log::warn!(
                    "Combat Profile '{unknown}' does not exist! Falling back to Modern Combat Profile instead."
                );
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
    },
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatType {
    Classic,
    Modern,
}

impl CombatType {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Modern => "Modern",
        }
    }
}

#[allow(dead_code)]
pub trait CombatProfile: Send + Sync {
    fn apply_attack_knockback(&self, attacker: &Entity, target: &Entity, strength: f64);
    fn receive_knockback(&self, strength: f64, entity: &Entity, knockback_x: f64, knockback_z: f64);
    fn combat_type(&self) -> CombatType;
    fn friction(&self) -> f64;
    fn horizontal_kb(&self) -> f64;
    fn vertical_kb(&self) -> f64;
    fn vertical_limit(&self) -> f64;
    fn extra_horizontal_kb(&self) -> f64;
    fn extra_vertical_kb(&self) -> f64;
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
    /// Getting called from an attacker, when attacking an entity
    fn apply_attack_knockback(&self, attacker: &Entity, target: &Entity, strength: f64) {
        let yaw: f64 = f64::from(attacker.yaw.load());
        let yaw_rad = yaw.to_radians();

        // The `extra_horizontal_kb` is 0.5 and `extra_vertical_kb` 0.1 by default in java mc 1.8
        let knockback_x = yaw_rad.sin() * strength * self.extra_horizontal_kb;
        let knockback_z = -yaw_rad.cos() * strength * self.extra_horizontal_kb;

        let velocity = target.velocity.load();

        target.velocity.store(Vector3::new(
            velocity.x,
            velocity.y + self.extra_vertical_kb,
            velocity.z,
        ));

        if let Some(attacker) = attacker.get_living_entity() {
            let velo = attacker.entity.velocity.load();
            let magnitude_3d = (square(velo.x) + square(velo.y) + square(velo.z)).sqrt();
            //let mut velocity_multiplier = magnitude_3d / 5.6;
            //velocity_multiplier = velocity_multiplier.clamp(0.1, 1.0);

            //send_packet(&CPlayDisconnect::new(&reason)).await;
            // TODO: This needs to go away, I think
            target.knockback(
                (strength + magnitude_3d / 5.6) * 0.5,
                knockback_x,
                knockback_z,
            );
        } else {
            target.knockback(strength * 0.5, knockback_x, knockback_z);
        }
    }

    /// Getting called on a target, when being attacked
    ///
    /// The field `knockback_resistance` is currently missing from Pumpkin. Refer to [Issue #1013](https://github.com/Pumpkin-MC/Pumpkin/issues/1013) for details.
    // the `float p_70653_2_` from java is dead code, so I removed it
    fn receive_knockback(
        &self,
        _strength: f64,
        entity: &Entity,
        knockback_x: f64,
        knockback_z: f64,
    ) {
        let mut rng = rand::rng();
        // TODO: Use the actual value as soon as this field gets added to `Entity`.
        let knockback_resistance = 0.0;

        if rng.random::<f64>() >= knockback_resistance {
            entity.on_ground.store(false, Release);
            let magnitude = (square(knockback_x) + square(knockback_z)).sqrt();
            let mut velocity = entity.velocity.load();

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

            entity.velocity.store(velocity);
        }
    }

    fn combat_type(&self) -> CombatType {
        CombatType::Classic
    }

    fn friction(&self) -> f64 {
        self.friction
    }

    fn horizontal_kb(&self) -> f64 {
        self.horizontal_kb
    }

    fn vertical_kb(&self) -> f64 {
        self.vertical_kb
    }

    fn vertical_limit(&self) -> f64 {
        self.vertical_limit
    }

    fn extra_horizontal_kb(&self) -> f64 {
        self.extra_horizontal_kb
    }

    fn extra_vertical_kb(&self) -> f64 {
        self.extra_vertical_kb
    }
}

/// An `enum` of possible classic `CombatProfile` attack outcomes to be returned in `classic_attack_entity_success`.
pub enum ClassicEntityAttackSuccessType {
    /// Full damage and knockback are applied, because `hurt_time` already reset.
    SmallerHrt,
    /// Reduced damage and no knockback are applied, due to `hurt_time` not having reset yet.
    GreaterHrt,
    /// No damage and no knockback are applied/something went wrong.
    False,
}

/// Determines the outcome of a classic `CombatProfile` attack.
pub async fn classic_attack_entity_success(
    victim: &Arc<dyn EntityBase>,
    damage: f64,
) -> ClassicEntityAttackSuccessType {
    victim
        .get_living_entity()
        .map_or(ClassicEntityAttackSuccessType::False, |living| {
            let hurt_resistant_time = living.hurt_resistant_time.load(Acquire);
            let max_hurt_resistant_time = living.max_hurt_resistant_time.load(Acquire);
            let last_damage_taken = living.last_damage_taken.load();

            if hurt_resistant_time > max_hurt_resistant_time / 2 {
                if damage <= f64::from(last_damage_taken) {
                    ClassicEntityAttackSuccessType::False
                } else {
                    ClassicEntityAttackSuccessType::GreaterHrt
                }
            } else if hurt_resistant_time == 0 {
                ClassicEntityAttackSuccessType::SmallerHrt
            } else {
                ClassicEntityAttackSuccessType::False
            }
        })
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

// These are taken from the existing implementation
impl CombatProfile for ModernProfile {
    fn apply_attack_knockback(&self, attacker: &Entity, target: &Entity, strength: f64) {
        let yaw = attacker.yaw.load();

        target.knockback(
            strength * 0.5,
            f64::from(yaw.to_radians().sin()),
            f64::from(-yaw.to_radians().cos()),
        );
    }

    fn receive_knockback(
        &self,
        strength: f64,
        entity: &Entity,
        knockback_x: f64,
        knockback_z: f64,
    ) {
        // This has some vanilla magic
        let mut x = knockback_x;
        let mut z = knockback_z;

        while x.mul_add(x, z * z) < 1.0E-5 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * strength;
        let velocity = entity.velocity.load();
        entity.velocity.store(Vector3::new(
            velocity.x / 2.0 - var8.x,
            if entity.on_ground.load(Acquire) {
                (velocity.y / 2.0 + strength).min(0.4)
            } else {
                velocity.y
            },
            velocity.z / 2.0 - var8.z,
        ));
    }

    fn combat_type(&self) -> CombatType {
        CombatType::Modern
    }

    fn friction(&self) -> f64 {
        self.friction
    }

    fn horizontal_kb(&self) -> f64 {
        self.horizontal_kb
    }

    fn vertical_kb(&self) -> f64 {
        self.vertical_kb
    }

    fn vertical_limit(&self) -> f64 {
        self.vertical_limit
    }

    fn extra_horizontal_kb(&self) -> f64 {
        self.extra_horizontal_kb
    }

    fn extra_vertical_kb(&self) -> f64 {
        self.extra_vertical_kb
    }
}
