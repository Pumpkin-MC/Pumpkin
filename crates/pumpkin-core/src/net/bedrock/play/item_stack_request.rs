use crate::entity::player::ScreenOwner;

#[allow(clippy::wildcard_imports)]
use super::*;
use crate::block::entities::beacon::BeaconBlockEntity;
use pumpkin_data::screen::WindowType;
use pumpkin_inventory::mount_screen_handler::MountScreenHandler;
use pumpkin_inventory::{
    anvil::anvil_screen_handler::AnvilScreenHandler, beacon_screen_handler::BeaconScreenHandler,
    cartography_table_screen_handler::CartographyTableScreenHandler,
    grindstone_screen_handler::GrindstoneScreenHandler, loom_screen_handler::LoomScreenHandler,
    smithing_table_screen_handler::SmithingTableScreenHandler,
    stonecutter_screen_handler::StonecutterScreenHandler,
};
use pumpkin_protocol::bedrock::client::item_stack_response::{
    CItemStackResponse, ItemStackResponseContainerInfo, ItemStackResponseInfo,
    ItemStackResponseSlotInfo,
};
use pumpkin_protocol::bedrock::server::item_stack_request::{
    ItemStackRequestSlotInfo, SItemStackRequest,
};

impl BedrockClient {
    pub fn handle_item_stack_request(&self, player: &Arc<Player>, packet: SItemStackRequest) {
        self.handle_item_stack_request_for_owner(player, packet, &player.screen_owner());
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn handle_item_stack_request_for_owner(
        &self,
        player: &Arc<Player>,
        packet: SItemStackRequest,
        owner: &ScreenOwner,
    ) {
        use pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestAction;

        let (
            responses,
            inventory_updated,
            sync,
            screen,
            epoch,
            effects,
            affected_slots,
            equipment_before,
            equipment_committed,
        ) = {
            let Some((active, mut screen_handler)) = player.lock_screen_action(owner) else {
                self.try_enqueue_client_packet(&CItemStackResponse {
                    responses: packet
                        .requests
                        .into_iter()
                        .map(|request| ItemStackResponseInfo {
                            result: 1,
                            client_request_id: request.request_id,
                            containers: Vec::new(),
                        })
                        .collect(),
                });
                resync_stack_request_screens(player);
                return;
            };
            let current_screen_handler = owner.handler.clone();
            drop(active);
            let inventory = player.inventory();
            let physical_inventory: Arc<dyn pumpkin_inventory::inventory::Inventory> =
                inventory.clone();
            let equipment_slots =
                Player::SCREEN_EQUIPMENT.map(|slot| inventory.equipment_slot_index(&slot));
            let equipment_before = player.screen_equipment();
            let mut equipment_committed = equipment_before.clone();
            let previous_sync = screen_handler.get_behaviour_mut().defer_updates();
            let mut responses = Vec::with_capacity(packet.requests.len());
            let mut pending_slots = Vec::with_capacity(packet.requests.len());

            let mut recover_partial_request = false;
            let mut beacon_payment_changed = false;
            for mut request in packet.requests {
                let request_epoch = screen_handler.get_behaviour().sync_epoch();
                let mut created_item: Option<ItemStack> = None;
                let mut crafting_inputs_consumed = false;
                let mut updates = Vec::new();
                let mut result = 0u8; // 0 = Success, 1 = Error

                if request
                    .actions
                    .iter()
                    .any(|action| touches_unroutable_slot(&*screen_handler, action))
                {
                    tracing::debug!(
                        "Item stack request targets armor/offhand outside the player screen"
                    );
                    result = 1;
                    request.actions.clear();
                }

                for action in request.actions {
                    match action {
                        ItemStackRequestAction::CraftCreative {
                            creative_item_id,
                            repetitions,
                        } => {
                            let index = (creative_item_id.0.saturating_sub(1)) as usize;
                            if index < pumpkin_data::bedrock_creative::CREATIVE_ENTRIES.len() {
                                let entry = pumpkin_data::bedrock_creative::CREATIVE_ENTRIES[index];
                                if let Some(mapping) =
                                    pumpkin_data::item::JavaToBedrockItemMapping::from_bedrock(
                                        entry.item_id,
                                        entry.item_aux_value,
                                    )
                                {
                                    // Bedrock `repetitions` represents how many stacks to create; use the item's max stack size
                                    let max_stack =
                                        ItemStack::static_new_java(1, mapping.java_item)
                                            .get_max_stack_size();
                                    let count = ((max_stack as u16) * (repetitions as u16))
                                        .min(u8::MAX as u16)
                                        as u8;
                                    created_item = Some(ItemStack::new(count, mapping.java_item));
                                } else {
                                    tracing::warn!(
                                        "Failed to map bedrock item id {} and data {} to Java item",
                                        entry.item_id,
                                        entry.item_aux_value
                                    );
                                    result = 1;
                                    break;
                                }
                            } else {
                                tracing::warn!(
                                    "Creative item index {} out of bounds (len: {})",
                                    index,
                                    pumpkin_data::bedrock_creative::CREATIVE_ENTRIES.len()
                                );
                                result = 1;
                                break;
                            }
                        }
                        ItemStackRequestAction::Take {
                            count,
                            source,
                            destination,
                        }
                        | ItemStackRequestAction::Place {
                            count,
                            source,
                            destination,
                        } => {
                            let mut source_stack =
                                get_slot_stack(&*screen_handler, &source, created_item.as_ref());
                            if source_stack.is_empty() && created_item.is_none() {
                                tracing::debug!("Source stack is empty in Take/Place");
                                result = 1;
                                break;
                            }
                            let count = count.min(source_stack.item_count);
                            if count > 0 {
                                let mut dest_stack = get_slot_stack(
                                    &*screen_handler,
                                    &destination,
                                    created_item.as_ref(),
                                );
                                if dest_stack.is_empty() {
                                    dest_stack = source_stack.copy_with_count(count);
                                } else if dest_stack.are_items_and_components_equal(&source_stack) {
                                    dest_stack.item_count =
                                        dest_stack.item_count.saturating_add(count);
                                } else {
                                    tracing::debug!(
                                        "Destination stack is not compatible with source stack"
                                    );
                                    result = 1;
                                    break;
                                }

                                if let Some(dest_screen_slot) = map_bedrock_container_slot(
                                    &*screen_handler,
                                    destination.container_name.container_name,
                                    destination.slot_id,
                                ) {
                                    let slot =
                                        &screen_handler.get_behaviour().slots[dest_screen_slot];
                                    if !slot.can_insert(&dest_stack) {
                                        result = 1;
                                        break;
                                    }
                                }

                                let merchant_result = screen_handler.window_type()
                                    == Some(pumpkin_data::screen::WindowType::Merchant)
                                    && matches!(
                                        source.container_name.container_name,
                                        ContainerName::CreatedOutput
                                            | ContainerName::TradeResultPreview
                                            | ContainerName::Trade2ResultPreview
                                    );
                                if merchant_result {
                                    if count != source_stack.item_count {
                                        result = 1;
                                        break;
                                    }
                                    let Some(handler) = screen_handler
                                    .as_any_mut()
                                    .downcast_mut::<pumpkin_inventory::merchant::merchant_screen_handler::MerchantScreenHandler>()
                                else {
                                    result = 1;
                                    break;
                                };
                                    if !handler.complete_bedrock_trade(player.as_ref()) {
                                        result = 1;
                                        break;
                                    }

                                    update_slot_stack(handler, &destination, dest_stack.clone());
                                    for (container_name, slot_id, screen_slot) in [
                                        (ContainerName::Trade2Ingredient1, 4, 0),
                                        (ContainerName::Trade2Ingredient2, 5, 1),
                                        (ContainerName::Trade2ResultPreview, 50, 2),
                                    ] {
                                        let stack = handler.get_behaviour().slots[screen_slot]
                                            .get_cloned_stack();
                                        record_update(
                                            &mut updates,
                                            FullContainerName {
                                                container_name,
                                                dynamic_id: None,
                                            },
                                            slot_id,
                                            &stack,
                                        );
                                    }
                                    record_update(
                                        &mut updates,
                                        source.container_name.clone(),
                                        source.slot_id,
                                        ItemStack::EMPTY,
                                    );
                                    record_update(
                                        &mut updates,
                                        destination.container_name.clone(),
                                        destination.slot_id,
                                        &dest_stack,
                                    );
                                    continue;
                                }

                                if let Some(out_slot) =
                                    get_output_screen_slot(&*screen_handler, &source)
                                    && !virtual_craft_source(&source, created_item.as_ref())
                                {
                                    let slot =
                                        screen_handler.get_behaviour().slots[out_slot].clone();
                                    if !slot.can_take_items(player.as_ref()) {
                                        result = 1;
                                        break;
                                    }
                                    slot.on_take_item(player.as_ref(), &source_stack);
                                    record_specialized_container_input_updates(
                                        &*screen_handler,
                                        &mut updates,
                                    );
                                }

                                source_stack.decrement(count);
                                record_consumed_source(
                                    &mut *screen_handler,
                                    &mut created_item,
                                    &mut updates,
                                    &source,
                                    &source_stack,
                                );
                                update_slot_stack(
                                    &mut *screen_handler,
                                    &destination,
                                    dest_stack.clone(),
                                );

                                record_update(
                                    &mut updates,
                                    destination.container_name.clone(),
                                    destination.slot_id,
                                    &dest_stack,
                                );
                            }
                        }
                        ItemStackRequestAction::Swap { slot1, slot2 } => {
                            if get_output_screen_slot(&*screen_handler, &slot1).is_some()
                                || get_output_screen_slot(&*screen_handler, &slot2).is_some()
                            {
                                result = 1;
                                break;
                            }

                            let stack1 =
                                get_slot_stack(&*screen_handler, &slot1, created_item.as_ref());
                            let stack2 =
                                get_slot_stack(&*screen_handler, &slot2, created_item.as_ref());

                            update_slot_stack(&mut *screen_handler, &slot1, stack2.clone());
                            update_slot_stack(&mut *screen_handler, &slot2, stack1.clone());

                            record_update(
                                &mut updates,
                                slot1.container_name.clone(),
                                slot1.slot_id,
                                &stack2,
                            );
                            record_update(
                                &mut updates,
                                slot2.container_name.clone(),
                                slot2.slot_id,
                                &stack1,
                            );
                        }
                        ItemStackRequestAction::Drop {
                            count,
                            source,
                            randomly: _,
                        } => {
                            let mut source_stack =
                                get_slot_stack(&*screen_handler, &source, created_item.as_ref());
                            if source_stack.is_empty() {
                                result = 1;
                                break;
                            }
                            if let Some(out_slot) =
                                get_output_screen_slot(&*screen_handler, &source)
                                && !virtual_craft_source(&source, created_item.as_ref())
                            {
                                let slot = screen_handler.get_behaviour().slots[out_slot].clone();
                                if !slot.can_take_items(player.as_ref()) {
                                    result = 1;
                                    break;
                                }
                                slot.on_take_item(player.as_ref(), &source_stack);
                                record_specialized_container_input_updates(
                                    &*screen_handler,
                                    &mut updates,
                                );
                            }
                            let count = count.min(source_stack.item_count);
                            if count > 0 {
                                let dropped_stack = source_stack.copy_with_count(count);
                                source_stack.decrement(count);
                                record_consumed_source(
                                    &mut *screen_handler,
                                    &mut created_item,
                                    &mut updates,
                                    &source,
                                    &source_stack,
                                );
                                screen_handler.get_behaviour_mut().drop_item(
                                    player.as_ref(),
                                    dropped_stack,
                                    true,
                                );
                            }
                        }
                        ItemStackRequestAction::Destroy { count, source }
                        | ItemStackRequestAction::Consume { count, source } => {
                            if screen_handler.window_type()
                                == Some(pumpkin_data::screen::WindowType::Merchant)
                                && matches!(
                                    source.container_name.container_name,
                                    ContainerName::TradeIngredient1
                                        | ContainerName::TradeIngredient2
                                        | ContainerName::Trade2Ingredient1
                                        | ContainerName::Trade2Ingredient2
                                )
                            {
                                // MerchantScreenHandler consumes both payment slots atomically when
                                // Bedrock subsequently takes the CreatedOutput result.
                                continue;
                            }
                            if crafting_inputs_consumed
                                && source.container_name.container_name
                                    == ContainerName::CraftingInput
                            {
                                let source_stack = get_slot_stack(
                                    &*screen_handler,
                                    &source,
                                    created_item.as_ref(),
                                );
                                record_update(
                                    &mut updates,
                                    source.container_name.clone(),
                                    source.slot_id,
                                    &source_stack,
                                );
                                continue;
                            }

                            let mut source_stack =
                                get_slot_stack(&*screen_handler, &source, created_item.as_ref());
                            if source_stack.is_empty() {
                                result = 1;
                                break;
                            }
                            let count = count.min(source_stack.item_count);
                            if count > 0 {
                                source_stack.decrement(count);
                                record_consumed_source(
                                    &mut *screen_handler,
                                    &mut created_item,
                                    &mut updates,
                                    &source,
                                    &source_stack,
                                );
                            }
                        }
                        ItemStackRequestAction::CraftRecipe {
                            recipe_id,
                            repetitions,
                        }
                        | ItemStackRequestAction::CraftRecipeAuto {
                            recipe_id,
                            repetitions,
                        } => {
                            if screen_handler.window_type()
                                == Some(pumpkin_data::screen::WindowType::Merchant)
                            {
                                let Some(handler) = screen_handler
                                .as_any_mut()
                                .downcast_mut::<pumpkin_inventory::merchant::merchant_screen_handler::MerchantScreenHandler>()
                            else {
                                result = 1;
                                break;
                            };
                                let trade = recipe_id.0.saturating_sub(1) as usize;
                                if trade >= handler.offers.len() {
                                    result = 1;
                                    break;
                                }
                                handler.set_selected_offer(trade);
                                for (container_name, slot_id, screen_slot) in [
                                    (ContainerName::Trade2Ingredient1, 4, 0),
                                    (ContainerName::Trade2Ingredient2, 5, 1),
                                    (ContainerName::Trade2ResultPreview, 50, 2),
                                ] {
                                    let stack = handler.get_behaviour().slots[screen_slot]
                                        .get_cloned_stack();
                                    record_update(
                                        &mut updates,
                                        FullContainerName {
                                            container_name,
                                            dynamic_id: None,
                                        },
                                        slot_id,
                                        &stack,
                                    );
                                }
                                continue;
                            }

                            if screen_handler.window_type()
                                == Some(pumpkin_data::screen::WindowType::Stonecutter)
                            {
                                let Some(handler) = screen_handler
                                    .as_any_mut()
                                    .downcast_mut::<StonecutterScreenHandler>()
                                else {
                                    result = 1;
                                    break;
                                };
                                let input_stack = handler.input_inventory.get_stack(0);
                                if input_stack.is_empty() {
                                    result = 1;
                                    break;
                                }
                                let available = handler.get_available_recipes(&input_stack);
                                let server = player.world().server.upgrade();
                                let recipe_idx =
                                    crate::net::bedrock::recipe::stonecutter_output_for_recipe_id(
                                        server.as_deref().map(|s| s.recipe_manager.as_ref()),
                                        recipe_id.0,
                                    )
                                    .and_then(|(out_id, out_aux)| {
                                        available.iter().position(|r| {
                                            let item_id = r
                                                .result_id
                                                .strip_prefix("minecraft:")
                                                .unwrap_or(&r.result_id);
                                            pumpkin_data::item::Item::from_registry_key(item_id)
                                                .and_then(|item| {
                                                    pumpkin_data::item::JavaToBedrockItemMapping::from_java_item_id(
                                                        item.id,
                                                    )
                                                })
                                                .is_some_and(|m| {
                                                    m.bedrock_item.id == out_id
                                                        && m.bedrock_data == out_aux
                                                })
                                        })
                                    })
                                    .or_else(|| {
                                        ((recipe_id.0 as usize) < available.len())
                                            .then_some(recipe_id.0 as usize)
                                    });

                                let Some(idx) = recipe_idx else {
                                    result = 1;
                                    break;
                                };

                                handler.set_selected_recipe(idx as u8);
                                handler.update_output();

                                let output_slot = handler.get_behaviour().slots[1].clone();
                                let output_stack = output_slot.get_cloned_stack();
                                if output_stack.is_empty() {
                                    result = 1;
                                    break;
                                }

                                let reps = repetitions.max(1);
                                let mut total_crafted = output_stack.clone();
                                total_crafted.item_count =
                                    total_crafted.item_count.saturating_mul(reps);
                                created_item = Some(total_crafted);

                                for _ in 0..reps {
                                    output_slot.on_take_item(player.as_ref(), &output_stack);
                                }
                                crafting_inputs_consumed = true;

                                record_specialized_container_input_updates(
                                    &*screen_handler,
                                    &mut updates,
                                );
                                record_preview_update(
                                    &mut updates,
                                    ContainerName::StonecutterResultPreview,
                                    1,
                                    &output_stack,
                                );
                                continue;
                            }

                            if screen_handler.window_type()
                                == Some(pumpkin_data::screen::WindowType::Enchantment)
                            {
                                let button_id = if recipe_id.0 < 3 {
                                    recipe_id.0 as i32
                                } else {
                                    recipe_id.0.saturating_sub(1) as i32
                                };
                                if (0..3).contains(&button_id)
                                    && screen_handler.on_button_click(player.as_ref(), button_id)
                                {
                                    let enchanted_item =
                                        screen_handler.get_behaviour().slots[0].get_cloned_stack();
                                    let lapis_item =
                                        screen_handler.get_behaviour().slots[1].get_cloned_stack();
                                    record_update(
                                        &mut updates,
                                        FullContainerName {
                                            container_name: ContainerName::EnchantingInput,
                                            dynamic_id: None,
                                        },
                                        0,
                                        &enchanted_item,
                                    );
                                    record_update(
                                        &mut updates,
                                        FullContainerName {
                                            container_name: ContainerName::EnchantingMaterial,
                                            dynamic_id: None,
                                        },
                                        1,
                                        &lapis_item,
                                    );
                                } else {
                                    result = 1;
                                    break;
                                }
                                continue;
                            }

                            if repetitions > 0 {
                                screen_handler.update_to_client();

                                let is_player = screen_handler.window_type().is_none();
                                let grid_size = if is_player { 4 } else { 9 };
                                let bedrock_grid_start = if is_player { 28 } else { 32 };
                                for i in 0..grid_size {
                                    let grid_slot_index = 1 + i;
                                    let grid_slot = screen_handler.get_behaviour().slots
                                        [grid_slot_index]
                                        .clone();
                                    let grid_stack = grid_slot.get_cloned_stack();
                                    tracing::info!(
                                        "Crafting Grid slot {i} (slot index {grid_slot_index}): Item ID: {}, Count: {}",
                                        grid_stack.item.id,
                                        grid_stack.item_count
                                    );
                                }

                                let output_slot = screen_handler.get_behaviour().slots[0].clone();
                                let output_stack = output_slot.get_cloned_stack();

                                if output_stack.is_empty()
                                    || repetitions > output_slot.get_max_item_count()
                                {
                                    tracing::warn!("Client sent an invalid crafting request");
                                    result = 1;
                                    break;
                                }

                                let mut total_crafted = output_stack.clone();
                                total_crafted.item_count =
                                    total_crafted.item_count.saturating_mul(repetitions);
                                created_item = Some(total_crafted);

                                for _ in 0..repetitions {
                                    output_slot.on_take_item(player.as_ref(), &output_stack);
                                }
                                crafting_inputs_consumed = true;

                                // Record updates for all grid slots so Bedrock client is notified of consumed ingredients!
                                let is_player = screen_handler.window_type().is_none();
                                let grid_size = if is_player { 4 } else { 9 };
                                for i in 0..grid_size {
                                    let grid_slot_index = 1 + i;
                                    let grid_slot = screen_handler.get_behaviour().slots
                                        [grid_slot_index]
                                        .clone();
                                    let grid_stack = grid_slot.get_cloned_stack();
                                    record_update(
                                        &mut updates,
                                        FullContainerName {
                                            container_name: ContainerName::CraftingInput,
                                            dynamic_id: None,
                                        },
                                        (bedrock_grid_start + i) as u8,
                                        &grid_stack,
                                    );
                                }
                            }
                        }
                        ItemStackRequestAction::BeaconPayment {
                            primary_effect_id,
                            secondary_effect_id,
                        } => {
                            let Some(beacon_handler) = screen_handler
                                .as_any_mut()
                                .downcast_mut::<BeaconScreenHandler>()
                            else {
                                result = 1;
                                break;
                            };
                            let Some(beacon_entity) = beacon_handler
                                .inventory
                                .as_any()
                                .downcast_ref::<BeaconBlockEntity>()
                            else {
                                result = 1;
                                break;
                            };
                            if beacon_entity
                                .payment
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .is_empty()
                            {
                                result = 1;
                                break;
                            }
                            let levels = beacon_entity.levels.load(Ordering::Relaxed);
                            let primary_id =
                                (primary_effect_id.0 > 0).then_some(primary_effect_id.0);
                            let secondary_id =
                                (secondary_effect_id.0 > 0).then_some(secondary_effect_id.0);

                            if BeaconBlockEntity::validate_effects(primary_id, secondary_id, levels)
                            {
                                beacon_entity
                                    .primary_effect
                                    .store(primary_id.unwrap_or(-1), Ordering::Relaxed);
                                beacon_entity
                                    .secondary_effect
                                    .store(secondary_id.unwrap_or(-1), Ordering::Relaxed);

                                beacon_entity.remove_stack_specific(0, 1);
                                beacon_entity.mark_dirty();
                                let payment_stack = beacon_entity.get_stack(0);
                                beacon_payment_changed = true;
                                record_update(
                                    &mut updates,
                                    FullContainerName {
                                        container_name: ContainerName::BeaconPayment,
                                        dynamic_id: None,
                                    },
                                    0,
                                    &payment_stack,
                                );
                            } else {
                                result = 1;
                                break;
                            }
                        }
                        ItemStackRequestAction::MineBlock {
                            hotbar_slot,
                            predicted_durability: _,
                            stack_id: _,
                        } => {
                            if (0..9).contains(&hotbar_slot.0)
                                && let Some(screen_slot) = map_bedrock_container_slot(
                                    &*screen_handler,
                                    ContainerName::HotBar,
                                    hotbar_slot.0 as u8,
                                )
                            {
                                let current_stack = screen_handler.get_behaviour().slots
                                    [screen_slot]
                                    .get_cloned_stack();
                                record_physical_read(
                                    &mut updates,
                                    FullContainerName {
                                        container_name: ContainerName::HotBar,
                                        dynamic_id: None,
                                    },
                                    hotbar_slot.0 as u8,
                                    &current_stack,
                                );
                            }
                        }
                        ItemStackRequestAction::Optional {
                            recipe_id: _,
                            filter_string_index,
                        } => {
                            if screen_handler.window_type()
                                == Some(pumpkin_data::screen::WindowType::Anvil)
                                && let Some(anvil) = screen_handler
                                    .as_any_mut()
                                    .downcast_mut::<AnvilScreenHandler>()
                            {
                                let name = if filter_string_index >= 0
                                    && (filter_string_index as usize) < request.filter_strings.len()
                                {
                                    &request.filter_strings[filter_string_index as usize]
                                } else if let Some(first) = request.filter_strings.first() {
                                    first
                                } else {
                                    ""
                                };
                                anvil.set_item_name(name, player.has_infinite_materials());
                                let result_stack = anvil.inventory.get_stack(2);
                                record_preview_update(
                                    &mut updates,
                                    ContainerName::AnvilResultPreview,
                                    2,
                                    &result_stack,
                                );
                            }
                        }
                        ItemStackRequestAction::Grindstone {
                            recipe_id: _,
                            repair_cost: _,
                            repetitions: _,
                        } => {
                            if screen_handler.window_type()
                                == Some(pumpkin_data::screen::WindowType::Grindstone)
                                && let Some(grindstone) = screen_handler
                                    .as_any_mut()
                                    .downcast_mut::<GrindstoneScreenHandler>()
                            {
                                grindstone.create_result();
                                let result_stack = grindstone.inventory.get_stack(2);
                                record_preview_update(
                                    &mut updates,
                                    ContainerName::GrindstoneResultPreview,
                                    2,
                                    &result_stack,
                                );
                            }
                        }
                        ItemStackRequestAction::Loom {
                            pattern_id,
                            repetitions: _,
                        } => {
                            if screen_handler.window_type()
                                == Some(pumpkin_data::screen::WindowType::Loom)
                                && let Some(loom) = screen_handler
                                    .as_any_mut()
                                    .downcast_mut::<LoomScreenHandler>()
                            {
                                loom.select_pattern(&pattern_id);
                                let result_stack = loom.output_inventory.get_stack(0);
                                record_preview_update(
                                    &mut updates,
                                    ContainerName::LoomResultPreview,
                                    3,
                                    &result_stack,
                                );
                            }
                        }
                        ItemStackRequestAction::CraftResultsDeprecated { .. }
                        | ItemStackRequestAction::CraftReserved { .. }
                        | ItemStackRequestAction::Create { .. }
                        | ItemStackRequestAction::LabTableCombine
                        | ItemStackRequestAction::CraftNonImplemented => {
                            // Successful no-ops to prevent client-side transaction rollbacks
                        }
                    }
                }

                if result == 0 {
                    refresh_specialized_screen_results(
                        player.as_ref(),
                        &mut *screen_handler,
                        &request.filter_strings,
                        &mut updates,
                    );
                }

                let mut equipment_writes = [None; Player::SCREEN_EQUIPMENT.len()];
                for update in &updates {
                    let Some((order, written)) = &update.committed else {
                        continue;
                    };
                    if update.container_name.container_name == ContainerName::Cursor {
                        continue;
                    }
                    let Some(slot) = map_bedrock_container_slot(
                        &*screen_handler,
                        update.container_name.container_name,
                        update.slot_id,
                    )
                    .and_then(|index| screen_handler.get_behaviour().slots.get(index)) else {
                        continue;
                    };
                    if Arc::ptr_eq(&slot.get_inventory(), &physical_inventory)
                        && let Some(index) = equipment_slots
                            .iter()
                            .position(|physical| *physical == Some(slot.get_index()))
                        && equipment_writes[index].is_none_or(|previous| previous < *order)
                    {
                        equipment_writes[index] = Some(*order);
                        equipment_committed[index] = written.clone();
                    }
                }

                let mut container_infos = Vec::new();
                let mut physical = Vec::new();
                for update in updates {
                    let name = update.container_name.container_name;
                    let recorded = if result == 0 {
                        update
                            .physical
                            .then_some((update.write_order, update.expected))
                    } else {
                        update.committed
                    };
                    if let Some((order, expected)) = recorded
                        && (name == ContainerName::Cursor
                            || map_bedrock_container_slot(&*screen_handler, name, update.slot_id)
                                .is_some())
                    {
                        physical.push((
                            order,
                            ItemStackRequestSlotInfo {
                                container_name: update.container_name.clone(),
                                slot_id: update.slot_id,
                                stack_id: update.stack_id.0,
                            },
                            expected,
                        ));
                    }
                    if result == 0 {
                        let container_info = container_infos.iter_mut().find(
                            |info: &&mut ItemStackResponseContainerInfo| {
                                info.full_container_name == update.container_name
                            },
                        );

                        let slot_info = ItemStackResponseSlotInfo {
                            requested_slot: update.slot_id,
                            slot: update.slot_id,
                            amount: update.count,
                            item_stack_net_id: update.stack_id,
                            custom_name: String::new(),
                            filtered_custom_name: String::new(),
                            durability_correction: VarInt(0),
                        };

                        if let Some(info) = container_info {
                            info.slots.push(slot_info);
                        } else {
                            container_infos.push(ItemStackResponseContainerInfo {
                                full_container_name: update.container_name,
                                slots: vec![slot_info],
                            });
                        }
                    }
                }

                // A failed acknowledgement cannot describe writes already made by
                // its earlier actions. Reconcile the client from the live menu.
                // Slot and cursor writes must advance the sync epoch for this check.
                recover_partial_request |=
                    result != 0 && screen_handler.get_behaviour().sync_epoch() != request_epoch;
                physical.sort_unstable_by_key(|(order, _, _)| *order);
                pending_slots.push(
                    physical
                        .into_iter()
                        .map(|(_, slot, expected)| (slot, expected))
                        .collect(),
                );
                responses.push(ItemStackResponseInfo {
                    result,
                    client_request_id: request.request_id,
                    containers: container_infos,
                });
            }

            if recover_partial_request || beacon_payment_changed {
                screen_handler.sync_state();
            } else {
                screen_handler.send_content_updates();
            }

            // Collect inventory updates if we modified player inventory
            let mut inventory_updated = recover_partial_request;
            for response in &responses {
                if response.result == 0 {
                    for info in &response.containers {
                        if info.full_container_name.container_name == ContainerName::Inventory
                            || info.full_container_name.container_name
                                == ContainerName::CombinedHotBarAndInventory
                            || info.full_container_name.container_name == ContainerName::HotBar
                        {
                            inventory_updated = true;
                        }
                    }
                }
            }

            let sync = screen_handler
                .get_behaviour_mut()
                .finish_deferred_updates(previous_sync);
            let epoch = screen_handler.get_behaviour().sync_epoch();
            let effects = screen_handler.get_behaviour_mut().take_deferred_effects();
            let affected_slots =
                final_acknowledged_slots(&*screen_handler, pending_slots, &mut responses);
            if equipment_slots[0] != inventory.equipment_slot_index(&EquipmentSlot::MAIN_HAND) {
                equipment_committed[0] = equipment_before[0].clone();
            }
            drop(screen_handler);
            (
                responses,
                inventory_updated,
                sync,
                current_screen_handler,
                epoch,
                effects,
                affected_slots,
                equipment_before,
                equipment_committed,
            )
        };

        let response = CItemStackResponse { responses };
        let current = self.enqueue_stack_request_response(
            player,
            &screen,
            &affected_slots,
            &response,
            Some(owner),
        );
        if current {
            player.flush_screen_updates(&screen, epoch, sync, true);
        } else {
            resync_stack_request_screens(player);
        }
        for effect in effects {
            effect.publish(player.as_ref());
        }
        player.publish_committed_screen_equipment(&equipment_before, &equipment_committed);
        if inventory_updated {
            player.update_bedrock_shield_sneaking();
        }
    }

    pub(super) fn enqueue_stack_request_response(
        &self,
        player: &Player,
        screen: &Arc<std::sync::Mutex<dyn ScreenHandler>>,
        slots: &[AcknowledgedSlots],
        response: &CItemStackResponse,
        owner: Option<&ScreenOwner>,
    ) -> bool {
        use pumpkin_protocol::{Packet, serial::PacketRead};

        let Ok(data) = self.serialize_packet(response) else {
            return false;
        };
        let invalid = std::cell::RefCell::new(None);
        self.try_enqueue_packet_data_projecting(
            data,
            || {
                let (active, handler) = if let Some(owner) = owner {
                    let Some((active, handler)) = player.lock_screen_action(owner) else {
                        invalid.replace(Some(vec![true; slots.len()]));
                        return None;
                    };
                    (Some(active), handler)
                } else {
                    (
                        None,
                        screen
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner),
                    )
                };
                let superseded: Vec<_> = slots
                    .iter()
                    .map(|request| {
                        request.iter().any(|(slot, expected)| {
                            let actual = get_slot_stack(&*handler, slot, None);
                            actual.uid != expected.uid || !actual.are_equal(expected)
                        })
                    })
                    .collect();
                if superseded.iter().any(|changed| *changed) {
                    invalid.replace(Some(superseded));
                    return None;
                }
                Some((active, handler))
            },
            |data, primary, keep| {
                if keep.is_some() {
                    Some(data)
                } else {
                    (!primary
                        && !VarUInt::read(&mut data.as_ref())
                            .is_ok_and(|id| id.0 == CItemStackResponse::PACKET_ID as u32))
                    .then_some(data)
                }
            },
            |_, _, _| {},
        );
        let Some(invalid) = invalid.into_inner() else {
            return true;
        };
        // These are status acknowledgements of already committed actions. Send them
        // through normal protocol translation, and follow with authoritative slots.
        self.try_enqueue_client_packet(&CItemStackResponse {
            responses: response
                .responses
                .iter()
                .zip(invalid)
                .map(|(original, changed)| ItemStackResponseInfo {
                    result: if changed { 1 } else { original.result },
                    client_request_id: original.client_request_id,
                    containers: Vec::new(),
                })
                .collect(),
        });
        false
    }
}

