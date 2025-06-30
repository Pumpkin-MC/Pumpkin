use crate::entity::player::Player;
use crate::world::World;
use futures::future::join_all;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;
use uuid::Uuid;

#[allow(dead_code)]
impl Player {
    async fn get_item(&self, slot: usize) -> Option<ItemStack> {
        if slot < self.inventory().main_inventory.len() {
            let guard = self.inventory().main_inventory[slot].lock().await;
            Some(guard.clone())
        } else {
            None
        }
    }

    async fn set_item(&self, slot: i16, mut item: ItemStack) {
        self.remove_stack(slot.try_into().unwrap()).await;
        self.inventory().insert_stack(slot, &mut item).await;
    }

    // TODO: Consider clearing off-hand and armor slots.
    async fn fill_inventory_with_item(&self, item: ItemStack) {
        let futures = (0..36).map(|i| self.set_item(i, item));
        join_all(futures).await;
    }

    // TODO: Also clear off-hand and armor slots.
    async fn clear_inventory(&self) {
        let futures = (0..36).map(|i| self.remove_stack(i));
        join_all(futures).await;
    }

    async fn remove_stack(&self, slot: usize) -> ItemStack {
        if slot < self.inventory().main_inventory.len() {
            let mut removed = ItemStack::EMPTY;
            let mut guard = self.inventory().main_inventory[slot].lock().await;
            std::mem::swap(&mut removed, &mut *guard);
            removed
        } else {
            let slot = self.inventory().equipment_slots.get(&slot).unwrap();
            self.inventory()
                .entity_equipment
                .lock()
                .await
                .put(slot, ItemStack::EMPTY)
                .await
        }
    }

    async fn get_food_level(&self) -> u8 {
        self.hunger_manager.level.load()
    }

    async fn set_food_level(&self, level: u8) {
        self.hunger_manager.level.store(level.clamp(0, 20));
        self.send_health().await;
    }

    async fn is_hungry(&self) -> bool {
        self.get_food_level().await < 20
    }

    async fn get_saturation(&self) -> f32 {
        self.hunger_manager.saturation.load()
    }

    // TODO: Find out the actual max, this makes no sense
    async fn set_saturation(&self, level: f32) {
        self.hunger_manager.saturation.store(level.clamp(0.0, 20.0));
        self.send_health().await;
    }

    async fn get_health(&self) -> f32 {
        self.living_entity.health.load()
    }

    // Consider clamping, as there is no max_health rn
    // Also, send update packet
    /* async fn set_health(&self, health: f32) {
        self.living_entity.health.store(health);
    } */

    async fn damage(&self, damage: f32) {
        self.set_health(self.get_health().await - damage).await;
        self.send_health().await;
    }

    async fn get_uuid(&self) -> Uuid {
        self.gameprofile.id
    }

    async fn get_name(&self) -> String {
        self.gameprofile.name.clone()
    }

    pub async fn send_message(&self, text: TextComponent) {
        self.send_system_message(&text).await
    }

    pub async fn get_location(&self) -> Option<Vector3<f64>> {
        Some(self.living_entity.entity.pos.load())
    }

    pub async fn set_location(&self, pos: Vector3<f64>) {
        self.living_entity.entity.pos.store(pos);
    }

    pub async fn get_world(&self) -> Option<Arc<World>> {
        Some(self.living_entity.entity.world.read().await.clone())
    }

    // TODO
    // get_offhand
    // clear armor slots
    // keep in mind to make armor slots available in get_item or add get_armor_slot
    // teleport
    //
}
