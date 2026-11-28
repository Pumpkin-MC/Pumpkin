//! Velocity exchanged by entity pushes: vanilla `Entity.push(Entity)`, `AbstractMinecart.push`
//! and `pushOtherMinecart`.

use std::sync::atomic::Ordering;

use pumpkin_util::math::vector3::Vector3;

use crate::entity::Entity;

/// `Entity.push(Entity)` push factor.
pub(crate) const PUSH_STRENGTH: f64 = 0.05f32 as f64;
/// `AbstractMinecart.push`: `0.1F * 0.5`, applied in two steps.
const MINECART_PUSH_STRENGTH: f64 = 0.1f32 as f64;
const MINECART_PUSH_SCALE: f64 = 0.5;
/// `pushOtherMinecart`: carts only exchange velocity when hit head-on.
const MINECART_ALIGNMENT: f64 = 0.8f32 as f64;
/// `pushOtherMinecart`: horizontal velocity kept by a cart before the exchange.
const MINECART_KEEP: f64 = 0.2;
/// `pushOtherMinecart`: horizontal velocity kept by a furnace cart hitting a plain one.
const FURNACE_KEEP: f64 = 0.95;

/// Position and velocity of a push partner, loaded once per push.
#[derive(Clone, Copy)]
pub struct Motion {
    pub pos: Vector3<f64>,
    pub velocity: Vector3<f64>,
}

impl Motion {
    #[must_use]
    pub fn of(entity: &Entity) -> Self {
        Self {
            pos: entity.pos.load(),
            velocity: entity.velocity.load(),
        }
    }
}

/// Which push rules a cart follows in `pushOtherMinecart`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CartClass {
    Plain,
    Furnace,
}

/// When `pushOtherMinecart` exchanges velocity.
#[derive(Clone, Copy)]
pub enum ExchangeRule {
    /// Old movement: only with a cart in front of or behind `this`, by position.
    HeadOn { this_yaw: f32 },
    /// Experimental movement: always.
    Always,
}

/// New velocity of one cart. `Push` marks it for resync like vanilla `Entity.push(Vec3)`.
#[derive(Clone, Copy)]
pub enum CartVelocity {
    Set(Vector3<f64>),
    Push(Vector3<f64>),
}

impl CartVelocity {
    pub fn apply(self, entity: &Entity) {
        match self {
            Self::Set(velocity) => entity.velocity.store(velocity),
            Self::Push(velocity) => {
                entity.velocity.store(velocity);
                entity.velocity_dirty.store(true, Ordering::SeqCst);
            }
        }
    }
}

/// Vanilla `Entity.push(Entity)`: impulse on `other`, its negation goes on `this`.
#[must_use]
pub fn entity_push(this: &Motion, other: &Motion) -> Option<Vector3<f64>> {
    let mut dx = other.pos.x - this.pos.x;
    let mut dz = other.pos.z - this.pos.z;
    let distance = dx.abs().max(dz.abs());
    if distance < 0.01 {
        return None;
    }
    let distance = distance.sqrt();
    dx /= distance;
    dz /= distance;
    let falloff = (1.0 / distance).min(1.0);
    Some(Vector3::new(
        dx * falloff * PUSH_STRENGTH,
        0.0,
        dz * falloff * PUSH_STRENGTH,
    ))
}

/// Vanilla `AbstractMinecart.push`: offset towards `other`, `None` when they overlap.
#[must_use]
pub fn minecart_push(this: &Motion, other: &Motion) -> Option<Vector3<f64>> {
    let mut xa = other.pos.x - this.pos.x;
    let mut za = other.pos.z - this.pos.z;
    let distance_squared = xa * xa + za * za;
    if distance_squared < f64::from(1.0e-4f32) {
        return None;
    }
    let distance = distance_squared.sqrt();
    xa /= distance;
    za /= distance;
    let falloff = (1.0 / distance).min(1.0);
    xa *= falloff;
    za *= falloff;
    xa *= MINECART_PUSH_STRENGTH;
    za *= MINECART_PUSH_STRENGTH;
    Some(Vector3::new(
        xa * MINECART_PUSH_SCALE,
        0.0,
        za * MINECART_PUSH_SCALE,
    ))
}

