use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_data::item::Item;
use pumpkin_data::screen::WindowType;
use pumpkin_inventory::Container;
use pumpkin_inventory::container_click::{
    Click, ClickType, DropType, KeyClick, MouseClick, MouseDragState, MouseDragType,
};
use pumpkin_inventory::drag_handler::DragHandler;
use pumpkin_inventory::player::{SLOT_BOOT, SLOT_CHEST, SLOT_HELM, SLOT_HOTBAR_START, SLOT_LEG};
use pumpkin_inventory::window_property::{WindowProperty, WindowPropertyTrait};
use pumpkin_inventory::{InventoryError, OptionallyCombinedContainer, container_click};
use pumpkin_protocol::client::play::{
    CCloseContainer, COpenScreen, CSetContainerContent, CSetContainerProperty, CSetContainerSlot,
};
use pumpkin_protocol::codec::slot::Slot;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::server::play::SClickContainer;
use pumpkin_util::text::TextComponent;
use pumpkin_util::{GameMode, MutableSplitSlice};
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
            .enqueue_packet(&COpenScreen::new(
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

        let carried_item = self.carried_item.lock().await;
        let carried_item = carried_item
            .as_ref()
            .map_or_else(Slot::empty, std::convert::Into::into);

        inventory.increment_state_id();
        let packet = CSetContainerContent::new(
            id.into(),
            (inventory.state_id).into(),
            &slots,
            &carried_item,
        );
        self.client.enqueue_packet(&packet).await;
    }

    /// The official Minecraft client is weird, and will always just close *any* window that is opened when this gets sent
    // TODO: is this just bc ids are not synced?
    pub async fn close_container(&self) {
        let mut inventory = self.inventory().lock().await;
        inventory.total_opened_containers += 1;
        self.client
            .enqueue_packet(&CCloseContainer::new(
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
            .enqueue_packet(&CSetContainerProperty::new(
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
                combined.crafted_item_slot().cloned(),
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
        self.client.enqueue_packet(&packet).await;
        Ok(())
    }

    async fn match_click_behaviour(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        click: Click,
        drag_handler: &DragHandler,
        update_whole_container: &mut bool,
        using_crafting_slot: bool,
    ) -> Result<(), InventoryError> {
        match click.click_type {
            ClickType::MouseClick(mouse_click) => {
                self.mouse_click(
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
                    let slots = container.all_slots();

                    if let Some(item_stack) = slots[slot].as_mut() {
                        match drop_type {
                            DropType::FullStack => {
                                self.drop_item(
                                    item_stack.item.id,
                                    u32::from(item_stack.item_count),
                                )
                                .await;
                                *slots[slot] = None;
                            }
                            DropType::SingleItem => {
                                self.drop_item(item_stack.item.id, 1).await;
                                item_stack.item_count -= 1;
                                if item_stack.item_count == 0 {
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
        opened_container: Option<&mut Box<dyn Container>>,
        mouse_click: MouseClick,
        slot: container_click::Slot,
        taking_crafted: bool,
    ) -> Result<(), InventoryError> {
        let mut inventory = self.inventory().lock().await;
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);
        let mut carried_item = self.carried_item.lock().await;
        match slot {
            container_click::Slot::Normal(slot) => {
                container.handle_item_change(&mut carried_item, slot, mouse_click, taking_crafted)
            }
            container_click::Slot::OutsideInventory => {
                if let Some(item_stack) = carried_item.as_mut() {
                    match mouse_click {
                        MouseClick::Left => {
                            self.drop_item(item_stack.item.id, u32::from(item_stack.item_count))
                                .await;
                            *carried_item = None;
                        }
                        MouseClick::Right => {
                            self.drop_item(item_stack.item.id, 1).await;
                            item_stack.item_count -= 1;
                            if item_stack.item_count == 0 {
                                *carried_item = None;
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
                let (item_stack, mut split_slice) =
                    MutableSplitSlice::extract_ith(&mut all_slots, slot);
                let Some(clicked_item_stack) = item_stack else {
                    return Ok(());
                };

                // Define the two inventories and determine which one contains the source slot
                let (inv1_range, inv2_range) = if has_container {
                    // When container is open:
                    // Inv1 = Container slots (0 to container_size-1)
                    // Inv2 = Player inventory (container_size to end)
                    ((0..container_size), (container_size..split_slice.len()))
                } else {
                    // When no container:
                    // Inv1 = Hotbar (36-45)
                    // Inv2 = Main inventory (9-36)
                    ((36..45), (9..36))
                };

                // Determine which inventory we're moving from and to
                let (source_inv, target_inv) = if inv1_range.contains(&slot) {
                    (&inv1_range, &inv2_range)
                } else if inv2_range.contains(&slot) {
                    (&inv2_range, &inv1_range)
                } else {
                    // When moving from top slots to inventory
                    (&(0..9), &(9..45))
                };

                // If moving to hotbar, reverse the order to fill from right to left
                let target_slots: Vec<usize> =
                    if has_container && source_inv.contains(&slot) && source_inv == &inv1_range {
                        target_inv.clone().rev().collect()
                    } else {
                        target_inv.clone().collect()
                    };

                //Handle armor slots
                if !has_container {
                    let temp_item_stack = ItemStack::new(1, clicked_item_stack.item.clone());
                    if slot != SLOT_HELM
                        && temp_item_stack.is_helmet()
                        && split_slice[SLOT_HELM].is_none()
                    {
                        *split_slice[SLOT_HELM] = Some(temp_item_stack);
                        **item_stack = None;
                        return Ok(());
                    } else if slot != SLOT_CHEST
                        && temp_item_stack.is_chestplate()
                        && split_slice[SLOT_CHEST].is_none()
                    {
                        *split_slice[SLOT_CHEST] = Some(temp_item_stack);
                        **item_stack = None;
                        return Ok(());
                    } else if slot != SLOT_LEG
                        && temp_item_stack.is_leggings()
                        && split_slice[SLOT_LEG].is_none()
                    {
                        *split_slice[SLOT_LEG] = Some(temp_item_stack);
                        **item_stack = None;
                        return Ok(());
                    } else if slot != SLOT_BOOT
                        && temp_item_stack.is_boots()
                        && split_slice[SLOT_BOOT].is_none()
                    {
                        *split_slice[SLOT_BOOT] = Some(temp_item_stack);
                        **item_stack = None;
                        return Ok(());
                    }
                }

                // First try to stack with existing items
                let max_stack_size = clicked_item_stack.item.components.max_stack_size;
                for target_idx in &target_slots {
                    if let Some(target_item) = split_slice[*target_idx].as_mut() {
                        if target_item.item.id == clicked_item_stack.item.id
                            && target_item.item_count < max_stack_size
                        {
                            let space_in_stack = max_stack_size - target_item.item_count;
                            let amount_to_add = clicked_item_stack.item_count.min(space_in_stack);
                            target_item.item_count += amount_to_add;
                            clicked_item_stack.item_count -= amount_to_add;

                            if clicked_item_stack.item_count == 0 {
                                **item_stack = None;
                                return Ok(());
                            }
                        }
                    }
                }

                // Then try to place in empty slots
                for target_idx in target_slots {
                    if split_slice[target_idx].is_none()
                        || split_slice[target_idx]
                            .as_ref()
                            .is_some_and(|item| item.item_count == 0)
                    {
                        std::mem::swap(split_slice[target_idx], *item_stack);
                        return Ok(());
                    }
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
            KeyClick::Slot(slot) => slot as usize + SLOT_HOTBAR_START,
            KeyClick::Offhand => 45,
        };
        let mut inventory = self.inventory().lock().await;
        let mut changing_item_slot = inventory.get_slot(changing_slot)?.clone();
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);

        container.handle_item_change(
            &mut changing_item_slot,
            slot,
            MouseClick::Left,
            taking_crafted,
        )?;
        *inventory.get_slot(changing_slot)? = changing_item_slot;
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
            let mut carried_item = self.carried_item.lock().await;
            *carried_item = Some(item.clone());
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
        let mut carried_item = self.carried_item.lock().await;
        let Some(carried_item) = carried_item.as_mut() else {
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
                let mut carried_item = self.carried_item.lock().await;
                drag_handler
                    .apply_drag(&mut carried_item, &mut container, &container_id, player_id)
                    .await
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
            player.client.enqueue_packet(&packet).await;
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

    // TODO: Use this method when actually picking up items instead of just the command
    async fn pickup_items(&self, item: Item, amount: u32) {
        let mut amount_left = amount;
        let max_stack = item.components.max_stack_size;
        let mut inventory = self.inventory().lock().await;

        while let Some(slot) = inventory.get_pickup_item_slot(item.id) {
            let item_stack = inventory
                .get_slot(slot)
                .expect("We just called a method that said this was a valid slot");

            if let Some(item_stack) = item_stack {
                let amount_to_add = max_stack - item_stack.item_count;
                if let Some(new_amount_left) = amount_left.checked_sub(u32::from(amount_to_add)) {
                    item_stack.item_count = max_stack;
                    amount_left = new_amount_left;
                } else {
                    // This is safe because amount left is less than amount_to_add which is a u8
                    item_stack.item_count = max_stack - (amount_to_add - amount_left as u8);
                    // Return here because if we have less than the max amount left then the whole
                    // stack will be moved
                    return;
                }
            } else if let Some(new_amount_left) = amount_left.checked_sub(u32::from(max_stack)) {
                *item_stack = Some(ItemStack {
                    item: item.clone(),
                    item_count: max_stack,
                });
                amount_left = new_amount_left;
            } else {
                *item_stack = Some(ItemStack {
                    item: item.clone(),
                    // This is safe because amount left is less than max_stack which is a u8
                    item_count: amount_left as u8,
                });
                // Return here because if we have less than the max amount left then the whole
                // stack will be moved
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
