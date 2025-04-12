use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::screen::WindowType;
use tokio::sync::Mutex;

use crate::{
    inventory::Inventory,
    slot::{NormalSlot, Slot},
};

//ScreenHandler.java
// TODO: Fully implement this
#[async_trait]
pub trait ScreenHandler {
    fn window_type(&self) -> Option<WindowType>;

    fn size(&self) -> usize;

    /// Add it into array
    fn add_slot_internal(&mut self, slot: Arc<dyn Slot>);

    fn add_slot<S: Slot + 'static>(&mut self, mut slot: S) -> Arc<S> {
        slot.set_id(self.size());
        let slot = Arc::new(slot);
        self.add_slot_internal(slot.clone());
        slot
    }

    fn add_player_hotbar_slots(&mut self, player_inventory: &Arc<Mutex<dyn Inventory>>) {
        for i in 0..9 {
            self.add_slot(NormalSlot::new(player_inventory.clone(), i));
        }
    }

    fn add_player_inventory_slots(&mut self, player_inventory: &Arc<Mutex<dyn Inventory>>) {
        for i in 0..3 {
            for j in 0..9 {
                self.add_slot(NormalSlot::new(player_inventory.clone(), j + (i + 1) * 9));
            }
        }
    }

    fn add_player_slots(&mut self, player_inventory: &Arc<Mutex<dyn Inventory>>) {
        self.add_player_inventory_slots(player_inventory);
        self.add_player_hotbar_slots(player_inventory);
    }
}
