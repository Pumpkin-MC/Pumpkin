use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering::Relaxed},
};

use async_trait::async_trait;
use pumpkin_data::damage::DamageType;
use pumpkin_protocol::{
    codec::item_stack_seralizer::ItemStackSerializer,
    java::client::play::{CTakeItemEntity, MetaDataType, Metadata},
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
        let entity = &self.entity;
        entity.tick(caller, server).await;
        {
            let mut delay = self.pickup_delay.lock().await;
            *delay = delay.saturating_sub(1);
        };

        let age = self.item_age.fetch_add(1, Relaxed);
        if age >= 6000 {
            entity.remove().await;
        }
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
            && player.living_entity.health.load() > 0.0
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
}
