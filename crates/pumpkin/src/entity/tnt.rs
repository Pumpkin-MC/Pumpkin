use super::{Entity, EntityBase, living::LivingEntity};
use crate::{entity::EntityBaseFuture, server::Server};
use core::f32;
use pumpkin_data::Block;
use pumpkin_protocol::{codec::var_int::VarInt, java::client::play::Metadata};
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

/// Vanilla `EntityTypes.TNT`'s `updateInterval(10)`: how often the entity tracker resyncs the
/// client's own simulation of this entity. See [`Entity::send_tracked_position`].
const UPDATE_INTERVAL: i32 = 10;

pub struct TNTEntity {
    entity: Entity,
    power: f32,
    fuse: AtomicU32,
}

impl TNTEntity {
    pub const fn new(entity: Entity, power: f32, fuse: u32) -> Self {
        Self {
            entity,
            power,
            fuse: AtomicU32::new(fuse),
        }
    }
}

impl EntityBase for TNTEntity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.entity;

            let mut velo = entity.velocity.load();
            velo.y -= self.get_gravity();

            entity.move_entity(caller, velo).await;
            entity.tick_block_collisions(caller, server).await;

            // Vanilla scales by air drag (0.98) unconditionally every tick, then multiplies by
            // (0.7, -0.5, 0.7) on top of that when on the ground. The two are cumulative, not
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
            // below observes it.
            let fuse = self.fuse.load(Relaxed).saturating_sub(1);
            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::tnt::FUSE_ID,
                    VarInt(fuse as i32),
                )],
                None,
            );

            if entity.velocity_dirty.swap(false, Ordering::SeqCst) {
                entity.send_pos_rot();
                entity.send_velocity();
            } else {
                entity.send_tracked_position(UPDATE_INTERVAL);
            }

            if fuse == 0 {
                // TNT explodes now
                self.entity.remove().await;
                let world = self.entity.world.load();
                if world.level_info.load().game_rules.tnt_explodes {
                    world
                        .explode(
                            self.entity.pos.load(),
                            self.power,
                            crate::world::ExplosionInteraction::Tnt,
                        )
                        .await;
                }
            } else {
                self.fuse.store(fuse, Relaxed);
                entity.update_fluid_state(caller).await;
            }
        })
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {
            let pos: f64 = rand::random::<f64>() * TAU;

            self.entity
                .set_velocity(Vector3::new(-pos.sin() * 0.02, 0.2, -pos.cos() * 0.02));

            self.entity.send_meta_data(
                &[
                    Metadata::new(
                        pumpkin_data::tracked_data::tnt::FUSE_ID,
                        VarInt(self.fuse.load(Relaxed) as i32),
                    ),
                    Metadata::new(
                        pumpkin_data::tracked_data::tnt::BLOCK_STATE_ID,
                        VarInt(i32::from(Block::TNT.default_state.id.as_u16())),
                    ),
                ],
                None,
            );
        })
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
