use std::sync::atomic::Ordering;

use pumpkin_data::{
    Enchantment,
    attributes::Attributes,
    damage::DamageType,
    entity::EntityType,
    particle::Particle,
    sound::{Sound, SoundCategory},
    tag::{self, Taggable},
};
use pumpkin_util::math::vector3::Vector3;

use crate::{
    entity::{Entity, EntityBase, player::Player},
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

        let sprinting = entity.sprinting.load(Ordering::Relaxed);
        let on_ground = entity.on_ground.load(Ordering::Relaxed);
        let fall_distance = player.living_entity.fall_distance.load();
        let sword = player.inventory().held_item().lock().await.is_sword();

        let is_strong = attack_cooldown_progress > 0.9;
        if sprinting && is_strong {
            return Self::Knockback;
        }

        if is_strong && !on_ground && fall_distance > 0.0 && !sprinting {
            return Self::Critical;
        }

        if sword && is_strong && on_ground && !sprinting {
            return Self::Sweeping;
        }

        if is_strong { Self::Strong } else { Self::Weak }
    }
}

pub fn handle_knockback(
    attacker: &Entity,
    victim: &Entity,
    strength: f64,
    victim_knockback_resistance: f64,
) {
    // Apply knockback resistance (0.0 = no resistance, 1.0 = full immunity)
    let effective_strength = strength * (1.0 - victim_knockback_resistance);
    if effective_strength <= 0.0 {
        return;
    }

    let yaw = attacker.yaw.load();
    victim.knockback(
        effective_strength * 0.5,
        f64::from((yaw.to_radians()).sin()),
        f64::from(-(yaw.to_radians()).cos()),
    );

    let velocity = attacker.velocity.load();
    attacker.velocity.store(velocity.multiply(0.6, 1.0, 0.6));
}

/// Calculate enchantment bonus damage for the given weapon enchantments against the victim.
///
/// Returns the extra damage from Sharpness, Smite, or Bane of Arthropods.
pub fn get_enchantment_damage(
    sharpness_level: i32,
    smite_level: i32,
    bane_level: i32,
    victim: &dyn EntityBase,
) -> f64 {
    let victim_type = victim.get_entity().entity_type;

    // Smite: +2.5 per level vs undead
    if smite_level > 0 && victim_type.has_tag(&tag::EntityType::MINECRAFT_SENSITIVE_TO_SMITE) {
        return 2.5 * f64::from(smite_level);
    }

    // Bane of Arthropods: +2.5 per level vs arthropods
    if bane_level > 0
        && victim_type.has_tag(&tag::EntityType::MINECRAFT_SENSITIVE_TO_BANE_OF_ARTHROPODS)
    {
        return 2.5 * f64::from(bane_level);
    }

    // Sharpness: +0.5 * level + 0.5 extra damage
    if sharpness_level > 0 {
        return f64::from(sharpness_level).mul_add(0.5, 0.5);
    }

    0.0
}

/// Calculate the sweep damage ratio based on Sweeping Edge enchantment level.
///
/// Formula: level / (level + 1)
/// Level 0: 0% of attack damage (just 1 base)
/// Level 1: 50%
/// Level 2: 66.7%
/// Level 3: 75%
pub fn get_sweep_damage_ratio(sweeping_edge_level: i32) -> f32 {
    if sweeping_edge_level > 0 {
        sweeping_edge_level as f32 / (sweeping_edge_level as f32 + 1.0)
    } else {
        0.0
    }
}

/// Apply sweep attack damage to all nearby entities around the victim.
pub async fn apply_sweep_attack(
    attacker: &Player,
    victim_entity_id: i32,
    victim_pos: &Vector3<f64>,
    base_damage: f64,
    sweeping_edge_level: i32,
    knockback_enchant_level: i32,
    world: &World,
) {
    let attacker_entity = &attacker.living_entity.entity;

    // Sweep damage: 1 + sweeping_edge_ratio * base_damage
    let sweep_ratio = get_sweep_damage_ratio(sweeping_edge_level);
    let sweep_damage = 1.0 + sweep_ratio as f64 * base_damage;

    // Find entities within range (vanilla uses 1.0 block horizontal + 0.25 vertical from victim)
    let nearby = world.get_nearby_entities(*victim_pos, 2.0);

    for entity in nearby.values() {
        let ent = entity.get_entity();

        // Skip the attacker and the original victim
        if ent.entity_id == attacker_entity.entity_id || ent.entity_id == victim_entity_id {
            continue;
        }

        // Only damage living entities
        if entity.get_living_entity().is_none() {
            continue;
        }

        // Check distance more precisely (1 block from victim)
        let entity_pos = ent.pos.load();
        let dx = entity_pos.x - victim_pos.x;
        let dz = entity_pos.z - victim_pos.z;
        let horizontal_dist_sq = dx * dx + dz * dz;
        if horizontal_dist_sq > 9.0 {
            // 3.0^2 - vanilla uses attack range + sweep range
            continue;
        }

        // Vertical check
        let dy = (entity_pos.y - victim_pos.y).abs();
        if dy > 2.0 {
            continue;
        }

        // Apply sweep damage
        entity
            .damage_with_context(
                &**entity,
                sweep_damage as f32,
                DamageType::PLAYER_ATTACK,
                None,
                Some(attacker),
                Some(attacker),
            )
            .await;

        // Apply knockback to swept entities
        let knockback_strength = 0.4 + f64::from(knockback_enchant_level) * 0.5;
        let kb_resist = entity.get_living_entity().map_or(0.0, |le| {
            le.get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE)
        });
        handle_knockback(attacker_entity, ent, knockback_strength, kb_resist);
        ent.send_velocity().await;
    }

    // Spawn sweep particle
    spawn_sweep_particle(attacker_entity, world, victim_pos).await;
}

