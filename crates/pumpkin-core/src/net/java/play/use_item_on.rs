#[allow(clippy::wildcard_imports)]
use super::*;
use crate::entity::player::advancement::predicate::CriterionConditions;
use crate::item::items::firework_rocket::FireworkRocketItem;
use crate::item::registry::should_try_block_placement;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_protocol::ser::NetworkReadExt;
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::version::JavaMinecraftVersion;
use std::cell::Cell;

const BLOCK_HIT_DISTANCE_FROM_CENTER: f64 = 1.000_000_1;

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

        let version = self.version.load();
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

        let mode = player.gamemode.load();
        let was_spectator = mode == GameMode::Spectator;
        let inventory = player.inventory();
        let held_item = inventory.held_item();
        let off_hand_item = inventory.off_hand_item();
        let held_item_empty = held_item.is_empty();
        let off_hand_item_empty = off_hand_item.is_empty();

        let equipment_slot = crate::entity::hand_equipment_slot(hand);
        let original_slot = inventory.equipment_slot_index(&equipment_slot);
        let Some(slot_index) = original_slot else {
            return Ok(());
        };
        let mut item = inventory.get_slot(slot_index);
        let before = item.clone();
        let item_id = item.item.id;
        let is_boat = server.item_registry.uses_boat_placement(item_id);
        let minecart = server.item_registry.minecart_item(item_id);
        let is_vehicle = !was_spectator && (is_boat || minecart.is_some());
        let is_firework = server
            .item_registry
            .get_pumpkin_item(item_id)
            .is_some_and(|item| item.as_any().is::<FireworkRocketItem>());
        if is_vehicle
            && ![cursor_pos.x, cursor_pos.y, cursor_pos.z]
                .into_iter()
                .all(|offset| (f64::from(offset) - 0.5).abs() < BLOCK_HIT_DISTANCE_FROM_CENTER)
        {
            return Ok(());
        }
        let entity = &player.get_entity();
        let world = entity.world.load_full();
        let block = world.get_block(&position);
        if is_vehicle && !world.is_in_height_limit(position.0.y) {
            Self::send_vehicle_build_limit_feedback(
                player,
                &world,
                position.0.y > world.get_top_y(),
            );
            return Ok(());
        }
        if is_vehicle && !crate::net::vehicle_block_use_is_eligible(player, &world, &position) {
            if crate::net::vehicle_spawn_is_protected(player, &world, &position) {
                player.send_system_message_raw(
                    &TextComponent::translate(
                        translation::java::BUILD_SPAWN_PROTECTION,
                        [TextComponent::text(position.to_string())],
                    )
                    .color_named(NamedColor::Red),
                    true,
                );
            }
            self.correct_block_use(player, &world, &position, face);
            return Ok(());
        }

        let source_is_current = || {
            let current = inventory.get_slot(slot_index);
            Arc::ptr_eq(&world, &player.world())
                && player.gamemode.load() == mode
                && inventory.equipment_slot_index(&equipment_slot) == original_slot
                && current.uid == before.uid
                && current.are_equal(&before)
        };
        if !is_vehicle && !is_firework && !was_spectator {
            player.increment_stat(StatisticCategory::Used, i32::from(item_id), 1);
            if !source_is_current() || world.get_block(&position) != block {
                return Ok(());
            }
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
                if is_vehicle {
                    self.correct_block_use(player, &world, &position, face);
                } else {
                    let state_id = world.get_block_state_id(&position);
                    player.try_send_client_packet(&CBlockUpdate::new(
                        position,
                        VarInt(i32::from(state_id.as_u16())),
                    ));
                }
                return Ok(());
            }
        }}

        if was_spectator {
            if Arc::ptr_eq(&world, &player.world())
                && player.gamemode.load() == mode
                && world.get_block(&position) == block
                && let Some(factory) = server
                    .block_registry
                    .get_screen_handler_factory(block, player, &position, server, &world)
            {
                player.open_handled_screen(factory.as_ref(), Some(position));
            }
            return Ok(());
        }

        if !source_is_current() || world.get_block(&position) != block {
            if is_vehicle {
                self.correct_block_use(player, &world, &position, face);
            }
            return Ok(());
        }

        if is_vehicle && !crate::net::vehicle_block_use_is_eligible(player, &world, &position) {
            self.correct_block_use(player, &world, &position, face);
            return Ok(());
        }

        let sneaking = player.get_entity().is_sneaking();

        // Code based on the java class ServerPlayerInteractionManager
        if !(sneaking && (!held_item_empty || !off_hand_item_empty)) {
            let (result, used_item) = Self::call_use_item_on(
                player,
                &position,
                &cursor_pos,
                face,
                &mut item,
                &equipment_slot,
                original_slot,
                &world,
                block,
                server,
            );
            if result.consumes_action() {
                if used_item || !is_vehicle {
                    trigger_item_used_on_block_advancement(
                        player, &world, &position, block, &before, is_vehicle,
                    );
                }
                sync_used_hand_item(player, hand, slot_index, &before, &item);

                if is_vehicle {
                    Self::send_vehicle_block_use_feedback(
                        version, player, &world, hand, &before, &result,
                    );
                    self.correct_block_use(player, &world, &position, face);
                } else if matches!(result, BlockActionResult::SuccessServer) {
                    player.swing_hand(hand, true);
                }
                return Ok(());
            }
        }

        if item.is_empty() || !source_is_current() || world.get_block(&position) != block {
            // TODO item cool down
            // If the hand is empty we stop here
            if is_vehicle {
                self.correct_block_use(player, &world, &position, face);
            }
            return Ok(());
        }

        if minecart.is_some() {
            let result = crate::item::items::minecart::MinecartItem::place_from_slot(
                &before, player, hand, slot_index, &world, position,
            );
            if result.consumes_action() {
                trigger_item_used_on_block_advancement(
                    player, &world, &position, block, &before, true,
                );
                Self::send_vehicle_block_use_feedback(
                    version, player, &world, hand, &before, &result,
                );
            }
            self.correct_block_use(player, &world, &position, face);
            return Ok(());
        }

        if is_firework {
            let hit = BlockHitResult {
                face: &face,
                cursor_pos: &cursor_pos,
            };
            let result = FireworkRocketItem::use_from_slot(
                &before,
                slot_index,
                player,
                &world,
                position,
                &hit,
                || source_is_current() && world.get_block(&position) == block,
            );
            if result.consumes_action() {
                trigger_item_used_on_block_advancement(
                    player, &world, &position, block, &before, false,
                );
            }
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
                        player, &world, &position, block, &before, is_vehicle,
                    );
                    publish_used_hand_item(player, hand, slot_index, &before, &remainder);
                }
                return Ok(());
            }
        }

        if item_result.consumes_action() || !item.are_equal(&before) {
            trigger_item_used_on_block_advancement(
                player, &world, &position, block, &before, is_vehicle,
            );
        }

        if matches!(item_result, BlockActionResult::SuccessServer) {
            player.swing_hand(hand, true);
        }

        sync_used_hand_item(player, hand, slot_index, &before, &item);
        if is_vehicle {
            self.correct_block_use(player, &world, &position, face);
        }

        Ok(())
    }

    fn send_vehicle_build_limit_feedback(player: &Player, world: &World, too_high: bool) {
        let (key, limit) = if too_high {
            (translation::java::BUILD_TOOHIGH, world.get_top_y())
        } else {
            (translation::java::BUILD_TOOLOW, world.get_bottom_y())
        };
        player.send_system_message_raw(
            &TextComponent::translate(key, [TextComponent::text(limit.to_string())])
                .color_named(NamedColor::Red),
            true,
        );
    }

    fn send_vehicle_block_use_feedback(
        version: JavaMinecraftVersion,
        player: &Player,
        world: &Arc<World>,
        hand: Hand,
        item: &ItemStack,
        result: &BlockActionResult,
    ) {
        let include_placer = matches!(result, BlockActionResult::SuccessServer);
        if version < JavaMinecraftVersion::V_26_3 {
            if include_placer && Arc::ptr_eq(world, &player.world()) {
                player.swing_hand(hand, true);
            }
            return;
        }
        if !matches!(
            result,
            BlockActionResult::Success | BlockActionResult::SuccessServer
        ) {
            return;
        }

        // Java 26.3 processes the accepted result in both block-feedback branches.
        for _ in 0..2 {
            if !Arc::ptr_eq(world, &player.world()) {
                break;
            }
            player.swing_after_vehicle_interaction(hand, item, include_placer, world);
        }
    }

    fn correct_block_use(
        &self,
        player: &Player,
        world: &Arc<World>,
        position: &BlockPos,
        face: BlockDirection,
    ) {
        for target in [*position, position.offset(face.to_offset())] {
            self.send_block_correction(player, world, target);
        }
    }

    fn send_block_correction(&self, player: &Player, world: &Arc<World>, target: BlockPos) {
        // Re-observe a fresh correction once so legacy translators can map the new state ID.
        // A second observer mutation supersedes even that attempt.
        for _ in 0..2 {
            if !Arc::ptr_eq(world, &player.world()) {
                return;
            }
            let state_id = world.get_block_state_id(&target);
            let Ok(data) = self.serialize_packet(&CBlockUpdate::new(
                target,
                VarInt(i32::from(state_id.as_u16())),
            )) else {
                return;
            };
            let version = self.version.load();
            let changed = Cell::new(false);
            self.try_enqueue_packet_data_projecting(
                data,
                || {
                    if !Arc::ptr_eq(world, &player.world()) {
                        return false;
                    }
                    let same_state = world.get_block_state_id(&target) == state_id;
                    changed.set(!same_state);
                    same_state
                },
                |data, primary, current| {
                    (*current || (!primary && !matches_block_update(&data, version, target)))
                        .then_some(data)
                },
                |_, _, _| {},
            );
            if !changed.get() {
                break;
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn call_use_item_on(
        player: &Arc<Player>,
        position: &BlockPos,
        cursor_pos: &Vector3<f32>,
        face: BlockDirection,
        held_item: &mut ItemStack,
        equipment_slot: &EquipmentSlot,
        vehicle_slot: Option<usize>,
        world: &Arc<World>,
        block: &Block,
        server: &Arc<Server>,
    ) -> (BlockActionResult, bool) {
        let result = server.block_registry.use_with_item_from_slot(
            crate::block::UseWithItemArgs {
                server,
                world,
                block,
                position,
                player,
                hit: &BlockHitResult {
                    face: &face,
                    cursor_pos,
                },
                item_stack: held_item,
                equipment_slot,
            },
            vehicle_slot,
        );

        if result.consumes_action() {
            return (result, true);
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
                return (result, false);
            }
        }

        (BlockActionResult::Pass, false)
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

fn matches_block_update(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
    target: BlockPos,
) -> bool {
    // Historical Java play-state block_update registrations, at the observer's wire version.
    let packet_id = match version.protocol_version() {
        4 | 5 | 47 => 35,
        107..=498 | 735..=754 => 11,
        573..=578 | 755..=758 => 12,
        759..=761 | 764..=769 => 9,
        762 | 763 => 10,
        770..=777 => 8,
        _ => return false,
    };
    if !payload.get_var_int().is_ok_and(|id| id.0 == packet_id) {
        return false;
    }
    if version < JavaMinecraftVersion::V_1_8 {
        return payload.get_i32_be().is_ok_and(|x| x == target.0.x)
            && payload.get_u8().is_ok_and(|y| i32::from(y) == target.0.y)
            && payload.get_i32_be().is_ok_and(|z| z == target.0.z)
            && payload.get_var_int().is_ok()
            && payload.len() == 1;
    }
    payload
        .get_block_pos(&version)
        .is_ok_and(|pos| pos == target)
        && payload.get_var_int().is_ok()
        && payload.is_empty()
}

fn trigger_item_used_on_block_advancement(
    player: &Player,
    world: &World,
    position: &BlockPos,
    block: &Block,
    used_item: &ItemStack,
    is_vehicle: bool,
) {
    let is_smokey = crate::block::blocks::campfire::CampfireBlock::is_smokey_pos(world, position);
    let biome = world.get_biome(position).registry_id;
    if is_vehicle {
        let (clicked_block, state_id) = world.get_block_and_state_id(position);
        let block_id = format!("minecraft:{}", clicked_block.name);
        player.trigger_criterion_event(
            "minecraft:item_used_on_block",
            |condition| match condition {
                CriterionConditions::ItemUsedOnBlock(predicate) => {
                    let state = predicate.state_key.as_deref().and_then(|key| {
                        clicked_block
                            .property_values(state_id)
                            .find(|property| property.key().to_str() == key)
                            .map(|property| (key, property.value_str()))
                    });
                    predicate.matches(
                        &block_id,
                        used_item.item.registry_key,
                        state,
                        Some(is_smokey),
                        Some(biome),
                    )
                }
                CriterionConditions::Custom { .. } => true,
                _ => false,
            },
        );
        return;
    }

    let state = pumpkin_data::block_properties::CopperBulbLikeProperties::handles_block_id(
        block.id,
    )
    .then(|| {
        let props = pumpkin_data::block_properties::CopperBulbLikeProperties::from_state_id(
            world.get_block_state(position).id,
        );
        ("lit".to_string(), props.r#lit.to_string())
    });
    player.trigger_advancement(
        crate::entity::player::advancement::trigger::AdvancementTrigger::ItemUsedOnBlock {
            block_id: format!("minecraft:{}", block.name),
            item_id: used_item.item.registry_key.to_string(),
            state,
            smokey: Some(is_smokey),
            biome: Some(biome.to_string()),
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
