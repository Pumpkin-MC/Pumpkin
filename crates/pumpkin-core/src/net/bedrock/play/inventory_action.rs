use super::item_stack_request::SlotUpdate;
#[allow(clippy::wildcard_imports)]
use super::*;
use crate::block::UseWithItemArgs;
use crate::item::{
    items::{
        cart_air_use::use_cart_in_air, firework_rocket::FireworkRocketItem, minecart::MinecartItem,
    },
    registry::should_try_block_placement,
};
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_protocol::bedrock::server::inventory_transaction::HandSlot;
use pumpkin_protocol::bedrock::server::inventory_transaction::UseItemTransactionData;

struct VehicleBlockClick<'a> {
    player: &'a Arc<Player>,
    server: &'a Arc<Server>,
    world: &'a Arc<World>,
    data: &'a UseItemTransactionData,
    block: &'static Block,
    hand: Hand,
    slot_type: EquipmentSlot,
    slot: usize,
    source: ItemStack,
}

impl VehicleBlockClick<'_> {
    fn source_is_current(&self) -> bool {
        let inventory = self.player.inventory();
        let current = inventory.get_slot(self.slot);
        crate::net::vehicle_block_use_is_eligible(
            self.player,
            self.world,
            &self.data.block_position,
        ) && inventory.equipment_slot_index(&self.slot_type) == Some(self.slot)
            && current.uid == self.source.uid
            && current.are_equal(&self.source)
            && self.world.get_block(&self.data.block_position).id == self.block.id
    }

    fn publish_changed_source(&self, update: ItemStack) {
        if !update.are_equal(&self.source)
            && self
                .player
                .inventory()
                .replace_slot_if_same(self.slot, &self.source, update.clone())
        {
            self.player.sync_hand_slot(self.slot, update);
        }
    }

    fn handle(&self, vehicle_click: bool, minecart_click: bool) -> bool {
        let Self {
            player,
            server,
            world,
            data,
            block,
            hand,
            slot_type,
            slot,
            source,
        } = self;
        let event = PlayerInteractEvent::new(
            player,
            InteractAction::RightClickBlock,
            block,
            Some(data.block_position),
        );
        send_cancellable_blocking! {{
            server;
            event;
            'cancelled: {
                return false;
            }
        }}
        if !self.source_is_current() {
            return false;
        }
        if player.gamemode.load() == GameMode::Spectator {
            let Some(factory) = server.block_registry.get_screen_handler_factory(
                block,
                player,
                &data.block_position,
                server,
                world,
            ) else {
                return false;
            };
            return player
                .open_handled_screen(factory.as_ref(), Some(data.block_position))
                .is_some();
        }

        let mut held = source.clone();
        let face = BedrockClient::interaction_face(data.block_face);
        let hit = BlockHitResult {
            face: &face,
            cursor_pos: &data.click_position,
        };
        let result = server.block_registry.use_with_item_from_slot(
            UseWithItemArgs {
                server,
                world,
                block,
                position: &data.block_position,
                player,
                hit: &hit,
                item_stack: &mut held,
                equipment_slot: slot_type,
            },
            Some(*slot),
        );

        if result.consumes_action() {
            self.publish_changed_source(held);
            return true;
        }
        if matches!(result, BlockActionResult::PassToDefaultBlockAction)
            && server
                .block_registry
                .on_use(block, player, &data.block_position, &hit, server, world)
                .consumes_action()
        {
            return true;
        }
        if !self.source_is_current()
            || player.gamemode.load() == GameMode::Spectator
            || held.is_empty()
        {
            return false;
        }
        if minecart_click {
            return MinecartItem::place_from_slot(
                source,
                player,
                *hand,
                *slot,
                world,
                data.block_position,
            )
            .consumes_action();
        }
        if vehicle_click {
            return false;
        }

        self.use_offhand_item(held, &hit)
    }

    fn use_offhand_item(&self, mut held: ItemStack, hit: &BlockHitResult<'_>) -> bool {
        let Self {
            player,
            server,
            data,
            block,
            world,
            ..
        } = self;
        if server
            .item_registry
            .get_pumpkin_item(held.get_item().id)
            .is_some_and(|item| item.as_any().is::<FireworkRocketItem>())
        {
            return FireworkRocketItem::use_from_slot(
                &self.source,
                self.slot,
                player,
                world,
                data.block_position,
                hit,
                || self.source_is_current(),
            )
            .consumes_action();
        }

        let face = *hit.face;
        let result = server.item_registry.use_on_block(
            &mut held,
            player,
            data.block_position,
            face,
            data.click_position,
            block,
            server,
        );
        if should_try_block_placement(&result)
            && let Some(placed_block) = pumpkin_data::Block::from_item_id(held.get_item().id)
        {
            let request = pumpkin_protocol::java::server::play::SUseItemOn {
                hand: VarInt(1),
                position: data.block_position,
                face: VarInt(i32::from(data.block_face)),
                cursor_pos: data.click_position,
                inside_block: false,
                is_against_world_border: false,
                sequence: VarInt(0),
            };
            let mut remainder = held;
            remainder.decrement_unless_creative(player.gamemode.load(), 1);
            let changed = !remainder.are_equal(&self.source);
            let inventory = player.inventory();
            let consumed = matches!(
                server.block_registry.place_block_with_commit(
                    player,
                    placed_block,
                    server,
                    &request,
                    data.block_position,
                    face,
                    || self.source_is_current()
                        && (!changed
                            || inventory.replace_slot_if_same(
                                self.slot,
                                &self.source,
                                remainder.clone()
                            )),
                ),
                Ok(Some(_))
            );
            if consumed {
                if changed {
                    player.sync_hand_slot(self.slot, inventory.get_slot(self.slot));
                }
                player.increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Used,
                    i32::from(self.source.get_item().id),
                    1,
                );
            }
            return consumed;
        }
        let consumed = result.consumes_action();
        if consumed {
            self.publish_changed_source(held);
            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Used,
                i32::from(self.source.get_item().id),
                1,
            );
        }
        consumed
    }
}

