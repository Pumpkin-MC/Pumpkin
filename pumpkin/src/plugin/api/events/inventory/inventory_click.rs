use crate::entity::player::Player;
use crate::plugin::inventory::{ClickType, InventoryAction, SlotType};
use pumpkin_macros::{Event, cancellable};
use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryClickEvent {
    /// The player who performs the event.
    pub player: Arc<Player>,

    /// The type of slot.
    pub slot_type: SlotType,

    /// The raw slot number.
    pub raw_slot: usize,

    ///The raw slot number, converted to the slot id in the current inventory view.
    //pub which_slot

    /// The type of click that was performed.
    pub click: ClickType,

    /// The type of action that was performed.
    pub action: InventoryAction,

    /// The item in the currently clicked slot.
    pub current: Option<ItemStack>,

    /// The owner of the current Inventory
    pub owner: Arc<dyn Inventory>,

    /// Seems pretty useless, so idk, but returns 0-8 hotbar key but who uses vanillahotkeys anyways
    pub hotbar_key: Option<i8>,
}

#[allow(clippy::too_many_arguments)]
impl InventoryClickEvent {
    pub fn new(
        player: Arc<Player>,
        slot_type: SlotType,
        raw_slot: usize,
        click: ClickType,
        action: InventoryAction,
        current: Option<ItemStack>,
        owner: Arc<dyn Inventory>,
        hotbar_key: Option<i8>,
    ) -> Self {
        Self {
            player,
            slot_type,
            raw_slot,
            click,
            action,
            current,
            owner,
            hotbar_key,
            cancelled: false,
        }
    }

    #[must_use]
    pub fn get_slot_type(&self) -> SlotType {
        self.slot_type.clone()
    }

    #[must_use]
    pub fn get_raw_slot(&self) -> usize {
        self.raw_slot
    }

    // /// Gets the currently held item
    /*pub fn get_cursor(&self) -> ItemStack {
        // todo()!
    }*/

    pub async fn get_current_item(&self, owner: Arc<dyn Inventory>) -> ItemStack {
        if matches!(self.slot_type, SlotType::Outside) {
            self.current.unwrap_or(ItemStack::EMPTY)
        } else {
            let stack = owner.get_stack(self.raw_slot).await;
            let guard = stack.lock().await;
            *guard
        }
    }

    pub async fn set_current_item(&mut self, owner: Arc<dyn Inventory>, item: ItemStack) {
        if matches!(self.slot_type, SlotType::Outside) {
            self.current = Some(item);
        } else {
            let slot = owner.get_stack(self.raw_slot).await;
            let mut guard = slot.lock().await;
            *guard = item;
        }
    }

    #[must_use]
    pub fn is_left_click(&self) -> bool {
        self.click.is_left_click()
    }

    #[must_use]
    pub fn is_right_click(&self) -> bool {
        self.click.is_right_click()
    }

    #[must_use]
    pub fn is_shift_click(&self) -> bool {
        self.click.is_shift_click()
    }

    #[must_use]
    pub fn get_hotbar_button(&self) -> Option<i8> {
        self.hotbar_key
    }

    #[must_use]
    pub fn get_action(&self) -> InventoryAction {
        self.action.clone()
    }

    #[must_use]
    pub fn get_click(&self) -> ClickType {
        self.click.clone()
    }

    #[must_use]
    pub fn get_current(&self) -> Option<ItemStack> {
        self.current
    }
}
