use super::Player;
use crate::plugin::player::inventory_interact::InventoryClickEvent;
use crate::server::Server;
use pumpkin_data::screen::WindowType;
use pumpkin_inventory::player::player_screen_handler::PlayerScreenHandler;
use pumpkin_inventory::screen_handler::ClickType;
use pumpkin_inventory::screen_handler::ScreenHandler;
use pumpkin_inventory::screen_handler::ScreenHandlerFactory;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::bedrock::client::container_open::CContainerOpen;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::java::client::play::CCloseContainer;
use pumpkin_protocol::java::client::play::COpenScreen;
use pumpkin_protocol::java::server::play::SClickSlot;
use pumpkin_protocol::java::server::play::SContainerButtonClick;
use pumpkin_protocol::java::server::play::SRenameItem;
use pumpkin_protocol::java::server::play::SlotActionType;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;
use tracing::warn;

impl Player {
    pub fn increment_screen_handler_sync_id(&self) {
        let current_id = self.screen_handler_sync_id.load(Ordering::Relaxed);
        self.screen_handler_sync_id
            .store(current_id % 100 + 1, Ordering::Relaxed);
    }

    pub async fn close_handled_screen(self: &Arc<Self>) {
        let (sync_id, bedrock_window_type) = {
            let current_handler_guard = self.current_screen_handler.lock().await;
            let handler = current_handler_guard.lock().await;
            let sync_id = handler.sync_id();
            let window_type = handler.window_type();
            let bedrock_window_type = match window_type {
                Some(WindowType::Crafting) => 1,
                Some(WindowType::Furnace) => 2,
                Some(WindowType::Enchantment) => 3,
                Some(WindowType::BrewingStand) => 4,
                Some(WindowType::Anvil) => 5,
                Some(WindowType::Hopper) => 8,
                Some(WindowType::Beacon) => 13,
                Some(WindowType::BlastFurnace) => 27,
                Some(WindowType::Smoker) => 28,
                Some(WindowType::Stonecutter) => 29,
                Some(WindowType::CartographyTable) => 30,
                Some(WindowType::Grindstone) => 26,
                Some(WindowType::Loom) => 24,
                Some(WindowType::Smithing) => 34,
                _ => 0,
            };
            (sync_id, bedrock_window_type)
        };

        self.client
            .enqueue_packet_editioned(
                &CCloseContainer::new(sync_id.into()),
                &pumpkin_protocol::bedrock::server::container_close::SContainerClose {
                    container_id: sync_id,
                    container_type: bedrock_window_type,
                    server_initiated: true,
                },
            )
            .await;
        self.on_handled_screen_closed().await;
    }

    pub async fn on_handled_screen_closed(self: &Arc<Self>) {
        let current_screen_handler: Arc<Mutex<dyn ScreenHandler>> =
            self.current_screen_handler.lock().await.clone();

        let window_type = {
            let mut handler = current_screen_handler.lock().await;
            let wt = handler.window_type();
            handler.on_closed(self.as_ref()).await;
            wt
        };

        if let Some(server) = self.living_entity.entity.world.load().server.upgrade() {
            server
                .plugin_manager
                .fire(
                    crate::plugin::api::events::player::inventory_close::InventoryCloseEvent::new(
                        self,
                        window_type,
                    ),
                )
                .await;
        }

        let player_screen_handler: Arc<Mutex<dyn ScreenHandler>> =
            self.player_screen_handler.clone();

        if !Arc::ptr_eq(&player_screen_handler, &current_screen_handler) {
            player_screen_handler
                .lock()
                .await
                .copy_shared_slots(current_screen_handler)
                .await;
        }

        *self.current_screen_handler.lock().await = self.player_screen_handler.clone();
        self.open_container_pos.store(None);
    }

    pub async fn on_screen_handler_opened(&self, screen_handler: Arc<Mutex<dyn ScreenHandler>>) {
        let mut screen_handler = screen_handler.lock().await;

        screen_handler
            .add_listener(self.screen_handler_listener.clone())
            .await;

        screen_handler
            .update_sync_handler(self.screen_handler_sync_handler.clone())
            .await;
    }

    pub async fn on_rename_item(self: &Arc<Self>, packet: SRenameItem<'_>) {
        self.update_last_action_time();
        let screen_handler_arc = self.current_screen_handler.lock().await.clone();
        let mut screen_handler = screen_handler_arc.lock().await;

        if let Some(anvil_handler) = screen_handler
            .as_any_mut()
            .downcast_mut::<pumpkin_inventory::anvil::AnvilScreenHandler>()
        {
            anvil_handler
                .update_item_name(packet.item_name.to_string())
                .await;
        }
    }

