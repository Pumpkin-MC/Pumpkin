use crate::entity::item::ItemEntity;
use crate::entity::living::LivingEntity;
use crate::entity::player::Player;
use crate::{entity::EntityBaseFuture, server::Server};
use core::f64;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{damage::DamageType, meta_data_type::MetaDataType, tracked_data::TrackedData};
use pumpkin_protocol::{
    codec::item_stack_seralizer::ItemStackSerializer, java::client::play::Metadata,
};
use pumpkin_util::math::vector3::Vector3;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use tokio::sync::Mutex;

use super::{Entity, EntityBase, NBTStorage};

/// Maximum horizontal distance the eye will travel before signalling at an elevated height.
const TOO_FAR_DISTANCE: f64 = 12.0;

/// Height offset used when the target is beyond [`TOO_FAR_DISTANCE`].
const TOO_FAR_SIGNAL_HEIGHT: f64 = 8.0;

/// The eye despawns after this many ticks.
const MAX_LIFE: u32 = 80;

/// Chance (1-in-5) the eye drops its item instead of playing the break effect.
const SURVIVE_CHANCE: u32 = 5;

pub struct EyeOfEnder {
    entity: Entity,
    item_stack: Mutex<ItemStack>,
    target: Mutex<Option<Vector3<f64>>>,
    life: AtomicU32,
    survive_after_death: AtomicBool,
}

impl EyeOfEnder {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            item_stack: Mutex::new(ItemStack::new(1, &Item::ENDER_EYE)),
            target: Mutex::new(None),
            life: AtomicU32::new(0),
            survive_after_death: AtomicBool::new(false),
        }
    }

    /// Aim the eye at `target`, clamping to [`TOO_FAR_DISTANCE`] if necessary,
    /// and randomly decide whether it should drop its item on expiry.
    pub async fn signal_to(&self, target: Vector3<f64>) {
        let pos = self.entity.pos.load();
        let delta = target.sub(&pos);
        let horizontal_dist = delta.x.hypot(delta.z);

        let clamped_target = if horizontal_dist > TOO_FAR_DISTANCE {
            Vector3::new(
                pos.x + delta.x / horizontal_dist * TOO_FAR_DISTANCE,
                pos.y + TOO_FAR_SIGNAL_HEIGHT,
                pos.z + delta.z / horizontal_dist * TOO_FAR_DISTANCE,
            )
        } else {
            target
        };

        *self.target.lock().await = Some(clamped_target);
        self.life.store(0, Ordering::Relaxed);

        // 4-in-5 chance to survive (drop item); 1-in-5 plays the break effect.
        let survive = !rand::random::<u32>().is_multiple_of(SURVIVE_CHANCE);
        self.survive_after_death.store(survive, Ordering::Relaxed);
    }

    /// Compute the next delta-movement so the eye smoothly homes toward `target`.
    ///
    /// Mirrors the vanilla `updateDeltaMovement` logic.
    fn compute_new_velocity(
        old_velo: Vector3<f64>,
        position: Vector3<f64>,
        target: Vector3<f64>,
    ) -> Vector3<f64> {
        let horizontal_delta = Vector3::new(target.x - position.x, 0.0, target.z - position.z);
        let horizontal_len = horizontal_delta.x.hypot(horizontal_delta.z);

        let old_h_speed = old_velo.x.hypot(old_velo.z);
        let mut wanted_speed = lerp(0.0025, old_h_speed, horizontal_len);
        let mut move_y = old_velo.y;

        if horizontal_len < 1.0 {
            wanted_speed *= 0.8;
            move_y *= 0.8;
        }

        let wanted_y_dir = if position.y - old_velo.y < target.y {
            1.0f64
        } else {
            -1.0f64
        };

        let scale = wanted_speed / horizontal_len;
        Vector3::new(
            horizontal_delta.x * scale,
            move_y + (wanted_y_dir - move_y) * 0.015,
            horizontal_delta.z * scale,
        )
    }
}

/// Linear interpolation helper (`start + t * (end - start)`).
#[inline]
fn lerp(t: f64, start: f64, end: f64) -> f64 {
    start + t * (end - start)
}

impl NBTStorage for EyeOfEnder {}

impl EntityBase for EyeOfEnder {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.entity;
            entity.tick(caller, server).await;

            // Advance position by current velocity.
            let velocity = entity.velocity.load();
            let new_pos = entity.pos.load().add(&velocity);

            // Server-side: steer toward target if one is set.
            {
                let target_guard = self.target.lock().await;
                if let Some(target) = *target_guard {
                    let new_velo = Self::compute_new_velocity(velocity, new_pos, target);
                    entity.velocity.store(new_velo);
                }
            }

            entity.set_pos(new_pos);
            entity.send_pos_rot().await;

            // Tick lifetime and handle expiry.
            let life = self.life.fetch_add(1, Ordering::Relaxed) + 1;
            if life > MAX_LIFE {
                entity.play_sound(Sound::EntityEnderEyeDeath).await;
                entity.remove().await;

                if self.survive_after_death.load(Ordering::Relaxed) {
                    // Drop the item at the current position.
                    let item_stack = self.item_stack.lock().await.clone();
                    let world = entity.world.load();
                    let entity = Entity::new(world.clone(), new_pos, &EntityType::ITEM);
                    let item_entity = Arc::new(ItemEntity::new(entity, item_stack).await);
                    world.spawn_entity(item_entity).await;
                } else {
                    entity
                        .world
                        .load()
                        .sync_world_event(
                            WorldEvent::ParticlesEyeOfEnderDeath,
                            new_pos.to_block_pos(),
                            0,
                        )
                        .await;
                }
            }
        })
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {
            self.entity
                .send_meta_data(&[Metadata::new(
                    TrackedData::ITEM,
                    MetaDataType::ITEM_STACK,
                    &ItemStackSerializer::from(self.item_stack.lock().await.clone()),
                )])
                .await;
        })
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        // Eye of Ender is not attackable.
        Box::pin(async { false })
    }

    fn on_player_collision<'a>(&'a self, _player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        // Eye of Ender cannot be picked up.
        Box::pin(async {})
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn get_item_entity(self: Arc<Self>) -> Option<Arc<ItemEntity>> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
