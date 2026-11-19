use std::sync::atomic::{AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use std::sync::Arc;

use crate::entity::{Entity, EntityBase};
use crate::world::{Explosion, ExplosionInteraction};

pub(super) struct TntMinecart {
    fuse: AtomicI32,
    explosion_power: AtomicCell<f32>,
    explosion_speed_factor: AtomicCell<f32>,
}

impl TntMinecart {
    pub(super) const fn new() -> Self {
        Self {
            fuse: AtomicI32::new(-1),
            explosion_power: AtomicCell::new(4.0),
            explosion_speed_factor: AtomicCell::new(1.0),
        }
    }

    pub(super) fn prime(&self, entity: &Entity, fuse: i32) {
        if self.fuse.load(Ordering::Relaxed) >= 0
            || !entity
                .world
                .load()
                .level_info
                .load()
                .game_rules
                .tnt_explodes
        {
            return;
        }

        self.fuse.store(fuse, Ordering::Relaxed);
        let world = entity.world.load();
        world.send_entity_status(entity, EntityStatus::TntPrime, ActorEventID::PrimeTNTCart);
        world.play_sound(
            Sound::EntityTntPrimed,
            SoundCategory::Blocks,
            &entity.pos.load(),
        );
    }

    pub(super) fn tick(&self, minecart: &dyn EntityBase) -> bool {
        let entity = minecart.get_entity();
        let fuse = self.fuse.load(Ordering::Relaxed);
        if fuse > 0 {
            self.fuse.store(fuse - 1, Ordering::Relaxed);
            let mut smoke_pos = entity.pos.load();
            smoke_pos.y += 0.5;
            entity.world.load().spawn_particle(
                smoke_pos,
                Vector3::new(0.0, 0.0, 0.0),
                0.0,
                1,
                Particle::Smoke,
            );
        } else if fuse == 0 {
            let velocity = entity.velocity.load();
            self.explode(
                minecart,
                velocity.x.mul_add(velocity.x, velocity.z * velocity.z),
                None,
            );
            return true;
        }
        false
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
            if self.fuse.load(Ordering::Relaxed) > -1 {
                entity.remove();
            }
            return;
        }

        let power = Self::explosion_strength(
            self.explosion_power.load(),
            self.explosion_speed_factor.load(),
            horizontal_speed_squared,
            rand::rng().random_range(0.0..1.0),
        );
        let pos = entity.pos.load();
        let primed = self.fuse.load(Ordering::Relaxed) > -1;
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
        random: f32,
    ) -> f32 {
        let speed = horizontal_speed_squared.sqrt().min(5.0) as f32;
        base + speed_factor * random * 1.5 * speed
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
