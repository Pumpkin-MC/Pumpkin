use std::sync::{
    atomic::{AtomicI8, AtomicU8},
    Arc,
};

use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_protocol::{
    client::play::{CTakeItemEntity, MetaDataType, Metadata},
    codec::{slot::Slot, var_int::VarInt},
};
use pumpkin_world::item::ItemStack;

use super::{living::LivingEntity, player::Player, Entity, EntityBase};

pub struct ItemEntity {
    entity: Entity,
    item: Slot,
    id: u16,
    count: AtomicU8,
    pickup_delay: AtomicI8,
}

impl ItemEntity {
    pub fn new(entity: Entity, stack: &ItemStack) -> Self {
        let slot = Slot::from(stack);
        Self {
            entity,
            id: stack.item.id,
            count: AtomicU8::new(stack.item_count),
            item: slot,
            pickup_delay: AtomicI8::new(10), // Vanilla
        }
    }
    pub async fn send_meta_packet(&self) {
        self.entity
            .send_meta_data(Metadata::new(8, MetaDataType::ItemStack, &self.item))
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
    }
    async fn on_player_collision(&self, player: Arc<Player>) {
        if self.pickup_delay.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            let mut inv = player.inventory.lock().await;
            // Check if we have space in inv
            if let Some(slot) = inv.collect_item_slot(self.id) {
                let mut item = self.item.clone();
                let max_stack = Item::from_id(self.id)
                    .unwrap_or(Item::AIR)
                    .components
                    .max_stack_size;
                if let Some(stack) = inv.get_slot(slot).unwrap() {
                    if stack.item_count + self.count.load(std::sync::atomic::Ordering::Relaxed)
                        > max_stack
                    {
                        // Fill the stack to max and store the overflow
                        let overflow = stack.item_count
                            + self.count.load(std::sync::atomic::Ordering::Relaxed)
                            - max_stack;

                        stack.item_count = max_stack;
                        item.item_count = VarInt(i32::from(stack.item_count));

                        self.count
                            .store(overflow, std::sync::atomic::Ordering::Relaxed);
                    } else {
                        // Add the item to the stack
                        stack.item_count += self.count.load(std::sync::atomic::Ordering::Relaxed);
                        item.item_count = VarInt(i32::from(stack.item_count));

                        player
                            .client
                            .send_packet(&CTakeItemEntity::new(
                                self.entity.entity_id.into(),
                                player.entity_id().into(),
                                1.into(),
                            ))
                            .await;
                        self.entity.remove().await;
                    }
                } else {
                    // Add the item as a new stack
                    item.item_count = VarInt(i32::from(
                        self.count.load(std::sync::atomic::Ordering::Relaxed),
                    ));

                    player
                        .client
                        .send_packet(&CTakeItemEntity::new(
                            self.entity.entity_id.into(),
                            player.entity_id().into(),
                            1.into(),
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
