use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering::Relaxed},
};

use async_trait::async_trait;
use pumpkin_data::{damage::DamageType, item::Item};
use pumpkin_nbt::compound::NbtCompound;
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
    pub async fn new(entity: Entity, item_id: u16, count: u32) -> Self {
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
            item_stack: Mutex::new(ItemStack::new(
                count as u8,
                Item::from_id(item_id).expect("We passed a bad item id into ItemEntity"),
            )),
            item_age: AtomicU32::new(0),
            pickup_delay: Mutex::new(10), // Vanilla pickup delay is 10 ticks
        }
    }

    pub async fn new_with_velocity(
        entity: Entity,
        item_id: u16,
        count: u32,
        velocity: Vector3<f64>,
        pickup_delay: u8,
    ) -> Self {
        entity.set_velocity(velocity).await;
        entity.yaw.store(rand::random::<f32>() * 360.0);
        Self {
            entity,
            item_stack: Mutex::new(ItemStack::new(
                count as u8,
                Item::from_id(item_id).expect("We passed a bad item id into ItemEntity"),
            )),
            item_age: AtomicU32::new(0),
            pickup_delay: Mutex::new(pickup_delay), // Vanilla pickup delay is 10 ticks
        }
    }
}

#[async_trait]
impl EntityBase for ItemEntity {
    async fn tick(&self, server: &Server) {
        let entity = &self.entity;
        entity.tick(server).await;
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
                &ItemStackSerializer::from(self.item_stack.lock().await.clone()),
            )])
            .await;
    }

    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.entity.write_nbt(nbt).await;
        nbt.put_short("Age", self.item_age.load(Relaxed) as i16);
        let pickup_delay = self.pickup_delay.lock().await;
        nbt.put_short("PickupDelay", i16::from(*pickup_delay));
        // TODO: put stack
    }

    async fn read_nbt(&self, nbt: &NbtCompound) {
        self.entity.read_nbt(nbt).await;
        self.item_age
            .store(nbt.get_short("Age").unwrap_or(0) as u32, Relaxed);
        *self.pickup_delay.lock().await = nbt.get_short("PickupDelay").unwrap_or(0) as u8;
        // TODO: get stack
    }

    async fn damage(&self, _amount: f32, _damage_type: DamageType) -> bool {
        false
    }

    async fn on_player_collision(&self, player: Arc<Player>) {
        let can_pickup = {
            let delay = self.pickup_delay.lock().await;
            *delay == 0
        };

        if can_pickup {
            let mut inv = player.inventory.lock().await;
            let mut total_pick_up = 0;
            let mut slot_updates = Vec::new();
            let remove_entity = {
                let item_stack = self.item_stack.lock().await.clone();
                let mut stack_size = item_stack.item_count;
                let max_stack = item_stack.item.components.max_stack_size;
                while stack_size > 0 {
                    if let Some(slot) = inv.get_pickup_item_slot(item_stack.item.id) {
                        // Fill the inventory while there are items in the stack and space in the inventory
                        let maybe_stack = inv.get_slot(slot).unwrap();

                        if let Some(existing_stack) = maybe_stack {
                            // We have the item in this stack already

                            // This is bounded to `u8::MAX`
                            let amount_to_fill = u32::from(max_stack - existing_stack.item_count);
                            // This is also bounded to `u8::MAX` since `amount_to_fill` is max `u8::MAX`
                            let amount_to_add = amount_to_fill.min(u32::from(stack_size));
                            // Therefore this is safe

                            // Update referenced stack so next call to `get_pickup_item_slot` is
                            // correct
                            existing_stack.item_count += amount_to_add as u8;
                            total_pick_up += amount_to_add;

                            debug_assert!(amount_to_add > 0);
                            stack_size = stack_size.saturating_sub(amount_to_add as u8);

                            slot_updates.push((slot, existing_stack.clone()));
                        } else {
                            // A new stack

                            // This is bounded to `u8::MAX`
                            let amount_to_fill = u32::from(max_stack);
                            // This is also bounded to `u8::MAX` since `amount_to_fill` is max `u8::MAX`
                            let amount_to_add = amount_to_fill.min(u32::from(stack_size));
                            total_pick_up += amount_to_add;

                            debug_assert!(amount_to_add > 0);
                            stack_size = stack_size.saturating_sub(amount_to_add as u8);

                            slot_updates.push((slot, self.item_stack.lock().await.clone()));
                        }
                    } else {
                        // We can't pick anything else up
                        break;
                    }
                }

                stack_size == 0
            };

            if total_pick_up > 0 {
                player
                    .client
                    .enqueue_packet(&CTakeItemEntity::new(
                        self.entity.entity_id.into(),
                        player.entity_id().into(),
                        total_pick_up.try_into().unwrap(),
                    ))
                    .await;
            }

            // TODO: Can we batch slot updates?
            for (slot, stack) in slot_updates {
                player.update_single_slot(&mut inv, slot, stack).await;
            }

            if remove_entity {
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
