use std::{
    num::{NonZero, NonZeroI32},
    sync::{Arc, atomic::Ordering},
};

use pumpkin_data::{
    data_component_impl::{
        BlocksAttacksImpl, ConsumableImpl, EquipmentSlot, EquippableImpl, FoodImpl,
    },
    item_stack::ItemStack,
};
use pumpkin_inventory::{
    player::player_inventory::PlayerInventory,
    screen_handler::{InventoryPlayer, ScreenHandler},
};
use pumpkin_protocol::bedrock::{
    client::{inventory_content::CInventoryContent, respawn::CRespawn},
    network_item::{
        ContainerName, FullContainerName, NetworkItemDescriptor, NetworkItemStackDescriptor,
    },
    respawn::RespawnState,
};
use pumpkin_protocol::{
    bedrock::{
        client::{
            chunk_radius_update::CChunkRadiusUpdate, container_open::CContainerOpen,
            player_hotbar::CPlayerHotbar, update_block::CUpdateBlock,
        },
        server::{
            animate::{AnimateAction, SAnimate},
            block_pick_request::SBlockPickRequest,
            command_request::SCommandRequest,
            container_close::SContainerClose,
            emote::SEmote,
            emote_list::SEmoteList,
            interaction::{Action, SInteraction},
            inventory_transaction::{SInventoryTransaction, TransactionData},
            mob_equipment::SMobEquipment,
            player_action::{Action as PlayerAction, SPlayerAction},
            player_auth_input::{InputData, SPlayerAuthInput},
            request_chunk_radius::SRequestChunkRadius,
            respawn::SRespawn,
            set_local_player_as_initialized::SSetLocalPlayerAsInitialized,
            text::SText,
        },
    },
    codec::{var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong},
    java::client::play::{Animation, CEntityAnimation, CSetSelectedSlot, CSystemChatMessage},
};
use pumpkin_util::{GameMode, Hand, math::position::BlockPos, text::TextComponent};

use pumpkin_world::inventory::Inventory;
use pumpkin_world::world::BlockFlags;

use crate::{
    block::{BlockHitResult, registry::BlockActionResult},
    entity::{
        EntityBase,
        player::{MINE_BLOCK_EXHAUSTION, Player},
    },
    net::{DisconnectReason, bedrock::BedrockClient},
    plugin::player::{
        item_held::PlayerItemHeldEvent,
        player_chat::PlayerChatEvent,
        player_command_send::PlayerCommandSendEvent,
        player_interact_event::{InteractAction, PlayerInteractEvent},
        player_toggle_flight_event::PlayerToggleFlightEvent,
    },
    server::{Server, seasonal_events},
    world::{BlockBreakingProgress, chunker},
};
use pumpkin_data::BlockDirection;
use tracing::{debug, info};

const MIN_PREDICTED_BREAK_PROGRESS: f32 = 0.65;

fn descriptor_to_stack(desc: &NetworkItemDescriptor) -> ItemStack {
    if desc.id.0 == 0 || desc.stack_size == 0 {
        ItemStack::EMPTY.clone()
    } else {
        pumpkin_data::item::JavaToBedrockItemMapping::from_bedrock(
            desc.id.0 as i16,
            desc.aux_value.0,
        )
        .map_or_else(
            || {
                tracing::warn!(
                    "Failed to map bedrock item id {} and data {} to Java item",
                    desc.id.0,
                    desc.aux_value.0
                );
                ItemStack::EMPTY.clone()
            },
            |mapping| ItemStack::new(desc.stack_size as u8, mapping.java_item),
        )
    }
}

const fn map_bedrock_slot_to_screen_handler(window_id: i32, slot: u32) -> Option<usize> {
    match window_id {
        0 => {
            // WINDOW_ID_INVENTORY
            if slot < 9 {
                // Hotbar: Bedrock 0-8 -> Screen Handler 36-44
                Some(slot as usize + 36)
            } else if slot < 36 {
                // Main Inventory: Bedrock 9-35 -> Screen Handler 9-35
                Some(slot as usize)
            } else {
                None
            }
        }
        120 => {
            // WINDOW_ID_ARMOUR
            if slot < 4 {
                // Armor: Bedrock 0-3 -> Screen Handler 5-8
                Some(slot as usize + 5)
            } else {
                None
            }
        }
        119 => {
            // WINDOW_ID_OFF_HAND
            if slot == 0 {
                // Offhand: Bedrock 0 -> Screen Handler 45
                Some(45)
            } else {
                None
            }
        }
        _ => None,
    }
}

