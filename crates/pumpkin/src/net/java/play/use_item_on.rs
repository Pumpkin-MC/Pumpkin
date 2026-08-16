#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
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

        let Ok(hand) = Hand::from_packet_id(use_item_on.hand.0) else {
            return Err(BlockPlacingError::InvalidHand);
        };

        if player.gamemode.load() == GameMode::Spectator {
            // TODO: openMenu
            return Ok(());
        }

        let inventory = player.inventory();
        let held_item = inventory.held_item().await;
        let off_hand_item = inventory.off_hand_item().await;
        let held_item_empty = held_item.is_empty();
        let off_hand_item_empty = off_hand_item.is_empty();

        let mut item = inventory.get_stack_in_hand(hand).await;
        let item_id = item.item.id;
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
                let state_id = world.get_block_state_id(&position);
                self.enqueue_packet(&CBlockUpdate::new(
                    position,
                    VarInt(i32::from(state_id.as_u16())),
                ))
                .await;
                return Ok(());
            }
        }}

        let equipment_slot = if matches!(hand, Hand::Right) {
            EquipmentSlot::MAIN_HAND
        } else {
            EquipmentSlot::OFF_HAND
        };

        let sneaking = player.get_entity().is_sneaking();

        // Code based on the java class ServerPlayerInteractionManager
        if !(sneaking && (!held_item_empty || !off_hand_item_empty)) {
            let result = self
                .call_use_item_on(
                    player,
                    &position,
                    &cursor_pos,
                    &face,
                    &mut item,
                    &equipment_slot,
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
                return Ok(());
            }
        }

        let slot_index = if matches!(hand, Hand::Right) {
            inventory.get_selected_slot() as usize
        } else {
            PlayerInventory::OFF_HAND_SLOT
        };

        if item.is_empty() {
            // TODO item cool down
            // If the hand is empty we stop here
            return Ok(());
        }

        let before = item.clone();

        server
            .item_registry
            .use_on_block(&mut item, player, position, face, cursor_pos, block, server)
            .await;

        // Check if the item is a block, because not every item can be placed :D
        let item_id = item.item.id;
        if let Some(block) = Block::from_item_id(item_id) {
            should_try_decrement = self
                .run_is_block_place(player, block, server, use_item_on, position, face)
                .await?;
        }

        if should_try_decrement {
            // TODO: Config
            // Decrease block count
            if player.gamemode.load() != GameMode::Creative {
                item.decrement(1);
            }
        }

        let after = item.clone();

        // Broadcast the break entity status before the slot sync; the client
        // needs the old item texture in the slot for break particles.
        if !before.is_empty() && after.is_empty() {
            let slot = if slot_index == player.inventory.get_selected_slot() as usize {
                &EquipmentSlot::MAIN_HAND
            } else {
                &EquipmentSlot::OFF_HAND
            };
            player.world().send_entity_status(
                player.get_entity(),
                equipment_break_status(slot),
                None,
            );
        }

        if !after.are_equal(&before) {
            player.sync_hand_slot(slot_index, after.clone()).await;
            inventory.set_stack_in_hand(hand, after).await;
        }

        Ok(())
    }

    #[expect(clippy::too_many_arguments)]
    async fn call_use_item_on(
        &self,
        player: &Arc<Player>,
        position: &BlockPos,
        cursor_pos: &Vector3<f32>,
        face: &BlockDirection,
        held_item: &mut ItemStack,
        equipment_slot: &EquipmentSlot,
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
                equipment_slot,
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

    async fn run_is_block_place(
        &self,
        player: &Arc<Player>,
        block: &'static Block,
        server: &Arc<Server>,
        use_item_on: SUseItemOn,
        location: BlockPos,
        face: BlockDirection,
    ) -> Result<bool, BlockPlacingError> {
        match server
            .block_registry
            .place_block(player, block, server, &use_item_on, location, face)
            .await
        {
            Ok(Some((final_block_pos, new_state))) => {
                self.send_packet_now(&CBlockUpdate::new(
                    final_block_pos,
                    VarInt(i32::from(new_state.as_u16())),
                ))
                .await;
                Ok(true)
            }
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