fn resync_stack_request_screens(player: &Player) {
    let personal: pumpkin_inventory::screen_handler::SharedScreenHandler =
        player.player_screen_handler.clone();
    if !player.is_active_screen(&personal) {
        player.resync_player_inventory_screen();
    }
    player.resync_active_inventory_screen();
}

pub(super) type AcknowledgedSlots = Vec<(ItemStackRequestSlotInfo, ItemStack)>;

fn final_acknowledged_slots(
    screen: &dyn ScreenHandler,
    mut requests: Vec<AcknowledgedSlots>,
    responses: &mut [ItemStackResponseInfo],
) -> Vec<AcknowledgedSlots> {
    let mut final_stacks = std::collections::HashMap::new();
    // Several wire slot names can refer to the same physical slot. Validate
    // every acknowledgement against the final committed or reported stack.
    for (request, response) in requests.iter_mut().zip(responses).rev() {
        let failed = response.result != 0;
        for (slot, expected) in request.iter_mut().rev() {
            let name = slot.container_name.container_name;
            let index = map_bedrock_container_slot(screen, name, slot.slot_id);
            if index.is_some() || name == ContainerName::Cursor {
                let (final_stack, from_failed_request) = final_stacks
                    .entry(index)
                    .or_insert_with(|| (expected.clone(), failed));
                if *from_failed_request && !failed {
                    response.containers.clear();
                }
                *expected = final_stack.clone();
            }
        }
        if failed {
            // Failed requests contribute writes for coalescing, but acknowledge no slots.
            request.clear();
        }
    }
    requests
}

