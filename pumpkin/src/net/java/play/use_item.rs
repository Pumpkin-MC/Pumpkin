use super::BlockPlacingError;
use crate::block;
use crate::block::BlockHitResult;
use crate::block::entities::sign::SignBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::entity::equipment_break_status;
use crate::entity::player::Player;
use crate::entity::player::statistics::StatisticCategory;
use crate::net::java::JavaClient;
use crate::plugin::player::fish::PlayerFishEvent;
use crate::plugin::player::fish::PlayerFishState;
use crate::plugin::player::player_interact_event::InteractAction;
use crate::plugin::player::player_interact_event::PlayerInteractEvent;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::data_component_impl::BlocksAttacksImpl;
use pumpkin_data::data_component_impl::ConsumableImpl;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::data_component_impl::EquippableImpl;
use pumpkin_data::data_component_impl::FoodImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::java::client::play::COpenSignEditor;
use pumpkin_protocol::java::server::play::SUpdateSign;
use pumpkin_protocol::java::server::play::SUseItem;
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::GameMode;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;

impl JavaClient {
    async fn sync_use_item_on_block_states(
        &self,
        world: &World,
        position: BlockPos,
        face: BlockDirection,
    ) {
        let placed_position = position.offset(face.to_offset());
        let mut positions = [position; 14];
        let mut position_count = 1;

        if placed_position != position {
            positions[position_count] = placed_position;
            position_count += 1;
        }

        // The client predicts only the interacted block. Correct directly adjacent
        // dust now so a lever, torch, or redstone-block change cannot leave its
        // source wire visually unpowered until the next batched world update.
        for origin in [position, placed_position] {
            for direction in BlockDirection::all() {
                let neighbor = origin.offset(direction.to_offset());
                if world.get_block(&neighbor) == &Block::REDSTONE_WIRE
                    && !positions[..position_count].contains(&neighbor)
                {
                    positions[position_count] = neighbor;
                    position_count += 1;
                }
            }
        }

        world
            .enqueue_block_state_corrections(self, &positions[..position_count])
            .await;
    }

    #[allow(clippy::too_many_lines)]
    pub async fn handle_use_item_on(
        &self,
        player: &Arc<Player>,
        use_item_on: SUseItemOn,
        server: &Arc<Server>,
    ) -> Result<(), BlockPlacingError> {
        if !player.has_client_loaded() {
            return Ok(());
        }
        player.update_last_action_time();
        self.update_sequence(player, use_item_on.sequence.0);

        let position = use_item_on.position;
        let cursor_pos = use_item_on.cursor_pos;

        let mut should_try_decrement = false;

        if !player.can_interact_with_block_at(&position, 1.0) {
            // TODO: maybe log?
            return Err(BlockPlacingError::BlockOutOfReach);
        }

        let Ok(face) = BlockDirection::try_from(use_item_on.face.0) else {
            return Err(BlockPlacingError::InvalidBlockFace);
        };

        let Ok(hand) = Hand::try_from(use_item_on.hand.0) else {
            return Err(BlockPlacingError::InvalidHand);
        };

        if player.gamemode.load() == GameMode::Spectator {
            // TODO: openMenu
            return Ok(());
        }

        let inventory = player.inventory();
        let held_item = inventory.held_item();
        let off_hand_item = inventory.off_hand_item().await;
        let held_item_empty = held_item.lock().await.is_empty();
        let off_hand_item_empty = off_hand_item.lock().await.is_empty();

        let item = if matches!(hand, Hand::Left) {
            held_item
        } else {
            off_hand_item
        };

        let item_id = item.lock().await.item.id;
        player
            .increment_stat(StatisticCategory::Used, item_id as i32, 1)
            .await;

        let entity = &player.get_entity();
        let world = entity.world.load_full();
        let block = world.get_block(&position);

        let event = PlayerInteractEvent::new(
            player,
            InteractAction::RightClickBlock,
            block,
            Some(position),
        );

        send_cancellable! {{
            server;
            event;
            'cancelled: {
                self.sync_use_item_on_block_states(&world, position, face)
                    .await;
                return Ok(());
            }
        }}

        let sneaking = player.get_entity().is_sneaking();

        // Code based on the java class ServerPlayerInteractionManager
        if !(sneaking && (!held_item_empty || !off_hand_item_empty)) {
            let result = self
                .call_use_item_on(
                    player,
                    &position,
                    &cursor_pos,
                    &face,
                    &item,
                    &world,
                    block,
                    server,
                )
                .await;
            if result.consumes_action() {
                // TODO: Trigger ANY_BLOCK_USE Criteria

                if matches!(result, BlockActionResult::SuccessServer) {
                    player.swing_hand(hand, true).await;
                }
                self.sync_use_item_on_block_states(&world, position, face)
                    .await;
                return Ok(());
            }
        }

