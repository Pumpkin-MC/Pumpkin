use std::sync::{
    Arc,
    atomic::{AtomicI8, AtomicU8, AtomicU32},
};

use async_trait::async_trait;
use pumpkin_protocol::{
    client::play::{CTakeItemEntity, MetaDataType, Metadata},
    codec::slot::Slot,
};
use pumpkin_world::item::ItemStack;

use super::{Entity, EntityBase, living::LivingEntity, player::Player};

pub struct ItemEntity {
    entity: Entity,
    item: ItemStack,
    count: AtomicU8,
    item_age: AtomicU32,
    pickup_delay: AtomicI8,
}

impl ItemEntity {
    pub fn new(entity: Entity, stack: ItemStack) -> Self {
        Self {
            entity,
            item: stack,
            count: AtomicU8::new(stack.item_count),
            item_age: AtomicU32::new(0),
            pickup_delay: AtomicI8::new(10), // Vanilla
        }
    }
    pub async fn send_meta_packet(&self) {
        let slot = Slot::from(&self.item);
        self.entity
            .send_meta_data(Metadata::new(8, MetaDataType::ItemStack, &slot))
            .await;
    }
}

#[async_trait]
impl EntityBase for ItemEntity {
    async fn tick(&self) {
        if self.pickup_delay.load(std::sync::atomic::Ordering::Relaxed) > 0 {
            self.pickup_delay
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }

        let age = self
            .item_age
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if age >= 6000 {
            self.entity.remove().await;
        }
    }
    async fn on_player_collision(&self, player: Arc<Player>) {
        if self.pickup_delay.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            let mut inv = player.inventory.lock().await;
            let mut item = self.item;
            // Check if we have space in inv
            if let Some(slot) = inv.collect_item_slot(item.item.id) {
                let max_stack = item.item.components.max_stack_size;
                if let Some(stack) = inv.get_slot(slot).unwrap() {
                    if stack.item_count + self.count.load(std::sync::atomic::Ordering::Relaxed)
                        > max_stack
                    {
                        // Fill the stack to max and store the overflow
                        let overflow = stack.item_count
                            + self.count.load(std::sync::atomic::Ordering::Relaxed)
                            - max_stack;

                        stack.item_count = max_stack;
                        item.item_count = stack.item_count;

                        self.count
                            .store(overflow, std::sync::atomic::Ordering::Relaxed);
                    } else {
                        // Add the item to the stack
                        stack.item_count += self.count.load(std::sync::atomic::Ordering::Relaxed);
                        item.item_count = stack.item_count;

                        player
                            .client
                            .send_packet(&CTakeItemEntity::new(
                                self.entity.entity_id.into(),
                                player.entity_id().into(),
                                item.item_count.into(),
                            ))
                            .await;
                        self.entity.remove().await;
                    }
                } else {
                    // Add the item as a new stack
                    item.item_count = self.count.load(std::sync::atomic::Ordering::Relaxed);

                    player
                        .client
                        .send_packet(&CTakeItemEntity::new(
                            self.entity.entity_id.into(),
                            player.entity_id().into(),
                            item.item_count.into(),
                        ))
                        .await;
                    self.entity.remove().await;
                }
                player.update_single_slot(&mut inv, slot as i16, item).await;
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