/// Whether `action` touches an armor or offhand slot that doesn't resolve on the open screen.
/// Such requests are rejected whole, before any stack changes.
fn touches_unroutable_slot(
    screen_handler: &dyn ScreenHandler,
    action: &pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestAction,
) -> bool {
    use pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestAction as A;
    let slots = match action {
        A::Take {
            source,
            destination,
            ..
        }
        | A::Place {
            source,
            destination,
            ..
        } => [Some(source), Some(destination)],
        A::Swap { slot1, slot2 } => [Some(slot1), Some(slot2)],
        A::Drop { source, .. } | A::Destroy { source, .. } | A::Consume { source, .. } => {
            [Some(source), None]
        }
        _ => [None, None],
    };
    slots.into_iter().flatten().any(|slot| {
        let name = slot.container_name.container_name;
        matches!(name, ContainerName::Armor | ContainerName::Offhand)
            && map_bedrock_container_slot(screen_handler, name, slot.slot_id).is_none()
    })
}

#[allow(clippy::too_many_lines)]
pub(crate) fn map_bedrock_container_slot(
    screen_handler: &dyn ScreenHandler,
    container_name: ContainerName,
    slot_id: u8,
) -> Option<usize> {
    let mount_equip_slot = screen_handler
        .as_any()
        .downcast_ref::<MountScreenHandler>()
        .and_then(|mount| mount.bedrock_equip_slot);
    let is_player_screen = screen_handler.window_type().is_none() && mount_equip_slot.is_none();
    let container_slots = screen_handler
        .get_behaviour()
        .slots
        .len()
        .saturating_sub(PlayerInventory::MAIN_SIZE);

    if let Some(equip_slot) = mount_equip_slot {
        match container_name {
            ContainerName::HorseEquip => return Some(equip_slot),
            ContainerName::LevelEntity => {
                let slot = slot_id as usize + 1;
                return (slot >= MountScreenHandler::SLOT_INVENTORY_START
                    && slot < container_slots)
                    .then_some(slot);
            }
            _ => {}
        }
    }

    match container_name {
        ContainerName::HotBar => {
            if is_player_screen {
                Some(36 + slot_id as usize)
            } else {
                Some(container_slots + 27 + slot_id as usize)
            }
        }
        ContainerName::Inventory | ContainerName::CombinedHotBarAndInventory => {
            if slot_id < 9 {
                if is_player_screen {
                    Some(36 + slot_id as usize)
                } else {
                    Some(container_slots + 27 + slot_id as usize)
                }
            } else if slot_id < 36 {
                if is_player_screen {
                    Some(slot_id as usize)
                } else {
                    Some(container_slots + (slot_id - 9) as usize)
                }
            } else {
                None
            }
        }
        // Armor and offhand live only on the player's own screen.
        ContainerName::Armor => (is_player_screen && slot_id < 4).then(|| 5 + slot_id as usize),
        ContainerName::Offhand => {
            (is_player_screen && u32::from(slot_id) <= super::OFF_HAND_BEDROCK_SLOT).then_some(45)
        }
        ContainerName::Cursor => None,
        ContainerName::CraftingInput => {
            if is_player_screen {
                if slot_id < 4 {
                    Some(1 + slot_id as usize)
                } else if (28..32).contains(&slot_id) {
                    Some(1 + (slot_id - 28) as usize)
                } else {
                    None
                }
            } else if screen_handler.window_type()
                == Some(pumpkin_data::screen::WindowType::Crafting)
            {
                if slot_id < 9 {
                    Some(1 + slot_id as usize)
                } else if (32..41).contains(&slot_id) {
                    Some(1 + (slot_id - 32) as usize)
                } else {
                    None
                }
            } else {
                None
            }
        }
        ContainerName::CraftingOutputPreview | ContainerName::CreatedOutput => {
            if is_player_screen {
                Some(0)
            } else if let Some(window_type) = screen_handler.window_type() {
                match window_type {
                    pumpkin_data::screen::WindowType::Crafting => Some(0),
                    pumpkin_data::screen::WindowType::Stonecutter => Some(1),
                    pumpkin_data::screen::WindowType::Anvil
                    | pumpkin_data::screen::WindowType::Furnace
                    | pumpkin_data::screen::WindowType::BlastFurnace
                    | pumpkin_data::screen::WindowType::Smoker
                    | pumpkin_data::screen::WindowType::Grindstone
                    | pumpkin_data::screen::WindowType::Merchant
                    | pumpkin_data::screen::WindowType::CartographyTable => Some(2),
                    pumpkin_data::screen::WindowType::Loom
                    | pumpkin_data::screen::WindowType::Smithing => Some(3),
                    _ => None,
                }
            } else {
                None
            }
        }
        ContainerName::AnvilInput => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Anvil)
        )
        .then_some(0),
        ContainerName::AnvilMaterial => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Anvil)
        )
        .then_some(1),
        ContainerName::AnvilResultPreview => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Anvil)
        )
        .then_some(2),
        ContainerName::BeaconPayment => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Beacon)
        )
        .then_some(0),
        ContainerName::BrewingStandResult => (matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::BrewingStand)
        ) && slot_id < 3)
            .then_some(slot_id as usize),
        ContainerName::BrewingStandInput => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::BrewingStand)
        )
        .then_some(3),
        ContainerName::BrewingStandFuel => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::BrewingStand)
        )
        .then_some(4),
        ContainerName::FurnaceIngredient
        | ContainerName::BlastFurnaceIngredient
        | ContainerName::SmokerIngredient => matches!(
            screen_handler.window_type(),
            Some(
                pumpkin_data::screen::WindowType::Furnace
                    | pumpkin_data::screen::WindowType::BlastFurnace
                    | pumpkin_data::screen::WindowType::Smoker
            )
        )
        .then_some(0),
        ContainerName::FurnaceFuel => matches!(
            screen_handler.window_type(),
            Some(
                pumpkin_data::screen::WindowType::Furnace
                    | pumpkin_data::screen::WindowType::BlastFurnace
                    | pumpkin_data::screen::WindowType::Smoker
            )
        )
        .then_some(1),
        ContainerName::FurnaceResult => matches!(
            screen_handler.window_type(),
            Some(
                pumpkin_data::screen::WindowType::Furnace
                    | pumpkin_data::screen::WindowType::BlastFurnace
                    | pumpkin_data::screen::WindowType::Smoker
            )
        )
        .then_some(2),
        ContainerName::EnchantingInput => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Enchantment)
        )
        .then_some(0),
        ContainerName::EnchantingMaterial => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Enchantment)
        )
        .then_some(1),
        ContainerName::GrindstoneInput => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Grindstone)
        )
        .then_some(0),
        ContainerName::GrindstoneAdditional => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Grindstone)
        )
        .then_some(1),
        ContainerName::GrindstoneResultPreview => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Grindstone)
        )
        .then_some(2),
        ContainerName::LoomInput => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Loom)
        )
        .then_some(0),
        ContainerName::LoomDye => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Loom)
        )
        .then_some(1),
        ContainerName::LoomMaterial => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Loom)
        )
        .then_some(2),
        ContainerName::LoomResultPreview => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Loom)
        )
        .then_some(3),
        ContainerName::StonecutterInput => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Stonecutter)
        )
        .then_some(0),
        ContainerName::StonecutterResultPreview => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Stonecutter)
        )
        .then_some(1),
        ContainerName::CartographyInput => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::CartographyTable)
        )
        .then_some(0),
        ContainerName::CartographyAdditional => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::CartographyTable)
        )
        .then_some(1),
        ContainerName::CartographyResultPreview => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::CartographyTable)
        )
        .then_some(2),
        ContainerName::SmithingTableTemplate => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Smithing)
        )
        .then_some(0),
        ContainerName::SmithingTableInput => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Smithing)
        )
        .then_some(1),
        ContainerName::SmithingTableMaterial => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Smithing)
        )
        .then_some(2),
        ContainerName::SmithingTableResultPreview => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Smithing)
        )
        .then_some(3),
        ContainerName::TradeIngredient1 | ContainerName::Trade2Ingredient1 => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Merchant)
        )
        .then_some(0),
        ContainerName::TradeIngredient2 | ContainerName::Trade2Ingredient2 => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Merchant)
        )
        .then_some(1),
        ContainerName::TradeResultPreview | ContainerName::Trade2ResultPreview => matches!(
            screen_handler.window_type(),
            Some(pumpkin_data::screen::WindowType::Merchant)
        )
        .then_some(2),
        _ => ((slot_id as usize) < container_slots).then_some(slot_id as usize),
    }
}

