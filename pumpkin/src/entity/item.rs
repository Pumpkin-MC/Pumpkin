use std::sync::{Arc, atomic::AtomicU32};

use async_trait::async_trait;
use pumpkin_data::damage::DamageType;
use pumpkin_protocol::{
    client::play::{CTakeItemEntity, MetaDataType, Metadata},
    codec::item_stack_seralizer::ItemStackSerializer,
};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::inventory::inventory::Inventory;
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
    pub async fn send_meta_packet(&self) {
        self.entity
            .send_meta_data(&[Metadata::new(
                8,
                MetaDataType::ItemStack,
                &ItemStackSerializer::from(*self.item_stack.lock().await),
            )])
            .await;
    }
}

#[async_trait]
impl EntityBase for ItemEntity {
    async fn tick(&self, server: &Server) {
        self.entity.tick(server).await;
        {
            let mut delay = self.pickup_delay.lock().await;
            *delay = delay.saturating_sub(1);
        };

        let age = self
            .item_age
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if age >= 6000 {
            self.entity.remove().await;
        }
    }
    async fn damage(&self, _amount: f32, _damage_type: DamageType) -> bool {
        false
    }

    async fn on_player_collision(&self, player: Arc<Player>) {
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
                self.send_meta_packet().await;
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
