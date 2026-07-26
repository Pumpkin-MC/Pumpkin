use super::record_update;
use crate::block::BlockHitResult;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::net::bedrock::BedrockClient;
use crate::plugin::player::item_held::PlayerItemHeldEvent;
use crate::plugin::player::player_interact_event::InteractAction;
use crate::plugin::player::player_interact_event::PlayerInteractEvent;
use pumpkin_data::BlockDirection;
use pumpkin_data::data_component_impl::BlocksAttacksImpl;
use pumpkin_data::data_component_impl::ConsumableImpl;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::data_component_impl::EquippableImpl;
use pumpkin_data::data_component_impl::FoodImpl;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler};
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::bedrock::client::inventory_content::CInventoryContent;
use pumpkin_protocol::bedrock::client::player_hotbar::CPlayerHotbar;
use pumpkin_protocol::bedrock::network_item::ContainerName;
use pumpkin_protocol::bedrock::network_item::FullContainerName;
use pumpkin_protocol::bedrock::network_item::NetworkItemDescriptor;
use pumpkin_protocol::bedrock::network_item::NetworkItemStackDescriptor;
use pumpkin_protocol::bedrock::server::block_pick_request::SBlockPickRequest;
use pumpkin_protocol::bedrock::server::container_close::SContainerClose;
use pumpkin_protocol::bedrock::server::inventory_transaction::SInventoryTransaction;
use pumpkin_protocol::bedrock::server::inventory_transaction::TransactionData;
use pumpkin_protocol::bedrock::server::mob_equipment::SMobEquipment;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_uint::VarUInt;
use pumpkin_protocol::java::client::play::CSetSelectedSlot;
use pumpkin_util::GameMode;
use pumpkin_util::Hand;
use pumpkin_world::inventory::Inventory;
use std::sync::Arc;
use std::sync::atomic::Ordering;

fn descriptor_to_stack(desc: &NetworkItemDescriptor, is_creative: bool) -> ItemStack {
    if desc.id.0 == 0 || desc.stack_size == 0 {
        ItemStack::EMPTY.clone()
    } else {
        let mut mapped_item = None;

        if is_creative {
            let index = (desc.id.0.saturating_sub(1)) as usize;
            if index < pumpkin_data::bedrock_creative::CREATIVE_ENTRIES.len() {
                let entry = pumpkin_data::bedrock_creative::CREATIVE_ENTRIES[index];
                if let Some(mapping) = pumpkin_data::item::JavaToBedrockItemMapping::from_bedrock(
                    entry.item_id,
                    entry.item_aux_value,
                ) {
                    mapped_item = Some(mapping.java_item);
                }
            }
        }

        if mapped_item.is_none()
            && let Some(mapping) = pumpkin_data::item::JavaToBedrockItemMapping::from_bedrock(
                desc.id.0 as i16,
                desc.aux_value.0,
            )
        {
            mapped_item = Some(mapping.java_item);
        }

        mapped_item.map_or_else(
            || {
                tracing::warn!(
                    "Failed to map bedrock item id {} and data {} to Java item",
                    desc.id.0,
                    desc.aux_value.0
                );
                ItemStack::EMPTY.clone()
            },
            |item| ItemStack::new(desc.stack_size as u8, item),
        )
    }
}

const fn map_bedrock_slot_to_screen_handler(window_id: i32, slot: u32) -> Option<usize> {
    match window_id {
        0 => {
            // WINDOW_ID_INVENTORY
            if slot < 9 {
                // Hotbar: Bedrock 0-8 -> Screen Handler 36-44
                Some(slot as usize + 36)
            } else if slot < 36 {
                // Main Inventory: Bedrock 9-35 -> Screen Handler 9-35
                Some(slot as usize)
            } else {
                None
            }
        }
        120 => {
            // WINDOW_ID_ARMOUR
            if slot < 4 {
                // Armor: Bedrock 0-3 -> Screen Handler 5-8
                Some(slot as usize + 5)
            } else {
                None
            }
        }
        119 => {
            // WINDOW_ID_OFF_HAND
            if slot == 0 {
                // Offhand: Bedrock 0 -> Screen Handler 45
                Some(45)
            } else {
                None
            }
        }
        _ => None,
    }
}

