use super::{Entity, EntityBase, living::LivingEntity};
use crate::server::Server;
use core::f32;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::Block;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::vector3::Vector3;
use std::{
    f64::consts::TAU,
    sync::atomic::{
        AtomicU32,
        Ordering::{self, Relaxed},
    },
};

/// Vanilla `EntityTypes.TNT`'s `updateInterval(10)`: how often the entity tracker resyncs the
/// client's own simulation of this entity. See [`Entity::send_tracked_position`].
const UPDATE_INTERVAL: i32 = 10;

/// Vanilla `PrimedTnt.explosionPower`'s default, and the value `readAdditionalSaveData` falls
/// back to. Only a deviating power is written, matching vanilla.
const DEFAULT_EXPLOSION_POWER: f32 = 4.0;

/// Vanilla `PrimedTnt`'s default fuse, and `readAdditionalSaveData`'s fallback.
const DEFAULT_FUSE: u32 = 80;

pub struct TNTEntity {
    entity: Entity,
    power: AtomicCell<f32>,
    fuse: AtomicU32,
}

impl TNTEntity {
    pub const fn new(entity: Entity, power: f32, fuse: u32) -> Self {
        Self {
            entity,
            power: AtomicCell::new(power),
            fuse: AtomicU32::new(fuse),
        }
    }

    /// Vanilla primed-TNT ctor impulse. Not in `init_data_tracker`: that also runs on
    /// chunk load, where a reconstructed entity already has its saved `Motion` and should
    /// not get a fresh random launch.
    pub fn apply_prime_impulse(&self) {
        let yaw: f64 = rand::random::<f64>() * TAU;
        self.entity
            .set_velocity(Vector3::new(-yaw.sin() * 0.02, 0.2, -yaw.cos() * 0.02));
    }
}

impl EntityBase for TNTEntity {
    /// Vanilla `PrimedTnt.addAdditionalSaveData`. Without this the fuse is lost whenever the
    /// entity is serialized, so a chunk reload or relog resets a nearly-spent fuse and the
    /// TNT explodes late.
    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_short("fuse", self.fuse.load(Relaxed) as i16);
        let power = self.power.load();
        if (power - DEFAULT_EXPLOSION_POWER).abs() > f32::EPSILON {
            nbt.put_float("explosion_power", power);
        }
    }

    /// Vanilla `PrimedTnt.readAdditionalSaveData`; `explosion_power` is clamped the same way.
    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        let fuse = nbt
            .get_short("fuse")
            .map_or(DEFAULT_FUSE, |fuse| fuse.max(0) as u32);
        self.fuse.store(fuse, Relaxed);
        self.power.store(
            nbt.get_float("explosion_power")
                .unwrap_or(DEFAULT_EXPLOSION_POWER)
                .clamp(0.0, 128.0),
        );
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = &self.entity;

        let mut velo = entity.velocity.load();
        velo.y -= self.get_gravity();

        entity.move_entity(caller, velo);
        entity.tick_block_collisions(caller);

        // Read back what actually happened instead of reusing the pre-move
        // value: `move_entity` clamps on collision, and an explosion may have
        // pushed us while we were moving above.
        // Vanilla scales by air drag (0.98) unconditionally every tick, then multiplies by
        // (0.7, -0.5, 0.7) on top of that when on the ground -- the two are cumulative, not
        // an either/or choice.
        let velo = entity.velocity.load().multiply(0.98, 0.98, 0.98);
        let velo = if entity.on_ground.load(Ordering::Relaxed) {
            velo.multiply(0.7, -0.5, 0.7)
        } else {
            velo
        };
        entity.velocity.store(velo);

        // Vanilla's `PrimedTnt.tick()` calls `setFuse()` unconditionally every tick, which
        // marks its `SynchedEntityData` dirty every tick, and `ServerEntity.sendChanges`
        // ORs that dirty flag into the same gate that guards position/velocity resync. TNT
        // therefore tracks almost every tick via the fuse, not on the 10-tick interval.
        // Push the metadata first so `entity_data_dirty` is set before the resync check
        // below observes it. `saturating_sub` keeps the fuse from underflowing at 0.
        let fuse = self.fuse.load(Relaxed).saturating_sub(1);
        entity.set_synced_data(
            pumpkin_data::tracked_data::tnt::FUSE_ID,
            VarInt(fuse as i32),
        );

        if entity.velocity_dirty.swap(false, Ordering::SeqCst) {
            entity.send_pos_rot();
            entity.send_velocity();
        } else {
            entity.send_tracked_position(UPDATE_INTERVAL);
        }

        if fuse == 0 {
            // TNT explodes now
            self.entity.remove();
            let world = self.entity.world.load_full();
            if world.level_info.load().game_rules.tnt_explodes {
                // Vanilla `PrimedTnt.explode`: `getY(0.0625)`.
                let pos = self.entity.pos.load();
                let explode_y = pos.y + f64::from(self.entity.entity_type.dimension[1]) * 0.0625;
                world.explode(
                    Vector3::new(pos.x, explode_y, pos.z),
                    self.power.load(),
                    crate::world::ExplosionInteraction::Tnt,
                );
            }
        } else {
            self.fuse.store(fuse, Relaxed);
            entity.update_fluid_state(caller);
        }
    }

    fn init_data_tracker(&self) {
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::tnt::FUSE_ID,
            VarInt(self.fuse.load(Relaxed) as i32),
        );
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::tnt::BLOCK_STATE_ID,
            VarInt(i32::from(Block::TNT.default_state.id.as_u16())),
        );
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn get_gravity(&self) -> f64 {
        0.04
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
