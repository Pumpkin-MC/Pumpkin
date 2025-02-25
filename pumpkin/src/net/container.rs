use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_data::item::Item;
use pumpkin_data::screen::WindowType;
use pumpkin_inventory::Container;
use pumpkin_inventory::container_click::{
    Click, ClickType, DropType, KeyClick, MouseClick, MouseDragState, MouseDragType,
};
use pumpkin_inventory::drag_handler::DragHandler;
use pumpkin_inventory::window_property::{WindowProperty, WindowPropertyTrait};
use pumpkin_inventory::{InventoryError, OptionallyCombinedContainer, container_click};
use pumpkin_protocol::client::play::{
    CCloseContainer, COpenScreen, CSetContainerContent, CSetContainerProperty, CSetContainerSlot,
};
use pumpkin_protocol::codec::slot::Slot;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::server::play::SClickContainer;
use pumpkin_util::GameMode;
use pumpkin_util::text::TextComponent;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

impl Player {
    pub async fn open_container(&self, server: &Server, window_type: WindowType) {
        let mut inventory = self.inventory().lock().await;
        //inventory.state_id = 0;
        inventory.increment_state_id();
        inventory.total_opened_containers += 1;
        let mut container = self.get_open_container(server).await;
        let mut container = match container.as_mut() {
            Some(container) => Some(container.lock().await),
            None => None,
        };
        let window_title = container.as_ref().map_or_else(
            || inventory.window_name(),
            |container| container.window_name(),
        );
        let title = TextComponent::text(window_title);

        self.client
            .send_packet(&COpenScreen::new(
                inventory.total_opened_containers.into(),
                VarInt(window_type as i32),
                &title,
            ))
            .await;
        drop(inventory);
        self.set_container_content(container.as_deref_mut()).await;
    }

    pub async fn set_container_content(&self, container: Option<&mut Box<dyn Container>>) {
        let mut inventory = self.inventory().lock().await;

        let total_opened_containers = inventory.total_opened_containers;
        let id = if container.is_some() {
            total_opened_containers
        } else {
            0
        };

        let container = OptionallyCombinedContainer::new(&mut inventory, container);

        let slots: Vec<Slot> = container
            .all_slots_ref()
            .into_iter()
            .map(Slot::from)
            .collect();

        let carried_item = self
            .carried_item
            .load()
            .as_ref()
            .map_or_else(Slot::empty, std::convert::Into::into);

        inventory.increment_state_id();
        let packet = CSetContainerContent::new(
            id.into(),
            (inventory.state_id).into(),
            &slots,
            &carried_item,
        );
        self.client.send_packet(&packet).await;
    }

    /// The official Minecraft client is weird, and will always just close *any* window that is opened when this gets sent
    // TODO: is this just bc ids are not synced?
    pub async fn close_container(&self) {
        let mut inventory = self.inventory().lock().await;
        inventory.total_opened_containers += 1;
        self.client
            .send_packet(&CCloseContainer::new(
                inventory.total_opened_containers.into(),
            ))
            .await;
    }

    pub async fn set_container_property<T: WindowPropertyTrait>(
        &mut self,
        window_property: WindowProperty<T>,
    ) {
        let (id, value) = window_property.into_tuple();
        self.client
            .send_packet(&CSetContainerProperty::new(
                self.inventory().lock().await.total_opened_containers.into(),
                id,
                value,
            ))
            .await;
    }

