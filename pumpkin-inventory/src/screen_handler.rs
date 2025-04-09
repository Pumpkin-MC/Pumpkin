use std::sync::Arc;

use pumpkin_data::screen::WindowType;
use tokio::sync::Mutex;

use crate::{
    inventory::Inventory,
    slot::{NormalSlot, Slot},
};

pub trait ScreenHandler {
    fn new(window_type: WindowType, sync_id: u8) -> Self;

    fn window_type(&self) -> WindowType;

    fn size(&self) -> usize;

    /// Add it into array
    fn add_slot_internal<S: Slot<I>, I: Inventory>(&mut self, slot: Arc<S>);

    fn add_slot<S: Slot<I>, I: Inventory>(&mut self, mut slot: S) -> Arc<S> {
        slot.set_id(self.size());
        let slot = Arc::new(slot);
        self.add_slot_internal(slot.clone());
        slot
    }

    fn add_player_hotbar_slots<I: Inventory>(&mut self, player_inventory: &Arc<Mutex<I>>) {
        for i in 0..9 {
            self.add_slot(NormalSlot::new(player_inventory.clone(), i));
        }
    }

    fn add_player_inventory_slots<I: Inventory>(&mut self, player_inventory: &Arc<Mutex<I>>) {
        for i in 0..3 {
            for j in 0..9 {
                self.add_slot(NormalSlot::new(player_inventory.clone(), j + (i + 1) * 9));
            }
        }
    }

    fn add_player_slots<I: Inventory>(&mut self, player_inventory: &Arc<Mutex<I>>) {
        self.add_player_inventory_slots(player_inventory);
        self.add_player_hotbar_slots(player_inventory);
    }
}
