use crate::inventory::Inventory;
use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour},
    slot::NormalSlot,
};
use pumpkin_data::{item_stack::ItemStack, screen::WindowType};
use std::{any::Any, sync::Arc};

/// Builder for custom GUIs.
pub struct GUIBuilder {
    screen_type: WindowType,
    rows: u8,
    columns: u8,
    inventory: Arc<dyn Inventory>,
    allow_grab_items: bool,
    allow_put_items: bool,
}

impl GUIBuilder {
    /// Creates a new `GUIBuilder` with a basic 9x3 layout.
    pub fn new(screen_type: WindowType, inventory: Arc<dyn Inventory>) -> Self {
        let (rows, columns) = match screen_type {
            WindowType::Generic9x1 => (1, 9),
            WindowType::Generic9x2 => (2, 9),
            WindowType::Generic9x4 => (4, 9),
            WindowType::Generic9x5 => (5, 9),
            WindowType::Generic9x6 => (6, 9),
            WindowType::Generic3x3 | WindowType::Crafter3x3 => (3, 3),
            WindowType::Hopper => (1, 5),
            _ => (3, 9), // Default to 9x3
        };

        Self {
            screen_type,
            rows,
            columns,
            inventory,
            allow_grab_items: true,
            allow_put_items: true,
        }
    }

    /// Sets whether players can grab items out of the inventory.
    #[must_use]
    pub const fn allow_grab_items(mut self, allow: bool) -> Self {
        self.allow_grab_items = allow;
        self
    }

    /// Sets whether players can put items into the inventory from their own.
    #[must_use]
    pub const fn allow_put_items(mut self, allow: bool) -> Self {
        self.allow_put_items = allow;
        self
    }

    /// Builds the `GUIScreenHandler`.
    pub fn build(self, sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> GUIScreenHandler {
        let mut behaviour = ScreenHandlerBehaviour::new(sync_id, Some(self.screen_type));
        behaviour.allow_grab_items = self.allow_grab_items;
        behaviour.allow_put_items = self.allow_put_items;
        behaviour.container_slots = (self.rows * self.columns) as usize;

        let mut handler = GUIScreenHandler {
            inventory: self.inventory.clone(),
            rows: self.rows,
            columns: self.columns,
            behaviour,
        };

        self.inventory.on_open();

        handler.add_inventory_slots();
        let player_inventory_trait: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory_trait);

        handler
    }
}

pub struct GUIScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    pub rows: u8,
    pub columns: u8,
    behaviour: ScreenHandlerBehaviour,
}

impl GUIScreenHandler {
    fn add_inventory_slots(&mut self) {
        for i in 0..self.rows {
            for j in 0..self.columns {
                self.add_slot(Arc::new(NormalSlot::new(
                    self.inventory.clone(),
                    (j + i * self.columns) as usize,
                )));
            }
        }
    }
}