        let slot_index = if matches!(hand, Hand::Left) {
            inventory.get_selected_slot() as usize
        } else {
            PlayerInventory::OFF_HAND_SLOT
        };

        let mut stack = item.lock().await;

        if stack.is_empty() {
            // TODO item cool down
            // If the hand is empty we stop here
            drop(stack);
            self.sync_use_item_on_block_states(&world, position, face)
                .await;
            return Ok(());
        }

        let before = stack.clone();

        server
            .item_registry
            .use_on_block(
                &mut stack, player, position, face, cursor_pos, block, server,
            )
            .await;

        // Check if the item is a block, because not every item can be placed :D
        let item_id = stack.item.id;
        if let Some(block) = Block::from_item_id(item_id) {
            should_try_decrement = self
                .run_is_block_place(player, block, server, use_item_on, position, face)
                .await?;
        }

        if should_try_decrement {
            // TODO: Config
            // Decrease block count
            if player.gamemode.load() != GameMode::Creative {
                stack.decrement(1);
            }
        }

        let after = stack.clone();
        drop(stack);

        // Broadcast the break entity status before the slot sync; the client
        // needs the old item texture in the slot for break particles.
        if !before.is_empty() && after.is_empty() {
            let slot = if slot_index == player.inventory.get_selected_slot() as usize {
                &EquipmentSlot::MAIN_HAND
            } else {
                &EquipmentSlot::OFF_HAND
            };
            player
                .world()
                .send_entity_status(player.get_entity(), equipment_break_status(slot));
        }

        if !after.are_equal(&before) {
            player.sync_hand_slot(slot_index, after).await;
        }

        self.sync_use_item_on_block_states(&world, position, face)
            .await;

