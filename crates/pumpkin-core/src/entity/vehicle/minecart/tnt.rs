use std::sync::atomic::{AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use rand::RngExt;

use std::sync::Arc;

use crate::entity::{Entity, EntityBase};
use crate::world::{Explosion, ExplosionInteraction};

/// Vanilla `0.01F`: squared horizontal speed that sets off a moving cart.
pub(super) const IGNITE_SPEED_SQUARED: f64 = 0.01f32 as f64;
/// `causeFallDamage`: a fall at least this long sets the cart off.
pub(super) const FALL_EXPLODE_DISTANCE: f64 = 3.0;

/// Vanilla `MinecartTNT.damageSourceIgnitesTnt`: a burning projectile, fire or an explosion.
pub(super) fn damage_source_ignites_tnt(
    damage_type: &DamageType,
    source: Option<&dyn EntityBase>,
) -> bool {
    // Every projectile has an owner slot, the `instanceof Projectile` test.
    source
        .filter(|source| source.get_projectile_owner().is_some())
        .map_or_else(
            || {
                damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_FIRE)
                    || damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_EXPLOSION)
            },
            |projectile| projectile.get_entity().is_on_fire(),
        )
}

// TODO: vanilla `ignitionSource`: the damage source that primed the cart, credited by
// `explode`
pub(super) struct TntMinecart {
    fuse: AtomicI32,
    explosion_power: AtomicCell<f32>,
    explosion_speed_factor: AtomicCell<f32>,
}

impl TntMinecart {
    /// Vanilla `primeFuse` fuse.
    pub(super) const FUSE: i32 = 80;

    pub(super) const fn new() -> Self {
        Self {
            fuse: AtomicI32::new(-1),
            explosion_power: AtomicCell::new(4.0),
            explosion_speed_factor: AtomicCell::new(1.0),
        }
    }

    /// Vanilla `primeFuse`; callers check `isPrimed` first like vanilla.
    pub(super) fn prime_fuse(&self, entity: &Entity) {
        let world = entity.world.load();
        if !world.level_info.load().game_rules.tnt_explodes {
            return;
        }

        self.fuse.store(Self::FUSE, Ordering::Relaxed);
        world.send_entity_status(entity, EntityStatus::TntPrime, ActorEventID::PrimeTNTCart);
        if !entity.silent.load(Ordering::Relaxed) {
            world.play_sound(
                Sound::EntityTntPrimed,
                SoundCategory::Blocks,
                &entity.pos.load(),
            );
        }
    }

    /// Vanilla `MinecartTNT.tick` after `super.tick()`. Smoke is client side.
    pub(super) fn tick(&self, minecart: &dyn EntityBase) {
        let entity = minecart.get_entity();
        let fuse = self.fuse.load(Ordering::Relaxed);
        if fuse > 0 {
            self.fuse.store(fuse - 1, Ordering::Relaxed);
        } else if fuse == 0 {
            self.explode(
                minecart,
                entity.velocity.load().horizontal_length_squared(),
                None,
            );
        }

        if entity.horizontal_collision.load(Ordering::Relaxed) {
            let speed_squared = entity.velocity.load().horizontal_length_squared();
            if speed_squared >= IGNITE_SPEED_SQUARED {
                self.explode(minecart, speed_squared, None);
            }
        }
    }

    pub(super) fn is_primed(&self) -> bool {
        self.fuse.load(Ordering::Relaxed) > -1
    }

    pub(super) fn fuse(&self) -> i32 {
        self.fuse.load(Ordering::Relaxed)
    }

    /// The strongest blast `explode` can make, speed counts up to its cap of 5.
    pub(super) fn max_explosion_power(&self) -> f32 {
        Self::explosion_strength(
            self.explosion_power.load(),
            self.explosion_speed_factor.load(),
            f64::INFINITY,
            1.0,
        )
    }

    /// `cause` is who gets the credit, e.g. the shooter of the burning arrow that set it off.
    pub(super) fn explode(
        &self,
        minecart: &dyn EntityBase,
        horizontal_speed_squared: f64,
        cause: Option<Arc<dyn EntityBase>>,
    ) {
        let entity = minecart.get_entity();
        let world = entity.world.load_full();
        if !world.level_info.load().game_rules.tnt_explodes {
            if self.is_primed() {
                entity.remove();
            }
            return;
        }

        let power = Self::explosion_strength(
            self.explosion_power.load(),
            self.explosion_speed_factor.load(),
            horizontal_speed_squared,
            rand::rng().random::<f64>(),
        );
        let pos = entity.pos.load();
        let primed = self.is_primed();
        entity.remove();
        let mut explosion = Explosion::new(
            power,
            pos,
            world.get_block_interaction(ExplosionInteraction::Tnt),
        )
        .with_source(minecart);
        if cause.is_some() {
            explosion = explosion.with_indirect_source(cause);
        }
        if primed {
            explosion = explosion.preserving_rails();
        }
        world.run_explosion(&explosion);
    }

    pub(super) fn set_fuse(&self, fuse: i32) {
        self.fuse.store(fuse, Ordering::Relaxed);
    }

    pub(super) fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("fuse", self.fuse.load(Ordering::Relaxed));
        let power = self.explosion_power.load();
        if power != 4.0 {
            nbt.put_float("explosion_power", power);
        }
        let speed_factor = self.explosion_speed_factor.load();
        if speed_factor != 1.0 {
            nbt.put_float("explosion_speed_factor", speed_factor);
        }
    }

    pub(super) fn read_nbt(&self, nbt: &NbtCompound) {
        self.fuse
            .store(nbt.get_int("fuse").unwrap_or(-1), Ordering::Relaxed);
        self.explosion_power.store(
            nbt.get_float("explosion_power")
                .unwrap_or(4.0)
                .clamp(0.0, 128.0),
        );
        self.explosion_speed_factor.store(
            nbt.get_float("explosion_speed_factor")
                .unwrap_or(1.0)
                .clamp(0.0, 128.0),
        );
    }

    fn explosion_strength(
        base: f32,
        speed_factor: f32,
        horizontal_speed_squared: f64,
        random: f64,
    ) -> f32 {
        // Vanilla sums in double and casts once.
        let speed = horizontal_speed_squared.sqrt().min(5.0);
        (f64::from(base) + f64::from(speed_factor) * random * 1.5 * speed) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::TntMinecart;

    #[test]
    fn tnt_minecart_explosion_bonus_is_speed_capped() {
        assert_eq!(TntMinecart::explosion_strength(4.0, 1.0, 100.0, 1.0), 11.5);
        assert_eq!(TntMinecart::explosion_strength(4.0, 1.0, 100.0, 0.0), 4.0);
    }
}
