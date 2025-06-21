use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering::Relaxed},
};

use async_trait::async_trait;
use pumpkin_data::damage::DamageType;
use pumpkin_protocol::{
    client::play::{CTakeItemEntity, MetaDataType, Metadata},
    codec::item_stack_seralizer::ItemStackSerializer,
};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::server::Server;

use super::{Entity, EntityBase, living::LivingEntity, player::Player};

pub struct ItemEntity {
    entity: Entity,
    item_age: AtomicU32,
    // These cannot be atomic values because we mutate their state based on what they are; we run
    // into the ABA problem
    item_stack: Mutex<ItemStack>,
    pickup_delay: Mutex<u8>,
}

impl ItemEntity {
    pub async fn new(entity: Entity, item_stack: ItemStack) -> Self {
        entity
            .set_velocity(Vector3::new(
                rand::random::<f64>() * 0.2 - 0.1,
                0.2,
                rand::random::<f64>() * 0.2 - 0.1,
            ))
            .await;
        entity.yaw.store(rand::random::<f32>() * 360.0);
        Self {
            entity,
            item_stack: Mutex::new(item_stack),
            item_age: AtomicU32::new(0),
            pickup_delay: Mutex::new(10), // Vanilla pickup delay is 10 ticks
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
            pickup_delay: Mutex::new(pickup_delay), // Vanilla pickup delay is 10 ticks
        }
    }
}

#[async_trait]
impl EntityBase for ItemEntity {
    async fn tick(&self, caller: Arc<dyn EntityBase>, server: &Server) {
        self.entity.tick(caller.clone(), server).await;

        {
            let mut delay = self.pickup_delay.lock().await;
            *delay = delay.saturating_sub(1);
        };

        // Gravity and buoyancy
        let mut velo = self.entity.velocity.load();
        if self.entity.touching_water.load(Relaxed) && self.entity.water_height.load() > 0.1 {
            velo.x *= 0.99;
            velo.z *= 0.99;
            if velo.y < 0.06 {
                velo.y += 5.0e-4;
            }
        } else if self.entity.touching_lava.load(Relaxed) && self.entity.lava_height.load() > 0.1 {
            velo.x *= 0.95;
            velo.z *= 0.95;
            if velo.y < 0.06 {
                velo.y += 5.0e-4;
            }
        } else {
            velo.y -= self.get_gravity();
        }
        self.entity.velocity.store(velo);

        let pos = self.entity.pos.load();
        let bounding_box = self.entity.bounding_box.load();

        let no_clip = self
            .entity
            .world
            .read()
            .await
            .is_space_empty(bounding_box.expand(-1.0e-7, -1.0e-7, -1.0e-7))
            .await;
        self.entity.no_clip.store(no_clip, Relaxed);
        if no_clip {
            self.entity
                .push_out_of_blocks(Vector3::new(
                    pos.x,
                    f64::midpoint(bounding_box.min.y, bounding_box.max.y),
                    pos.z,
                ))
                .await;
        }

        let mut velo = self.entity.velocity.load(); // In case push_out_of_blocks modifies it
        let mut tick_move =
            !self.entity.on_ground.load(Relaxed) || velo.horizontal_length_squared() > 1.0e-5;
        if !tick_move {
            let Ok(item_age) = i32::try_from(self.item_age.load(Relaxed)) else {
                self.entity.remove().await;
                return;
            };
            tick_move = (item_age + self.entity.entity_id) % 4 == 0;
        }
        if tick_move {
            self.entity.move_entity(caller.clone(), velo).await;

            self.entity.tick_block_collisions(&caller, server).await;

            let mut friction = 0.98;
            if self.entity.on_ground.load(Relaxed) {
                let block_affecting_velo = self.entity.get_block_with_y_offset(0.999_999).await.1;
                friction *= f64::from(block_affecting_velo.slipperiness);
            }
            velo = velo.multiply(friction, 0.98, friction);
            if velo.y < 0.0 && self.entity.on_ground.load(Relaxed) {
                velo = velo.multiply(1.0, -0.5, 1.0);
            }
            self.entity.velocity.store(velo);
        }

        // TODO merge;

        self.entity.update_fluid_state(&caller).await;

        let age = self.item_age.fetch_add(1, Relaxed);
        if age >= 6000 {
            self.entity.remove().await;
        }

        self.entity.send_pos_rot().await;
        self.entity.send_velocity().await;
    }

    async fn init_data_tracker(&self) {
        self.entity
            .send_meta_data(&[Metadata::new(
                8,
                MetaDataType::ItemStack,
                &ItemStackSerializer::from(*self.item_stack.lock().await),
            )])
            .await;
    }

    async fn damage(&self, _amount: f32, _damage_type: DamageType) -> bool {
        false
    }

    async fn on_player_collision(&self, player: &Arc<Player>) {
        let can_pickup = {
            let delay = self.pickup_delay.lock().await;
            *delay == 0
        };

        if can_pickup
            && player
                .inventory
                .insert_stack_anywhere(&mut *self.item_stack.lock().await)
                .await
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
}