impl BedrockClient {
    fn use_vehicle_in_air(
        server: &Arc<Server>,
        world: &Arc<World>,
        player: &Arc<Player>,
        hand: Hand,
        slot: usize,
        expected: &ItemStack,
    ) -> bool {
        if player.gamemode.load() == GameMode::Spectator {
            return false;
        }
        let event =
            PlayerInteractEvent::new(player, InteractAction::RightClickAir, &Block::AIR, None);
        let event = server.plugin_manager.fire_blocking(server, event);
        if event.cancelled
            || !Arc::ptr_eq(world, &player.world())
            || player.gamemode.load() == GameMode::Spectator
            || player
                .inventory()
                .equipment_slot_index(&crate::entity::hand_equipment_slot(hand))
                != Some(slot)
        {
            return false;
        }
        let held = player.inventory().get_slot(slot);
        if held.uid != expected.uid || !held.are_equal(expected) {
            return false;
        }
        if let Some(boat) = server.item_registry.boat_item(held.get_item().id) {
            let (yaw, pitch) = player.rotation();
            return boat.place_from_hand_in_world(&held, player, yaw, pitch, hand, world);
        }
        use_cart_in_air(player, hand, slot, &held, world).consumes_action()
    }

    fn handle_vehicle_transaction(
        player: &Arc<Player>,
        data: &UseItemTransactionData,
    ) -> Option<bool> {
        if !matches!(data.action_type.0, 0 | 1) {
            return None;
        }
        let world = player.world();
        let server = world.server.upgrade()?;
        let hand = match data.hand {
            HandSlot::Mainhand => Hand::Right,
            HandSlot::Offhand => Hand::Left,
        };
        let inventory = player.inventory();
        let slot_type = crate::entity::hand_equipment_slot(hand);
        let slot = inventory.equipment_slot_index(&slot_type)?;
        let held = inventory.get_slot(slot);
        let descriptor = descriptor_to_stack(&data.item_in_hand);
        let minecart = server.item_registry.minecart_item(held.get_item().id);
        let uses_vehicle = |id| {
            server.item_registry.uses_boat_placement(id)
                || server.item_registry.minecart_item(id).is_some()
        };
        let vehicle_click =
            uses_vehicle(held.get_item().id) || uses_vehicle(descriptor.get_item().id);
        let protect_main_hand = data.action_type.0 == 0
            && matches!(hand, Hand::Left)
            && uses_vehicle(inventory.held_item().get_item().id);
        if !vehicle_click && !protect_main_hand {
            return None;
        }
        if descriptor.get_item().id != held.get_item().id
            || !data.matches_source_slot(slot, held.uid)
        {
            return Some(false);
        }

        if data.action_type.0 == 1 {
            return Some(Self::use_vehicle_in_air(
                &server, &world, player, hand, slot, &held,
            ));
        }
        if !crate::net::vehicle_block_use_is_eligible(player, &world, &data.block_position) {
            return Some(false);
        }
        let block = world.get_block(&data.block_position);

        Some(
            VehicleBlockClick {
                player,
                server: &server,
                world: &world,
                data,
                block,
                hand,
                slot_type,
                slot,
                source: held,
            }
            .handle(vehicle_click, minecart.is_some()),
        )
    }