    pub async fn open_handled_screen(
        self: &Arc<Self>,
        screen_handler_factory: &dyn ScreenHandlerFactory,
        block_pos: Option<BlockPos>,
    ) -> Option<u8> {
        if !self
            .current_screen_handler
            .lock()
            .await
            .lock()
            .await
            .as_any()
            .is::<PlayerScreenHandler>()
        {
            self.close_handled_screen().await;
        }

        self.increment_screen_handler_sync_id();

        if let Some(screen_handler) = screen_handler_factory
            .create_screen_handler(
                self.screen_handler_sync_id.load(Ordering::Relaxed),
                &self.inventory,
                self.as_ref(),
            )
            .await
        {
            let screen_handler_temp = screen_handler.lock().await;
            let sync_id = screen_handler_temp.sync_id();
            let window_type = screen_handler_temp
                .window_type()
                .expect("Can't open PlayerScreenHandler");

            let display_name = screen_handler_factory.get_display_name();
            let java_packet =
                COpenScreen::new(sync_id.into(), (window_type as i32).into(), &display_name);

            let bedrock_window_type = match window_type {
                WindowType::Crafting => 1,
                WindowType::Furnace => 2,
                WindowType::Enchantment => 3,
                WindowType::BrewingStand => 4,
                WindowType::Anvil => 5,
                WindowType::Hopper => 8,
                WindowType::Beacon => 13,
                WindowType::BlastFurnace => 27,
                WindowType::Smoker => 28,
                WindowType::Stonecutter => 29,
                WindowType::CartographyTable => 30,
                WindowType::Grindstone => 26,
                WindowType::Loom => 24,
                WindowType::Smithing => 34,
                _ => 0,
            };

            let bedrock_packet = CContainerOpen {
                container_id: sync_id,
                container_type: bedrock_window_type,
                position: block_pos.unwrap_or(BlockPos::ZERO),
                target_entity_id: VarLong(-1),
            };

            self.client
                .enqueue_packet_editioned(&java_packet, &bedrock_packet)
                .await;

            drop(screen_handler_temp);
            self.on_screen_handler_opened(screen_handler.clone()).await;
            *self.current_screen_handler.lock().await = screen_handler;
            self.open_container_pos.store(block_pos);
            Some(self.screen_handler_sync_id.load(Ordering::Relaxed))
        } else {
            //TODO: Send message if spectator

            None
        }
    }

    pub async fn open_handled_screen_direct(
        self: &Arc<Self>,
        screen_handler: Arc<Mutex<dyn ScreenHandler>>,
        title: TextComponent,
    ) {
        if !self
            .current_screen_handler
            .lock()
            .await
            .lock()
            .await
            .as_any()
            .is::<PlayerScreenHandler>()
        {
            self.close_handled_screen().await;
        }

        let screen_handler_temp = screen_handler.lock().await;
        let sync_id = screen_handler_temp.sync_id();
        let window_type = screen_handler_temp
            .window_type()
            .expect("Can't open PlayerScreenHandler");

        let java_packet = COpenScreen::new(sync_id.into(), (window_type as i32).into(), &title);

        let bedrock_window_type = match window_type {
            WindowType::Crafting => 1,
            WindowType::Furnace => 2,
            WindowType::Enchantment => 3,
            WindowType::BrewingStand => 4,
            WindowType::Anvil => 5,
            WindowType::Hopper => 8,
            WindowType::Beacon => 13,
            WindowType::BlastFurnace => 27,
            WindowType::Smoker => 28,
            WindowType::Stonecutter => 29,
            WindowType::CartographyTable => 30,
            WindowType::Grindstone => 26,
            WindowType::Loom => 24,
            WindowType::Smithing => 34,
            _ => 0,
        };

        let bedrock_packet = CContainerOpen {
            container_id: sync_id,
            container_type: bedrock_window_type,
            position: BlockPos::ZERO,
            target_entity_id: VarLong(-1),
        };

        self.client
            .enqueue_packet_editioned(&java_packet, &bedrock_packet)
            .await;

        drop(screen_handler_temp);
        self.on_screen_handler_opened(screen_handler.clone()).await;
        *self.current_screen_handler.lock().await = screen_handler;
        self.open_container_pos.store(None);
    }

