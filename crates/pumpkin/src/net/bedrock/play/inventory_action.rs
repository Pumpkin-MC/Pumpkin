#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    #[allow(clippy::too_many_lines, clippy::collapsible_if, clippy::unreachable)]
    pub async fn handle_inventory_action(
        &self,
        player: &Arc<Player>,
        packet: SInventoryTransaction,
    ) {
        tracing::debug!("handle_inventory_action: packet={:?}", packet);
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
                                ItemStack::EMPTY,
                            );
                            inventory_updated = true;
                        }
                    }
                }
            }
            player_screen_handler.send_content_updates().await;
        }

        for action in &packet.actions {
            use pumpkin_protocol::bedrock::server::inventory_transaction::InventoryActionSource;
            let source_type = InventoryActionSource::from(action.source_type);
            if source_type == InventoryActionSource::World {
                let old_stack = descriptor_to_stack(&action.old_item);
                let new_stack = descriptor_to_stack(&action.new_item);
                if old_stack.is_empty() && !new_stack.is_empty() {
                    player.drop_item(new_stack).await;
                }
            } else if let Some(window_id) = action.window_id {
                if let Some(screen_slot) =
                    map_bedrock_slot_to_screen_handler(window_id, action.inventory_slot)
                {
                    let item_stack = descriptor_to_stack(&action.new_item);

                    let mut player_screen_handler = player.player_screen_handler.lock().await;

                    let is_armor_equipped = player_screen_handler
                        .get_slot(screen_slot)
                        .get_stack()
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
                slots: player
                    .inventory()
                    .main_inventory
                    .read()
                    .await
                    .iter()
                    .map(NetworkItemStackDescriptor::from)
                    .collect(),
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
                let Some(server) = world.server.upgrade() else {
                    return;
                };

                if player.gamemode.load() == GameMode::Spectator {
                    // TODO: openMenu ?
                    return;
                }

                if data.action_type.0 == 0 {
                    // Click block
                    let client_stack = descriptor_to_stack(&data.item_in_hand);

                    let mut held_item = player.inventory().held_item().await;
                    if !client_stack.is_empty() {
                        if held_item.is_empty() || held_item.item.id != client_stack.item.id {
                            held_item = client_stack.clone();
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
                            &mut held_item,
                            &EquipmentSlot::MAIN_HAND,
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

                    let mut stack = held_item;
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
                                    face: VarInt(i32::from(data.block_face)),
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
                        player.inventory().set_held_item(stack).await;
                    }
                } else if data.action_type.0 == 1 {
                    // Click air / Use item
                    let client_stack = descriptor_to_stack(&data.item_in_hand);

                    let mut held = player.inventory.held_item().await;
                    if !client_stack.is_empty()
                        && (held.is_empty() || held.item.id != client_stack.item.id)
                    {
                        held = client_stack.clone();
                        player.inventory.set_held_item(held.clone()).await;
                    }

                    let event = PlayerInteractEvent::new(
                        player,
                        InteractAction::RightClickAir,
                        &pumpkin_data::Block::AIR,
                        None,
                    );

                    let stack_for_use = held.clone();

                    {
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
                                let mut equipment_guard = inventory.entity_equipment.lock().await;
                                let current_equipped = equipment_guard.get(equippable.slot);
                                if !current_equipped.are_items_and_components_equal(&held) {
                                    player
                                        .enqueue_equipment_change(equippable.slot, &held)
                                        .await;

                                    let equip_item = equipment_guard
                                        .equipment
                                        .entry(equippable.slot.clone())
                                        .or_insert_with(|| ItemStack::EMPTY.clone());
                                    if equip_item.is_empty() {
                                        *equip_item = held.clone();
                                        held.decrement_unless_creative(player.gamemode.load(), 1);
                                    } else {
                                        let old_held = held.clone();
                                        held = equip_item.clone();
                                        *equip_item = old_held;
                                    }
                                    player.inventory().set_held_item(held.clone()).await;
                                }
                            }
                        }
                    }

                    send_cancellable! {{
                        &server;
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
                            let mut stack = player.inventory().held_item().await;
                            if !target.interact(player, &mut stack).await {
                                let Some(server) = world.server.upgrade() else {
                                    return;
                                };
                                server
                                    .item_registry
                                    .use_on_entity(&mut stack, player, target)
                                    .await;
                                player.inventory().set_held_item(stack).await;
                            }
                        }
                    }
                    // Attack
                    1 => {
                        let world = player.world();
                        if let Some(target) = world.get_entity_by_id(target_runtime_id) {
                            player.attack(target).await;
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
                    let Some(server) = player.world().server.upgrade() else {
                        return;
                    };
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
}
