use std::sync::atomic::AtomicI8;

use async_trait::async_trait;
use pumpkin_protocol::{
    client::play::{MetaDataType, Metadata},
    codec::slot::Slot,
};
use pumpkin_world::item::ItemStack;

use super::{living::LivingEntity, Entity, EntityBase};

pub struct ItemEntity {
    entity: Entity,
    item: Slot,
    pickup_delay: AtomicI8,
}

impl ItemEntity {
    pub fn new(entity: Entity, stack: &ItemStack) -> Self {
        let slot = Slot::from(stack);
        Self {
            entity,
            item: slot,
            pickup_delay: AtomicI8::new(10), // Vanilla
        }
    }
    pub async fn send_meta_packet(&self) {
        self.entity
            .send_meta_data(Metadata::new(8, MetaDataType::ItemStack, self.item.clone()))
            .await;
    }
}

#[async_trait]
impl EntityBase for ItemEntity {
    async fn tick(&self) {
        if self.pickup_delay.load(std::sync::atomic::Ordering::Relaxed) >= 0 {
            self.pickup_delay
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    async fn on_player_collision(&self) {
        if self.pickup_delay.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            // check if inventory is full
            self.entity.remove().await;
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }
}