impl BedrockClient {
    #[allow(clippy::too_many_lines, clippy::collapsible_if, clippy::unreachable)]
    pub async fn handle_inventory_action(
        &self,
        player: &Arc<Player>,
        packet: SInventoryTransaction,
    ) {
        tracing::info!("handle_inventory_action: packet={:?}", packet);
        let mut inventory_updated = false;
        let mut updates = Vec::new();
        let result = 0u8;

        if packet.actions.is_empty() && packet.legacy_request_id.0 != 0 {
            let mut player_screen_handler = player.player_screen_handler.lock().await;
            for legacy_slot in &packet.legacy_set_item_slots {
                let mapped_window_id = match legacy_slot.container_id {
                    28 | 29 => 0,    // HotBar or Inventory
                    6 | 120 => 120,  // Armor
                    34 | 119 => 119, // Offhand
                    other => other as i32,
                };
                for &slot_id in &legacy_slot.slots {
                    if let Some(screen_slot) =
                        map_bedrock_slot_to_screen_handler(mapped_window_id, slot_id as u32)
                    {
                        let current_stack = player_screen_handler
                            .get_slot(screen_slot)
                            .get_cloned_stack()
                            .await;
                        if !current_stack.is_empty() {
                            player.drop_item(current_stack.clone()).await;

                            player_screen_handler
                                .get_slot(screen_slot)
                                .set_stack(ItemStack::EMPTY.clone())
                                .await;
                            player_screen_handler
                                .set_received_stack(screen_slot, ItemStack::EMPTY.clone());

                            record_update(
                                &mut updates,
                                FullContainerName {
                                    container_name: match legacy_slot.container_id {
                                        28 => ContainerName::HotBar,
                                        _ => ContainerName::Inventory,
                                    },
                                    dynamic_id: None,
                                },
                                slot_id,
                                0,
                                VarInt(0),
                            );
                            inventory_updated = true;
                        }
                    }
                }
            }
            player_screen_handler.send_content_updates().await;
        }

        let is_creative = player.gamemode.load() == GameMode::Creative;
        for action in &packet.actions {
            use pumpkin_protocol::bedrock::server::inventory_transaction::InventoryActionSource;
            let source_type = InventoryActionSource::from(action.source_type);
            if source_type == InventoryActionSource::World {
                let old_stack = descriptor_to_stack(&action.old_item, is_creative);
                let new_stack = descriptor_to_stack(&action.new_item, is_creative);
                if old_stack.is_empty() && !new_stack.is_empty() {
                    player.drop_item(new_stack).await;
                }
            } else if let Some(window_id) = action.window_id {
                if let Some(screen_slot) =
                    map_bedrock_slot_to_screen_handler(window_id, action.inventory_slot)
                {
                    let item_stack = descriptor_to_stack(&action.new_item, is_creative);

                    let mut player_screen_handler = player.player_screen_handler.lock().await;

                    let is_armor_equipped = player_screen_handler
                        .get_slot(screen_slot)
                        .get_stack()
                        .await
                        .lock()
                        .await
                        .are_equal(&item_stack);

                    if !is_armor_equipped {
                        if (5..9).contains(&screen_slot) {
                            player
                                .enqueue_equipment_change(
                                    &match screen_slot {
                                        5 => EquipmentSlot::HEAD,
                                        6 => EquipmentSlot::CHEST,
                                        7 => EquipmentSlot::LEGS,
                                        8 => EquipmentSlot::FEET,
                                        _ => unreachable!(),
                                    },
                                    &item_stack,
                                )
                                .await;
                        } else if (36..45).contains(&screen_slot) {
                            let hotbar_slot = screen_slot - 36;
                            if player.inventory().get_selected_slot() == hotbar_slot as u8 {
                                let equipment = &[(EquipmentSlot::MAIN_HAND, item_stack.clone())];
                                player.living_entity.send_equipment_changes(equipment);
                            }
                        }
                    }

                    player_screen_handler
                        .get_slot(screen_slot)
                        .set_stack(item_stack.clone())
                        .await;
                    player_screen_handler.set_received_stack(screen_slot, item_stack);
                    player_screen_handler.send_content_updates().await;

                    inventory_updated = true;
                }
            }
        }

        if inventory_updated {
            self.enqueue_packet(&CInventoryContent {
                container_id: VarUInt(0),
                slots: futures::future::join_all(player.inventory().main_inventory.iter().map(
                    async |s| {
                        let stack = s.lock().await;
                        NetworkItemStackDescriptor::from(&*stack)
                    },
                ))
                .await,
                full_container_name: FullContainerName {
                    container_name: ContainerName::Inventory,
                    dynamic_id: None,
                },
                storage_item: NetworkItemStackDescriptor::default(),
            })
            .await;
        }

        match packet.transaction_data {
            TransactionData::Normal(_data) => {
                // Actions are already applied to the inventory screen handler above.
            }
            TransactionData::Mismatch(_data) => {
                // Actions are already applied to the inventory screen handler above.
            }
            TransactionData::UseItem(data) => {
                let face = match data.block_face {
                    0 => BlockDirection::Down,
                    2 => BlockDirection::North,
                    3 => BlockDirection::South,
                    4 => BlockDirection::West,
                    5 => BlockDirection::East,
                    _ => BlockDirection::Up,
                };
                let world = player.world();
                let block = world.get_block(&data.block_position);
                let server = world.server.upgrade().expect("Server is gone");

                if player.gamemode.load() == GameMode::Spectator {
                    // TODO: openMenu ?
                    return;
                }

                if data.action_type.0 == 0 {
                    // Click block
                    let is_creative = player.gamemode.load() == GameMode::Creative;
                    let client_stack = descriptor_to_stack(&data.item_in_hand, is_creative);

                    let held_item = player.inventory.held_item();
                    if !client_stack.is_empty() {
                        let mut server_stack = held_item.lock().await;
                        if server_stack.is_empty() || server_stack.item.id != client_stack.item.id {
                            *server_stack = client_stack.clone();
                        }
                    }

                    let result = server
                        .block_registry
                        .use_with_item(
                            block,
                            player,
                            &data.block_position,
                            &BlockHitResult {
                                face: &face,
                                cursor_pos: &data.click_position,
                            },
                            &held_item,
                            &server,
                            &world,
                        )
                        .await;

                    if result.consumes_action() {
                        return;
                    }

                    if matches!(result, BlockActionResult::PassToDefaultBlockAction) {
                        server
                            .block_registry
                            .on_use(
                                block,
                                player,
                                &data.block_position,
                                &BlockHitResult {
                                    face: &face,
                                    cursor_pos: &data.click_position,
                                },
                                &server,
                                &world,
                            )
                            .await;
                    }

                    let mut stack = held_item.lock().await;
                    if !stack.is_empty() {
                        server
                            .item_registry
                            .use_on_block(
                                &mut stack,
                                player,
                                data.block_position,
                                face,
                                data.click_position,
                                block,
                                &server,
                            )
                            .await;

                        let item_id = stack.item.id;
                        if let Some(placed_block) = pumpkin_data::Block::from_item_id(item_id) {
                            let dummy_use_item_on =
                                pumpkin_protocol::java::server::play::SUseItemOn {
                                    hand: VarInt(0),
                                    position: data.block_position,
                                    face: VarInt(data.block_face),
                                    cursor_pos: data.click_position,
                                    inside_block: false,
                                    is_against_world_border: false,
                                    sequence: VarInt(0),
                                };

                            if let Ok(Some(_)) = server
                                .block_registry
                                .place_block(
                                    player,
                                    placed_block,
                                    &server,
                                    &dummy_use_item_on,
                                    data.block_position,
                                    face,
                                )
                                .await
                            {
                                if player.gamemode.load() != GameMode::Creative {
                                    stack.decrement(1);
                                }
                            }
                        }
                    }
                } else if data.action_type.0 == 1 {
                    // Click air / Use item
                    let is_creative = player.gamemode.load() == GameMode::Creative;
                    let client_stack = descriptor_to_stack(&data.item_in_hand, is_creative);

                    let held_item = player.inventory.held_item();
                    if !client_stack.is_empty() {
                        let mut server_stack = held_item.lock().await;
                        if server_stack.is_empty() || server_stack.item.id != client_stack.item.id {
                            *server_stack = client_stack.clone();
                        }
                    }

                    let event = PlayerInteractEvent::new(
                        player,
                        InteractAction::RightClickAir,
                        &pumpkin_data::Block::AIR,
                        None,
                    );

                    let stack_for_use = held_item.lock().await.clone();

                    {
                        let mut held = held_item.lock().await;
                        let mut cooldown_active = false;
                        if let Some(cooldown) = held.get_use_cooldown() {
                            let group = cooldown
                                .cooldown_group
                                .clone()
                                .unwrap_or_else(|| held.item.registry_key.to_string());
                            if player.is_on_cooldown(&group).await {
                                cooldown_active = true;
                            }
                        }

                        if !cooldown_active {
                            if held.get_data_component::<ConsumableImpl>().is_some()
                                || held.get_data_component::<BlocksAttacksImpl>().is_some()
                            {
                                if let Some(food) = held.get_data_component::<FoodImpl>() {
                                    if player.abilities.lock().await.invulnerable
                                        || food.can_always_eat
                                        || player.hunger_manager.level.load() < 20
                                    {
                                        player
                                            .living_entity
                                            .set_active_hand(
                                                Hand::Left,
                                                held.clone(),
                                                held.get_max_use_time(),
                                            )
                                            .await;
                                    }
                                } else {
                                    player
                                        .living_entity
                                        .set_active_hand(
                                            Hand::Left,
                                            held.clone(),
                                            held.get_max_use_time(),
                                        )
                                        .await;
                                }
                            }
                            if let Some(equippable) = held.get_data_component::<EquippableImpl>() {
                                let inventory = player.inventory();
                                if !inventory
                                    .is_already_equipped(&held_item, equippable.slot)
                                    .await
                                {
                                    player
                                        .enqueue_equipment_change(equippable.slot, &held)
                                        .await;

                                    let binding = {
                                        let mut equipment = inventory.entity_equipment.lock().await;
                                        equipment.get_or_insert(equippable.slot)
                                    };
                                    let mut equip_item = binding.lock().await;
                                    if equip_item.is_empty() {
                                        *equip_item = held.clone();
                                        held.decrement_unless_creative(player.gamemode.load(), 1);
                                    } else {
                                        let binding = held.clone();
                                        *held = equip_item.clone();
                                        *equip_item = binding;
                                    }
                                }
                            }
                        }
                    }

                    send_cancellable! {{
                        server;
                        event;
                        'after: {
                            server.item_registry.on_use(&stack_for_use, player).await;
                        }
                    }}
                }
            }
            TransactionData::UseItemOnEntity(data) => {
                let target_runtime_id = data.target_entity_runtime_id.0 as i32;
                // TODO: replace with consts, i'm too lazy
                match data.action_type.0 {
                    // Interact / Item Interact
                    0 | 2 => {
                        let world = player.world();
                        if let Some(target) = world.get_entity_by_id(target_runtime_id) {
                            let held = player.inventory.held_item();
                            let mut stack = held.lock().await;
                            if !target.interact(player, &mut stack).await {
                                let server = world.server.upgrade().expect("Server is gone");
                                server
                                    .item_registry
                                    .use_on_entity(&mut stack, player, target)
                                    .await;
                            }
                        }
                    }
                    // Attack
                    1 => {
                        let world = player.world();
                        if let Some(target) = world.get_entity_by_id(target_runtime_id) {
                            let target_bounds = target.get_entity().bounding_box.load();
                            if player.is_within_entity_interaction_range(&target_bounds, 3.0) {
                                player.attack(target).await;
                            }
                        }
                    }
                    _ => {
                        tracing::warn!(
                            "invalid UseItemOnEntity action type {}",
                            data.action_type.0
                        );
                        // Kick?
                    }
                }
            }
            TransactionData::ReleaseItem(_data) => {
                let item_in_use = player.living_entity.item_in_use.lock().await.clone();
                if let Some(stack) = item_in_use {
                    let server = player.world().server.upgrade().expect("Server is gone");
                    server.item_registry.on_stopped_using(&stack, player).await;
                }
                player.living_entity.clear_active_hand().await;
            }
        }