    pub async fn handle_click_container(
        &self,
        server: &Arc<Server>,
        packet: SClickContainer,
    ) -> Result<(), InventoryError> {
        let opened_container = self.get_open_container(server).await;
        let mut opened_container = match opened_container.as_ref() {
            Some(container) => Some(container.lock().await),
            None => None,
        };
        let drag_handler = &server.drag_handler;

        let state_id = self.inventory().lock().await.state_id;
        // This is just checking for regular desync, client hasn't done anything malicious
        if state_id != packet.state_id.0 as u32 {
            self.set_container_content(opened_container.as_deref_mut())
                .await;
            return Ok(());
        }

        if opened_container.is_some() {
            let total_containers = self.inventory().lock().await.total_opened_containers;
            if packet.window_id.0 != total_containers {
                return Err(InventoryError::ClosedContainerInteract(self.entity_id()));
            }
        } else if packet.window_id.0 != 0 {
            return Err(InventoryError::ClosedContainerInteract(self.entity_id()));
        }

        let click = Click::new(packet.mode, packet.button, packet.slot)?;
        let (crafted_item, crafted_item_slot) = {
            let mut inventory = self.inventory().lock().await;
            let combined =
                OptionallyCombinedContainer::new(&mut inventory, opened_container.as_deref_mut());
            (
                combined.crafted_item_slot(),
                combined.crafting_output_slot(),
            )
        };
        let crafted_is_picked = crafted_item.is_some()
            && match click.slot {
                container_click::Slot::Normal(slot) => {
                    crafted_item_slot.is_some_and(|crafted_slot| crafted_slot == slot)
                }
                container_click::Slot::OutsideInventory => false,
            };
        let mut update_whole_container = false;

        let click_slot = click.slot;
        self.match_click_behaviour(
            server,
            opened_container.as_deref_mut(),
            click,
            drag_handler,
            &mut update_whole_container,
            crafted_is_picked,
        )
        .await?;
        // Checks for if crafted item has been taken
        {
            let mut inventory = self.inventory().lock().await;
            let mut combined =
                OptionallyCombinedContainer::new(&mut inventory, opened_container.as_deref_mut());
            if combined.crafted_item_slot().is_none() && crafted_item.is_some() {
                combined.recipe_used();
            }

            // TODO: `combined.craft` uses rayon! It should be called from `rayon::spawn` and its
            // result passed to the tokio runtime via a channel!
            if combined.craft() {
                drop(inventory);
                self.set_container_content(opened_container.as_deref_mut())
                    .await;
            }
        }

        if let Some(mut opened_container) = opened_container {
            if update_whole_container {
                drop(opened_container);
                self.send_whole_container_change(server).await?;
            } else if let container_click::Slot::Normal(slot_index) = click_slot {
                let mut inventory = self.inventory().lock().await;
                let combined_container =
                    OptionallyCombinedContainer::new(&mut inventory, Some(&mut opened_container));
                if let Some(slot) = combined_container.get_slot_excluding_inventory(slot_index) {
                    let slot = Slot::from(slot);
                    drop(opened_container);
                    self.send_container_changes(server, slot_index, slot)
                        .await?;
                }
            }
        }
        Ok(())
    }

    pub async fn handle_decrease_item(
        &self,
        _server: &Server,
        slot_index: i16,
        item_stack: Option<&ItemStack>,
        state_id: &mut u32,
    ) -> Result<(), InventoryError> {
        // TODO: this will not update hotbar when server admin is peeking
        // TODO: check and iterate over all players in player inventory
        let slot = Slot::from(item_stack);
        *state_id += 1;
        let packet = CSetContainerSlot::new(0, *state_id as i32, slot_index, &slot);
        self.client.send_packet(&packet).await;
        Ok(())
    }