/// Vanilla `pushOtherMinecart`: new velocities of `this` and `other`, `None` when the rule
/// keeps them apart.
#[must_use]
pub fn minecart_exchange(
    this: (&Motion, CartClass),
    other: (&Motion, CartClass),
    rule: ExchangeRule,
    push: Vector3<f64>,
) -> Option<(CartVelocity, CartVelocity)> {
    let ((this, this_class), (other, other_class)) = (this, other);
    if let ExchangeRule::HeadOn { this_yaw } = rule {
        let direction =
            Vector3::new(other.pos.x - this.pos.x, 0.0, other.pos.z - this.pos.z).normalize();
        let yaw = this_yaw * (std::f32::consts::PI / 180.0);
        let facing = Vector3::new(
            f64::from(pumpkin_util::math::cos(yaw)),
            0.0,
            f64::from(pumpkin_util::math::sin(yaw)),
        )
        .normalize();
        if direction.dot(&facing).abs() < MINECART_ALIGNMENT {
            return None;
        }
    }

    let keep = |velocity: Vector3<f64>, factor| velocity.multiply(factor, 1.0, factor);
    let pushed = |velocity: Vector3<f64>, x, z| {
        CartVelocity::Push(keep(velocity, MINECART_KEEP).add_raw(x, 0.0, z))
    };
    let (mine, theirs) = (this.velocity, other.velocity);
    Some(match (this_class, other_class) {
        (CartClass::Plain, CartClass::Furnace) => (
            pushed(mine, theirs.x - push.x, theirs.z - push.z),
            CartVelocity::Set(keep(theirs, FURNACE_KEEP)),
        ),
        (CartClass::Furnace, CartClass::Plain) => (
            CartVelocity::Set(keep(mine, FURNACE_KEEP)),
            pushed(theirs, mine.x + push.x, mine.z + push.z),
        ),
        _ => {
            #[expect(clippy::manual_midpoint, reason = "vanilla rounding")]
            let (x, z) = ((theirs.x + mine.x) / 2.0, (theirs.z + mine.z) / 2.0);
            (
                pushed(mine, x - push.x, z - push.z),
                pushed(theirs, x + push.x, z + push.z),
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carts_align_by_position_not_velocity() {
        let still = Vector3::new(0.0, 0.0, 0.0);
        let this = Motion {
            pos: Vector3::new(0.0, 64.0, 0.0),
            velocity: Vector3::new(0.3, 0.0, 0.0),
        };
        let beside = Motion {
            pos: Vector3::new(0.0, 64.0, 0.5),
            velocity: still,
        };
        let ahead = Motion {
            pos: Vector3::new(0.5, 64.0, 0.0),
            velocity: still,
        };
        // Yaw 0 faces +X: moving along it does not matter, the partner's side does.
        let exchange = |other: &Motion| {
            minecart_push(&this, other).and_then(|push| {
                minecart_exchange(
                    (&this, CartClass::Plain),
                    (other, CartClass::Plain),
                    ExchangeRule::HeadOn { this_yaw: 0.0 },
                    push,
                )
            })
        };
        assert!(exchange(&beside).is_none());
        assert!(exchange(&ahead).is_some());
    }

    #[test]
    fn furnace_keeps_its_speed_and_the_plain_cart_is_pushed() {
        let plain = Motion {
            pos: Vector3::new(0.0, 64.0, 0.0),
            velocity: Vector3::new(0.1, 0.0, 0.0),
        };
        let furnace = Motion {
            pos: Vector3::new(0.5, 64.0, 0.0),
            velocity: Vector3::new(0.3, 0.0, 0.0),
        };
        let exchange = |this: (&Motion, CartClass), other: (&Motion, CartClass)| {
            let push = minecart_push(this.0, other.0).unwrap();
            minecart_exchange(this, other, ExchangeRule::Always, push).unwrap()
        };
        // 0.1 * 0.2 + 0.3 - 0.05 (the push) for the plain cart, 0.3 * 0.95 for the furnace.
        let resync_and_x = |result: CartVelocity| match result {
            CartVelocity::Set(velocity) => (false, velocity.x),
            CartVelocity::Push(velocity) => (true, velocity.x),
        };
        let assert_outcome = |plain_result: CartVelocity, furnace_result: CartVelocity| {
            let (plain_resync, plain_x) = resync_and_x(plain_result);
            let (furnace_resync, furnace_x) = resync_and_x(furnace_result);
            assert!(plain_resync && !furnace_resync);
            assert!((plain_x - 0.27).abs() < 1e-8, "plain x {plain_x}");
            assert!((furnace_x - 0.285).abs() < 1e-12, "furnace x {furnace_x}");
        };

        let (this, other) = exchange((&plain, CartClass::Plain), (&furnace, CartClass::Furnace));
        assert_outcome(this, other);
        let (this, other) = exchange((&furnace, CartClass::Furnace), (&plain, CartClass::Plain));
        assert_outcome(other, this);
    }
}
