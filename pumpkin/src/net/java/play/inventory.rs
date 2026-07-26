use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::net::java::JavaClient;
use crate::plugin::player::item_held::PlayerItemHeldEvent;
use crate::server::Server;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::InventoryError;
use pumpkin_inventory::merchant::merchant_screen_handler::MerchantScreenHandler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_inventory::screen_handler::ScreenHandler;
use pumpkin_protocol::java::client::play::CSetSelectedSlot;
use pumpkin_protocol::java::server::play::SBundleItemSelected;
use pumpkin_protocol::java::server::play::SCloseContainer;
use pumpkin_protocol::java::server::play::SPickItemFromBlock;
use pumpkin_protocol::java::server::play::SPickItemFromEntity;
use pumpkin_protocol::java::server::play::SPlaceRecipe;
use pumpkin_protocol::java::server::play::SRecipeBookChangeSettings;
use pumpkin_protocol::java::server::play::SRecipeBookSeenRecipe;
use pumpkin_protocol::java::server::play::SSelectTrade;
use pumpkin_protocol::java::server::play::SSetCreativeSlot;
use pumpkin_protocol::java::server::play::SSetHeldItem;
use pumpkin_util::GameMode;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use tracing::debug;

impl JavaClient {
    pub async fn handle_pick_item_from_block(
        &self,
        player: &Arc<Player>,
        pick_item: SPickItemFromBlock,
    ) {
        if !player.can_interact_with_block_at(&pick_item.pos, 1.0) {
            return;
        }

        let world = player.world();
        let block = world.get_block(&pick_item.pos);

        if block.item_id == 0 {
            // Invalid block id (blocks such as tall seagrass)
            return;
        }

        let Some(item) = Item::from_id(block.item_id) else {
            return;
        };
        let stack = ItemStack::new(1, item);

        let slot_with_stack = player.inventory().get_slot_with_stack(&stack).await;

        if slot_with_stack != -1 {
            if PlayerInventory::is_valid_hotbar_index(slot_with_stack as usize) {
                player.inventory.set_selected_slot(slot_with_stack as u8);
            } else {
                player
                    .inventory
                    .swap_slot_with_hotbar(slot_with_stack as usize)
                    .await;
            }
        } else if player.gamemode.load() == GameMode::Creative {
            player.inventory.swap_stack_with_hotbar(stack).await;
        }

        player
            .client
            .enqueue_packet(&CSetSelectedSlot::new(
                player.inventory.get_selected_slot() as i8
            ))
            .await;
        player
            .player_screen_handler
            .lock()
            .await
            .send_content_updates()
            .await;
    }

    pub async fn handle_pick_item_from_entity(
        &self,
        player: &Arc<Player>,
        pick_item: SPickItemFromEntity,
    ) {
        use pumpkin_data::entity::{entity_from_egg, spawn_egg_ids};

        let world = player.world();
        let Some(target) = world.get_entity_by_id(pick_item.id.0) else {
            return;
        };

        let p_eye = player.get_entity().get_eye_pos();
        let t_eye = target.get_eye_pos();
        let dx = p_eye.x - t_eye.x;
        let dy = p_eye.y - t_eye.y;
        let dz = p_eye.z - t_eye.z;
        if dx * dx + dy * dy + dz * dz > 64.0 {
            return;
        }

        let target_type_id = target.get_entity().entity_type.id;
        let mut found_egg: Option<u16> = None;
        for &egg_id in &spawn_egg_ids() {
            if let Some(et) = entity_from_egg(egg_id)
                && et.id == target_type_id
            {
                found_egg = Some(egg_id);
                break;
            }
        }

        if let Some(item) = found_egg.and_then(Item::from_id) {
            let stack = ItemStack::new(1, item);

            let slot_with_stack = player.inventory().get_slot_with_stack(&stack).await;

            if slot_with_stack != -1 {
                if PlayerInventory::is_valid_hotbar_index(slot_with_stack as usize) {
                    player.inventory.set_selected_slot(slot_with_stack as u8);
                } else {
                    player
                        .inventory
                        .swap_slot_with_hotbar(slot_with_stack as usize)
                        .await;
                }
            } else if player.gamemode.load() == GameMode::Creative {
                player.inventory.swap_stack_with_hotbar(stack).await;
            }

            player
                .client
                .enqueue_packet(&CSetSelectedSlot::new(
                    player.inventory.get_selected_slot() as i8
                ))
                .await;
            player
                .player_screen_handler
                .lock()
                .await
                .send_content_updates()
                .await;
        }
    }

