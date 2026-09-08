use std::any::Any;
use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_world::inventory::Inventory;

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour};
use crate::slot::{ArmorSlot, NormalSlot};

pub struct MountScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    pub mount_inventory: Arc<dyn Inventory>,
    pub saddle_inventory: Arc<dyn Inventory>,
    pub armor_inventory: Arc<dyn Inventory>,
    pub inventory_columns: usize,
}

impl MountScreenHandler {
    pub const SLOT_SADDLE: usize = 0;
    pub const SLOT_BODY_ARMOR: usize = 1;
    pub const SLOT_INVENTORY_START: usize = 2;
    pub const INVENTORY_ROWS: usize = 3;

    #[must_use]
    pub const fn get_inventory_size(inventory_columns: usize) -> usize {
        inventory_columns * Self::INVENTORY_ROWS
    }

    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        mount_inventory: Arc<dyn Inventory>,
        saddle_inventory: Arc<dyn Inventory>,
        armor_inventory: Arc<dyn Inventory>,
        inventory_columns: usize,
    ) -> Self {
        let mut handler = Self {
            behaviour: ScreenHandlerBehaviour::new(sync_id, None),
            mount_inventory,
            saddle_inventory: saddle_inventory.clone(),
            armor_inventory: armor_inventory.clone(),
            inventory_columns,
        };

        handler.add_slot(Arc::new(ArmorSlot::new(
            saddle_inventory,
            0,
            EquipmentSlot::SADDLE,
        )));
        handler.add_slot(Arc::new(ArmorSlot::new(
            armor_inventory,
            0,
            EquipmentSlot::BODY,
        )));

        let mount_size = handler.mount_inventory.size();
        for i in 0..mount_size {
            handler.add_slot(Arc::new(NormalSlot::new(
                handler.mount_inventory.clone(),
                i,
            )));
        }

        let pi: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&pi);

        handler
    }
}

impl ScreenHandler for MountScreenHandler {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player);
    }

    fn quick_move(&mut self, _player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let mut clicked = ItemStack::EMPTY.clone();
        let slot = self.get_behaviour().slots.get(slot_index as usize).cloned();

        if let Some(slot) = slot {
            if !slot.has_stack() {
                return clicked;
            }

            let mut stack = slot.get_stack();
            clicked = stack.clone();

            let mount_container_size = self.mount_inventory.size();
            let player_container_start = 2 + mount_container_size as i32;
            let total_slots = self.get_behaviour().slots.len() as i32;

            if slot_index < player_container_start {
                if !self.insert_item(&mut stack, player_container_start, total_slots, true) {
                    return ItemStack::EMPTY.clone();
                }
            } else if self.get_behaviour().slots[1].can_insert(&stack)
                && !self.get_behaviour().slots[1].has_stack()
            {
                if !self.insert_item(&mut stack, 1, 2, false) {
                    return ItemStack::EMPTY.clone();
                }
            } else if self.get_behaviour().slots[0].can_insert(&stack)
                && !self.get_behaviour().slots[0].has_stack()
            {
                if !self.insert_item(&mut stack, 0, 1, false) {
                    return ItemStack::EMPTY.clone();
                }
            } else if mount_container_size == 0
                || !self.insert_item(&mut stack, 2, player_container_start, false)
            {
                let player_container_end = player_container_start + 27;
                let player_hotbar_start = player_container_end;
                let player_hotbar_end = player_hotbar_start + 9;

                if (player_hotbar_start..player_hotbar_end).contains(&slot_index) {
                    if !self.insert_item(
                        &mut stack,
                        player_container_start,
                        player_container_end,
                        false,
                    ) {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (player_container_start..player_container_end).contains(&slot_index) {
                    if !self.insert_item(&mut stack, player_hotbar_start, player_hotbar_end, false)
                    {
                        return ItemStack::EMPTY.clone();
                    }
                } else if !self.insert_item(
                    &mut stack,
                    player_hotbar_start,
                    player_container_end,
                    false,
                ) {
                    return ItemStack::EMPTY.clone();
                }

                return ItemStack::EMPTY.clone();
            }

            if stack.is_empty() {
                slot.set_stack(ItemStack::EMPTY.clone());
            } else {
                slot.set_stack(stack);
            }
        }

        clicked
    }
}

pub struct NautilusInventoryScreenHandler {
    pub mount_handler: MountScreenHandler,
}

impl NautilusInventoryScreenHandler {
    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        nautilus_inventory: Arc<dyn Inventory>,
        saddle_inventory: Arc<dyn Inventory>,
        armor_inventory: Arc<dyn Inventory>,
        inventory_columns: usize,
    ) -> Self {
        Self {
            mount_handler: MountScreenHandler::new(
                sync_id,
                player_inventory,
                nautilus_inventory,
                saddle_inventory,
                armor_inventory,
                inventory_columns,
            ),
        }
    }
}

impl ScreenHandler for NautilusInventoryScreenHandler {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        self.mount_handler.get_behaviour()
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        self.mount_handler.get_behaviour_mut()
    }

    fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.mount_handler.on_closed(player);
    }

    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        self.mount_handler.quick_move(player, slot_index)
    }
}