impl ScreenHandler for GUIScreenHandler {
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
        self.inventory.on_close();
    }

    fn quick_move(&mut self, _player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let mut stack_left = ItemStack::EMPTY.clone();
        let slot = self.get_behaviour().slots[slot_index as usize].clone();

        if slot.has_stack() {
            let mut slot_stack = slot.get_stack();
            stack_left = slot_stack.clone();
            let container_slots = i32::from(self.rows * self.columns);

            if slot_index < container_slots {
                // From container to player
                if !self.get_behaviour().allow_grab_items {
                    return ItemStack::EMPTY.clone();
                }
                if !self.insert_item(
                    &mut slot_stack,
                    container_slots,
                    self.get_behaviour().slots.len() as i32,
                    true,
                ) {
                    return ItemStack::EMPTY.clone();
                }
            } else {
                // From player to container
                if !self.get_behaviour().allow_put_items {
                    return ItemStack::EMPTY.clone();
                }
                if !self.insert_item(&mut slot_stack, 0, container_slots, false) {
                    // Move from player area to inventory (start)
                    return ItemStack::EMPTY.clone();
                }
            }

            if slot_stack.is_empty() {
                slot.set_stack(ItemStack::EMPTY.clone());
            } else {
                slot.set_stack(slot_stack);
            }
        }

        stack_left
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InventoryHolder, SimpleInventory, entity_equipment::EntityEquipment};
    use pumpkin_data::item::Item;
    use std::sync::Mutex;

    struct MenuOwner {
        session: u32,
    }

    impl InventoryHolder for MenuOwner {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    fn player_inventory() -> Arc<PlayerInventory> {
        Arc::new(PlayerInventory::new(
            Arc::new(Mutex::new(EntityEquipment::new())),
            Arc::new(crate::build_equipment_slots()),
        ))
    }

    #[test]
    fn gui_slots_preserve_inventory_holder() -> Result<(), &'static str> {
        let holder: Arc<dyn InventoryHolder> = Arc::new(MenuOwner { session: 7 });
        let inventory: Arc<dyn Inventory> =
            Arc::new(SimpleInventory::new(27).with_holder(holder.clone()));
        let handler = GUIBuilder::new(WindowType::Generic9x3, inventory.clone())
            .build(1, &player_inventory());

        assert!(Arc::ptr_eq(&handler.inventory, &inventory));
        let slots = &handler.get_behaviour().slots;
        assert_eq!(slots.len(), inventory.size() + PlayerInventory::MAIN_SIZE);

        for slot in &slots[..inventory.size()] {
            let slot_inventory = slot.get_inventory();
            assert!(Arc::ptr_eq(&slot_inventory, &inventory));
            let slot_holder = slot_inventory.holder().ok_or("Container holder missing")?;
            assert!(Arc::ptr_eq(&slot_holder, &holder));
            let owner = slot_holder
                .as_any()
                .downcast_ref::<MenuOwner>()
                .ok_or("Incorrect holder type")?;
            assert_eq!(owner.session, 7);
        }

        for slot in &slots[inventory.size()..] {
            assert!(slot.get_inventory().holder().is_none());
        }

        let slot = slots.first().ok_or("Container slot missing")?;
        slot.set_stack(ItemStack::new(3, &Item::STONE));
        assert_eq!(inventory.get_stack(0).item_count, 3);
        inventory.clear();
        assert!(slot.get_stack().is_empty());
        let updated_holder = inventory.holder().ok_or("Holder lost after clearing")?;
        assert!(Arc::ptr_eq(&updated_holder, &holder));

        Ok(())
    }

    #[test]
    fn shared_holder_does_not_merge_gui_inventories() -> Result<(), &'static str> {
        let holder: Arc<dyn InventoryHolder> = Arc::new(MenuOwner { session: 7 });
        let first_inventory: Arc<dyn Inventory> =
            Arc::new(SimpleInventory::new(27).with_holder(holder.clone()));
        let second_inventory: Arc<dyn Inventory> =
            Arc::new(SimpleInventory::new(27).with_holder(holder));
        let player_inventory = player_inventory();
        let first = GUIBuilder::new(WindowType::Generic9x3, first_inventory.clone())
            .build(1, &player_inventory);
        let second = GUIBuilder::new(WindowType::Generic9x3, second_inventory.clone())
            .build(2, &player_inventory);

        let first_slot = first
            .get_behaviour()
            .slots
            .first()
            .ok_or("First GUI slot missing")?;
        let second_slot = second
            .get_behaviour()
            .slots
            .first()
            .ok_or("Second GUI slot missing")?;
        let first_slot_inventory = first_slot.get_inventory();
        let second_slot_inventory = second_slot.get_inventory();
        assert!(!Arc::ptr_eq(&first_slot_inventory, &second_slot_inventory));
        let first_holder = first_slot_inventory
            .holder()
            .ok_or("First holder missing")?;
        let second_holder = second_slot_inventory
            .holder()
            .ok_or("Second holder missing")?;
        assert!(Arc::ptr_eq(&first_holder, &second_holder));

        first_slot.set_stack(ItemStack::new(3, &Item::STONE));
        assert_eq!(first_inventory.get_stack(0).item_count, 3);
        assert!(second_slot.get_stack().is_empty());
        second_slot.set_stack(ItemStack::new(1, &Item::DIRT));
        assert_eq!(second_inventory.get_stack(0).get_item().id, Item::DIRT.id);
        assert_eq!(first_slot.get_stack().get_item().id, Item::STONE.id);

        Ok(())
    }

    #[test]
    fn gui_slots_retain_holder_until_last_inventory_reference_is_dropped()
    -> Result<(), &'static str> {
        let holder: Arc<dyn InventoryHolder> = Arc::new(MenuOwner { session: 7 });
        let weak_holder = Arc::downgrade(&holder);
        let inventory: Arc<dyn Inventory> =
            Arc::new(SimpleInventory::new(27).with_holder(holder.clone()));
        let weak_inventory = Arc::downgrade(&inventory);
        let handler = GUIBuilder::new(WindowType::Generic9x3, inventory.clone())
            .build(1, &player_inventory());
        let slot = handler
            .get_behaviour()
            .slots
            .first()
            .ok_or("Container slot missing")?
            .clone();

        drop(holder);
        drop(inventory);
        drop(handler);
        assert!(weak_inventory.upgrade().is_some());
        assert!(weak_holder.upgrade().is_some());
        assert!(slot.get_inventory().holder().is_some());

        drop(slot);
        assert!(weak_inventory.upgrade().is_none());
        assert!(weak_holder.upgrade().is_none());

        Ok(())
    }
}
