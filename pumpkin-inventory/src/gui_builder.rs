use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture,
    },
    slot::NormalSlot,
};
use pumpkin_data::{item_stack::ItemStack, screen::WindowType};
use pumpkin_world::inventory::Inventory;
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
            WindowType::Generic3x3 => (3, 3),
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
    pub async fn build(
        self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
    ) -> GUIScreenHandler {
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

        self.inventory.on_open().await;

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

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            self.inventory.on_close().await;
        })
    }

    fn quick_move<'a>(
        &'a mut self,
        _player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let mut stack_left = ItemStack::EMPTY.clone();
            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if slot.has_stack().await {
                let slot_stack_lock = slot.get_stack().await;
                let slot_stack_guard = slot_stack_lock.lock().await;
                stack_left = slot_stack_guard.clone();
                drop(slot_stack_guard);

                let mut slot_stack_mut = slot_stack_lock.lock().await;
                let container_slots = i32::from(self.rows * self.columns);

                if slot_index < container_slots {
                    // From container to player
                    if !self.get_behaviour().allow_grab_items {
                        return ItemStack::EMPTY.clone();
                    }
                    if !self
                        .insert_item(
                            &mut slot_stack_mut,
                            container_slots,
                            self.get_behaviour().slots.len() as i32,
                            true,
                        )
                        .await
                    {
                        return ItemStack::EMPTY.clone();
                    }
                } else {
                    // From player to container
                    if !self.get_behaviour().allow_put_items {
                        return ItemStack::EMPTY.clone();
                    }
                    if !self
                        .insert_item(&mut slot_stack_mut, 0, container_slots, false)
                        .await
                    {
                        // Move from player area to inventory (start)
                        return ItemStack::EMPTY.clone();
                    }
                }

                if slot_stack_mut.is_empty() {
                    drop(slot_stack_mut);
                    slot.set_stack(ItemStack::EMPTY.clone()).await;
                } else {
                    drop(slot_stack_mut);
                    slot.mark_dirty().await;
                }
            }

            stack_left
        })
    }
}
