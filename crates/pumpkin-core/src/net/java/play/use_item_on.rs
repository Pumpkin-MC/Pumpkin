#[allow(clippy::wildcard_imports)]
use super::*;
use crate::item::registry::should_try_block_placement;
use pumpkin_data::block_properties::BlockProperties;

impl JavaClient {
    #[allow(clippy::too_many_lines)]
    pub fn handle_use_item_on(
        &self,
        player: &Arc<Player>,
        use_item_on: &SUseItemOn,
        server: &Arc<Server>,
    ) -> Result<(), BlockPlacingError> {
        if !player.has_client_loaded() {
            return Ok(());
        }
        player.update_last_action_time();
        self.update_sequence(use_item_on.sequence.0);

        let position = use_item_on.position;
        let cursor_pos = use_item_on.cursor_pos;

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
            let entity = &player.get_entity();
            let world = entity.world.load_full();
            let block = world.get_block(&position);

            let event = PlayerInteractEvent::new(
                player,
                InteractAction::RightClickBlock,
                block,
                Some(position),
            );

            send_cancellable_blocking! {{
                server;
                event;
                'cancelled: {
                    let state_id = world.get_block_state_id(&position);
                    player.try_send_client_packet(&CBlockUpdate::new(
                        position,
                        VarInt(i32::from(state_id.as_u16())),
                    ));
                    return Ok(());
                }
            }}