        if packet.legacy_request_id.0 != 0 {
            use pumpkin_protocol::bedrock::client::item_stack_response::{
                CItemStackResponse, ItemStackResponse, ItemStackResponseContainerInfo,
                ItemStackResponseSlotInfo,
            };

            let mut container_infos = Vec::new();
            if result == 0 {
                for update in updates {
                    let container_info = container_infos.iter_mut().find(
                        |info: &&mut ItemStackResponseContainerInfo| {
                            info.container_name == update.container_name
                        },
                    );

                    let slot_info = ItemStackResponseSlotInfo {
                        slot: update.slot_id,
                        hotbar_slot: update.slot_id,
                        count: update.count,
                        item_stack_id: update.stack_id,
                        custom_name: String::new(),
                        filtered_custom_name: String::new(),
                        durability_correction: VarInt(0),
                    };

                    if let Some(info) = container_info {
                        info.slots.push(slot_info);
                    } else {
                        container_infos.push(ItemStackResponseContainerInfo {
                            container_name: update.container_name.clone(),
                            slots: vec![slot_info],
                        });
                    }
                }
            }

            self.enqueue_packet(&CItemStackResponse {
                responses: vec![ItemStackResponse {
                    result,
                    request_id: packet.legacy_request_id,
                    container_infos,
                }],
            })
            .await;
        }
    }
    pub async fn handle_container_close(&self, player: &Arc<Player>, packet: SContainerClose) {
        if packet.container_id == 0 || packet.container_id == 0xff {
            self.inventory_opened.store(false, Ordering::Relaxed);
        }
        player.on_handled_screen_closed().await;

        self.enqueue_packet(&SContainerClose {
            container_id: packet.container_id,
            container_type: packet.container_type,
            server_initiated: false,
        })
        .await;

        // Sync the cursor (make it empty) to Bedrock client
        self.enqueue_packet(&CInventoryContent {
            container_id: VarUInt(59), // Cursor container ID
            slots: vec![NetworkItemStackDescriptor::default()],
            full_container_name: FullContainerName {
                container_name: ContainerName::Cursor,
                dynamic_id: None,
            },
            storage_item: NetworkItemStackDescriptor::default(),
        })
        .await;

        // Sync the inventory content to Bedrock client
        self.enqueue_packet(&CInventoryContent {
            container_id: VarUInt(0), // player inventory
            slots: futures::future::join_all(player.inventory().main_inventory.iter().map(
                async |s| {
                    let stack = s.lock().await;
                    NetworkItemStackDescriptor::from(&*stack)
                },
            ))
            .await,
            full_container_name: FullContainerName {
                container_name: ContainerName::Inventory,
                dynamic_id: None,
            },
            storage_item: NetworkItemStackDescriptor::default(),
        })
        .await;
    }
    #[allow(clippy::too_many_lines)]
    pub async fn handle_block_pick_request(&self, player: &Arc<Player>, packet: SBlockPickRequest) {
        if !player.can_interact_with_block_at(&packet.block_pos, 1.0) {
            return;
        }

        let world = player.world();
        let block = world.get_block(&packet.block_pos);

        if block.item_id == 0 {
            return;
        }

        let Some(item) = pumpkin_data::item::Item::from_id(block.item_id) else {
            return;
        };
        let stack = ItemStack::new(1, item);

        let target_hotbar_slot = packet.hotbar_slot as usize;
        if target_hotbar_slot >= 9 {
            return;
        }

        let slot_with_stack = player.inventory().get_slot_with_stack(&stack).await;

        if slot_with_stack != -1 {
            if pumpkin_inventory::player::player_inventory::PlayerInventory::is_valid_hotbar_index(
                slot_with_stack as usize,
            ) {
                if slot_with_stack as usize != target_hotbar_slot {
                    let target_stack = player.inventory.main_inventory[target_hotbar_slot]
                        .lock()
                        .await
                        .clone();
                    let source_stack = player.inventory.main_inventory[slot_with_stack as usize]
                        .lock()
                        .await
                        .clone();
                    player
                        .inventory
                        .set_stack(target_hotbar_slot, source_stack)
                        .await;
                    player
                        .inventory
                        .set_stack(slot_with_stack as usize, target_stack)
                        .await;
                }
            } else {
                let target_stack = player.inventory.main_inventory[target_hotbar_slot]
                    .lock()
                    .await
                    .clone();
                let source_stack = player.inventory.main_inventory[slot_with_stack as usize]
                    .lock()
                    .await
                    .clone();
                player
                    .inventory
                    .set_stack(target_hotbar_slot, source_stack)
                    .await;
                player
                    .inventory
                    .set_stack(slot_with_stack as usize, target_stack)
                    .await;
            }
        } else if player.gamemode.load() == GameMode::Creative {
            player.inventory.set_stack(target_hotbar_slot, stack).await;
        } else {
            return;
        }

        player.inventory.set_selected_slot(target_hotbar_slot as u8);

        // Send hotbar updates
        player
            .client
            .enqueue_packet_editioned(
                &CSetSelectedSlot::new(player.inventory.get_selected_slot() as i8),
                &CPlayerHotbar {
                    selected_slot: VarUInt(player.inventory.get_selected_slot() as u32),
                    container_id: 0,
                    should_select_block: true,
                },
            )
            .await;

        // Send screen handler / Java inventory updates
        player
            .player_screen_handler
            .lock()
            .await
            .send_content_updates()
            .await;

        // Sync main hand equipment to other players
        let stack_in_hand = player.inventory().held_item().lock().await.clone();
        let equipment = &[(EquipmentSlot::MAIN_HAND, stack_in_hand)];
        player.living_entity.send_equipment_changes(equipment);

        // Sync bedrock inventory updates
        self.enqueue_packet(&CInventoryContent {
            container_id: VarUInt(0),
            slots: futures::future::join_all(player.inventory().main_inventory.iter().map(
                async |s| {
                    let stack = s.lock().await;
                    NetworkItemStackDescriptor::from(&*stack)
                },
            ))
            .await,
            full_container_name: FullContainerName {
                container_name: ContainerName::Inventory,
                dynamic_id: None,
            },
            storage_item: NetworkItemStackDescriptor::default(),
        })
        .await;
    }

    pub async fn handle_mob_equipment(&self, player: &Arc<Player>, packet: SMobEquipment) {
        player.update_last_action_time();
        let slot = packet.hotbar_slot;
        if slot >= 9 {
            return;
        }
        let previous_slot = player.inventory.get_selected_slot();
        if let Some(server) = player.world().server.upgrade() {
            let event = PlayerItemHeldEvent::new(player.clone(), previous_slot, slot);
            let event = server.plugin_manager.fire(event).await;
            if event.cancelled {
                self.enqueue_packet(&CPlayerHotbar {
                    selected_slot: VarUInt(previous_slot as u32),
                    container_id: 0,
                    should_select_block: true,
                })
                .await;
                return;
            }
        }

        let inv = player.inventory();
        inv.set_selected_slot(slot);
        let stack = inv.held_item().lock().await.clone();
        let equipment = &[(EquipmentSlot::MAIN_HAND, stack)];
        player.living_entity.send_equipment_changes(equipment);
    }
}
