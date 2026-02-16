use crate::{entity::EntityBaseFuture, server::Server};
use core::f32;
use pumpkin_data::{damage::DamageType, meta_data_type::MetaDataType, tracked_data::TrackedData};
use pumpkin_protocol::{
    codec::item_stack_seralizer::ItemStackSerializer,
    java::client::play::{CTakeItemEntity, Metadata},
};
use pumpkin_util::math::atomic_f32::AtomicF32;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;
use std::sync::atomic::Ordering::{AcqRel, Relaxed};
use std::sync::{
    Arc,
    atomic::{
        AtomicBool, AtomicU8, AtomicU32,
        Ordering::{self},
    },
};
use tokio::sync::Mutex;

use super::{Entity, EntityBase, NBTStorage, living::LivingEntity, player::Player};

pub struct ItemEntity {
    entity: Entity,
    item_age: AtomicU32,
    // These cannot be atomic values because we mutate their state based on what they are; we run
    // into the ABA problem
    item_stack: Mutex<ItemStack>,
    pickup_delay: AtomicU8,
    health: AtomicF32,
    never_despawn: AtomicBool,
    never_pickup: AtomicBool,
}

impl ItemEntity {
    pub async fn new(entity: Entity, item_stack: ItemStack) -> Self {
        entity
            .set_velocity(Vector3::new(
                rand::random::<f64>().mul_add(0.2, -0.1),
                0.2,
                rand::random::<f64>().mul_add(0.2, -0.1),
            ))
            .await;
        entity.yaw.store(rand::random::<f32>() * 360.0);
        Self {
            entity,
            item_stack: Mutex::new(item_stack),
            item_age: AtomicU32::new(0),
            pickup_delay: AtomicU8::new(10), // Vanilla pickup delay is 10 ticks
            health: AtomicF32::new(5.0),
            never_despawn: AtomicBool::new(false),
            never_pickup: AtomicBool::new(false),
        }
    }

    pub async fn new_with_velocity(
        entity: Entity,
        item_stack: ItemStack,
        velocity: Vector3<f64>,
        pickup_delay: u8,
    ) -> Self {
        entity.set_velocity(velocity).await;
        entity.yaw.store(rand::random::<f32>() * 360.0);
        Self {
            entity,
            item_stack: Mutex::new(item_stack),
            item_age: AtomicU32::new(0),
            pickup_delay: AtomicU8::new(pickup_delay), // Vanilla pickup delay is 10 ticks
            health: AtomicF32::new(5.0),
            never_despawn: AtomicBool::new(false),
            never_pickup: AtomicBool::new(false),
        }
    }

    async fn can_merge(&self) -> bool {
        if self.never_pickup.load(Ordering::Relaxed) || self.entity.removed.load(Ordering::Relaxed)
        {
            return false;
        }

        let item_stack = self.item_stack.lock().await;

        item_stack.item_count < item_stack.get_max_stack_size()
    }

    async fn try_merge(&self) {
        let bounding_box = self.entity.bounding_box.load().expand(0.5, 0.0, 0.5);

        let world = self.entity.world.load();
        let entities = world.entities.load();
        let items = entities.iter().filter_map(|entity: &Arc<dyn EntityBase>| {
            entity.clone().get_item_entity().filter(|item| {
                item.entity.entity_id != self.entity.entity_id
                    && !item.never_despawn.load(Ordering::Relaxed)
                    && item.entity.bounding_box.load().intersects(&bounding_box)
            })
        });

        for item in items {
            if item.can_merge().await {
                self.try_merge_with(&item).await;

                if self.entity.removed.load(Ordering::SeqCst) {
                    break;
                }
            }
        }
    }

    async fn try_merge_with(&self, other: &Self) {
        let self_stack = self.item_stack.lock().await;

        let other_stack = other.item_stack.lock().await;

        if !self_stack.are_equal(&other_stack)
            || self_stack.item_count + other_stack.item_count > self_stack.get_max_stack_size()
        {
            return;
        }

        let (target, mut stack1, source, mut stack2) =
            if other_stack.item_count < self_stack.item_count {
                (self, self_stack, other, other_stack)
            } else {
                (other, other_stack, self, self_stack)
            };

        // Vanilla code adds a .min(64). Not needed with Vanilla item data

        let max_size = stack1.get_max_stack_size();

        let j = stack2.item_count.min(max_size - stack1.item_count);

        stack1.increment(j);

        stack2.decrement(j);

        let empty1 = stack1.item_count == 0;

        let empty2 = stack2.item_count == 0;

        drop(stack1);

        drop(stack2);

        let never_despawn = source.never_despawn.load(Ordering::Relaxed);

        target.never_despawn.store(never_despawn, Ordering::Relaxed);

        if !never_despawn {
            let age = target
                .item_age
                .load(Ordering::Relaxed)
                .min(source.item_age.load(Ordering::Relaxed));

            target.item_age.store(age, Ordering::Relaxed);
        }

        let never_pickup = source.never_pickup.load(Ordering::Relaxed);

        target.never_pickup.store(never_pickup, Ordering::Relaxed);

        if !never_pickup {
            let source_delay = source.pickup_delay.load(Ordering::Relaxed);
            target
                .pickup_delay
                .fetch_max(source_delay, Ordering::Relaxed);
        }

        if empty1 {
            target.entity.remove().await;
        } else {
            target.init_data_tracker().await;
        }

        if empty2 {
            source.entity.remove().await;
        } else {
            source.init_data_tracker().await;
        }
    }
}