impl BedrockClient {
    pub async fn handle_request_chunk_radius(
        &self,
        player: &Arc<Player>,
        packet: SRequestChunkRadius,
    ) {
        let chunk_radius = packet.chunk_radius;
        if chunk_radius.0 < 1 {
            self.kick(
                DisconnectReason::Kicked,
                "Cannot have zero or negative view distance!".to_string(),
            )
            .await;
            return;
        }
        let Some(server) = player.world().server.upgrade() else {
            return;
        };

        let view_distance = chunk_radius.clamp(
            2,
            NonZeroI32::from(server.advanced_config.networking.bedrock.view_distance).get(),
        );

        self.enqueue_packet(&CChunkRadiusUpdate {
            chunk_radius: VarInt(view_distance),
        })
        .await;

        let old_view_distance = {
            let current_config = player.config.load();
            let old_vd = current_config.view_distance;
            let mut new_config = (**current_config).clone();

            new_config.view_distance =
                NonZero::new(view_distance as u8).unwrap_or(NonZero::<u8>::MIN);
            player.config.store(std::sync::Arc::new(new_config));

            old_vd
        };

        debug!(
            "Player {} updated their render distance: {} -> {}.",
            player.gameprofile.name, old_view_distance, view_distance
        );
        chunker::update_position(player).await;
    }

    pub fn handle_set_local_player_as_initialized(
        &self,
        player: &Arc<Player>,
        packet: &SSetLocalPlayerAsInitialized,
    ) {
        debug!(
            "Player {} initialized (Runtime ID: {})",
            player.gameprofile.name, packet.runtime_entity_id.0
        );
        // This is sent when the client has finished loading and rendering the world.
        player.set_client_loaded(true);
    }