    #[allow(clippy::unused_async)]
    pub async fn handle_recipe_book_change_settings(
        &self,
        _player: &Arc<Player>,
        _packet: SRecipeBookChangeSettings,
    ) {
        // Client is updating its recipe book filter/open state; no server action needed.
    }

    #[allow(clippy::unused_async)]
    pub async fn handle_recipe_book_seen_recipe(
        &self,
        _player: &Arc<Player>,
        _packet: SRecipeBookSeenRecipe,
    ) {
        // Client acknowledged a recipe display; no server action needed.
    }

    #[allow(clippy::too_many_lines)]
    pub async fn handle_place_recipe(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        packet: SPlaceRecipe,
    ) {
        use super::super::recipe_helper::{
            GenericIngredient, compute_biggest_craftable, take_n_ingredient,
        };
        use crate::server::recipe::DynamicRecipe;
        use pumpkin_data::recipes::{CraftingRecipeTypes, RECIPES_COOKING, RECIPES_CRAFTING};
        use pumpkin_data::screen::WindowType;
        use pumpkin_inventory::crafting::recipe_provider::RecipeProvider;

        let target_id = packet.recipe_display_id.0 as usize;
        let use_max = packet.use_max_items;

        // Count crafting display IDs.
        let crafting_display_count = RECIPES_CRAFTING
            .iter()
            .filter(|r| {
                !matches!(
                    r,
                    CraftingRecipeTypes::CraftingSpecial
                        | CraftingRecipeTypes::CraftingDecoratedPot { .. }
                )
            })
            .count();
        let cooking_display_count = RECIPES_COOKING.len();
        let dynamic_recipes = server.recipe_manager.get_dynamic_recipes().await;

        let (grid_width, crafting_inv) = {
            let screen_handler_arc = player.current_screen_handler.lock().await.clone();
            let handler = screen_handler_arc.lock().await;
            let grid_width: usize = match handler.window_type() {
                Some(WindowType::Crafting) => 3,
                None => 2, // player inventory 2x2
                _ => return,
            };
            (grid_width, handler.get_behaviour().slots[1].get_inventory())
        };

        let grid_size = grid_width * grid_width;
        let mut ingredient_slots: Vec<Option<GenericIngredient<'_>>> = vec![None; grid_size];

        if target_id < crafting_display_count {
            // Crafting recipe
            let mut counter = 0usize;
            let recipe = RECIPES_CRAFTING.iter().find(|r| {
                if matches!(
                    r,
                    CraftingRecipeTypes::CraftingSpecial
                        | CraftingRecipeTypes::CraftingDecoratedPot { .. }
                ) {
                    return false;
                }
                let found = counter == target_id;
                counter += 1;
                found
            });
            let Some(recipe) = recipe else { return };

            match recipe {
                CraftingRecipeTypes::CraftingShaped { pattern, key, .. } => {
                    for (row, row_str) in pattern.iter().enumerate() {
                        for (col, ch) in row_str.chars().enumerate() {
                            if ch != ' '
                                && let Some(ing) =
                                    key.iter().find_map(|(k, v)| (*k == ch).then_some(v))
                                && row * grid_width + col < grid_size
                            {
                                ingredient_slots[row * grid_width + col] =
                                    Some(GenericIngredient::Vanilla(ing));
                            }
                        }
                    }
                }
                CraftingRecipeTypes::CraftingShapeless { ingredients, .. } => {
                    for (i, ing) in ingredients.iter().enumerate().take(grid_size) {
                        ingredient_slots[i] = Some(GenericIngredient::Vanilla(ing));
                    }
                }
                CraftingRecipeTypes::CraftingTransmute {
                    input, material, ..
                } => {
                    if grid_size >= 2 {
                        ingredient_slots[0] = Some(GenericIngredient::Vanilla(input));
                        ingredient_slots[1] = Some(GenericIngredient::Vanilla(material));
                    }
                }
                _ => return,
            }
        } else if target_id < crafting_display_count + cooking_display_count {
            // TODO: cooking recipes
            return;
        } else {
            let dynamic_id = target_id - crafting_display_count - cooking_display_count;
            let Some(DynamicRecipe::Crafting(crafting)) = dynamic_recipes.get(dynamic_id) else {
                return;
            };

            match crafting {
                pumpkin_protocol::codec::recipe::OwnedCraftingRecipe::Shaped {
                    pattern,
                    key,
                    ..
                } => {
                    for (row, row_str) in pattern.iter().enumerate() {
                        for (col, ch) in row_str.chars().enumerate() {
                            if ch != ' '
                                && let Some((_, ing)) = key.iter().find(|(k, _)| *k == ch)
                                && row * grid_width + col < grid_size
                            {
                                ingredient_slots[row * grid_width + col] =
                                    Some(GenericIngredient::Dynamic(ing));
                            }
                        }
                    }
                }

                pumpkin_protocol::codec::recipe::OwnedCraftingRecipe::Shapeless {
                    ingredients,
                    ..
                } => {
                    for (i, ing) in ingredients.iter().enumerate().take(grid_size) {
                        ingredient_slots[i] = Some(GenericIngredient::Dynamic(ing));
                    }
                }
            }
        }

        // Check if this exact recipe is already placed (determines stacking vs fresh fill).
        let recipe_matches = {
            let mut ok = true;
            for (idx, ing) in ingredient_slots.iter().enumerate() {
                let slot_arc = crafting_inv.get_stack(idx).await;
                let stack = slot_arc.lock().await;
                match ing {
                    None => {
                        if !stack.is_empty() {
                            ok = false;
                            break;
                        }
                    }
                    Some(ingredient) => {
                        if stack.is_empty() || !ingredient.match_item(stack.item) {
                            ok = false;
                            break;
                        }
                    }
                }
            }
            ok
        };

        // Read minimum count from occupied slots before clearing (needed for stacking).
        let current_min = if recipe_matches && !use_max {
            let mut min = u8::MAX;
            for (idx, ing) in ingredient_slots.iter().enumerate() {
                if ing.is_some() {
                    let slot_arc = crafting_inv.get_stack(idx).await;
                    let stack = slot_arc.lock().await;
                    if !stack.is_empty() {
                        min = min.min(stack.item_count);
                    }
                }
            }
            if min == u8::MAX { 0 } else { min }
        } else {
            0
        };

        // Always clear the grid first, returning items to inventory.
        for i in 0..grid_size {
            let stack = crafting_inv.remove_stack(i).await;
            if !stack.is_empty() {
                player.inventory.offer(stack, false, player.as_ref()).await;
            }
        }

        // Determine how many of each ingredient to place per slot.
        let active_ingredients: Vec<GenericIngredient<'_>> =
            ingredient_slots.iter().flatten().copied().collect();
        let amount_to_craft = if use_max {
            compute_biggest_craftable(&active_ingredients, &player.inventory).await
        } else if recipe_matches {
            current_min.saturating_add(1)
        } else {
            1
        };

        if amount_to_craft == 0 {
            let screen_handler_arc = player.current_screen_handler.lock().await.clone();
            screen_handler_arc.lock().await.send_content_updates().await;
            return;
        }

        // Fill each grid slot with exactly `amount_to_craft` matching items.
        for (idx, ing) in ingredient_slots.iter().enumerate() {
            let Some(ingredient) = ing else { continue };
            let taken = take_n_ingredient(&player.inventory, ingredient, amount_to_craft).await;
            if !taken.is_empty() {
                *crafting_inv.get_stack(idx).await.lock().await = taken;
            }
        }

        let screen_handler_arc = player.current_screen_handler.lock().await.clone();
        screen_handler_arc.lock().await.send_content_updates().await;
    }