impl NBTStorage for ItemEntity {}

impl EntityBase for ItemEntity {
    fn tick<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.entity;
            self.pickup_delay
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
                    Some(val.saturating_sub(1))
                })
                .ok();

            let original_velo = entity.velocity.load();

            let mut velo = original_velo;

            if entity.touching_water.load(Ordering::SeqCst) && entity.water_height.load() > 0.1 {
                velo.x *= 0.99;

                velo.z *= 0.99;

                if velo.y < 0.06 {
                    velo.y += 5.0e-4;
                }
            } else if entity.touching_lava.load(Ordering::SeqCst) && entity.lava_height.load() > 0.1
            {
                velo.x *= 0.95;

                velo.z *= 0.95;

                if velo.y < 0.06 {
                    velo.y += 5.0e-4;
                }
            } else {
                velo.y -= self.get_gravity();
            }

            entity.velocity.store(velo);

            let pos = entity.pos.load();

            let bounding_box = entity.bounding_box.load();

            let no_clip = !self
                .entity
                .world
                .load()
                .is_space_empty(bounding_box.expand(-1.0e-7, -1.0e-7, -1.0e-7))
                .await;

            entity.no_clip.store(no_clip, Ordering::Relaxed);

            if no_clip {
                entity
                    .push_out_of_blocks(Vector3::new(
                        pos.x,
                        f64::midpoint(bounding_box.min.y, bounding_box.max.y),
                        pos.z,
                    ))
                    .await;
            }

            let move_velo = entity.velocity.load(); // In case push_out_of_blocks modifies it

            let mut tick_move = !entity.on_ground.load(Ordering::SeqCst)
                || move_velo.horizontal_length_squared() > 1.0e-5;

            if !tick_move {
                let Ok(item_age) = i32::try_from(self.item_age.load(Ordering::Relaxed)) else {
                    entity.remove().await;

                    return;
                };

                tick_move = (item_age + entity.entity_id) % 4 == 0;
            }

            if tick_move {
                entity.move_entity(caller.clone(), move_velo).await;

                entity.tick_block_collisions(&caller, server).await;

                let mut friction = 0.98;

                let on_ground = entity.on_ground.load(Ordering::SeqCst);

                // Vanilla Entity.move() runs block fall movement hooks before item friction/bounce.
                let mut velo = entity.velocity.load();
                if on_ground {
                    velo.y = 0.0;
                }

                if on_ground {
                    let block_affecting_velo = entity.get_block_with_y_offset(0.999_999).await.1;

                    friction *= f64::from(block_affecting_velo.slipperiness) * 0.98;
                }

                velo = velo.multiply(friction, 0.98, friction);

                if on_ground && velo.y < 0.0 {
                    velo = velo.multiply(1.0, -0.5, 1.0);
                }

                entity.velocity.store(velo);
            }

            if !self.never_despawn.load(Ordering::Relaxed) {
                let age = self.item_age.fetch_add(1, Ordering::Relaxed) + 1;

                if age >= 6000 {
                    entity.remove().await;

                    return;
                }

                let n = if entity
                    .last_pos
                    .load()
                    .sub(&entity.pos.load())
                    .length_squared()
                    == 0.0
                {
                    40
                } else {
                    2
                };

                if age.is_multiple_of(n) && self.can_merge().await {
                    self.try_merge().await;
                }
            }

            entity.update_fluid_state(&caller).await;

            let velocity_dirty = entity.velocity_dirty.swap(false, Ordering::SeqCst)
                || entity.touching_water.load(Ordering::SeqCst)
                || entity.touching_lava.load(Ordering::SeqCst)
                || entity.velocity.load().sub(&original_velo).length_squared() > 0.1;

            if velocity_dirty {
                entity.send_pos_rot().await;

                entity.send_velocity().await;
            }
        })
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {
            self.entity
                .send_meta_data(&[Metadata::new(
                    TrackedData::DATA_STACK,
                    MetaDataType::ItemStack,
                    &ItemStackSerializer::from(self.item_stack.lock().await.clone()),
                )])
                .await;
        })
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            // TODO: invulnerability, e.g. ancient debris
            loop {
                let current = self.health.load(Relaxed);
                let new = current - amount;
                if self
                    .health
                    .compare_exchange(current, new, AcqRel, Relaxed)
                    .is_ok()
                {
                    if new <= 0.0 {
                        self.entity.remove().await;
                    }
                    return true;
                }
            }
        })
    }

    fn on_player_collision<'a>(&'a self, player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {
            if self.pickup_delay.load(Ordering::Relaxed) > 0
                || player.living_entity.health.load() <= 0.0
            {
                return;
            }

            if player
                .inventory
                .insert_stack_anywhere(&mut *self.item_stack.lock().await)
                .await
                || player.is_creative()
            {
                player
                    .client
                    .enqueue_packet(&CTakeItemEntity::new(
                        self.entity.entity_id.into(),
                        player.entity_id().into(),
                        self.item_stack.lock().await.item_count.into(),
                    ))
                    .await;
                player
                    .current_screen_handler
                    .lock()
                    .await
                    .lock()
                    .await
                    .send_content_updates()
                    .await;
                if self.item_stack.lock().await.is_empty() {
                    self.entity.remove().await;
                } else {
                    // Update entity
                    self.init_data_tracker().await;
                }
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn get_item_entity(self: Arc<Self>) -> Option<Arc<ItemEntity>> {
        Some(self)
    }

    fn get_gravity(&self) -> f64 {
        0.04
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }
}
