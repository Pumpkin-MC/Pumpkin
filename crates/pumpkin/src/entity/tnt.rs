use super::{Entity, EntityBase, living::LivingEntity};
use crate::server::Server;
use crate::world::World;
use core::f32;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::{
    f64::consts::TAU,
    sync::{
        Arc,
        atomic::{
            AtomicU32,
            Ordering::{self, Relaxed},
        },
    },
};

/// Vanilla `Entity.getAirDrag`, applied every tick.
const AIR_DRAG: f32 = 0.98;

pub struct TNTEntity {
    entity: Entity,
    power: AtomicCell<f32>,
    fuse: AtomicU32,
}

impl TNTEntity {
    /// Vanilla `PrimedTnt.DEFAULT_FUSE_TIME`.
    pub const DEFAULT_FUSE: u32 = 80;
    /// Vanilla `PrimedTnt.DEFAULT_EXPLOSION_POWER`.
    pub const DEFAULT_POWER: f32 = 4.0;

    pub const fn new(entity: Entity, power: f32, fuse: u32) -> Self {
        Self {
            entity,
            power: AtomicCell::new(power),
            fuse: AtomicU32::new(fuse),
        }
    }

    /// Vanilla primed-TNT constructor: block centre plus the random launch impulse.
    pub fn primed(world: &Arc<World>, position: &BlockPos, fuse: u32) -> Arc<Self> {
        let pos = Vector3::new(
            f64::from(position.0.x) + 0.5,
            f64::from(position.0.y),
            f64::from(position.0.z) + 0.5,
        );
        let entity = Entity::new(world.clone(), pos, &EntityType::TNT);
        let rot = rand::random::<f64>() * TAU;
        entity
            .velocity
            .store(Vector3::new(-rot.sin() * 0.02, 0.2, -rot.cos() * 0.02));
        Arc::new(Self::new(entity, Self::DEFAULT_POWER, fuse))
    }

    /// Vanilla `PrimedTnt.getRandomShortFuse`.
    #[must_use]
    pub fn random_short_fuse(fuse: u32) -> u32 {
        rand::random_range(0..(fuse / 4).max(1)) + fuse / 8
    }
}

impl EntityBase for TNTEntity {
    /// Vanilla `PrimedTnt.addAdditionalSaveData`. Without it a chunk reload resets a
    /// nearly-detonated fuse and the TNT explodes late.
    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_short("fuse", self.fuse.load(Relaxed) as i16);
        let power = self.power.load();
        if (power - Self::DEFAULT_POWER).abs() > f32::EPSILON {
            nbt.put_float("explosion_power", power);
        }
    }

    /// Vanilla `PrimedTnt.readAdditionalSaveData`.
    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        let fuse = nbt
            .get_short("fuse")
            .map_or(Self::DEFAULT_FUSE, |fuse| fuse.max(0) as u32);
        self.fuse.store(fuse, Relaxed);
        self.power.store(
            nbt.get_float("explosion_power")
                .unwrap_or(Self::DEFAULT_POWER)
                .clamp(0.0, 128.0),
        );
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = &self.entity;

        let mut velo = entity.velocity.load();
        velo.y -= self.get_gravity();

        entity.move_entity(caller, velo);
        entity.tick_block_collisions(caller);

        // Read back what actually happened instead of reusing the pre-move value:
        // `move_entity` clamps on collision, and an explosion may have pushed us while we
        // were moving above. Air drag applies every tick, the ground factors on top of it.
        let drag = f64::from(AIR_DRAG);
        let velo = entity.velocity.load().multiply(drag, drag, drag);
        let velo = if entity.on_ground.load(Ordering::Relaxed) {
            velo.multiply(0.7, -0.5, 0.7)
        } else {
            velo
        };
        entity.velocity.store(velo);

        // Vanilla `setFuse` every tick; the tracker picks it up through `is_dirty`.
        let fuse = self.fuse.load(Relaxed).saturating_sub(1);
        self.fuse.store(fuse, Relaxed);
        entity.set_synced_data(
            pumpkin_data::tracked_data::tnt::FUSE_ID,
            VarInt(fuse as i32),
        );

        if fuse == 0 {
            entity.remove();
            let world = entity.world.load_full();
            if world.level_info.load().game_rules.tnt_explodes {
                // Vanilla `PrimedTnt.explode`: `getY(0.0625)`.
                let pos = entity.pos.load();
                let y = f64::from(entity.entity_type.dimension[1]).mul_add(0.0625, pos.y);
                world.explode(
                    Vector3::new(pos.x, y, pos.z),
                    self.power.load(),
                    crate::world::ExplosionInteraction::Tnt,
                );
            }
        } else {
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

    // TODO: Bedrock lacks fuse metadata, ignited flag, prime sound and particles: no blink.
    fn bedrock_y_offset(&self) -> f64 {
        0.49
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