        Ok(())
    }

    #[expect(clippy::too_many_arguments)]
    async fn call_use_item_on(
        &self,
        player: &Arc<Player>,
        position: &BlockPos,
        cursor_pos: &Vector3<f32>,
        face: &BlockDirection,
        held_item: &Arc<Mutex<ItemStack>>,
        world: &Arc<World>,
        block: &Block,
        server: &Arc<Server>,
    ) -> BlockActionResult {
        let result = server
            .block_registry
            .use_with_item(
                block,
                player,
                position,
                &BlockHitResult { face, cursor_pos },
                held_item,
                server,
                world,
            )
            .await;

        if result.consumes_action() {
            // TODO: Trigger ITEM_USED_ON_BLOCK Criteria
            return result;
        }

        if matches!(result, BlockActionResult::PassToDefaultBlockAction) {
            let result = server
                .block_registry
                .on_use(
                    block,
                    player,
                    position,
                    &BlockHitResult { face, cursor_pos },
                    server,
                    world,
                )
                .await;

            if result.consumes_action() {
                // TODO: Trigger DEFAULT_BLOCK_USE Criteria
                return result;
            }
        }

        BlockActionResult::Pass
    }

    pub async fn handle_sign_update(&self, player: &Player, sign_data: SUpdateSign<'_>) {
        let world = player.get_entity().world.load_full();
        let Some(block_entity) = world.get_block_entity(&sign_data.location) else {
            return;
        };
        let Some(sign_entity) = block_entity.as_any().downcast_ref::<SignBlockEntity>() else {
            return;
        };
        if sign_entity.is_waxed.load(Ordering::Relaxed) {
            return;
        }

        let text = if sign_data.is_front_text {
            &sign_entity.front_text
        } else {
            &sign_entity.back_text
        };

        *text.messages.lock().unwrap() = [
            sign_data.line_1.into(),
            sign_data.line_2.into(),
            sign_data.line_3.into(),
            sign_data.line_4.into(),
        ];
        *sign_entity.currently_editing_player.lock().await = None;
        world.update_block_entity(&block_entity);
    }

    pub async fn handle_use_item(
        &self,
        player: &Arc<Player>,
        use_item: &SUseItem,
        server: &Server,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        let inventory = player.inventory();
        let Ok(hand) = Hand::try_from(use_item.hand.0) else {
            self.kick(TextComponent::text("InvalidHand")).await;
            return;
        };
        self.update_sequence(player, use_item.sequence.0);

        let item_in_hand = if hand == Hand::Left {
            inventory.held_item()
        } else {
            inventory.off_hand_item().await
        };

        let (item_id, _item) = {
            let guard = item_in_hand.lock().await;
            (guard.item.id, guard.item)
        };
        player
            .increment_stat(StatisticCategory::Used, item_id as i32, 1)
            .await;

        let hit_result = player
            .world()
            .raycast(
                player.eye_position(),
                player.eye_position().add(
                    &(Vector3::rotation_vector(f64::from(use_item.pitch), f64::from(use_item.yaw))
                        * 4.5),
                ),
                async |pos, world| {
                    let block = world.get_block(pos);
                    block != &Block::AIR && block != &Block::WATER && block != &Block::LAVA
                },
            )
            .await;

        let event = if let Some((hit_pos, _hit_dir)) = hit_result {
            PlayerInteractEvent::new(
                player,
                InteractAction::RightClickBlock,
                player.world().get_block(&hit_pos),
                Some(hit_pos),
            )
        } else {
            PlayerInteractEvent::new(player, InteractAction::RightClickAir, &Block::AIR, None)
        };
        self.prepare_hand_item_for_use(player, hand, &item_in_hand)
            .await;

        let (item_for_use, stack_for_use) = {
            let held = item_in_hand.lock().await;
            (held.item, held.clone())
        };

        if !self
            .should_continue_use_after_fish_event(server, player, hand, item_for_use)
            .await
        {
            return;
        }

        send_cancellable! {{
            server;
            event;
            'after: {
                server.item_registry.on_use(&stack_for_use, player).await;
            }
        }}
    }

    async fn prepare_hand_item_for_use(
        &self,
        player: &Arc<Player>,
        hand: Hand,
        item_in_hand: &Arc<Mutex<ItemStack>>,
    ) {
        let inventory = player.inventory();
        let mut held = item_in_hand.lock().await;

        if let Some(cooldown) = held.get_use_cooldown() {
            let group = cooldown
                .cooldown_group
                .clone()
                .unwrap_or_else(|| held.item.registry_key.to_string());
            if player.is_on_cooldown(&group).await {
                return;
            }
        }

        if held.get_data_component::<ConsumableImpl>().is_some()
            || held.get_data_component::<BlocksAttacksImpl>().is_some()
        {
            // If its food we want to make sure we can actually consume it
            if let Some(food) = held.get_data_component::<FoodImpl>() {
                if player.abilities.lock().await.invulnerable
                    || food.can_always_eat
                    || player.hunger_manager.level.load() < 20
                {
                    player
                        .living_entity
                        .set_active_hand(hand, held.clone(), held.get_max_use_time())
                        .await;
                }
            } else {
                player
                    .living_entity
                    .set_active_hand(hand, held.clone(), held.get_max_use_time())
                    .await;
            }
        }
        if let Some(equippable) = held.get_data_component::<EquippableImpl>() {
            // Skip if the item is already in the target equipment slot.
            // This prevents a self-deadlock: `held` already locks the same
            // Mutex<ItemStack> that `get_or_insert` would return, and
            // Tokio's Mutex is not reentrant.
            if inventory
                .is_already_equipped(item_in_hand, equippable.slot)
                .await
            {
                return;
            }

            // If it can be equipped we want to make sure we can actually equip it
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

    async fn should_continue_use_after_fish_event(
        &self,
        server: &Server,
        player: &Arc<Player>,
        hand: Hand,
        item_for_use: &Item,
    ) -> bool {
        if item_for_use.id != Item::FISHING_ROD.id {
            return true;
        }

        // TODO: Apply fishing rod durability on retrieval based on catch type.
        let fish_event = PlayerFishEvent::new(
            player.clone(),
            None,
            uuid::Uuid::nil(),
            String::new(),
            PlayerFishState::Fishing,
            hand,
            0,
        );
        let fish_event = server.plugin_manager.fire(fish_event).await;
        !fish_event.cancelled
    }

    async fn run_is_block_place(
        &self,
        player: &Arc<Player>,
        block: &'static Block,
        server: &Server,
        use_item_on: SUseItemOn,
        location: BlockPos,
        face: BlockDirection,
    ) -> Result<bool, BlockPlacingError> {
        match server
            .block_registry
            .place_block(player, block, server, &use_item_on, location, face)
            .await
        {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(crate::block::registry::BlockPlacingError::InvalidGamemode) => {
                Err(BlockPlacingError::InvalidGamemode)
            }
            Err(crate::block::registry::BlockPlacingError::BlockOutOfWorld) => {
                Err(BlockPlacingError::BlockOutOfWorld)
            }
        }
    }

    /// Checks if the block placed was a sign, then opens a dialog.
    pub async fn send_sign_packet(&self, block_position: BlockPos, is_front_text: bool) {
        self.enqueue_packet(&COpenSignEditor::new(block_position, is_front_text))
            .await;
    }
}