    pub async fn handle_set_held_item(&self, player: &Player, held: SSetHeldItem) {
        player.update_last_action_time();
        let slot = held.slot;
        if !(0..=8).contains(&slot) {
            self.kick(TextComponent::text("Invalid held slot")).await;
            return;
        }
        let slot = slot as u8;
        let previous_slot = player.inventory.get_selected_slot();
        if let Some(server) = player.world().server.upgrade() {
            let Some(player_arc) = player.world().get_player_by_uuid(player.gameprofile.id) else {
                return;
            };
            let event = PlayerItemHeldEvent::new(player_arc, previous_slot, slot);
            let event = server.plugin_manager.fire(event).await;
            if event.cancelled {
                player
                    .client
                    .enqueue_packet(&CSetSelectedSlot::new(previous_slot as i8))
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

    pub async fn handle_set_creative_slot(
        &self,
        player: &Player,
        packet: SSetCreativeSlot,
    ) -> Result<(), InventoryError> {
        if player.gamemode.load() != GameMode::Creative {
            return Err(InventoryError::PermissionError);
        }
        let is_negative = packet.slot < 0;
        let valid_slot = packet.slot >= 1 && packet.slot as usize <= 45;
        let item_stack = packet
            .clicked_item
            .to_stack_for_version(&self.version.load());
        let is_legal =
            item_stack.is_empty() || item_stack.item_count <= item_stack.get_max_stack_size();

        if valid_slot && is_legal {
            let mut player_screen_handler = player.player_screen_handler.lock().await;

            let is_armor_equipped = player_screen_handler
                .get_slot(packet.slot as usize)
                .get_stack()
                .await
                .lock()
                .await
                .are_equal(&item_stack);
            if !is_armor_equipped {
                if (5..9).contains(&packet.slot) {
                    player
                        .enqueue_equipment_change(
                            &match packet.slot {
                                5 => EquipmentSlot::HEAD,
                                6 => EquipmentSlot::CHEST,
                                7 => EquipmentSlot::LEGS,
                                8 => EquipmentSlot::FEET,
                                _ => {
                                    tracing::error!("Invalid armor slot: {}", packet.slot);
                                    EquipmentSlot::HEAD
                                }
                            },
                            &item_stack,
                        )
                        .await;
                } else if (36..45).contains(&packet.slot) {
                    let slot = packet.slot - 36;
                    if player.inventory().get_selected_slot() == slot as u8 {
                        let equipment = &[(EquipmentSlot::MAIN_HAND, item_stack.clone())];
                        player.living_entity.send_equipment_changes(equipment);
                    }
                }
            }

            player_screen_handler
                .get_slot(packet.slot as usize)
                .set_stack(item_stack.clone())
                .await;
            player_screen_handler.set_received_stack(packet.slot as usize, item_stack);
            player_screen_handler.send_content_updates().await;
            drop(player_screen_handler);
        } else if is_negative && is_legal {
            // Item drop
            player.drop_item(item_stack).await;
        }
        Ok(())
    }

    pub async fn handle_close_container(
        &self,
        player: &Arc<Player>,
        _server: &Server,
        _packet: SCloseContainer,
    ) {
        player.on_handled_screen_closed().await;
    }

    pub async fn handle_select_trade(&self, player: &Arc<Player>, packet: SSelectTrade) {
        let screen_handler = player.current_screen_handler.lock().await;
        let mut screen_handler = screen_handler.lock().await;
        if let Some(merchant) = screen_handler
            .as_any_mut()
            .downcast_mut::<MerchantScreenHandler>()
        {
            merchant
                .set_selected_offer(packet.selected_slot.0 as usize)
                .await;
        }
    }

    pub async fn handle_bundle_item_selected(
        &self,
        player: &Arc<Player>,
        packet: SBundleItemSelected,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        let selected_item_index = packet.selected_item_index.0;
        if selected_item_index < 0 && selected_item_index != -1 {
            self.kick(TextComponent::text("Invalid selected item index"))
                .await;
            return;
        }

        debug!(
            "Bundle item selected: Slot ID {}, Selected Item Index {}",
            packet.slot_id.0, selected_item_index
        );
    }
}