    #[expect(clippy::too_many_lines)]
    pub async fn handle_player_auth_input(
        &self,
        player: &Arc<Player>,
        packet: SPlayerAuthInput,
        server: &Arc<Server>,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        if player.living_entity.dead.load(Ordering::Relaxed)
            || player.living_entity.health.load() <= 0.0
        {
            return;
        }
        let entity = player.get_entity();
        let on_ground = packet.input_data.get(InputData::VerticalCollision as usize)
            && packet.delta.y < 0.0
            && !entity.has_vehicle().await;
        entity.on_ground.store(on_ground, Ordering::Relaxed);

        let new_pos = packet
            .position
            .add_raw(0.0, -entity.entity_type.eye_height, 0.0)
            .to_f64();
        let old_pos = player.position();

        let new_pitch = packet.pitch;
        let new_yaw = packet.yaw;

        let old_pitch = entity.pitch.load();
        let old_yaw = entity.yaw.load();

        let pos_changed = new_pos != old_pos;
        let rot_changed = new_pitch != old_pitch || new_yaw != old_yaw;

        if pos_changed || rot_changed {
            let world = player.world();

            if pos_changed {
                player.get_entity().set_pos(new_pos);
            }
            if rot_changed {
                entity.pitch.store(new_pitch);
                entity.yaw.store(new_yaw);
            }

            let je_yaw = (new_yaw * 256.0 / 360.0).rem_euclid(256.0);
            let je_pitch = (new_pitch * 256.0 / 360.0).rem_euclid(256.0);

            let delta = pumpkin_util::math::vector3::Vector3::new(
                new_pos.x - old_pos.x,
                new_pos.y - old_pos.y,
                new_pos.z - old_pos.z,
            );

            let bedrock_move_packet = pumpkin_protocol::bedrock::client::CMovePlayer::new(
                pumpkin_protocol::codec::var_ulong::VarULong(player.entity_id() as u64),
                pumpkin_util::math::vector3::Vector3::new(
                    new_pos.x as f32,
                    new_pos.y as f32 + entity.entity_type.eye_height,
                    new_pos.z as f32,
                ),
                new_pitch,
                new_yaw,
                new_yaw, // Head yaw
                pumpkin_protocol::bedrock::client::CMovePlayer::MODE_NORMAL,
                on_ground,
                pumpkin_protocol::codec::var_ulong::VarULong(0),
                0,
                0,
                pumpkin_protocol::codec::var_ulong::VarULong(0),
            );

            if pos_changed && delta.length_squared() >= 64.0 {
                world.broadcast_packet_except(
                    &[player.gameprofile.id],
                    &pumpkin_protocol::java::client::play::CEntityPositionSync::new(
                        player.entity_id().into(),
                        new_pos,
                        pumpkin_util::math::vector3::Vector3::new(0.0, 0.0, 0.0),
                        je_yaw,
                        je_pitch,
                        on_ground,
                    ),
                );
            } else if pos_changed && rot_changed {
                world.broadcast_packet_except_editioned_sync(
                    &[player.gameprofile.id],
                    &pumpkin_protocol::java::client::play::CUpdateEntityPosRot::new(
                        player.entity_id().into(),
                        pumpkin_util::math::vector3::Vector3::new(
                            new_pos.x.mul_add(4096.0, -(old_pos.x * 4096.0)) as i16,
                            new_pos.y.mul_add(4096.0, -(old_pos.y * 4096.0)) as i16,
                            new_pos.z.mul_add(4096.0, -(old_pos.z * 4096.0)) as i16,
                        ),
                        je_yaw as u8,   // Use converted Java byte
                        je_pitch as u8, // Use converted Java byte
                        on_ground,
                    ),
                    &bedrock_move_packet,
                );
            } else if pos_changed {
                world.broadcast_packet_except_editioned_sync(
                    &[player.gameprofile.id],
                    &pumpkin_protocol::java::client::play::CUpdateEntityPos::new(
                        player.entity_id().into(),
                        pumpkin_util::math::vector3::Vector3::new(
                            new_pos.x.mul_add(4096.0, -(old_pos.x * 4096.0)) as i16,
                            new_pos.y.mul_add(4096.0, -(old_pos.y * 4096.0)) as i16,
                            new_pos.z.mul_add(4096.0, -(old_pos.z * 4096.0)) as i16,
                        ),
                        on_ground,
                    ),
                    &bedrock_move_packet,
                );
            } else if rot_changed {
                world.broadcast_packet_except_editioned_sync(
                    &[player.gameprofile.id],
                    &pumpkin_protocol::java::client::play::CUpdateEntityRot::new(
                        player.entity_id().into(),
                        je_yaw as u8,   // Use converted Java byte
                        je_pitch as u8, // Use converted Java byte
                        on_ground,
                    ),
                    &bedrock_move_packet,
                );
            }

            if rot_changed {
                world.broadcast_packet_except(
                    &[player.gameprofile.id],
                    // Adjust to `CHeadRot` if that is what your crate currently calls it
                    &pumpkin_protocol::java::client::play::CHeadRot::new(
                        player.entity_id().into(),
                        je_yaw as u8,
                    ),
                );
            }

            if pos_changed {
                chunker::update_position(player).await;
                player.progress_motion(delta).await;
            }
        }

        let input_data = packet.input_data;

        if input_data.get(InputData::StartSprinting as usize) {
            entity.set_sprinting(true).await;
        } else if input_data.get(InputData::StopSprinting as usize) {
            entity.set_sprinting(false).await;
        }

        if input_data.get(InputData::StartSneaking as usize) {
            entity.set_sneaking(true).await;
        } else if input_data.get(InputData::StopSneaking as usize) {
            entity.set_sneaking(false).await;
        }

        if input_data.get(InputData::StartFlying as usize) {
            let flying = { player.abilities.lock().await.flying };
            if !flying {
                send_cancellable! {{
                    server;
                    PlayerToggleFlightEvent::new(player.clone(), true);
                    'after: {
                        {
                            player.abilities.lock().await.flying = true;
                        };
                        player.send_abilities_update().await;
                    }
                    'cancelled: {
                        player.send_abilities_update().await;
                    }
                }}
            }
        } else if input_data.get(InputData::StopFlying as usize) {
            let flying = { player.abilities.lock().await.flying };
            if flying {
                send_cancellable! {{
                    server;
                    PlayerToggleFlightEvent::new(player.clone(), false);
                    'after: {
                        {
                            player.abilities.lock().await.flying = false;
                        };
                        player.send_abilities_update().await;
                    }
                    'cancelled: {
                        player.send_abilities_update().await;
                    }
                }}
            }
        }

        if let Some(block_actions) = packet.block_actions {
            for action in block_actions {
                self.handle_player_block_action(player, server, action)
                    .await;
            }
        }
    }

    pub async fn handle_player_block_action(
        &self,
        player: &Arc<Player>,
        server: &Server,
        packet: pumpkin_protocol::bedrock::server::player_auth_input::PlayerBlockAction,
    ) {
        use pumpkin_protocol::bedrock::server::player_action::Action as PlayerAction;
        let Ok(action) = PlayerAction::try_from(packet.action.0) else {
            return;
        };
        self.handle_player_action(
            player,
            server,
            SPlayerAction {
                runtime_id: VarULong(0), // Unused
                action,
                block_pos: packet.block_pos,
                result_pos: BlockPos::ZERO,
                face: packet.face,
            },
        )
        .await;
    }

    pub async fn handle_animate(&self, player: &Arc<Player>, _server: &Server, packet: &SAnimate) {
        if !player.has_client_loaded() {
            return;
        }

        let entity = &player.get_entity();
        let world = entity.world.load();

        let java_animation = match packet.action {
            AnimateAction::NoAction => None,
            AnimateAction::SwingArm => Some(Animation::SwingMainArm),
            AnimateAction::WakeUp => Some(Animation::LeaveBed),
            AnimateAction::CriticalHit => Some(Animation::CriticalEffect),
            AnimateAction::MagicCriticalHit => Some(Animation::MagicCriticaleffect),
        };

        if let Some(animation) = java_animation {
            let je_packet = CEntityAnimation::new(VarInt(entity.entity_id), animation);
            let be_packet = SAnimate {
                action: packet.action,
                runtime_entity_id: VarULong(entity.entity_id as u64),
                data: 0.0,
                swing_source: None,
            };
            world.broadcast_editioned(&je_packet, &be_packet).await;
        }
    }

    pub async fn handle_emote(&self, player: &Arc<Player>, _server: &Server, packet: SEmote<'_>) {
        if !player.has_client_loaded() {
            return;
        }

        tracing::info!(
            "handle_emote: player={} packet={:?}",
            player.gameprofile.name,
            packet
        );

        let entity = &player.living_entity.entity;
        let world = entity.world.load();

        let mut broadcast_packet = packet;
        broadcast_packet.runtime_entity_id = VarULong(entity.entity_id as u64);
        broadcast_packet.flags |= pumpkin_protocol::bedrock::server::emote::EMOTE_FLAG_SERVER_SIDE;

        world
            .broadcast_packet_except_editioned(
                &[player.gameprofile.id],
                &CEntityAnimation::new(
                    VarInt(entity.entity_id),
                    Animation::SwingMainArm, // Fallback for Java? Or just ignore
                ),
                &broadcast_packet,
            )
            .await;
    }

    pub fn handle_emote_list(&self, player: &Arc<Player>, _server: &Server, packet: &SEmoteList) {
        tracing::info!(
            "handle_emote_list: player={} packet={:?}",
            player.gameprofile.name,
            packet
        );
    }

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
                                    face: VarInt(data.block_face),
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

    pub async fn handle_interaction(&self, player: &Arc<Player>, packet: SInteraction) {
        match packet.action {
            Action::OpenInventory => {
                if self.inventory_opened.load(Ordering::Relaxed) {
                    return;
                }
                self.inventory_opened.store(true, Ordering::Relaxed);
                self.enqueue_packet(&CContainerOpen {
                    container_id: 0,
                    container_type: 0xff,
                    position: BlockPos::ZERO,
                    target_entity_id: VarLong(-1),
                })
                .await;
            }
            // No longer used in newer versions
            Action::Attack => {
                let target_runtime_id = packet.target_runtime_id.0 as i32;
                let world = player.world();
                if let Some(target) = world.get_entity_by_id(target_runtime_id) {
                    player.attack(target).await;
                }
            }
            _ => {}
        }
    }

    pub async fn handle_container_close(&self, player: &Arc<Player>, packet: SContainerClose) {
        if packet.container_id == 0 || packet.container_id == 0xff {
            self.inventory_opened.store(false, Ordering::Relaxed);
        }
        player.on_handled_screen_closed().await;

        self.enqueue_packet(&SContainerClose {
            container_id: packet.container_id,
            container_type: packet.container_type,
            server_initiated: false,
        })
        .await;

        // Sync the cursor (make it empty) to Bedrock client
        self.enqueue_packet(&CInventoryContent {
            container_id: VarUInt(59), // Cursor container ID
            slots: vec![NetworkItemStackDescriptor::default()],
            full_container_name: FullContainerName {
                container_name: ContainerName::Cursor,
                dynamic_id: None,
            },
            storage_item: NetworkItemStackDescriptor::default(),
        })
        .await;

        // Sync the inventory content to Bedrock client
        self.enqueue_packet(&CInventoryContent {
            container_id: VarUInt(0), // player inventory
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

    pub async fn handle_chat_message(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        packet: SText<'_>,
    ) {
        let gameprofile = &player.gameprofile;

        send_cancellable! {{
            server;
            PlayerChatEvent::new(player.clone(), packet.message.into_owned(), vec![]);

            'after: {
                info!("<chat> {}: {}", gameprofile.name, event.message);

                let config = &server.advanced_config;

                let message = match seasonal_events::modify_chat_message(&event.message, config) {
                    Some(m) => m,
                    None => event.message.clone(),
                };

                let decorated_message = TextComponent::chat_decorated(
                    &config.chat.format,
                    &gameprofile.name,
                    &message,
                );

                let entity = &player.get_entity();
                if server.basic_config.allow_chat_reports {
                    //TODO Alex help, what is this?
                    //world.broadcast_secure_player_chat(player, &message, decorated_message).await;
                } else {
                    let je_packet = CSystemChatMessage::new(
                        &decorated_message,
                        false,
                    );

                    let be_packet = SText::chat_with_xuid(
                        message,
                        gameprofile.name.clone(),
                        packet.xuid.into_owned(),
                        packet.filtered_message.map(std::borrow::Cow::into_owned),
                    );

                    entity.world.load().broadcast_editioned(&je_packet, &be_packet).await;
                }
            }
        }}
    }

    #[expect(clippy::match_same_arms)]
    #[expect(clippy::too_many_lines)]
    pub async fn handle_player_action(
        &self,
        player: &Arc<Player>,
        server: &Server,
        packet: SPlayerAction,
    ) {
        if !player.has_client_loaded()
            || ((player.living_entity.dead.load(Ordering::Relaxed)
                || player.living_entity.health.load() <= 0.0)
                && !matches!(packet.action, PlayerAction::Respawn))
        {
            return;
        }
        player.update_last_action_time();

        match packet.action {
            PlayerAction::StartBreak
            | PlayerAction::CreativePlayerDestroyBlock
            | PlayerAction::ContinueDestroyBlock => {
                let location = packet.block_pos;
                if !player.can_interact_with_block_at(&location, 1.0) {
                    return;
                }

                let entity = &player.get_entity();
                let world = entity.world.load_full();
                let (block, state) = world.get_block_and_state(&location);

                if player.mining.load(Ordering::Relaxed)
                    && *player.mining_pos.lock().await != location
                {
                    player.stop_mining().await;
                }

                if player.gamemode.load() == GameMode::Creative {
                    let new_state = world
                        .break_block(
                            &location,
                            Some(player.clone()),
                            BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::SKIP_DROPS,
                        )
                        .await;
                    if new_state.is_some() {
                        server
                            .block_registry
                            .broken(&world, block, player, &location, server, state)
                            .await;
                    }
                } else if !state.is_air() {
                    let speed = crate::block::calc_block_breaking(player, state, block).await;
                    if speed >= 1.0 {
                        player.stop_mining().await;
                        let broken_state = world.get_block_state(&location);
                        let can_harvest = player.can_harvest(broken_state, block).await;
                        let new_state = world
                            .break_block(
                                &location,
                                Some(player.clone()),
                                BlockFlags::NOTIFY_NEIGHBORS,
                            )
                            .await;
                        if new_state.is_some() {
                            server
                                .block_registry
                                .broken(&world, block, player, &location, server, broken_state)
                                .await;
                            player.apply_tool_damage_for_block_break(broken_state).await;
                            if can_harvest {
                                player.add_exhaustion(MINE_BLOCK_EXHAUSTION).await;
                            }
                        }
                    } else {
                        let mut mining_pos = player.mining_pos.lock().await;
                        let starts_breaking =
                            !player.mining.load(Ordering::Relaxed) || *mining_pos != location;
                        let progress = if starts_breaking {
                            player.start_mining_time.store(
                                player.tick_counter.load(Ordering::Relaxed),
                                Ordering::Relaxed,
                            );
                            player.mining.store(true, Ordering::Relaxed);
                            *mining_pos = location;
                            (speed * 10.0) as i32
                        } else {
                            player.current_block_destroy_stage.load(Ordering::Relaxed)
                        };
                        drop(mining_pos);
                        let old_speed = player
                            .current_block_breaking_speed
                            .swap(speed.to_bits(), Ordering::Relaxed);
                        if starts_breaking {
                            world
                                .set_block_breaking(
                                    entity,
                                    location,
                                    BlockBreakingProgress::Start {
                                        stage: progress,
                                        speed,
                                    },
                                )
                                .await;
                            player
                                .current_block_destroy_stage
                                .store(progress, Ordering::Relaxed);
                        } else if old_speed != speed.to_bits() {
                            world
                                .set_block_breaking(
                                    entity,
                                    location,
                                    BlockBreakingProgress::Update {
                                        stage: progress,
                                        speed: Some(speed),
                                    },
                                )
                                .await;
                        }
                    }
                }
            }
            action @ (PlayerAction::PredictDestroyBlock | PlayerAction::StopBreak) => {
                let location = packet.block_pos;
                if !player.can_interact_with_block_at(&location, 1.0) {
                    return;
                }

                let entity = &player.get_entity();
                let world = entity.world.load_full();

                let (block, state) = world.get_block_and_state(&location);
                if player.gamemode.load() != GameMode::Creative && !state.is_air() {
                    let speed = crate::block::calc_block_breaking(player, state, block).await;
                    let elapsed = player.tick_counter.load(Ordering::Relaxed)
                        - player.start_mining_time.load(Ordering::Relaxed)
                        + 1;
                    let same_block = *player.mining_pos.lock().await == location;
                    if player.mining.load(Ordering::Relaxed)
                        && same_block
                        && speed * elapsed as f32 >= MIN_PREDICTED_BREAK_PROGRESS
                    {
                        player.stop_mining().await;

                        let can_harvest = player.can_harvest(state, block).await;
                        let flags = if can_harvest {
                            BlockFlags::NOTIFY_NEIGHBORS
                        } else {
                            BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_NEIGHBORS
                        };
                        if world
                            .break_block(&location, Some(player.clone()), flags)
                            .await
                            .is_some()
                        {
                            server
                                .block_registry
                                .broken(&world, block, player, &location, server, state)
                                .await;
                            player.apply_tool_damage_for_block_break(state).await;
                            if can_harvest {
                                player.add_exhaustion(MINE_BLOCK_EXHAUSTION).await;
                            }
                        }
                    } else {
                        let runtime_id = pumpkin_data::BlockState::to_be_network_id(state.id);
                        self.enqueue_packet(&CUpdateBlock::new(location, runtime_id as u32))
                            .await;
                        if matches!(action, PlayerAction::StopBreak) {
                            player.stop_mining().await;
                        } else {
                            world
                                .set_block_breaking(
                                    entity,
                                    location,
                                    BlockBreakingProgress::Update {
                                        stage: player
                                            .current_block_destroy_stage
                                            .load(Ordering::Relaxed),
                                        speed: Some(speed),
                                    },
                                )
                                .await;
                        }
                    }
                } else if matches!(action, PlayerAction::StopBreak) {
                    player.stop_mining().await;
                }
            }
            PlayerAction::CrackBreak => {
                // Don't do anything for this action. It is no longer used. Block
                // cracking is done fully server-side.
            }
            PlayerAction::AbortBreak => {
                player.stop_mining().await;
            }
            PlayerAction::DropItem => {
                player.drop_held_item(false).await;
            }
            PlayerAction::Respawn
                if player.living_entity.dead.load(Ordering::Relaxed)
                    || player.living_entity.health.load() <= 0.0 =>
            {
                player.world().respawn_player(player, false).await;
            }
            // TODO
            _ => {}
        }
    }

    pub async fn handle_respawn(&self, player: &Arc<Player>, packet: SRespawn) {
        if packet.state != RespawnState::ClientReadyToSpawn
            || (!player.living_entity.dead.load(Ordering::Relaxed)
                && player.living_entity.health.load() > 0.0)
        {
            return;
        }

        let entity = player.get_entity();
        let position = entity.pos.load();
        self.enqueue_packet(&CRespawn::new(
            pumpkin_util::math::vector3::Vector3::new(
                position.x as f32,
                position.y as f32 + entity.entity_type.eye_height,
                position.z as f32,
            ),
            RespawnState::ReadyToSpawn,
            VarULong(player.entity_id() as u64),
        ))
        .await;
    }

    pub async fn handle_chat_command(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: SCommandRequest<'_>,
    ) {
        let player_clone = player.clone();
        let server_clone = server.clone();
        let command = packet.command.strip_prefix('/').unwrap_or(&packet.command);

        send_cancellable! {{
            server;
            PlayerCommandSendEvent {
                player: player.clone(),
                command: command.to_string(),
                cancelled: false
            };

            'after: {
                let command = event.command;
                let command_clone = command.clone();

                // Some commands can take a long time to execute. If they do, they block packet processing for the player.
                // That's why we will spawn a task instead.
                server.spawn_task(async move {
                    let dispatcher = server_clone.command_dispatcher.read().await;
                    dispatcher.handle_command(
                        &player_clone.get_command_source(&server_clone).await,
                        &command_clone
                    ).await;
                });

                if server.advanced_config.commands.log_console {
                    info!(
                        "Player ({}): executed command /{}",
                        player.gameprofile.name,
                        command
                    );
                }
            }
        }}
    }

    pub async fn handle_modal_form_response(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: pumpkin_protocol::bedrock::server::modal_form_response::SModalFormResponse<'_>,
    ) {
        let mut event =
            crate::plugin::api::events::player::bedrock_form_response::BedrockFormResponseEvent::new(
                player.clone(),
                packet.form_id.0,
                packet
                    .form_data
                    .filter(|data| data != "null")
                    .map(std::borrow::Cow::into_owned),
            );
        server.plugin_manager.fire(server, &mut event).await;
    }

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
        self.enqueue_packet(&CItemStackResponse { responses }).await;

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
    }

    #[allow(clippy::too_many_lines)]
    pub async fn handle_block_pick_request(&self, player: &Arc<Player>, packet: SBlockPickRequest) {
        if !player.can_interact_with_block_at(&packet.block_pos, 1.0) {
            return;
        }

        let world = player.world();
        let block = world.get_block(&packet.block_pos);

        if block.item_id == 0 {
            return;
        }

        let Some(item) = pumpkin_data::item::Item::from_id(block.item_id) else {
            return;
        };
        let stack = ItemStack::new(1, item);

        let target_hotbar_slot = packet.hotbar_slot as usize;
        if target_hotbar_slot >= 9 {
            return;
        }

        let slot_with_stack = player.inventory().get_slot_with_stack(&stack).await;

        if slot_with_stack != -1 {
            if pumpkin_inventory::player::player_inventory::PlayerInventory::is_valid_hotbar_index(
                slot_with_stack as usize,
            ) {
                if slot_with_stack as usize != target_hotbar_slot {
                    let target_stack = player.inventory().get_stack(target_hotbar_slot).await;
                    let source_stack = player.inventory().get_stack(slot_with_stack as usize).await;
                    player
                        .inventory()
                        .set_stack(target_hotbar_slot, source_stack)
                        .await;
                    player
                        .inventory()
                        .set_stack(slot_with_stack as usize, target_stack)
                        .await;
                }
            } else {
                let target_stack = player.inventory().get_stack(target_hotbar_slot).await;
                let source_stack = player.inventory().get_stack(slot_with_stack as usize).await;
                player
                    .inventory()
                    .set_stack(target_hotbar_slot, source_stack)
                    .await;
                player
                    .inventory
                    .set_stack(slot_with_stack as usize, target_stack)
                    .await;
            }
        } else if player.gamemode.load() == GameMode::Creative {
            player.inventory.set_stack(target_hotbar_slot, stack).await;
        } else {
            return;
        }

        player.inventory.set_selected_slot(target_hotbar_slot as u8);

        // Send hotbar updates
        player
            .client
            .enqueue_packet_editioned(
                &CSetSelectedSlot::new(player.inventory.get_selected_slot() as i8),
                &CPlayerHotbar {
                    selected_slot: VarUInt(player.inventory.get_selected_slot() as u32),
                    container_id: 0,
                    should_select_block: true,
                },
            )
            .await;

        // Send screen handler / Java inventory updates
        player
            .player_screen_handler
            .lock()
            .await
            .send_content_updates()
            .await;

        // Sync main hand equipment to other players
        let stack_in_hand = player.inventory().held_item().await;
        let equipment = &[(EquipmentSlot::MAIN_HAND, stack_in_hand)];
        player.living_entity.send_equipment_changes(equipment);

        // Sync bedrock inventory updates
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

    pub async fn handle_mob_equipment(
        &self,
        _server: &Arc<Server>,
        player: &Arc<Player>,
        packet: SMobEquipment,
    ) {
        player.update_last_action_time();
        let slot = packet.hotbar_slot;
        if slot >= 9 {
            return;
        }
        let previous_slot = player.inventory.get_selected_slot();
        let server = player.world().server.upgrade();
        if let Some(server) = server {
            let mut event = PlayerItemHeldEvent::new(player.clone(), previous_slot, slot);
            server.plugin_manager.fire(&server, &mut event).await;
            let is_cancelled = {
                use crate::plugin::Cancellable;
                event.cancelled()
            };
            if is_cancelled {
                self.enqueue_packet(&CPlayerHotbar {
                    selected_slot: VarUInt(previous_slot as u32),
                    container_id: 0,
                    should_select_block: true,
                })
                .await;
                return;
            }
        }

        let inv = player.inventory();
        inv.set_selected_slot(slot);
        let stack = inv.held_item().await;
        let equipment = &[(EquipmentSlot::MAIN_HAND, stack)];
        player.living_entity.send_equipment_changes(equipment);
    }

    pub async fn handle_request_ability(
        &self,
        player: &Arc<Player>,
        packet: pumpkin_protocol::bedrock::server::request_ability::SRequestAbility,
    ) {
        player.update_last_action_time();
        let ability_id = packet.ability.0;
        match ability_id {
            9 => {
                // Flying
                if let pumpkin_protocol::bedrock::server::request_ability::AbilityValue::Bool(
                    requested_flying,
                ) = packet.value
                {
                    let mut abilities = player.abilities.lock().await;
                    if abilities.allow_flying {
                        abilities.flying = requested_flying;
                    } else {
                        abilities.flying = false;
                    }
                    drop(abilities);
                    player.send_abilities_update().await;
                }
            }
            _ => {
                debug!("Received RequestAbility packet for unhandled ability {ability_id}");
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
fn map_bedrock_container_slot(
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

struct SlotUpdate {
    container_name: FullContainerName,
    slot_id: u8,
    count: u8,
    stack_id: VarInt,
}

fn record_update(
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