            if let Some(factory) = server
                .block_registry
                .get_screen_handler_factory(block, player, &position, server, &world)
            {
                player.open_handled_screen(factory.as_ref(), Some(position));
            }
            return Ok(());
        }

        let inventory = player.inventory();
        let held_item = inventory.held_item();
        let off_hand_item = inventory.off_hand_item();
        let held_item_empty = held_item.is_empty();
        let off_hand_item_empty = off_hand_item.is_empty();

        let slot_index = if matches!(hand, Hand::Right) {
            inventory.get_selected_slot() as usize
        } else {
            PlayerInventory::OFF_HAND_SLOT
        };

        let mut item = inventory.get_slot(slot_index);
        let before = item.clone();
        let mode = player.gamemode.load();
        let entity = &player.get_entity();
        let world = entity.world.load_full();
        let block = world.get_block(&position);

        let source_is_current = || {
            let current = inventory.get_slot(slot_index);
            Arc::ptr_eq(&world, &player.world())
                && player.gamemode.load() == mode
                && (matches!(hand, Hand::Left)
                    || inventory.get_selected_slot() as usize == slot_index)
                && current.uid == before.uid
                && current.are_equal(&before)
        };
        player.increment_stat(StatisticCategory::Used, i32::from(before.item.id), 1);
        if !source_is_current() || world.get_block(&position) != block {
            return Ok(());
        }

        let event = PlayerInteractEvent::new(
            player,
            InteractAction::RightClickBlock,
            block,
            Some(position),
        );

        send_cancellable_blocking! {{
            server;
            event;
            'cancelled: {
                let state_id = world.get_block_state_id(&position);
                player.try_send_client_packet(&CBlockUpdate::new(
                    position,
                    VarInt(i32::from(state_id.as_u16())),
                ));
                return Ok(());
            }
        }}

        if !source_is_current() || world.get_block(&position) != block {
            return Ok(());
        }

        let equipment_slot = if matches!(hand, Hand::Right) {
            EquipmentSlot::MAIN_HAND
        } else {
            EquipmentSlot::OFF_HAND
        };

        let sneaking = player.get_entity().is_sneaking();

        // Code based on the java class ServerPlayerInteractionManager
        if !(sneaking && (!held_item_empty || !off_hand_item_empty)) {
            let result = Self::call_use_item_on(
                player,
                &position,
                &cursor_pos,
                face,
                &mut item,
                &equipment_slot,
                slot_index,
                &world,
                block,
                server,
            );
            if result.consumes_action() {
                trigger_item_used_on_block_advancement(player, &world, &position, block, &before);
                sync_used_hand_item(player, hand, slot_index, &before, &item);

                if matches!(result, BlockActionResult::SuccessServer) {
                    player.swing_hand(hand, true);
                }
                return Ok(());
            }
        }

        if item.is_empty() || !source_is_current() || world.get_block(&position) != block {
            // TODO item cool down
            // If the hand is empty we stop here
            return Ok(());
        }

        let item_result = server
            .item_registry
            .use_on_block(&mut item, player, position, face, cursor_pos, block, server);

        if should_try_block_placement(&item_result)
            && source_is_current()
            && let Some(placed_block) = Block::from_item_id(item.item.id)
        {
            let mut remainder = item.clone();
            if mode != GameMode::Creative {
                remainder.decrement(1);
            }
            let changed = !remainder.are_equal(&before);
            if Self::run_is_block_place(
                player,
                placed_block,
                server,
                use_item_on,
                position,
                face,
                || {
                    source_is_current()
                        && (!changed
                            || inventory.replace_slot_if_same(
                                slot_index,
                                &before,
                                remainder.clone(),
                            ))
                },
            )? {
                if changed {
                    trigger_item_used_on_block_advancement(
                        player, &world, &position, block, &before,
                    );
                    publish_used_hand_item(player, hand, slot_index, &before, &remainder);
                }
                return Ok(());
            }
        }

        if item_result.consumes_action() || !item.are_equal(&before) {
            trigger_item_used_on_block_advancement(player, &world, &position, block, &before);
        }

        if matches!(item_result, BlockActionResult::SuccessServer) {
            player.swing_hand(hand, true);
        }

        sync_used_hand_item(player, hand, slot_index, &before, &item);

        Ok(())
    }

    #[expect(clippy::too_many_arguments)]
    fn call_use_item_on(
        player: &Arc<Player>,
        position: &BlockPos,
        cursor_pos: &Vector3<f32>,
        face: BlockDirection,
        held_item: &mut ItemStack,
        equipment_slot: &EquipmentSlot,
        slot_index: usize,
        world: &Arc<World>,
        block: &Block,
        server: &Arc<Server>,
    ) -> BlockActionResult {
        let result = server.block_registry.use_with_item_from_slot(
            crate::block::UseWithItemArgs {
                block,
                player,
                position,
                hit: &BlockHitResult {
                    face: &face,
                    cursor_pos,
                },
                item_stack: held_item,
                equipment_slot,
                server,
                world,
            },
            Some(slot_index),
        );

        if result.consumes_action() {
            return result;
        }

        if matches!(result, BlockActionResult::PassToDefaultBlockAction)
            && equipment_slot == &EquipmentSlot::MAIN_HAND
        {
            let result = server.block_registry.on_use(
                block,
                player,
                position,
                &BlockHitResult {
                    face: &face,
                    cursor_pos,
                },
                server,
                world,
            );

            if result.consumes_action() {
                return result;
            }
        }

        BlockActionResult::Pass
    }

    fn run_is_block_place(
        player: &Arc<Player>,
        block: &'static Block,
        server: &Arc<Server>,
        use_item_on: &SUseItemOn,
        location: BlockPos,
        face: BlockDirection,
        commit_item: impl Fn() -> bool,
    ) -> Result<bool, BlockPlacingError> {
        match server.block_registry.place_block_with_commit(
            player,
            block,
            server,
            use_item_on,
            location,
            face,
            commit_item,
        ) {
            Ok(Some((final_block_pos, new_state))) => {
                player.try_send_client_packet(&CBlockUpdate::new(
                    final_block_pos,
                    VarInt(i32::from(new_state.as_u16())),
                ));
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
}

fn trigger_item_used_on_block_advancement(
    player: &Player,
    world: &World,
    position: &BlockPos,
    block: &Block,
    used_item: &ItemStack,
) {
    let state = pumpkin_data::block_properties::CopperBulbLikeProperties::handles_block_id(
        block.id,
    )
    .then(|| {
        let props = pumpkin_data::block_properties::CopperBulbLikeProperties::from_state_id(
            world.get_block_state(position).id,
        );
        ("lit".to_string(), props.r#lit.to_string())
    });
    let is_smokey = crate::block::blocks::campfire::CampfireBlock::is_smokey_pos(world, position);
    let biome = Some(world.get_biome(position).registry_id.to_string());

    player.trigger_advancement(
        crate::entity::player::advancement::trigger::AdvancementTrigger::ItemUsedOnBlock {
            block_id: format!("minecraft:{}", block.name),
            item_id: used_item.item.registry_key.to_string(),
            state,
            smokey: Some(is_smokey),
            biome,
        },
    );
}

fn sync_used_hand_item(
    player: &Player,
    hand: Hand,
    slot_index: usize,
    before: &ItemStack,
    after: &ItemStack,
) {
    if after.are_equal(before)
        || !player
            .inventory()
            .replace_slot_if_same(slot_index, before, after.clone())
    {
        return;
    }

    publish_used_hand_item(player, hand, slot_index, before, after);
}

fn publish_used_hand_item(
    player: &Player,
    hand: Hand,
    slot_index: usize,
    before: &ItemStack,
    after: &ItemStack,
) {
    // Broadcast the break entity status before the slot sync; the client
    // needs the old item texture in the slot for break particles.
    if before.is_damageable() && after.is_empty() {
        let slot = if matches!(hand, Hand::Right) {
            &EquipmentSlot::MAIN_HAND
        } else {
            &EquipmentSlot::OFF_HAND
        };
        player.increment_stat(StatisticCategory::Broken, i32::from(before.item.id), 1);
        player.world().send_entity_status(
            player.get_entity(),
            equipment_break_status(slot),
            ActorEventID::None,
        );
    }

    player.sync_hand_slot(slot_index, player.inventory().get_slot(slot_index));
}
