#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    #[allow(clippy::too_many_lines)]
    pub async fn handle_item_stack_request(
        &self,
        player: &Arc<Player>,
        packet: pumpkin_protocol::bedrock::server::item_stack_request::SItemStackRequest,
    ) {
        use pumpkin_protocol::bedrock::client::item_stack_response::{
            CItemStackResponse, ItemStackResponse, ItemStackResponseContainerInfo,
            ItemStackResponseSlotInfo,
        };
        use pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestAction;

        let current_screen_handler = player.current_screen_handler.lock().await.clone();
        let mut screen_handler = current_screen_handler.lock().await;

        let mut responses = Vec::with_capacity(packet.requests.len());

        for request in packet.requests {
            let mut created_item: Option<ItemStack> = None;
            let mut crafting_inputs_consumed = false;
            let mut updates = Vec::new();
            let mut result = 0u8; // 0 = Success, 1 = Error

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
                                let max_stack = ItemStack::static_new_java(1, mapping.java_item)
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
                            get_slot_stack(&*screen_handler, &source, created_item.as_ref()).await;
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
                            )
                            .await;
                            if dest_stack.is_empty() {
                                dest_stack = source_stack.copy_with_count(count);
                            } else if dest_stack.are_items_and_components_equal(&source_stack) {
                                dest_stack.item_count = dest_stack.item_count.saturating_add(count);
                            } else {
                                tracing::debug!(
                                    "Destination stack is not compatible with source stack"
                                );
                                result = 1;
                                break;
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
                                if !handler.complete_bedrock_trade(player.as_ref()).await {
                                    result = 1;
                                    break;
                                }

                                update_slot_stack(
                                    player,
                                    handler,
                                    &destination,
                                    dest_stack.clone(),
                                )
                                .await;
                                for (container_name, slot_id, screen_slot) in [
                                    (ContainerName::Trade2Ingredient1, 4, 0),
                                    (ContainerName::Trade2Ingredient2, 5, 1),
                                    (ContainerName::Trade2ResultPreview, 50, 2),
                                ] {
                                    let stack = handler.get_behaviour().slots[screen_slot]
                                        .get_cloned_stack()
                                        .await;
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

                            source_stack.decrement(count);
                            if source.container_name.container_name == ContainerName::CreatedOutput
                                && let Some(ref mut stack) = created_item
                            {
                                stack.decrement(count);
                                if stack.is_empty() {
                                    created_item = None;
                                }
                            }
                            let source_stack = if source_stack.is_empty() {
                                ItemStack::EMPTY.clone()
                            } else {
                                source_stack
                            };

                            update_slot_stack(
                                player,
                                &mut *screen_handler,
                                &source,
                                source_stack.clone(),
                            )
                            .await;
                            update_slot_stack(
                                player,
                                &mut *screen_handler,
                                &destination,
                                dest_stack.clone(),
                            )
                            .await;

                            record_update(
                                &mut updates,
                                source.container_name.clone(),
                                source.slot_id,
                                &source_stack,
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
                        let stack1 =
                            get_slot_stack(&*screen_handler, &slot1, created_item.as_ref()).await;
                        let stack2 =
                            get_slot_stack(&*screen_handler, &slot2, created_item.as_ref()).await;

                        update_slot_stack(player, &mut *screen_handler, &slot1, stack2.clone())
                            .await;
                        update_slot_stack(player, &mut *screen_handler, &slot2, stack1.clone())
                            .await;

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
                            get_slot_stack(&*screen_handler, &source, created_item.as_ref()).await;
                        if source_stack.is_empty() {
                            result = 1;
                            break;
                        }
                        let count = count.min(source_stack.item_count);
                        if count > 0 {
                            let dropped_stack = source_stack.copy_with_count(count);
                            player.drop_item(dropped_stack).await;

                            source_stack.decrement(count);
                            let source_stack = if source_stack.is_empty() {
                                ItemStack::EMPTY.clone()
                            } else {
                                source_stack
                            };

                            update_slot_stack(
                                player,
                                &mut *screen_handler,
                                &source,
                                source_stack.clone(),
                            )
                            .await;

                            record_update(
                                &mut updates,
                                source.container_name.clone(),
                                source.slot_id,
                                &source_stack,
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
                            && source.container_name.container_name == ContainerName::CraftingInput
                        {
                            let source_stack =
                                get_slot_stack(&*screen_handler, &source, created_item.as_ref())
                                    .await;
                            record_update(
                                &mut updates,
                                source.container_name.clone(),
                                source.slot_id,
                                &source_stack,
                            );
                            continue;
                        }

                        let mut source_stack =
                            get_slot_stack(&*screen_handler, &source, created_item.as_ref()).await;
                        if source_stack.is_empty() {
                            result = 1;
                            break;
                        }
                        let count = count.min(source_stack.item_count);
                        if count > 0 {
                            source_stack.decrement(count);
                            let source_stack = if source_stack.is_empty() {
                                ItemStack::EMPTY.clone()
                            } else {
                                source_stack
                            };

                            update_slot_stack(
                                player,
                                &mut *screen_handler,
                                &source,
                                source_stack.clone(),
                            )
                            .await;

                            record_update(
                                &mut updates,
                                source.container_name.clone(),
                                source.slot_id,
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
                            handler.set_selected_offer(trade).await;
                            for (container_name, slot_id, screen_slot) in [
                                (ContainerName::Trade2Ingredient1, 4, 0),
                                (ContainerName::Trade2Ingredient2, 5, 1),
                                (ContainerName::Trade2ResultPreview, 50, 2),
                            ] {
                                let stack = handler.get_behaviour().slots[screen_slot]
                                    .get_cloned_stack()
                                    .await;
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

                        if repetitions > 0 {
                            screen_handler.update_to_client().await;

                            let is_player = screen_handler.window_type().is_none();
                            let grid_size = if is_player { 4 } else { 9 };
                            let bedrock_grid_start = if is_player { 28 } else { 32 };
                            for i in 0..grid_size {
                                let grid_slot_index = 1 + i;
                                let grid_slot =
                                    screen_handler.get_behaviour().slots[grid_slot_index].clone();
                                let grid_stack = grid_slot.get_cloned_stack().await;
                                tracing::info!(
                                    "Crafting Grid slot {i} (slot index {grid_slot_index}): Item ID: {}, Count: {}",
                                    grid_stack.item.id,
                                    grid_stack.item_count
                                );
                            }

                            let output_slot = screen_handler.get_behaviour().slots[0].clone();
                            let output_stack = output_slot.get_cloned_stack().await;

                            if output_stack.is_empty()
                                || repetitions > output_slot.get_max_item_count().await
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
                                output_slot
                                    .on_take_item(player.as_ref(), &output_stack)
                                    .await;
                            }
                            crafting_inputs_consumed = true;

                            // Record updates for all grid slots so Bedrock client is notified of consumed ingredients!
                            let is_player = screen_handler.window_type().is_none();
                            let grid_size = if is_player { 4 } else { 9 };
                            for i in 0..grid_size {
                                let grid_slot_index = 1 + i;
                                let grid_slot =
                                    screen_handler.get_behaviour().slots[grid_slot_index].clone();
                                let grid_stack = grid_slot.get_cloned_stack().await;
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
                    ItemStackRequestAction::CraftResultsDeprecated { .. }
                    | ItemStackRequestAction::MineBlock { .. }
                    | ItemStackRequestAction::BeaconPayment { .. }
                    | ItemStackRequestAction::Create { .. }
                    | ItemStackRequestAction::LabTableCombine
                    | ItemStackRequestAction::Optional { .. }
                    | ItemStackRequestAction::Grindstone { .. }
                    | ItemStackRequestAction::Loom { .. }
                    | ItemStackRequestAction::CraftNonImplemented => {
                        // Successful no-ops to prevent client-side transaction rollbacks
                    }
                }
            }

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
                            container_name: update.container_name,
                            slots: vec![slot_info],
                        });
                    }
                }
            }

            responses.push(ItemStackResponse {
                result,
                request_id: request.request_id,
                container_infos,
            });
        }

        // Send updates to Java client
        screen_handler.send_content_updates().await;

        // Collect inventory updates if we modified player inventory
        let mut inventory_updated = false;
        for response in &responses {
            if response.result == 0 {
                for info in &response.container_infos {
                    if info.container_name.container_name == ContainerName::Inventory
                        || info.container_name.container_name
                            == ContainerName::CombinedHotBarAndInventory
                        || info.container_name.container_name == ContainerName::HotBar
                    {
                        inventory_updated = true;
                    }
                }
            }
        }

        // Send Bedrock specific responses and updates
        self.enqueue_client_packet(&CItemStackResponse { responses })
            .await;

        if inventory_updated {
            self.enqueue_client_packet(&CInventoryContent {
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
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn map_bedrock_container_slot(
    screen_handler: &dyn ScreenHandler,
    container_name: ContainerName,
    slot_id: u8,
) -> Option<usize> {
    let is_player_screen = screen_handler.window_type().is_none();
    let container_slots = screen_handler
        .get_behaviour()
        .slots
        .len()
        .saturating_sub(PlayerInventory::MAIN_SIZE);

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
        ContainerName::Armor => (slot_id < 4).then(|| 5 + slot_id as usize),
        ContainerName::Offhand => (slot_id == 0).then_some(45),
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
                    | pumpkin_data::screen::WindowType::Merchant => Some(2),
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
}

pub(crate) fn record_update(
    updates: &mut Vec<SlotUpdate>,
    container_name: FullContainerName,
    slot_id: u8,
    stack: &ItemStack,
) {
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
    } else {
        updates.push(SlotUpdate {
            container_name,
            slot_id,
            count,
            stack_id,
        });
    }
}

async fn get_slot_stack(
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
            .await
            .clone();
    }
    if let Some(screen_slot) = map_bedrock_container_slot(
        screen_handler,
        slot_info.container_name.container_name,
        slot_info.slot_id,
    ) {
        screen_handler.get_behaviour().slots[screen_slot]
            .get_cloned_stack()
            .await
    } else {
        ItemStack::EMPTY.clone()
    }
}

#[allow(clippy::unreachable)]
async fn update_slot_stack(
    player: &Player,
    screen_handler: &mut dyn ScreenHandler,
    slot_info: &pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestSlotInfo,
    new_stack: ItemStack,
) {
    if slot_info.container_name.container_name == ContainerName::Cursor {
        let mut cursor_lock = screen_handler.get_behaviour().cursor_stack.lock().await;
        *cursor_lock = new_stack;
        return;
    }
    if let Some(screen_slot) = map_bedrock_container_slot(
        screen_handler,
        slot_info.container_name.container_name,
        slot_info.slot_id,
    ) {
        let is_player_screen = screen_handler.window_type().is_none();
        if is_player_screen {
            let current_stack = screen_handler.get_behaviour().slots[screen_slot]
                .get_cloned_stack()
                .await;
            if !current_stack.are_items_and_components_equal(&new_stack) {
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
                            &new_stack,
                        )
                        .await;
                } else if (36..45).contains(&screen_slot) {
                    let hotbar_slot = screen_slot - 36;
                    if player.inventory().get_selected_slot() == hotbar_slot as u8 {
                        let equipment = &[(EquipmentSlot::MAIN_HAND, new_stack.clone())];
                        player.living_entity.send_equipment_changes(equipment);
                    }
                }
            }
        }

        screen_handler.get_behaviour().slots[screen_slot]
            .set_stack(new_stack.clone())
            .await;
        screen_handler.set_received_stack(screen_slot, new_stack);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::item::Item;
    use pumpkin_inventory::{
        build_equipment_slots, crafting::crafting_screen_handler::CraftingTableScreenHandler,
        entity_equipment::EntityEquipment,
    };
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn crafting_table_maps_bedrock_player_inventory_after_its_ten_slots() {
        let inventory = Arc::new(PlayerInventory::new(
            Arc::new(Mutex::new(EntityEquipment::new())),
            Arc::new(build_equipment_slots()),
        ));
        let handler = CraftingTableScreenHandler::new(1, &inventory, None).await;

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
}