    #[allow(clippy::too_many_lines)]
    pub async fn on_slot_click(self: &Arc<Self>, packet: SClickSlot, server: &Server) {
        self.update_last_action_time();
        let screen_handler_arc = self.current_screen_handler.lock().await.clone();
        let mut screen_handler = screen_handler_arc.lock().await;

        let (sync_id, container_slots, allow_grab_items, allow_put_items) = {
            let b = screen_handler.get_behaviour();
            (
                b.sync_id,
                b.container_slots,
                b.allow_grab_items,
                b.allow_put_items,
            )
        };

        if i32::from(sync_id) != packet.sync_id.0 {
            return;
        }

        if self.gamemode.load() == GameMode::Spectator {
            screen_handler.sync_state().await;
            return;
        }

        if !screen_handler.can_use(self.as_ref()) {
            warn!(
                "Player {} interacted with invalid menu {:?}",
                self.gameprofile.name,
                screen_handler.window_type()
            );
            return;
        }

        let slot = packet.slot;

        if !screen_handler.is_slot_valid(i32::from(slot)).await {
            warn!(
                "Player {} clicked invalid slot index: {}, available slots: {}",
                self.gameprofile.name,
                slot,
                screen_handler.get_behaviour().slots.len()
            );
            return;
        }

        // Fire InventoryClickEvent
        let clicked_item = if slot >= 0 {
            let slot_obj = &screen_handler.get_behaviour().slots[slot as usize];
            Some(slot_obj.get_cloned_stack().await)
        } else {
            None
        };

        let cursor_item = Some(
            screen_handler
                .get_behaviour()
                .cursor_stack
                .lock()
                .await
                .clone(),
        );
        let raw_slot = slot; // For now raw_slot == slot, as we don't have separate view/inventory indexing yet
        let hotbar_button = if matches!(packet.mode, SlotActionType::Swap) {
            packet.button
        } else {
            -1
        };

        let click_type = match packet.mode {
            SlotActionType::Pickup => {
                if packet.button == 0 {
                    ClickType::Left
                } else {
                    ClickType::Right
                }
            }
            SlotActionType::QuickMove => {
                if packet.button == 0 {
                    ClickType::ShiftLeft
                } else {
                    ClickType::ShiftRight
                }
            }
            SlotActionType::Swap => ClickType::NumberKey(packet.button as u8),
            SlotActionType::Clone => ClickType::Middle,
            SlotActionType::Throw => {
                if packet.button == 0 {
                    ClickType::Drop
                } else {
                    ClickType::ControlDrop
                }
            }
            SlotActionType::QuickCraft => {
                if [0, 4, 8].contains(&packet.button) {
                    ClickType::Left
                } else if [1, 5, 9].contains(&packet.button) {
                    ClickType::Right
                } else {
                    ClickType::Middle
                }
            }
            SlotActionType::PickupAll => ClickType::DoubleClick,
        };

        send_cancellable! {{
            server;
            InventoryClickEvent::new(
                self,
                screen_handler.window_type(),
                click_type,
                slot,
                raw_slot,
                clicked_item,
                cursor_item,
                i32::from(hotbar_button),
            );
            'after: {}
            'cancelled: {
                screen_handler.cancel().await;
                return;
            }
        }}

        // Enforce flags
        let is_container_slot = slot >= 0 && i32::from(slot) < container_slots as i32;

        match packet.mode {
            SlotActionType::Pickup => {
                let cursor_stack = screen_handler.get_behaviour().cursor_stack.lock().await;
                if is_container_slot {
                    if !cursor_stack.is_empty() && !allow_put_items {
                        drop(cursor_stack);
                        screen_handler.cancel().await;
                        return;
                    }
                    if cursor_stack.is_empty() && !allow_grab_items {
                        drop(cursor_stack);
                        screen_handler.cancel().await;
                        return;
                    }
                }
            }
            SlotActionType::QuickMove => {
                if is_container_slot && !allow_grab_items {
                    screen_handler.cancel().await;
                    return;
                }
                if !is_container_slot && !allow_put_items {
                    screen_handler.cancel().await;
                    return;
                }
            }
            SlotActionType::Swap => {
                if is_container_slot && (!allow_grab_items || !allow_put_items) {
                    screen_handler.cancel().await;
                    return;
                }
            }
            SlotActionType::Throw => {
                if is_container_slot && !allow_grab_items {
                    screen_handler.cancel().await;
                    return;
                }
            }
            SlotActionType::QuickCraft => {
                if !allow_put_items {
                    // Dragging items into slots
                    screen_handler.cancel().await;
                    return;
                }
            }
            SlotActionType::PickupAll => {
                if !allow_grab_items {
                    screen_handler.cancel().await;
                    return;
                }
            }
            SlotActionType::Clone => {}
        }

        let not_in_sync = packet.revision.0
            != (screen_handler
                .get_behaviour()
                .revision
                .load(Ordering::Relaxed) as i32);

        screen_handler.disable_sync();
        screen_handler
            .on_slot_click(
                i32::from(slot),
                i32::from(packet.button),
                packet.mode.clone(),
                self.as_ref(),
            )
            .await;

        for (key, value) in packet.array_of_changed_slots {
            screen_handler.set_received_hash(key as usize, value);
        }

        screen_handler.set_received_cursor_hash(packet.carried_item);
        screen_handler.enable_sync();

        if not_in_sync {
            screen_handler.update_to_client().await;
        } else {
            screen_handler.send_content_updates().await;
        }
    }

    /// Handles when the player clicks a button in a container (e.g. Enchantment Table)
    pub async fn on_container_button_click(self: &Arc<Self>, packet: SContainerButtonClick) {
        let screen_handler = self.current_screen_handler.lock().await.clone();
        let mut screen_handler = screen_handler.lock().await;

        if i32::from(screen_handler.sync_id()) != packet.window_id.0 {
            return;
        }

        screen_handler
            .on_button_click(self.as_ref(), packet.button_id.0)
            .await;
    }
}