/// Spawn critical hit particles around the victim.
pub async fn spawn_crit_particles(world: &World, victim_pos: &Vector3<f64>, enchanted: bool) {
    let particle = if enchanted {
        Particle::EnchantedHit
    } else {
        Particle::Crit
    };

    world
        .spawn_particle(*victim_pos, Vector3::new(0.5, 0.5, 0.5), 0.2, 10, particle)
        .await;
}

pub async fn spawn_sweep_particle(attacker_entity: &Entity, world: &World, pos: &Vector3<f64>) {
    let yaw = attacker_entity.yaw.load();
    let d = -f64::from((yaw.to_radians()).sin());
    let e = f64::from((yaw.to_radians()).cos());

    let scale = 0.5;
    let body_y = f64::from(attacker_entity.height()).mul_add(scale, pos.y);

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

/// Get enchantment levels from the player's held item.
///
/// Returns a `CombatEnchantments` struct with all combat-relevant enchantment levels.
pub async fn get_combat_enchantments(player: &Player) -> CombatEnchantments {
    let item = player.inventory().held_item();
    let item_lock = item.lock().await;

    CombatEnchantments {
        sharpness: item_lock.get_enchantment_level(&Enchantment::SHARPNESS),
        smite: item_lock.get_enchantment_level(&Enchantment::SMITE),
        bane_of_arthropods: item_lock.get_enchantment_level(&Enchantment::BANE_OF_ARTHROPODS),
        knockback: item_lock.get_enchantment_level(&Enchantment::KNOCKBACK),
        fire_aspect: item_lock.get_enchantment_level(&Enchantment::FIRE_ASPECT),
        sweeping_edge: item_lock.get_enchantment_level(&Enchantment::SWEEPING_EDGE),
    }
}

/// Combat-related enchantment levels for the held weapon.
pub struct CombatEnchantments {
    pub sharpness: i32,
    pub smite: i32,
    pub bane_of_arthropods: i32,
    pub knockback: i32,
    pub fire_aspect: i32,
    pub sweeping_edge: i32,
}

/// Context for post-damage combat effects.
pub struct PostDamageContext<'a> {
    pub attacker: &'a Player,
    pub victim: &'a dyn EntityBase,
    pub damage: f64,
    pub enchant_damage: f64,
    pub attack_type: AttackType,
    pub enchants: &'a CombatEnchantments,
    pub config_knockback: bool,
}

/// Handle post-damage combat effects: knockback, sweep attack, fire aspect, and particles.
pub async fn handle_post_damage_effects(
    ctx: &PostDamageContext<'_>,
    world: &World,
    pos: &Vector3<f64>,
) {
    let attacker_entity = &ctx.attacker.living_entity.entity;
    let victim_entity = ctx.victim.get_entity();

    // Spawn particles
    if matches!(ctx.attack_type, AttackType::Critical) {
        spawn_crit_particles(world, pos, ctx.enchant_damage > 0.0).await;
    } else if ctx.enchant_damage > 0.0 {
        spawn_crit_particles(world, pos, true).await;
    }

    if ctx.victim.get_living_entity().is_some() {
        let mut knockback_strength = 1.0 + f64::from(ctx.enchants.knockback) * 0.5;

        match ctx.attack_type {
            AttackType::Knockback => knockback_strength += 1.0,
            AttackType::Sweeping => {
                apply_sweep_attack(
                    ctx.attacker,
                    victim_entity.entity_id,
                    pos,
                    ctx.damage,
                    ctx.enchants.sweeping_edge,
                    ctx.enchants.knockback,
                    world,
                )
                .await;
            }
            _ => {}
        }

        if ctx.config_knockback {
            // Armor stands only take knockback from sprint attacks or knockback enchantment
            let is_armor_stand = *victim_entity.entity_type == EntityType::ARMOR_STAND;
            let should_apply = !is_armor_stand
                || matches!(ctx.attack_type, AttackType::Knockback)
                || ctx.enchants.knockback > 0;

            if should_apply {
                let kb_resist = ctx.victim.get_living_entity().map_or(0.0, |le| {
                    le.get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE)
                });
                handle_knockback(
                    attacker_entity,
                    victim_entity,
                    knockback_strength,
                    kb_resist,
                );
                victim_entity.send_velocity().await;
                attacker_entity.send_velocity().await;
            }
        }

        // Sprint knockback attack should reset sprinting
        if matches!(ctx.attack_type, AttackType::Knockback) {
            attacker_entity.set_sprinting(false).await;
        }

        // Fire Aspect: set victim on fire (4 seconds per level)
        if ctx.enchants.fire_aspect > 0 {
            victim_entity.set_on_fire_for(4.0 * ctx.enchants.fire_aspect as f32);
            victim_entity.set_on_fire(true).await;
        }
    }
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
