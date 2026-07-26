use super::{get_slot_stack, record_update, update_slot_stack};
use crate::entity::player::Player;
use crate::net::bedrock::BedrockClient;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_protocol::bedrock::client::inventory_content::CInventoryContent;
use pumpkin_protocol::bedrock::network_item::ContainerName;
use pumpkin_protocol::bedrock::network_item::FullContainerName;
use pumpkin_protocol::bedrock::network_item::NetworkItemStackDescriptor;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_uint::VarUInt;
use std::sync::Arc;

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
            let mut updates = Vec::new();
            let mut result = 0u8; // 0 = Success, 1 = Error

            for action in request.actions {
                tracing::info!("Processing ItemStackRequestAction: {:?}", action);
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
                    }
                    | ItemStackRequestAction::PlaceInContainer {
                        count,
                        source,
                        destination,
                    }
                    | ItemStackRequestAction::TakeOutContainer {
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

                            source_stack.decrement(count);
                            if source.container_name.container_name == ContainerName::CreatedOutput
                            {
                                if let Some(ref mut stack) = created_item {
                                    stack.decrement(count);
                                    if stack.is_empty() {
                                        created_item = None;
                                    }
                                }
                            } else if source.container_name.container_name == ContainerName::Cursor
                            {
                                let cursor_is_empty = screen_handler
                                    .get_behaviour()
                                    .cursor_stack
                                    .lock()
                                    .await
                                    .is_empty();
                                if cursor_is_empty && let Some(ref mut stack) = created_item {
                                    stack.decrement(count);
                                    if stack.is_empty() {
                                        created_item = None;
                                    }
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
                                source_stack.item_count,
                                source.stack_id,
                            );
                            record_update(
                                &mut updates,
                                destination.container_name.clone(),
                                destination.slot_id,
                                dest_stack.item_count,
                                destination.stack_id,
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
                            stack2.item_count,
                            slot2.stack_id,
                        );
                        record_update(
                            &mut updates,
                            slot2.container_name.clone(),
                            slot2.slot_id,
                            stack1.item_count,
                            slot1.stack_id,
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
                                source_stack.item_count,
                                source.stack_id,
                            );
                        }
                    }
                    ItemStackRequestAction::Destroy { count, source }
                    | ItemStackRequestAction::Consume { count, source } => {
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
                                source_stack.item_count,
                                source.stack_id,
                            );
                        }
                    }
                    ItemStackRequestAction::CraftRecipe {
                        recipe_id: _,
                        repetitions,
                    }
                    | ItemStackRequestAction::CraftRecipeAuto {
                        recipe_id: _,
                        repetitions,
                        ..
                    } => {
                        if repetitions > 0 {
                            screen_handler.update_to_client().await;

                            let is_player = screen_handler.window_type().is_none();
                            let grid_size = if is_player { 4 } else { 9 };
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

                            if output_stack.is_empty() {
                                tracing::warn!("Client tried to craft, but output slot is empty!");
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
                                    i as u8,
                                    grid_stack.item_count,
                                    VarInt(0),
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
        self.enqueue_packet(&CItemStackResponse { responses }).await;

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
    }
}