pub(crate) struct SlotUpdate {
    pub(crate) container_name: FullContainerName,
    pub(crate) slot_id: u8,
    pub(crate) count: u8,
    pub(crate) stack_id: VarInt,
    physical: bool,
    expected: ItemStack,
    write_order: usize,
    committed: Option<(usize, ItemStack)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SlotRecord {
    Write,
    Read,
    Virtual,
}

pub(crate) fn record_update(
    updates: &mut Vec<SlotUpdate>,
    container_name: FullContainerName,
    slot_id: u8,
    stack: &ItemStack,
) {
    record_slot_update(updates, container_name, slot_id, stack, SlotRecord::Write);
}

fn record_physical_read(
    updates: &mut Vec<SlotUpdate>,
    container_name: FullContainerName,
    slot_id: u8,
    stack: &ItemStack,
) {
    record_slot_update(updates, container_name, slot_id, stack, SlotRecord::Read);
}

fn record_slot_update(
    updates: &mut Vec<SlotUpdate>,
    container_name: FullContainerName,
    slot_id: u8,
    stack: &ItemStack,
    record: SlotRecord,
) {
    let physical = record != SlotRecord::Virtual;
    let write_order = updates
        .iter()
        .map(|update| update.write_order)
        .max()
        .map_or(0, |order| order + 1);
    let committed = (record == SlotRecord::Write).then(|| (write_order, stack.clone()));
    let count = stack.item_count;
    let stack_id = if stack.is_empty() {
        VarInt(0)
    } else {
        VarInt(stack.uid.get())
    };
    if let Some(existing) = updates
        .iter_mut()
        .find(|u| u.container_name == container_name && u.slot_id == slot_id)
    {
        existing.count = count;
        existing.stack_id = stack_id;
        existing.physical = physical;
        existing.expected = stack.clone();
        existing.write_order = write_order;
        if committed.is_some() {
            existing.committed = committed;
        }
    } else {
        updates.push(SlotUpdate {
            container_name,
            slot_id,
            count,
            stack_id,
            physical,
            expected: stack.clone(),
            write_order,
            committed,
        });
    }
}

fn record_consumed_source(
    screen: &mut dyn ScreenHandler,
    created_item: &mut Option<ItemStack>,
    updates: &mut Vec<SlotUpdate>,
    source: &ItemStackRequestSlotInfo,
    stack: &ItemStack,
) {
    let physical = !virtual_craft_source(source, created_item.as_ref());
    let stack = if stack.is_empty() {
        ItemStack::EMPTY
    } else {
        stack
    };
    if physical {
        update_slot_stack(screen, source, stack.clone());
    } else {
        *created_item = (!stack.is_empty()).then(|| stack.clone());
    }
    record_slot_update(
        updates,
        source.container_name.clone(),
        source.slot_id,
        stack,
        if physical {
            SlotRecord::Write
        } else {
            SlotRecord::Virtual
        },
    );
}

fn virtual_craft_source(slot: &ItemStackRequestSlotInfo, created_item: Option<&ItemStack>) -> bool {
    slot.container_name.container_name == ContainerName::CreatedOutput && created_item.is_some()
}

fn get_slot_stack(
    screen_handler: &dyn ScreenHandler,
    slot_info: &pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestSlotInfo,
    created_item: Option<&ItemStack>,
) -> ItemStack {
    if let (ContainerName::CreatedOutput, Some(stack)) =
        (slot_info.container_name.container_name, created_item)
    {
        return stack.clone();
    }
    if slot_info.container_name.container_name == ContainerName::Cursor {
        return screen_handler
            .get_behaviour()
            .cursor_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
    }
    map_bedrock_container_slot(
        screen_handler,
        slot_info.container_name.container_name,
        slot_info.slot_id,
    )
    .map_or_else(
        || ItemStack::EMPTY.clone(),
        |screen_slot| screen_handler.get_behaviour().slots[screen_slot].get_cloned_stack(),
    )
}

#[allow(clippy::unreachable)]
fn update_slot_stack(
    screen_handler: &mut dyn ScreenHandler,
    slot_info: &pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestSlotInfo,
    new_stack: ItemStack,
) {
    if slot_info.container_name.container_name == ContainerName::Cursor {
        let mut cursor_lock = screen_handler
            .get_behaviour()
            .cursor_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *cursor_lock = new_stack;
        screen_handler.get_behaviour().invalidate_sync();
        return;
    }
    if let Some(screen_slot) = map_bedrock_container_slot(
        screen_handler,
        slot_info.container_name.container_name,
        slot_info.slot_id,
    ) {
        screen_handler.get_behaviour().slots[screen_slot].set_stack(new_stack.clone());
        screen_handler.set_received_stack(screen_slot, new_stack);
    }
}

fn get_output_screen_slot(
    screen_handler: &dyn ScreenHandler,
    source: &ItemStackRequestSlotInfo,
) -> Option<usize> {
    match source.container_name.container_name {
        ContainerName::CreatedOutput | ContainerName::CraftingOutputPreview => {
            map_bedrock_container_slot(
                screen_handler,
                source.container_name.container_name,
                source.slot_id,
            )
        }
        ContainerName::AnvilResultPreview => {
            (screen_handler.window_type() == Some(WindowType::Anvil)).then_some(2)
        }
        ContainerName::GrindstoneResultPreview => {
            (screen_handler.window_type() == Some(WindowType::Grindstone)).then_some(2)
        }
        ContainerName::StonecutterResultPreview => {
            (screen_handler.window_type() == Some(WindowType::Stonecutter)).then_some(1)
        }
        ContainerName::LoomResultPreview => {
            (screen_handler.window_type() == Some(WindowType::Loom)).then_some(3)
        }
        ContainerName::SmithingTableResultPreview => {
            (screen_handler.window_type() == Some(WindowType::Smithing)).then_some(3)
        }
        ContainerName::CartographyResultPreview => {
            (screen_handler.window_type() == Some(WindowType::CartographyTable)).then_some(2)
        }
        ContainerName::FurnaceResult => matches!(
            screen_handler.window_type(),
            Some(WindowType::Furnace | WindowType::BlastFurnace | WindowType::Smoker)
        )
        .then_some(2),
        ContainerName::TradeResultPreview | ContainerName::Trade2ResultPreview => {
            (screen_handler.window_type() == Some(WindowType::Merchant)).then_some(2)
        }
        ContainerName::BrewingStandResult => {
            (matches!(screen_handler.window_type(), Some(WindowType::BrewingStand))
                && source.slot_id < 3)
                .then_some(source.slot_id as usize)
        }
        _ => None,
    }
}

fn record_preview_update(
    updates: &mut Vec<SlotUpdate>,
    container_name: ContainerName,
    screen_slot: u8,
    stack: &ItemStack,
) {
    for slot_id in [0, screen_slot, 50] {
        record_update(
            updates,
            FullContainerName {
                container_name,
                dynamic_id: None,
            },
            slot_id,
            stack,
        );
    }
}

#[allow(clippy::too_many_lines)]
fn record_specialized_container_input_updates(
    screen_handler: &dyn ScreenHandler,
    updates: &mut Vec<SlotUpdate>,
) {
    let Some(window_type) = screen_handler.window_type() else {
        return;
    };
    match window_type {
        WindowType::Anvil => {
            let slot0 = screen_handler.get_behaviour().slots[0].get_cloned_stack();
            let slot1 = screen_handler.get_behaviour().slots[1].get_cloned_stack();
            for slot_id in [0, 1] {
                record_update(
                    updates,
                    FullContainerName {
                        container_name: ContainerName::AnvilInput,
                        dynamic_id: None,
                    },
                    slot_id,
                    &slot0,
                );
                record_update(
                    updates,
                    FullContainerName {
                        container_name: ContainerName::AnvilMaterial,
                        dynamic_id: None,
                    },
                    slot_id,
                    &slot1,
                );
            }
        }
        WindowType::Grindstone => {
            let slot0 = screen_handler.get_behaviour().slots[0].get_cloned_stack();
            let slot1 = screen_handler.get_behaviour().slots[1].get_cloned_stack();
            for slot_id in [0, 1] {
                record_update(
                    updates,
                    FullContainerName {
                        container_name: ContainerName::GrindstoneInput,
                        dynamic_id: None,
                    },
                    slot_id,
                    &slot0,
                );
                record_update(
                    updates,
                    FullContainerName {
                        container_name: ContainerName::GrindstoneAdditional,
                        dynamic_id: None,
                    },
                    slot_id,
                    &slot1,
                );
            }
        }
        WindowType::Stonecutter => {
            let slot0 = screen_handler.get_behaviour().slots[0].get_cloned_stack();
            record_update(
                updates,
                FullContainerName {
                    container_name: ContainerName::StonecutterInput,
                    dynamic_id: None,
                },
                0,
                &slot0,
            );
        }
        WindowType::Loom => {
            let slot0 = screen_handler.get_behaviour().slots[0].get_cloned_stack();
            let slot1 = screen_handler.get_behaviour().slots[1].get_cloned_stack();
            let slot2 = screen_handler.get_behaviour().slots[2].get_cloned_stack();
            for slot_id in [0, 1, 2] {
                record_update(
                    updates,
                    FullContainerName {
                        container_name: ContainerName::LoomInput,
                        dynamic_id: None,
                    },
                    slot_id,
                    &slot0,
                );
                record_update(
                    updates,
                    FullContainerName {
                        container_name: ContainerName::LoomDye,
                        dynamic_id: None,
                    },
                    slot_id,
                    &slot1,
                );
                record_update(
                    updates,
                    FullContainerName {
                        container_name: ContainerName::LoomMaterial,
                        dynamic_id: None,
                    },
                    slot_id,
                    &slot2,
                );
            }
        }
        WindowType::Smithing => {
            let slot0 = screen_handler.get_behaviour().slots[0].get_cloned_stack();
            let slot1 = screen_handler.get_behaviour().slots[1].get_cloned_stack();
            let slot2 = screen_handler.get_behaviour().slots[2].get_cloned_stack();
            for slot_id in [0, 1, 2] {
                record_update(
                    updates,
                    FullContainerName {
                        container_name: ContainerName::SmithingTableTemplate,
                        dynamic_id: None,
                    },
                    slot_id,
                    &slot0,
                );
                record_update(
                    updates,
                    FullContainerName {
                        container_name: ContainerName::SmithingTableInput,
                        dynamic_id: None,
                    },
                    slot_id,
                    &slot1,
                );
                record_update(
                    updates,
                    FullContainerName {
                        container_name: ContainerName::SmithingTableMaterial,
                        dynamic_id: None,
                    },
                    slot_id,
                    &slot2,
                );
            }
        }
        WindowType::CartographyTable => {
            let slot0 = screen_handler.get_behaviour().slots[0].get_cloned_stack();
            let slot1 = screen_handler.get_behaviour().slots[1].get_cloned_stack();
            for slot_id in [0, 1] {
                record_update(
                    updates,
                    FullContainerName {
                        container_name: ContainerName::CartographyInput,
                        dynamic_id: None,
                    },
                    slot_id,
                    &slot0,
                );
                record_update(
                    updates,
                    FullContainerName {
                        container_name: ContainerName::CartographyAdditional,
                        dynamic_id: None,
                    },
                    slot_id,
                    &slot1,
                );
            }
        }
        WindowType::Merchant => {
            let slot0 = screen_handler.get_behaviour().slots[0].get_cloned_stack();
            let slot1 = screen_handler.get_behaviour().slots[1].get_cloned_stack();
            record_update(
                updates,
                FullContainerName {
                    container_name: ContainerName::Trade2Ingredient1,
                    dynamic_id: None,
                },
                4,
                &slot0,
            );
            record_update(
                updates,
                FullContainerName {
                    container_name: ContainerName::Trade2Ingredient2,
                    dynamic_id: None,
                },
                5,
                &slot1,
            );
        }
        _ => {}
    }
}

#[allow(clippy::too_many_lines)]
fn refresh_specialized_screen_results(
    player: &Player,
    screen_handler: &mut dyn ScreenHandler,
    filter_strings: &[String],
    updates: &mut Vec<SlotUpdate>,
) {
    let Some(window_type) = screen_handler.window_type() else {
        return;
    };
    match window_type {
        WindowType::Anvil => {
            if let Some(anvil) = screen_handler
                .as_any_mut()
                .downcast_mut::<AnvilScreenHandler>()
            {
                let has_infinite = player.has_infinite_materials();
                if let Some(name) = filter_strings.first() {
                    anvil.set_item_name(name, has_infinite);
                } else {
                    anvil.create_result(has_infinite);
                }
                let result_stack = anvil.inventory.get_stack(2);
                record_preview_update(updates, ContainerName::AnvilResultPreview, 2, &result_stack);
            }
        }
        WindowType::Grindstone => {
            if let Some(grindstone) = screen_handler
                .as_any_mut()
                .downcast_mut::<GrindstoneScreenHandler>()
            {
                grindstone.create_result();
                let result_stack = grindstone.inventory.get_stack(2);
                record_preview_update(
                    updates,
                    ContainerName::GrindstoneResultPreview,
                    2,
                    &result_stack,
                );
            }
        }
        WindowType::Stonecutter => {
            if let Some(stonecutter) = screen_handler
                .as_any_mut()
                .downcast_mut::<StonecutterScreenHandler>()
            {
                stonecutter.update_output();
                let result_stack = stonecutter.output_inventory.get_stack(0);
                record_preview_update(
                    updates,
                    ContainerName::StonecutterResultPreview,
                    1,
                    &result_stack,
                );
            }
        }
        WindowType::Loom => {
            if let Some(loom) = screen_handler
                .as_any_mut()
                .downcast_mut::<LoomScreenHandler>()
            {
                loom.slots_changed();
                let result_stack = loom.output_inventory.get_stack(0);
                record_preview_update(updates, ContainerName::LoomResultPreview, 3, &result_stack);
            }
        }
        WindowType::Smithing => {
            if let Some(smithing) = screen_handler
                .as_any_mut()
                .downcast_mut::<SmithingTableScreenHandler>()
            {
                smithing.update_output();
                let result_stack = smithing.output_inventory.get_stack(0);
                record_preview_update(
                    updates,
                    ContainerName::SmithingTableResultPreview,
                    3,
                    &result_stack,
                );
            }
        }
        WindowType::CartographyTable => {
            if let Some(cartography) = screen_handler
                .as_any_mut()
                .downcast_mut::<CartographyTableScreenHandler>()
            {
                cartography.slots_changed();
                let result_stack = cartography.output_inventory.get_stack(0);
                record_preview_update(
                    updates,
                    ContainerName::CartographyResultPreview,
                    2,
                    &result_stack,
                );
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::item::Item;
    use pumpkin_inventory::{
        SimpleInventory, build_equipment_slots,
        crafting::crafting_screen_handler::CraftingTableScreenHandler,
        entity_equipment::EntityEquipment,
    };
    use std::sync::Mutex;

    #[tokio::test]
    async fn crafting_table_maps_bedrock_player_inventory_after_its_ten_slots() {
        let inventory = Arc::new(PlayerInventory::new(
            Arc::new(Mutex::new(EntityEquipment::new())),
            Arc::new(build_equipment_slots()),
        ));
        let handler = CraftingTableScreenHandler::new(1, &inventory, None);

        assert_eq!(
            map_bedrock_container_slot(&handler, ContainerName::Inventory, 26),
            Some(27)
        );
        assert_eq!(
            map_bedrock_container_slot(&handler, ContainerName::HotBar, 0),
            Some(37)
        );
        assert_eq!(
            map_bedrock_container_slot(&handler, ContainerName::CraftingInput, 32),
            Some(1)
        );
        assert_eq!(
            map_bedrock_container_slot(&handler, ContainerName::CraftingInput, 40),
            Some(9)
        );
    }

    #[test]
    fn physical_read_does_not_replace_the_committed_write() {
        let container = FullContainerName {
            container_name: ContainerName::HotBar,
            dynamic_id: None,
        };
        let first = ItemStack::new(1, &Item::STICK);
        let later = ItemStack::new(2, &Item::STICK);
        let mut updates = Vec::new();
        record_physical_read(&mut updates, container.clone(), 0, &first);
        assert!(updates[0].committed.is_none());

        record_update(&mut updates, container.clone(), 0, &first);
        record_physical_read(&mut updates, container, 0, &later);
        assert_eq!(updates[0].expected.uid, later.uid);
        assert_eq!(
            updates[0]
                .committed
                .as_ref()
                .map(|(order, stack)| (*order, stack.uid)),
            Some((1, first.uid))
        );
    }

    #[test]
    fn acknowledgements_keep_the_last_write_across_slot_aliases() {
        let inventory = Arc::new(PlayerInventory::new(
            Arc::new(Mutex::new(EntityEquipment::new())),
            Arc::new(build_equipment_slots()),
        ));
        let handler = CraftingTableScreenHandler::new(1, &inventory, None);
        let first = ItemStack::new(2, &Item::STICK);
        let last = ItemStack::new(3, &Item::STICK);
        let replacement = ItemStack::new(3, &Item::STICK);
        let ack = |container_name, stack: &ItemStack| {
            let container = FullContainerName {
                container_name,
                dynamic_id: None,
            };
            let mut updates = Vec::new();
            record_update(&mut updates, container.clone(), 0, stack);
            let update = updates.remove(0);
            vec![(
                ItemStackRequestSlotInfo {
                    container_name: container,
                    slot_id: update.slot_id,
                    stack_id: update.stack_id.0,
                },
                update.expected,
            )]
        };
        let requests = vec![
            ack(ContainerName::HotBar, &first),
            ack(ContainerName::Inventory, &last),
        ];
        inventory.set_slot(0, replacement.clone());

        let mut responses = std::array::from_fn::<_, 2, _>(|id| ItemStackResponseInfo {
            result: 0,
            client_request_id: (id as i32).into(),
            containers: Vec::new(),
        });
        let requests = final_acknowledged_slots(&handler, requests, &mut responses);
        assert_eq!(requests[0][0].0.stack_id, first.uid.get());
        assert_eq!(requests[1][0].0.stack_id, last.uid.get());
        for request in requests {
            let (slot, expected) = &request[0];
            assert_eq!(expected.uid, last.uid);
            assert_eq!(get_slot_stack(&handler, slot, None).uid, replacement.uid);
        }
    }

    #[test]
    fn item_stack_response_uses_the_authoritative_stack_id() {
        let stack = ItemStack::new(3, &Item::SPRUCE_DOOR);
        let container = FullContainerName {
            container_name: ContainerName::Cursor,
            dynamic_id: None,
        };
        let mut updates = Vec::new();

        record_update(&mut updates, container.clone(), 0, &stack);
        assert_eq!(updates[0].count, 3);
        assert_eq!(updates[0].stack_id, VarInt(stack.uid.get()));

        record_update(&mut updates, container, 0, ItemStack::EMPTY);
        assert_eq!(updates[0].count, 0);
        assert_eq!(updates[0].stack_id, VarInt(0));
    }

    #[tokio::test]
    async fn specialized_containers_map_output_slots_correctly() {
        let inventory = Arc::new(PlayerInventory::new(
            Arc::new(Mutex::new(EntityEquipment::new())),
            Arc::new(build_equipment_slots()),
        ));

        let make_slot = |container_name: ContainerName, slot_id: u8| ItemStackRequestSlotInfo {
            container_name: FullContainerName {
                container_name,
                dynamic_id: None,
            },
            slot_id,
            stack_id: 0,
        };

        // Stonecutter
        let stonecutter = StonecutterScreenHandler::new(1, &inventory, None);
        assert_eq!(
            get_output_screen_slot(
                &stonecutter,
                &make_slot(ContainerName::StonecutterResultPreview, 0)
            ),
            Some(1)
        );
        assert_eq!(
            get_output_screen_slot(&stonecutter, &make_slot(ContainerName::CreatedOutput, 0)),
            Some(1)
        );
        assert_eq!(
            get_output_screen_slot(&stonecutter, &make_slot(ContainerName::StonecutterInput, 0)),
            None
        );

        // Anvil
        let anvil_inv = Arc::new(SimpleInventory::new(3));
        let anvil = AnvilScreenHandler::new(1, &inventory, anvil_inv);
        assert_eq!(
            get_output_screen_slot(&anvil, &make_slot(ContainerName::AnvilResultPreview, 0)),
            Some(2)
        );
        assert_eq!(
            get_output_screen_slot(&anvil, &make_slot(ContainerName::CreatedOutput, 0)),
            Some(2)
        );
        assert_eq!(
            get_output_screen_slot(&anvil, &make_slot(ContainerName::AnvilInput, 0)),
            None
        );

        // Grindstone
        let grindstone_inv = Arc::new(SimpleInventory::new(3));
        let grindstone = GrindstoneScreenHandler::new(1, &inventory, grindstone_inv);
        assert_eq!(
            get_output_screen_slot(
                &grindstone,
                &make_slot(ContainerName::GrindstoneResultPreview, 0)
            ),
            Some(2)
        );
        assert_eq!(
            get_output_screen_slot(&grindstone, &make_slot(ContainerName::CreatedOutput, 0)),
            Some(2)
        );

        // Loom
        let loom = LoomScreenHandler::new(1, &inventory);
        assert_eq!(
            get_output_screen_slot(&loom, &make_slot(ContainerName::LoomResultPreview, 0)),
            Some(3)
        );
        assert_eq!(
            get_output_screen_slot(&loom, &make_slot(ContainerName::CreatedOutput, 0)),
            Some(3)
        );

        // Smithing Table
        let smithing = SmithingTableScreenHandler::new(1, &inventory, None);
        assert_eq!(
            get_output_screen_slot(
                &smithing,
                &make_slot(ContainerName::SmithingTableResultPreview, 0)
            ),
            Some(3)
        );
        assert_eq!(
            get_output_screen_slot(&smithing, &make_slot(ContainerName::CreatedOutput, 0)),
            Some(3)
        );

        // Cartography Table
        let cartography = CartographyTableScreenHandler::new(1, &inventory);
        assert_eq!(
            get_output_screen_slot(
                &cartography,
                &make_slot(ContainerName::CartographyResultPreview, 0)
            ),
            Some(2)
        );
        assert_eq!(
            get_output_screen_slot(&cartography, &make_slot(ContainerName::CreatedOutput, 0)),
            Some(2)
        );
    }
}
