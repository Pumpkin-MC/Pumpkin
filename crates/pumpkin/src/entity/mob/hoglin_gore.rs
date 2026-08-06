use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{EntityBase, mob::hoglin::HoglinEntity};

/// `HoglinBase.hurtAndThrowTarget` damage roll: adults deal `base/2 + rand(0..base)`,
/// babies deal flat `base` damage.
#[must_use]
pub fn gore_damage_roll(base_attack_damage: f32, is_baby: bool, rand_int: i32) -> f32 {
    if !is_baby && base_attack_damage as i32 > 0 {
        base_attack_damage / 2.0 + rand_int as f32
    } else {
        base_attack_damage
    }
}

/// `HoglinBase.throwTarget` knockback vector.
///
/// Takes the attacker->target horizontal delta and pre-rolled randomness. Returns
/// `None` when knockback is fully resisted (`effective_knockback <= 0`), matching
/// vanilla's early-return.
#[must_use]
pub fn gore_knockback_vector(
    attack_knockback: f64,
    target_knockback_resistance: f64,
    dx: f64,
    dz: f64,
    horizontal_angle_radians: f64,
    horizontal_rand01: f64,
    vertical_rand01: f64,
) -> Option<(f64, f64, f64)> {
    let effective = attack_knockback - target_knockback_resistance;
    if effective <= 0.0 {
        return None;
    }

    let len = dx.hypot(dz);
    let (nx, nz) = if len > 0.0 {
        (dx / len, dz / len)
    } else {
        (0.0, 0.0)
    };

    // Vec3::yRot (Vec3.java:241): xx = x*cos + z*sin; zz = z*cos - x*sin.
    let cos = horizontal_angle_radians.cos();
    let sin = horizontal_angle_radians.sin();
    let rx = nx * cos + nz * sin;
    let rz = nz * cos - nx * sin;

    let horizontal_scale = effective * (horizontal_rand01 * 0.5 + 0.2);
    let vertical_scale = effective * vertical_rand01 * 0.5;

    Some((rx * horizontal_scale, vertical_scale, rz * horizontal_scale))
}

/// Vanilla `HoglinBase.hurtAndThrowTarget`/`throwTarget`.
///
/// Replaces the generic `MobEntity::try_attack` damage/knockback path with the
/// randomized-damage, resistance-adjusted-knockback gore attack. Babies use the
/// same roll (which collapses to flat `base_attack_damage`) but never throw
/// (`if (!body.isBaby()) throwTarget(...)`).
pub async fn try_gore_attack(hoglin: &HoglinEntity, target: &dyn EntityBase) -> bool {
    let is_baby = !hoglin.is_adult();
    let living = &hoglin.mob_entity.living_entity;

    let base_attack_damage = living.get_attribute_value(&Attributes::ATTACK_DAMAGE) as f32;
    let rand_int = if base_attack_damage as i32 > 0 {
        rand::rng().random_range(0..base_attack_damage as i32)
    } else {
        0
    };
    let damage = gore_damage_roll(base_attack_damage, is_baby, rand_int);

    let pos = living.entity.pos.load();
    let caller = living
        .entity
        .world
        .load()
        .get_entity_by_id(living.entity.entity_id);

    let damaged = target
        .damage_with_context(
            target,
            damage,
            DamageType::MOB_ATTACK,
            Some(pos),
            caller.as_deref(),
            caller.as_deref(),
        )
        .await;

    if damaged && !is_baby {
        let Some(target_living) = target.get_living_entity() else {
            return damaged;
        };
        let attack_knockback = living.get_attribute_value(&Attributes::ATTACK_KNOCKBACK);
        let resistance = target_living.get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE);
        let target_pos = target.get_entity().pos.load();
        let dx = target_pos.x - pos.x;
        let dz = target_pos.z - pos.z;

        let mut rng = rand::rng();
        // `random.nextInt(21) - 10` (HoglinBase.java:46), fed directly as radians into
        // `Vec3.yRot` (verified against Vec3.java:241 -- vanilla passes the raw
        // -10..=10 value as radians, not degrees).
        let horizontal_angle_radians = f64::from(rng.random_range(-10..=10));
        let horizontal_rand01 = f64::from(rng.random_range(0.0f32..1.0));
        let vertical_rand01 = f64::from(rng.random_range(0.0f32..1.0));

        if let Some((vx, vy, vz)) = gore_knockback_vector(
            attack_knockback,
            resistance,
            dx,
            dz,
            horizontal_angle_radians,
            horizontal_rand01,
            vertical_rand01,
        ) {
            target.get_entity().add_velocity(Vector3::new(vx, vy, vz));
        }
    }

    damaged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baby_damage_is_flat() {
        assert_eq!(gore_damage_roll(6.0, true, 5), 6.0);
    }

    #[test]
    fn adult_damage_adds_half_base_and_roll() {
        assert_eq!(gore_damage_roll(6.0, false, 3), 6.0);
        assert_eq!(gore_damage_roll(6.0, false, 0), 3.0);
    }

    #[test]
    fn zero_base_damage_is_flat_even_for_adults() {
        assert_eq!(gore_damage_roll(0.0, false, 0), 0.0);
    }

    #[test]
    fn knockback_fully_resisted_returns_none() {
        assert_eq!(
            gore_knockback_vector(1.0, 1.0, 1.0, 0.0, 0.0, 0.5, 0.5),
            None
        );
        assert_eq!(
            gore_knockback_vector(1.0, 2.0, 1.0, 0.0, 0.0, 0.5, 0.5),
            None
        );
    }

    #[test]
    fn knockback_pushes_away_from_attacker_with_no_rotation() {
        let (vx, vy, vz) =
            gore_knockback_vector(1.0, 0.0, 1.0, 0.0, 0.0, 0.5, 0.4).expect("not resisted");
        assert!(vx > 0.0);
        assert_eq!(vz, 0.0);
        assert!(vy > 0.0);
    }

    #[test]
    fn knockback_scale_matches_vanilla_formula() {
        // effective=2.0, horizontal_rand=0.6 -> scale = 2.0 * (0.6*0.5+0.2) = 1.0
        let (vx, _, _) =
            gore_knockback_vector(3.0, 1.0, 1.0, 0.0, 0.0, 0.6, 0.0).expect("not resisted");
        assert!((vx - 1.0).abs() < 1e-9);
    }

    #[test]
    fn zero_delta_yields_zero_horizontal_push() {
        let (vx, _, vz) =
            gore_knockback_vector(2.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5).expect("not resisted");
        assert_eq!(vx, 0.0);
        assert_eq!(vz, 0.0);
    }
}
