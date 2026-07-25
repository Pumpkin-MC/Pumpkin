//! Test utilities: a minimal [`InventoryPlayer`] implementation for
//! screen-handler tests. Tracks XP and dropped stacks in memory instead of
//! talking to a world or network connection.

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_protocol::java::client::play::{
    CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
    CSetPlayerInventory, CSetSelectedSlot,
};
use tokio::sync::Mutex;

use crate::entity_equipment::EntityEquipment;
use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{InventoryPlayer, PlayerFuture};

pub struct MockPlayer {
    pub(crate) player_inventory: Arc<PlayerInventory>,
    pub(crate) xp_levels: AtomicI32,
    pub(crate) xp_awarded: AtomicI32,
    pub(crate) dropped: Mutex<Vec<ItemStack>>,
    creative: bool,
}

impl MockPlayer {
    pub(crate) fn new() -> Self {
        Self {
            player_inventory: Arc::new(PlayerInventory::new(
                Arc::new(Mutex::new(EntityEquipment::new())),
                Arc::new(crate::build_equipment_slots()),
            )),
            xp_levels: AtomicI32::new(0),
            xp_awarded: AtomicI32::new(0),
            dropped: Mutex::new(Vec::new()),
            creative: false,
        }
    }
}

impl Default for MockPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryPlayer for MockPlayer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn drop_item(&self, item: ItemStack, _retain_ownership: bool) -> PlayerFuture<'_, ()> {
        Box::pin(async move {
            self.dropped.lock().await.push(item);
        })
    }

    fn get_inventory(&self) -> Arc<PlayerInventory> {
        self.player_inventory.clone()
    }

    fn has_infinite_materials(&self) -> bool {
        self.creative
    }

    fn is_creative(&self) -> bool {
        self.creative
    }

    fn experience_level(&self) -> i32 {
        self.xp_levels.load(Ordering::Relaxed)
    }

    fn add_experience_levels(&self, levels: i32) -> PlayerFuture<'_, ()> {
        Box::pin(async move {
            self.xp_levels.fetch_add(levels, Ordering::Relaxed);
        })
    }

    fn enchantment_seed(&self) -> i32 {
        0
    }

    fn set_enchantment_seed(&self, _seed: i32) -> PlayerFuture<'_, ()> {
        Box::pin(async {})
    }

    fn enqueue_inventory_packet<'a>(
        &'a self,
        _packet: &'a CSetContainerContent,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async {})
    }

    fn enqueue_slot_packet<'a>(&'a self, _packet: &'a CSetContainerSlot) -> PlayerFuture<'a, ()> {
        Box::pin(async {})
    }

    fn enqueue_cursor_packet<'a>(&'a self, _packet: &'a CSetCursorItem) -> PlayerFuture<'a, ()> {
        Box::pin(async {})
    }

    fn enqueue_property_packet<'a>(
        &'a self,
        _packet: &'a CSetContainerProperty,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async {})
    }

    fn enqueue_slot_set_packet<'a>(
        &'a self,
        _packet: &'a CSetPlayerInventory,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async {})
    }

    fn enqueue_set_held_item_packet<'a>(
        &'a self,
        _packet: &'a CSetSelectedSlot,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async {})
    }

    fn enqueue_equipment_change<'a>(
        &'a self,
        _slot: &'a EquipmentSlot,
        _stack: &'a ItemStack,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async {})
    }

    fn award_experience(&self, amount: i32) -> PlayerFuture<'_, ()> {
        Box::pin(async move {
            self.xp_awarded.fetch_add(amount, Ordering::Relaxed);
        })
    }

    fn increment_stat(
        &self,
        _category: StatisticCategory,
        _stat_id: i32,
        _amount: i32,
    ) -> PlayerFuture<'_, ()> {
        Box::pin(async {})
    }
}