    async fn match_click_behaviour(
        &self,
        server: &Server,
        opened_container: Option<&mut Box<dyn Container>>,
        click: Click,
        drag_handler: &DragHandler,
        update_whole_container: &mut bool,
        using_crafting_slot: bool,
    ) -> Result<(), InventoryError> {
        match click.click_type {
            ClickType::MouseClick(mouse_click) => {
                self.mouse_click(
                    server,
                    opened_container,
                    mouse_click,
                    click.slot,
                    using_crafting_slot,
                )
                .await
            }
            ClickType::ShiftClick => {
                self.shift_mouse_click(opened_container, click.slot, using_crafting_slot)
                    .await
            }
            ClickType::KeyClick(key_click) => match click.slot {
                container_click::Slot::Normal(slot) => {
                    self.number_button_pressed(
                        opened_container,
                        key_click,
                        slot,
                        using_crafting_slot,
                    )
                    .await
                }
                container_click::Slot::OutsideInventory => Err(InventoryError::InvalidPacket),
            },
            ClickType::CreativePickItem => {
                if let container_click::Slot::Normal(slot) = click.slot {
                    self.creative_pick_item(opened_container, slot).await
                } else {
                    Err(InventoryError::InvalidPacket)
                }
            }
            ClickType::DoubleClick => {
                *update_whole_container = true;
                if let container_click::Slot::Normal(slot) = click.slot {
                    self.double_click(opened_container, slot).await
                } else {
                    Err(InventoryError::InvalidPacket)
                }
            }
            ClickType::MouseDrag { drag_state } => {
                if drag_state == MouseDragState::End {
                    *update_whole_container = true;
                }
                self.mouse_drag(drag_handler, opened_container, drag_state)
                    .await
            }
            ClickType::DropType(drop_type) => {
                if let container_click::Slot::Normal(slot) = click.slot {
                    let mut inventory = self.inventory().lock().await;
                    let mut container =
                        OptionallyCombinedContainer::new(&mut inventory, opened_container);
                    let mut slots = container.all_slots();

                    if let Some(item) = slots[slot].as_mut() {
                        match drop_type {
                            DropType::FullStack => {
                                self.drop_item(server, item.to_owned()).await;
                                *slots[slot] = None;
                            }
                            DropType::SingleItem => {
                                self.drop_item(server, ItemStack::new(1, item.item)).await;
                                item.item_count -= 1;
                                if item.item_count == 0 {
                                    *slots[slot] = None;
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    }

    async fn mouse_click(
        &self,
        server: &Server,
        opened_container: Option<&mut Box<dyn Container>>,
        mouse_click: MouseClick,
        slot: container_click::Slot,
        taking_crafted: bool,
    ) -> Result<(), InventoryError> {
        let mut inventory = self.inventory().lock().await;
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);
        let mut carried_item = self.carried_item.load();
        match slot {
            container_click::Slot::Normal(slot) => {
                let res = container.handle_item_change(
                    &mut carried_item,
                    slot,
                    mouse_click,
                    taking_crafted,
                );
                self.carried_item.store(carried_item);
                res
            }
            container_click::Slot::OutsideInventory => {
                if let Some(item) = carried_item {
                    match mouse_click {
                        MouseClick::Left => {
                            self.drop_item(server, item).await;
                            self.carried_item.store(None);
                        }
                        MouseClick::Right => {
                            let mut item = item;
                            self.drop_item(server, ItemStack::new(1, item.item)).await;
                            item.item_count -= 1;
                            if item.item_count == 0 {
                                self.carried_item.store(None);
                            } else {
                                self.carried_item.store(Some(item));
                            }
                        }
                    };
                }
                Ok(())
            }
        }
    }

    /// TODO: Allow equiping/de equiping armor and allow taking items from crafting grid
    async fn shift_mouse_click(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        slot: container_click::Slot,
        _taking_crafted: bool,
    ) -> Result<(), InventoryError> {
        let mut inventory = self.inventory().lock().await;
        let has_container = opened_container.is_some();
        let container_size = opened_container
            .as_ref()
            .map_or(0, |c| c.all_slots_ref().len());
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);

        match slot {
            container_click::Slot::Normal(slot) => {
                let mut all_slots = container.all_slots();

                // Get the item data we need before any mutable operations
                let (item, max_stack, initial_count) =
                    if let Some(item_in_pressed_slot) = all_slots[slot].as_ref() {
                        (
                            item_in_pressed_slot.item,
                            item_in_pressed_slot.item.components.max_stack_size,
                            item_in_pressed_slot.item_count,
                        )
                    } else {
                        return Ok(());
                    };

                let mut remaining_count = initial_count;

                // Define the two inventories and determine which one contains the source slot
                let (inv1_range, inv2_range) = if has_container {
                    // When container is open:
                    // Inv1 = Container slots (0 to container_size-1)
                    // Inv2 = Player inventory (container_size to end)
                    ((0..container_size), (container_size..all_slots.len()))
                } else {
                    // When no container:
                    // Inv1 = Hotbar (36-45)
                    // Inv2 = Main inventory (9-36)
                    ((36..45), (9..36))
                };

                let inv_1_clone = inv1_range.clone();

                // Determine which inventory we're moving from and to
                let (source_inv, target_inv) = if inv1_range.contains(&slot) {
                    (inv1_range, inv2_range)
                } else if inv2_range.contains(&slot) {
                    (inv2_range, inv1_range)
                } else {
                    // When moving from top slots to inventory
                    ((0..9), (9..45))
                };

                // If moving to hotbar, reverse the order to fill from right to left
                let target_slots: Vec<usize> =
                    if has_container && source_inv.contains(&slot) && source_inv == inv_1_clone {
                        target_inv.rev().collect()
                    } else {
                        target_inv.collect()
                    };

                //Handle armor slots
                if !has_container {
                    let temp_item_stack = ItemStack::new(1, item);
                    if slot != 5 && temp_item_stack.is_helmet() && all_slots[5].is_none() {
                        *all_slots[5] = Some(temp_item_stack);
                        *all_slots[slot] = None;
                        return Ok(());
                    } else if slot != 6 && temp_item_stack.is_chestplate() && all_slots[6].is_none()
                    {
                        *all_slots[6] = Some(temp_item_stack);
                        *all_slots[slot] = None;
                        return Ok(());
                    } else if slot != 7 && temp_item_stack.is_leggings() && all_slots[7].is_none() {
                        *all_slots[7] = Some(temp_item_stack);
                        *all_slots[slot] = None;
                        return Ok(());
                    } else if slot != 8 && temp_item_stack.is_boots() && all_slots[8].is_none() {
                        *all_slots[8] = Some(temp_item_stack);
                        *all_slots[slot] = None;
                        return Ok(());
                    }
                }

                // First try to stack with existing items
                for target_idx in &target_slots {
                    if let Some(target_item) = all_slots[*target_idx].as_mut() {
                        if target_item.item.id == item.id && target_item.item_count < max_stack {
                            let space_in_stack = max_stack - target_item.item_count;
                            let amount_to_add = remaining_count.min(space_in_stack);
                            target_item.item_count += amount_to_add;
                            remaining_count -= amount_to_add;

                            if remaining_count == 0 {
                                *all_slots[slot] = None;
                                return Ok(());
                            }
                        }
                    }
                }

                // Then try to place in empty slots
                for target_idx in &target_slots {
                    if all_slots[*target_idx].is_none()
                        || all_slots[*target_idx].is_some_and(|item| item.item_count == 0)
                    {
                        *all_slots[*target_idx] = Some(ItemStack {
                            item,
                            item_count: remaining_count,
                        });
                        *all_slots[slot] = None;
                        return Ok(());
                    }
                }

                // Update original slot if we moved some but not all items
                if remaining_count != initial_count {
                    *all_slots[slot] = Some(ItemStack {
                        item,
                        item_count: remaining_count,
                    });
                }
            }
            container_click::Slot::OutsideInventory => (),
        };
        Ok(())
    }

    async fn number_button_pressed(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        key_click: KeyClick,
        slot: usize,
        taking_crafted: bool,
    ) -> Result<(), InventoryError> {
        let changing_slot = match key_click {
            KeyClick::Slot(slot) => slot,
            KeyClick::Offhand => 45,
        };
        let mut inventory = self.inventory().lock().await;
        let mut changing_item_slot = inventory.get_slot(changing_slot as usize)?.to_owned();
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);

        container.handle_item_change(
            &mut changing_item_slot,
            slot,
            MouseClick::Left,
            taking_crafted,
        )?;
        *inventory.get_slot(changing_slot as usize)? = changing_item_slot;
        Ok(())
    }

    async fn creative_pick_item(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        slot: usize,
    ) -> Result<(), InventoryError> {
        if self.gamemode.load() != GameMode::Creative {
            return Err(InventoryError::PermissionError);
        }
        let mut inventory = self.inventory().lock().await;
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);
        if let Some(Some(item)) = container.all_slots().get_mut(slot) {
            self.carried_item.store(Some(item.to_owned()));
        }
        Ok(())
    }

    async fn double_click(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        _slot: usize,
    ) -> Result<(), InventoryError> {
        let mut inventory = self.inventory().lock().await;
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);

        let Some(mut carried_item) = self.carried_item.load() else {
            return Ok(());
        };

        // Iterate directly over container slots to modify them in place'
        for slot in container.all_slots() {
            if let Some(itemstack) = slot {
                if itemstack.item.id == carried_item.item.id {
                    if itemstack.item_count + carried_item.item_count
                        <= carried_item.item.components.max_stack_size
                    {
                        carried_item.item_count += itemstack.item_count;
                        *slot = None;
                    } else {
                        let overflow = itemstack.item_count
                            - (carried_item.item.components.max_stack_size
                                - carried_item.item_count);
                        carried_item.item_count = carried_item.item.components.max_stack_size;
                        itemstack.item_count = overflow;
                    }

                    if carried_item.item_count == carried_item.item.components.max_stack_size {
                        break;
                    }
                }
            }
        }
        self.carried_item.store(Some(carried_item));
        Ok(())
    }

    async fn mouse_drag(
        &self,
        drag_handler: &DragHandler,
        opened_container: Option<&mut Box<dyn Container>>,
        mouse_drag_state: MouseDragState,
    ) -> Result<(), InventoryError> {
        let player_id = self.entity_id();
        let container_id = opened_container
            .as_ref()
            .map_or(player_id as u64, |container| {
                container.internal_pumpkin_id()
            });
        match mouse_drag_state {
            MouseDragState::Start(drag_type) => {
                if drag_type == MouseDragType::Middle && self.gamemode.load() != GameMode::Creative
                {
                    Err(InventoryError::PermissionError)?;
                }
                drag_handler
                    .new_drag(container_id, player_id, drag_type)
                    .await
            }
            MouseDragState::AddSlot(slot) => {
                drag_handler.add_slot(container_id, player_id, slot).await
            }
            MouseDragState::End => {
                let mut inventory = self.inventory().lock().await;
                let mut container =
                    OptionallyCombinedContainer::new(&mut inventory, opened_container);
                let mut carried_item = self.carried_item.load();
                let res = drag_handler
                    .apply_drag(&mut carried_item, &mut container, &container_id, player_id)
                    .await;
                self.carried_item.store(carried_item);
                res
            }
        }
    }

    async fn get_current_players_in_container(&self, server: &Server) -> Vec<Arc<Self>> {
        let player_ids: Vec<i32> = {
            let open_containers = server.open_containers.read().await;
            open_containers
                .get(&self.open_container.load().unwrap())
                .unwrap()
                .all_player_ids()
                .into_iter()
                .filter(|player_id| *player_id != self.entity_id())
                .collect()
        };
        let player_token = self.gameprofile.id;

        // TODO: Figure out better way to get only the players from player_ids
        // Also refactor out a better method to get individual advanced state ids

        let players = self
            .living_entity
            .entity
            .world
            .read()
            .await
            .players
            .read()
            .await
            .iter()
            .filter_map(|(token, player)| {
                if *token == player_token {
                    None
                } else {
                    let entity_id = player.entity_id();
                    player_ids.contains(&entity_id).then(|| player.clone())
                }
            })
            .collect();
        players
    }

    pub async fn send_container_changes(
        &self,
        server: &Server,
        slot_index: usize,
        slot: Slot,
    ) -> Result<(), InventoryError> {
        for player in self.get_current_players_in_container(server).await {
            let mut inventory = player.inventory().lock().await;
            let total_opened_containers = inventory.total_opened_containers;

            // Returns previous value
            inventory.increment_state_id();
            let packet = CSetContainerSlot::new(
                total_opened_containers as i8,
                (inventory.state_id) as i32,
                slot_index as i16,
                &slot,
            );
            player.client.send_packet(&packet).await;
        }
        Ok(())
    }

    pub async fn send_whole_container_change(&self, server: &Server) -> Result<(), InventoryError> {
        let players = self.get_current_players_in_container(server).await;

        for player in players {
            let container = player.get_open_container(server).await;
            let mut container = match container.as_ref() {
                Some(container) => Some(container.lock().await),
                None => None,
            };
            player.set_container_content(container.as_deref_mut()).await;
        }
        Ok(())
    }

    pub async fn get_open_container(
        &self,
        server: &Server,
    ) -> Option<Arc<tokio::sync::Mutex<Box<dyn Container>>>> {
        match self.open_container.load() {
            Some(id) => server.try_get_container(self.entity_id(), id).await,
            None => None,
        }
    }

    async fn pickup_items(&self, item: Item, mut amount: u32) {
        let max_stack = item.components.max_stack_size;
        let mut inventory = self.inventory().lock().await;
        let slots = inventory.slots_with_hotbar_first();

        let matching_slots = slots.filter_map(|slot| {
            if let Some(item_slot) = slot.as_mut() {
                (item_slot.item.id == item.id && item_slot.item_count < max_stack).then(|| {
                    let item_count = item_slot.item_count;
                    (item_slot, item_count)
                })
            } else {
                None
            }
        });

        for (slot, item_count) in matching_slots {
            if amount == 0 {
                return;
            }
            let amount_to_add = max_stack - item_count;
            if let Some(amount_left) = amount.checked_sub(u32::from(amount_to_add)) {
                amount = amount_left;
                *slot = ItemStack {
                    item,
                    item_count: item.components.max_stack_size,
                };
            } else {
                *slot = ItemStack {
                    item,
                    item_count: max_stack - (amount_to_add - amount as u8),
                };
                return;
            }
        }

        let empty_slots = inventory
            .slots_with_hotbar_first()
            .filter(|slot| slot.is_none());
        for slot in empty_slots {
            if amount == 0 {
                return;
            }
            if let Some(remaining_amount) = amount.checked_sub(u32::from(max_stack)) {
                amount = remaining_amount;
                *slot = Some(ItemStack {
                    item,
                    item_count: max_stack,
                });
            } else {
                *slot = Some(ItemStack {
                    item,
                    item_count: amount as u8,
                });
                return;
            }
        }
        log::warn!(
            "{amount} items were discarded because dropping them to the ground is not implemented"
        );
    }

    /// Add items to inventory if there's space, else drop them to the ground.
    ///
    /// This method automatically syncs changes with the client.
    pub async fn give_items(&self, item: Item, amount: u32) {
        self.pickup_items(item, amount).await;
        self.set_container_content(None).await;
    }
}