    const fn interaction_face(face: u8) -> BlockDirection {
        match face {
            0 => BlockDirection::Down,
            2 => BlockDirection::North,
            3 => BlockDirection::South,
            4 => BlockDirection::West,
            5 => BlockDirection::East,
            _ => BlockDirection::Up,
        }
    }

    fn correct_rejected_food_use(&self, player: &Player) {
        // Holding use can repeat rejected transactions. Limit prediction corrections
        // to every 20 ticks, without indefinitely trusting a previously sent snapshot.
        const CORRECTION_INTERVAL_TICKS: i32 = 20;
        let tick = player.tick_counter.load(Ordering::Relaxed);
        let recently_corrected = self.last_food_rejection_tick.load().is_some_and(|last| {
            tick >= last && tick.saturating_sub(last) < CORRECTION_INTERVAL_TICKS
        });
        let has_active_use = player
            .living_entity
            .item_in_use
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some();
        if recently_corrected && !has_active_use {
            return;
        }
        if player.has_client_loaded() {
            self.last_food_rejection_tick.store(Some(tick));
        }
        player.living_entity.clear_active_hand();
        player.send_health();
    }

    #[allow(clippy::too_many_lines, clippy::collapsible_if, clippy::unreachable)]
    pub fn handle_inventory_action(&self, player: &Arc<Player>, packet: SInventoryTransaction) {
        tracing::debug!("handle_inventory_action: packet={:?}", packet);
        if let TransactionData::UseItem(data) = &packet.transaction_data
            && let Some(accepted) = Self::handle_vehicle_transaction(player, data)
        {
            // The interaction committed the source. The queued client changes are predictions.
            if !packet.actions.is_empty() || !packet.legacy_set_item_slots.is_empty() {
                player.resync_player_inventory_screen();
            }
            self.send_inventory_action_response(
                player,
                packet.legacy_request_id,
                u8::from(!accepted),
                Vec::new(),
            );
            return;
        }
        let equipment_before = player.screen_equipment();
        let mut inventory_updated = false;
        let mut updates = Vec::new();
        let mut drops = Vec::new();
        let result = 0u8;

        if packet.actions.is_empty() && packet.legacy_request_id.0 != 0 {
            let mut player_screen_handler = player
                .player_screen_handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for legacy_slot in &packet.legacy_set_item_slots {
                // Legacy slots may name the container instead of its id
                let mapped_window_id = match i32::from(legacy_slot.container_id) {
                    28 | 29 => WINDOW_ID_INVENTORY, // HotBar or Inventory
                    6 | WINDOW_ID_ARMOUR => WINDOW_ID_ARMOUR,
                    34 | WINDOW_ID_OFF_HAND => WINDOW_ID_OFF_HAND,
                    other => other,
                };
                for &slot_id in &legacy_slot.slots {
                    if let Some(screen_slot) =
                        map_bedrock_slot_to_screen_handler(mapped_window_id, slot_id as u32)
                    {
                        let current_stack = player_screen_handler
                            .get_slot(screen_slot)
                            .get_cloned_stack();
                        if !current_stack.is_empty() {
                            player_screen_handler
                                .get_slot(screen_slot)
                                .set_stack(ItemStack::EMPTY.clone());
                            player_screen_handler
                                .set_received_stack(screen_slot, ItemStack::EMPTY.clone());
                            drops.push(current_stack);

                            if let Some((_, container, slot)) =
                                bedrock_inventory_slot(screen_slot as i16)
                            {
                                let container_name = if legacy_slot.container_id == 28 {
                                    ContainerName::HotBar
                                } else {
                                    container
                                };
                                record_update(
                                    &mut updates,
                                    FullContainerName {
                                        container_name,
                                        dynamic_id: None,
                                    },
                                    slot as u8,
                                    ItemStack::EMPTY,
                                );
                            }
                            inventory_updated = true;
                        }
                    }
                }
            }
            drop(player_screen_handler);
        }

        for action in &packet.actions {
            use pumpkin_protocol::bedrock::server::inventory_transaction::InventoryActionSource;
            let source_type = InventoryActionSource::from(action.source_type);
            if source_type == InventoryActionSource::World {
                let old_stack = descriptor_to_stack(&action.old_item);
                let new_stack = descriptor_to_stack(&action.new_item);
                if old_stack.is_empty() && !new_stack.is_empty() {
                    drops.push(new_stack);
                }
            } else if let Some(window_id) = action.window_id {
                if let Some(screen_slot) =
                    map_bedrock_slot_to_screen_handler(window_id, action.inventory_slot)
                {
                    let item_stack = descriptor_to_stack(&action.new_item);

                    let mut player_screen_handler = player
                        .player_screen_handler
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);

                    player_screen_handler
                        .get_slot(screen_slot)
                        .set_stack(item_stack.clone());
                    player_screen_handler.set_received_stack(screen_slot, item_stack);
                    drop(player_screen_handler);

                    inventory_updated = true;
                }
            }
        }

        let equipment_committed = player.screen_equipment();
        if inventory_updated {
            player.resync_player_inventory_screen();
            player.publish_committed_screen_equipment(&equipment_before, &equipment_committed);
        }
        for stack in drops {
            player.drop_item(stack);
        }

        match packet.transaction_data {
            TransactionData::Normal(_data) => {
                // Actions are already applied to the inventory screen handler above.
            }
            TransactionData::Mismatch(_data) => {
                // Actions are already applied to the inventory screen handler above.
            }
            TransactionData::UseItem(data) => {
                let face = Self::interaction_face(data.block_face);
                let world = player.world();
                let block = world.get_block(&data.block_position);
                let Some(server) = world.server.upgrade() else {
                    return;
                };

                if player.gamemode.load() == GameMode::Spectator {
                    if data.action_type.0 == 0
                        && let Some(factory) = server.block_registry.get_screen_handler_factory(
                            block,
                            player,
                            &data.block_position,
                            &server,
                            &world,
                        )
                    {
                        player.open_handled_screen(factory.as_ref(), Some(data.block_position));
                    }
                    return;
                }

                if data.action_type.0 == 0 {
                    // Click block
                    let client_stack = descriptor_to_stack(&data.item_in_hand);
                    let client_item_id = client_stack.item.id;

                    let mut held_item = player.inventory().held_item();
                    if !client_stack.is_empty()
                        && (held_item.is_empty() || held_item.item.id != client_stack.item.id)
                    {
                        held_item = client_stack;
                    }

                    let before_use = held_item.clone();
                    let mut physical_source = None;
                    let hit = BlockHitResult {
                        face: &face,
                        cursor_pos: &data.click_position,
                    };
                    let result = if server
                        .block_registry
                        .get_pumpkin_block(block.id)
                        .is_some_and(|behaviour| behaviour.requires_physical_item_source())
                    {
                        let equipment_slot = match data.hand {
                            HandSlot::Mainhand => EquipmentSlot::MAIN_HAND,
                            HandSlot::Offhand => EquipmentSlot::OFF_HAND,
                        };
                        let inventory = player.inventory();
                        let Some(slot) = inventory.equipment_slot_index(&equipment_slot) else {
                            return;
                        };
                        let mut source = inventory.get_slot(slot);
                        if client_item_id != source.item.id
                            || !data.matches_source_slot(slot, source.uid)
                        {
                            player.sync_hand_slot(slot, source);
                            return;
                        }
                        physical_source = Some((
                            slot,
                            equipment_slot.clone(),
                            source.clone(),
                            player.gamemode.load(),
                        ));
                        let result = server.block_registry.use_with_item_from_slot(
                            crate::block::UseWithItemArgs {
                                block,
                                player,
                                position: &data.block_position,
                                hit: &hit,
                                item_stack: &mut source,
                                equipment_slot: &equipment_slot,
                                server: &server,
                                world: &world,
                            },
                            Some(slot),
                        );
                        if result.consumes_action() {
                            return;
                        }
                        result
                    } else {
                        server.block_registry.use_with_item(
                            block,
                            player,
                            &data.block_position,
                            &hit,
                            &mut held_item,
                            &EquipmentSlot::MAIN_HAND,
                            &server,
                            &world,
                        )
                    };

                    if result.consumes_action() {
                        if !held_item.are_equal(&before_use) {
                            if before_use.is_damageable() && held_item.is_empty() {
                                player.increment_stat(
                                    pumpkin_data::statistic::StatisticCategory::Broken,
                                    i32::from(before_use.item.id),
                                    1,
                                );
                                player.world().send_entity_status(
                                    player.get_entity(),
                                    crate::entity::equipment_break_status(
                                        &EquipmentSlot::MAIN_HAND,
                                    ),
                                    ActorEventID::None,
                                );
                            }
                            let selected = player.inventory().get_selected_slot() as usize;
                            player.inventory().set_held_item(held_item.clone());
                            player.sync_hand_slot(selected, held_item);
                        }
                        return;
                    }

                    if matches!(result, BlockActionResult::PassToDefaultBlockAction)
                        && (physical_source.is_none() || matches!(data.hand, HandSlot::Mainhand))
                    {
                        let result = server.block_registry.on_use(
                            block,
                            player,
                            &data.block_position,
                            &BlockHitResult {
                                face: &face,
                                cursor_pos: &data.click_position,
                            },
                            &server,
                            &world,
                        );

                        if result.consumes_action() {
                            return;
                        }
                    }

                    let source_is_current = || {
                        physical_source.as_ref().is_none_or(
                            |(slot, equipment_slot, before, mode)| {
                                let inventory = player.inventory();
                                let current = inventory.get_slot(*slot);
                                player.gamemode.load() == *mode
                                    && Arc::ptr_eq(&player.world(), &world)
                                    && world.get_block(&data.block_position).id == block.id
                                    && inventory.equipment_slot_index(equipment_slot) == Some(*slot)
                                    && current.uid == before.uid
                                    && current.are_equal(before)
                            },
                        )
                    };
                    let mut stack = physical_source
                        .as_ref()
                        .map_or(held_item, |(_, _, before, _)| before.clone());
                    if !stack.is_empty() && source_is_current() {
                        let item_id = stack.item.id;
                        let before = stack.clone();
                        if !server.item_registry.uses_boat_placement(item_id) {
                            player.increment_stat(
                                pumpkin_data::statistic::StatisticCategory::Used,
                                i32::from(item_id),
                                1,
                            );
                        }
                        if !source_is_current() {
                            return;
                        }
                        let item_result = server.item_registry.use_on_block(
                            &mut stack,
                            player,
                            data.block_position,
                            face,
                            data.click_position,
                            block,
                            &server,
                        );

                        if should_try_block_placement(&item_result) && source_is_current() {
                            let item_id = stack.item.id;
                            if let Some(placed_block) = pumpkin_data::Block::from_item_id(item_id) {
                                let dummy_use_item_on =
                                    pumpkin_protocol::java::server::play::SUseItemOn {
                                        hand: VarInt(i32::from(
                                            physical_source.is_some()
                                                && matches!(data.hand, HandSlot::Offhand),
                                        )),
                                        position: data.block_position,
                                        face: VarInt(i32::from(data.block_face)),
                                        cursor_pos: data.click_position,
                                        inside_block: false,
                                        is_against_world_border: false,
                                        sequence: VarInt(0),
                                    };

                                let placement = server.block_registry.place_block_with_commit(
                                    player,
                                    placed_block,
                                    &server,
                                    &dummy_use_item_on,
                                    data.block_position,
                                    face,
                                    || {
                                        physical_source.as_ref().is_none_or(
                                            |(slot, equipment_slot, source, mode)| {
                                                let inventory = player.inventory();
                                                let current = inventory.get_slot(*slot);
                                                if player.gamemode.load() != *mode
                                                    || inventory
                                                        .equipment_slot_index(equipment_slot)
                                                        != Some(*slot)
                                                    || current.uid != source.uid
                                                    || !current.are_equal(source)
                                                {
                                                    return false;
                                                }
                                                let mut remainder = stack.clone();
                                                if *mode != GameMode::Creative {
                                                    remainder.decrement(1);
                                                }
                                                remainder.are_equal(source)
                                                    || inventory.replace_slot_if_same(
                                                        *slot, source, remainder,
                                                    )
                                            },
                                        )
                                    },
                                );
                                if let Ok(Some(_)) = placement {
                                    if let Some((slot, _, _, _)) = physical_source {
                                        player.sync_hand_slot(
                                            slot,
                                            player.inventory().get_slot(slot),
                                        );
                                        return;
                                    }
                                    if player.gamemode.load() != GameMode::Creative {
                                        stack.decrement(1);
                                    }
                                }
                            }
                        }
                        if let Some((slot, _, source, _)) = &physical_source {
                            if !source_is_current()
                                || stack.are_equal(source)
                                || !player.inventory().replace_slot_if_same(
                                    *slot,
                                    source,
                                    stack.clone(),
                                )
                            {
                                return;
                            }
                        }
                        if before.is_damageable() && stack.is_empty() {
                            let equipment_slot = physical_source
                                .as_ref()
                                .map_or(&EquipmentSlot::MAIN_HAND, |(_, equipment_slot, _, _)| {
                                    equipment_slot
                                });
                            player.increment_stat(
                                pumpkin_data::statistic::StatisticCategory::Broken,
                                item_id as i32,
                                1,
                            );
                            player.world().send_entity_status(
                                player.get_entity(),
                                crate::entity::equipment_break_status(equipment_slot),
                                ActorEventID::None,
                            );
                        }
                        if let Some((slot, _, _, _)) = physical_source {
                            player.sync_hand_slot(slot, player.inventory().get_slot(slot));
                        } else {
                            player.inventory().set_held_item(stack);
                        }
                    }
                } else if data.action_type.0 == 1 {
                    // Click air / Use item
                    let client_stack = descriptor_to_stack(&data.item_in_hand);

                    let hand = match data.hand {
                        HandSlot::Mainhand => Hand::Right,
                        HandSlot::Offhand => Hand::Left,
                    };
                    let mut held = player.inventory().get_stack_in_hand(hand);
                    if !client_stack.is_empty()
                        && (held.is_empty() || held.item.id != client_stack.item.id)
                    {
                        held = client_stack;
                        player.inventory().set_stack_in_hand(hand, held.clone());
                    }
                    if !held.is_empty() {
                        player.increment_stat(
                            pumpkin_data::statistic::StatisticCategory::Used,
                            held.item.id as i32,
                            1,
                        );
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
                            if player.is_on_cooldown(&group) {
                                cooldown_active = true;
                            }
                        }

                        if !cooldown_active {
                            // Bedrock can repeat click-air while using an item. Do not
                            // restart its server-side use timer on these timed inputs.
                            let already_using =
                                player.living_entity.item_use_time.load(Ordering::Relaxed) > 0
                                    && player
                                        .living_entity
                                        .item_in_use
                                        .lock()
                                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                                        .as_ref()
                                        .is_some_and(|item| {
                                            item.are_items_and_components_equal(&held)
                                        });
                            if !already_using
                                && held.get_data_component::<ConsumableImpl>().is_some()
                            {
                                if held
                                    .get_data_component::<FoodImpl>()
                                    .is_none_or(|food| player.can_eat(food.can_always_eat))
                                {
                                    player.living_entity.set_active_hand(
                                        hand,
                                        held.clone(),
                                        held.get_max_use_time(),
                                    );
                                } else {
                                    // Correct predicted eating when the server's food
                                    // level is already full, even if it has not changed.
                                    self.correct_rejected_food_use(player);
                                }
                            }
                            if let Some(equippable) = held.get_data_component::<EquippableImpl>() {
                                let should_change = {
                                    let inventory = player.inventory();
                                    let equipment_guard = inventory
                                        .entity_equipment
                                        .lock()
                                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                                    let current_equipped = equipment_guard.get(equippable.slot);
                                    !current_equipped.are_items_and_components_equal(&held)
                                };
                                if should_change {
                                    let slot = equippable.slot.clone();
                                    let equipped = held.clone();
                                    let inventory = player.inventory();
                                    let mut equipment_guard = inventory
                                        .entity_equipment
                                        .lock()
                                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                                    let equip_item = equipment_guard
                                        .equipment
                                        .entry(slot.clone())
                                        .or_insert_with(|| ItemStack::EMPTY.clone());
                                    if equip_item.is_empty() {
                                        *equip_item = held.clone();
                                        held.decrement_unless_creative(player.gamemode.load(), 1);
                                    } else {
                                        let old_held = held.clone();
                                        held = equip_item.clone();
                                        *equip_item = old_held;
                                    }
                                    drop(equipment_guard);
                                    player.inventory().set_stack_in_hand(hand, held.clone());
                                    player.enqueue_equipment_change(&slot, &equipped);
                                }
                            }
                        }
                    }

                    send_cancellable_blocking! {{
                        &server;
                        event;
                        'after: {
                            server.item_registry.on_use(&stack_for_use, player, hand);
                        }
                    }}
                }
            }
            TransactionData::UseItemOnEntity(data) => {
                let action = match data.action_type.0 {
                    // Bedrock does not distinguish an entity hit position here. ItemInteract is
                    // therefore exposed as the general Interact action rather than InteractAt.
                    0 | 2 => ActionType::Interact,
                    1 => ActionType::Attack,
                    action => {
                        tracing::warn!("invalid UseItemOnEntity action type {action}");
                        return;
                    }
                };
                let Ok(target_runtime_id) = i32::try_from(data.target_entity_runtime_id.0) else {
                    tracing::warn!(
                        "invalid UseItemOnEntity target runtime ID {}",
                        data.target_entity_runtime_id.0
                    );
                    return;
                };

                let world = player.world();
                let Some(target) = world.get_entity_by_id(target_runtime_id) else {
                    return;
                };
                if !player.can_interact_with_entity(target.as_ref(), 1.0) {
                    return;
                }
                let Some(server) = world.server.upgrade() else {
                    return;
                };

                let mut event = PlayerInteractEntityEvent::new(
                    player,
                    target.clone(),
                    action,
                    None,
                    player.get_entity().is_sneaking(),
                );
                event = server.plugin_manager.fire_blocking(&server, event);
                if event.cancelled {
                    return;
                }

                match event.action {
                    ActionType::Interact | ActionType::InteractAt => {
                        let mut stack = player.inventory().held_item();
                        let item_id = stack.item.id;
                        let before = stack.clone();
                        let native_interaction = event.target.interact_with_hand(
                            player,
                            &mut stack,
                            pumpkin_util::Hand::Right,
                        );
                        if !native_interaction
                            .unwrap_or_else(|| event.target.interact(player, &mut stack))
                        {
                            server
                                .item_registry
                                .use_on_entity(&mut stack, player, event.target);
                        }
                        if native_interaction.is_none() && !stack.are_equal(&before) {
                            player.increment_stat(
                                pumpkin_data::statistic::StatisticCategory::Used,
                                item_id as i32,
                                1,
                            );
                            if before.is_damageable() && stack.is_empty() {
                                player.increment_stat(
                                    pumpkin_data::statistic::StatisticCategory::Broken,
                                    item_id as i32,
                                    1,
                                );
                                player.world().send_entity_status(
                                    player.get_entity(),
                                    crate::entity::equipment_break_status(
                                        &EquipmentSlot::MAIN_HAND,
                                    ),
                                    ActorEventID::None,
                                );
                            }
                        }
                        player.inventory().set_held_item(stack);
                    }
                    ActionType::Attack => player.attack(&event.target),
                }
            }
            TransactionData::ReleaseItem(_data) => {
                if let Some(server) = player.world().server.upgrade() {
                    server.item_registry.release_item(player);
                }
            }
        }

        self.send_inventory_action_response(player, packet.legacy_request_id, result, updates);
    }

    fn send_inventory_action_response(
        &self,
        player: &Arc<Player>,
        request_id: VarInt,
        result: u8,
        updates: Vec<SlotUpdate>,
    ) {
        if request_id.0 != 0 {
            use pumpkin_protocol::bedrock::client::item_stack_response::{
                CItemStackResponse, ItemStackResponseContainerInfo, ItemStackResponseInfo,
                ItemStackResponseSlotInfo,
            };

            use pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestSlotInfo;
            let mut container_infos = Vec::new();
            let mut affected = super::item_stack_request::AcknowledgedSlots::new();
            if result == 0 {
                for update in updates {
                    affected.push((
                        ItemStackRequestSlotInfo {
                            container_name: update.container_name.clone(),
                            slot_id: update.slot_id,
                            stack_id: update.stack_id.0,
                        },
                        ItemStack::EMPTY.clone(),
                    ));
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
                            full_container_name: update.container_name.clone(),
                            slots: vec![slot_info],
                        });
                    }
                }
            }

            let response = CItemStackResponse {
                responses: vec![ItemStackResponseInfo {
                    result,
                    client_request_id: request_id,
                    containers: container_infos,
                }],
            };
            let personal: pumpkin_inventory::screen_handler::SharedScreenHandler =
                player.player_screen_handler.clone();
            if !self.enqueue_stack_request_response(player, &personal, &[affected], &response, None)
            {
                player.resync_player_inventory_screen();
            }
        }
    }
}
