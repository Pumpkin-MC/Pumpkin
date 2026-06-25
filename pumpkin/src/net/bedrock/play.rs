use std::{
    num::{NonZero, NonZeroI32},
    sync::{Arc, atomic::Ordering},
};

use pumpkin_data::{data_component_impl::EquipmentSlot, item_stack::ItemStack};
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler};
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::bedrock::{
    client::inventory_content::CInventoryContent,
    network_item::{
        ContainerName, FullContainerName, NetworkItemDescriptor, NetworkItemStackDescriptor,
    },
};
use pumpkin_protocol::{
    bedrock::{
        client::{
            chunk_radius_update::CChunkRadiusUpdate, container_open::CContainerOpen,
            player_hotbar::CPlayerHotbar,
        },
        server::{
            animate::{AnimateAction, SAnimate},
            block_pick_request::SBlockPickRequest,
            command_request::SCommandRequest,
            container_close::SContainerClose,
            emote::SEmote,
            interaction::{Action, SInteraction},
            inventory_transaction::{SInventoryTransaction, TransactionData},
            mob_equipment::SMobEquipment,
            player_action::{Action as PlayerAction, SPlayerAction},
            player_auth_input::{InputData, SPlayerAuthInput},
            request_chunk_radius::SRequestChunkRadius,
            set_local_player_as_initialized::SSetLocalPlayerAsInitialized,
            text::SText,
        },
    },
    codec::{var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong},
    java::client::play::{Animation, CEntityAnimation, CSetSelectedSlot, CSystemChatMessage},
};
use pumpkin_util::{GameMode, math::position::BlockPos, text::TextComponent};

use pumpkin_world::inventory::Inventory;
use pumpkin_world::world::BlockFlags;

use crate::{
    block::{BlockHitResult, registry::BlockActionResult},
    entity::{EntityBase, player::Player},
    net::{DisconnectReason, bedrock::BedrockClient},
    plugin::player::{
        item_held::PlayerItemHeldEvent, player_chat::PlayerChatEvent,
        player_command_send::PlayerCommandSendEvent,
        player_toggle_flight_event::PlayerToggleFlightEvent,
    },
    server::{Server, seasonal_events},
    world::chunker::{self},
};
use pumpkin_data::BlockDirection;
use tracing::{debug, info};

fn descriptor_to_stack(desc: &NetworkItemDescriptor) -> ItemStack {
    if desc.id.0 == 0 || desc.stack_size == 0 {
        ItemStack::EMPTY.clone()
    } else if let Some(mapping) = pumpkin_data::item::JavaToBedrockItemMapping::from_bedrock(
        desc.id.0 as i16,
        desc.aux_value.0,
    ) {
        ItemStack::new(desc.stack_size as u8, mapping.java_item)
    } else {
        tracing::warn!(
            "Failed to map bedrock item id {} and data {} to Java item",
            desc.id.0,
            desc.aux_value.0
        );
        ItemStack::EMPTY.clone()
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
        let server = player.world().server.upgrade().unwrap();

        let view_distance =
            chunk_radius.clamp(2, NonZeroI32::from(server.basic_config.view_distance).get());

        self.enqueue_packet(&CChunkRadiusUpdate {
            chunk_radius: VarInt(view_distance),
        })
        .await;

        let old_view_distance = {
            let current_config = player.config.load();
            let old_vd = current_config.view_distance;
            let mut new_config = (**current_config).clone();

            new_config.view_distance =
                NonZero::new(view_distance as u8).expect("View distance must be > 0");
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
        server: &Server,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        let entity = player.get_entity();

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
            let on_ground = entity.on_ground.load(std::sync::atomic::Ordering::Relaxed);

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
            let mut abilities = player.abilities.lock().await;
            if !abilities.flying {
                send_cancellable! {{
                    server;
                    PlayerToggleFlightEvent::new(player.clone(), true);
                    'after: {
                        abilities.flying = true;
                        player.send_abilities_update().await;
                    }
                    'cancelled: {
                        player.send_abilities_update().await;
                    }
                }}
            }
        } else if input_data.get(InputData::StopFlying as usize) {
            let mut abilities = player.abilities.lock().await;
            if abilities.flying {
                send_cancellable! {{
                    server;
                    PlayerToggleFlightEvent::new(player.clone(), false);
                    'after: {
                        abilities.flying = false;
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
        let action = PlayerAction::try_from(packet.action.0).unwrap();
        self.handle_player_action(
            player,
            server,
            SPlayerAction {
                runtime_id: VarInt(0), // Unused
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
            AnimateAction::SwingArm => Some(Animation::SwingMainArm),
            AnimateAction::WakeUp => Some(Animation::LeaveBed),
            AnimateAction::CriticalHit => Some(Animation::CriticalEffect),
            AnimateAction::MagicCriticalHit => Some(Animation::MagicCriticaleffect),
            AnimateAction::StopSleep => None, // TODO
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

    pub async fn handle_emote(&self, player: &Arc<Player>, _server: &Server, packet: SEmote) {
        if !player.has_client_loaded() {
            return;
        }

        let entity = &player.living_entity.entity;
        let world = entity.world.load();

        let mut broadcast_packet = packet;
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

    // pub fn handle_emote_list(
    //     &self,
    //     player: &Arc<Player>,
    //     _server: &Server,
    //     packet: &SEmoteList,
    // ) {
    //     debug!(
    //         "Player {} sent emote list: {:?}",
    //         player.gameprofile.name, packet.emote_pieces
    //     );
    // }

    #[allow(clippy::too_many_lines, clippy::collapsible_if, clippy::unreachable)]
    pub async fn handle_inventory_action(
        &self,
        player: &Arc<Player>,
        packet: SInventoryTransaction,
    ) {
        let mut inventory_updated = false;

        for action in &packet.actions {
            if let Some(window_id) = action.window_id {
                if let Some(screen_slot) =
                    map_bedrock_slot_to_screen_handler(window_id, action.inventory_slot)
                {
                    let item_stack = descriptor_to_stack(&action.new_item);

                    let mut player_screen_handler = player.player_screen_handler.lock().await;

                    let is_armor_equipped = player_screen_handler
                        .get_slot(screen_slot)
                        .get_stack()
                        .await
                        .lock()
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
                let server = world.server.upgrade().expect("Server is gone");

                if player.gamemode.load() == GameMode::Spectator {
                    // TODO: openMenu ?
                    return;
                }

                if data.action_type.0 == 0 {
                    // Click block
                    let held_item = player.inventory.held_item();

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
                            &held_item,
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
                }
            }
            TransactionData::UseItemOnEntity(data) => {
                let target_runtime_id = data.target_entity_runtime_id.0 as i32;
                // TODO: replace with consts, i'm too lazy
                match data.action_type.0 {
                    // Interact
                    0 => {
                        let world = player.world();
                        if let Some(target) = world.get_entity_by_id(target_runtime_id) {
                            let held = player.inventory.held_item();
                            let mut stack = held.lock().await;
                            if !target.interact(player, &mut stack).await {
                                let server = world.server.upgrade().expect("Server is gone");
                                server
                                    .item_registry
                                    .use_on_entity(&mut stack, player, target)
                                    .await;
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
                    let server = player.world().server.upgrade().expect("Server is gone");
                    server.item_registry.on_stopped_using(&stack, player).await;
                }
                player.living_entity.clear_active_hand().await;
            }
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

    pub async fn handle_chat_message(&self, server: &Server, player: &Arc<Player>, packet: SText) {
        let gameprofile = &player.gameprofile;

        send_cancellable! {{
            server;
            PlayerChatEvent::new(player.clone(), packet.message, vec![]);

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

                    let be_packet = SText::new(
                        message, gameprofile.name.clone()
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
        if !player.has_client_loaded() {
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
                        let broken_state = world.get_block_state(&location);
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
                        }
                    } else {
                        player.mining.store(true, Ordering::Relaxed);
                        *player.mining_pos.lock().await = location;
                        let progress = (speed * 10.0) as i32;
                        world.set_block_breaking(entity, location, progress).await;
                        player
                            .current_block_destroy_stage
                            .store(progress, Ordering::Relaxed);
                    }
                }
            }
            PlayerAction::PredictDestroyBlock | PlayerAction::StopBreak => {
                let location = packet.block_pos;
                if !player.can_interact_with_block_at(&location, 1.0) {
                    return;
                }

                let entity = &player.get_entity();
                let world = entity.world.load_full();

                player.mining.store(false, Ordering::Relaxed);
                world.set_block_breaking(entity, location, -1).await;

                let (block, state) = world.get_block_and_state(&location);
                if player.gamemode.load() != GameMode::Creative {
                    let block_drop = player.can_harvest(state, block).await;

                    let new_state = world
                        .break_block(
                            &location,
                            Some(player.clone()),
                            if block_drop {
                                BlockFlags::NOTIFY_NEIGHBORS
                            } else {
                                BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_NEIGHBORS
                            },
                        )
                        .await;
                    if new_state.is_some() {
                        server
                            .block_registry
                            .broken(&world, block, player, &location, server, state)
                            .await;
                        player.apply_tool_damage_for_block_break(state).await;
                    }
                }
            }
            PlayerAction::CrackBreak => {
                // Don't do anything for this action. It is no longer used. Block
                // cracking is done fully server-side.
            }
            PlayerAction::AbortBreak => {
                let location = packet.block_pos;
                let entity = &player.get_entity();
                let world = entity.world.load();

                player.mining.store(false, Ordering::Relaxed);
                world.set_block_breaking(entity, location, -1).await;
            }
            // TODO
            _ => {}
        }
    }

    pub async fn handle_chat_command(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: SCommandRequest,
    ) {
        let player_clone = player.clone();
        let server_clone = server.clone();
        let command = packet.command.strip_prefix("/").unwrap_or(&packet.command);

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
        server: &Server,
        packet: pumpkin_protocol::bedrock::server::modal_form_response::SModalFormResponse,
    ) {
        let event = crate::plugin::api::events::player::bedrock_form_response::BedrockFormResponseEvent::new(
            player.clone(),
            packet.form_id.0 as u32,
            packet.form_data,
        );
        let _ = server.plugin_manager.fire(event).await;
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
                    let target_stack = player.inventory.main_inventory[target_hotbar_slot]
                        .lock()
                        .await
                        .clone();
                    let source_stack = player.inventory.main_inventory[slot_with_stack as usize]
                        .lock()
                        .await
                        .clone();
                    player
                        .inventory
                        .set_stack(target_hotbar_slot, source_stack)
                        .await;
                    player
                        .inventory
                        .set_stack(slot_with_stack as usize, target_stack)
                        .await;
                }
            } else {
                let target_stack = player.inventory.main_inventory[target_hotbar_slot]
                    .lock()
                    .await
                    .clone();
                let source_stack = player.inventory.main_inventory[slot_with_stack as usize]
                    .lock()
                    .await
                    .clone();
                player
                    .inventory
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
        let stack_in_hand = player.inventory().held_item().lock().await.clone();
        let equipment = &[(EquipmentSlot::MAIN_HAND, stack_in_hand)];
        player.living_entity.send_equipment_changes(equipment);

        // Sync bedrock inventory updates
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

    pub async fn handle_mob_equipment(&self, player: &Arc<Player>, packet: SMobEquipment) {
        player.update_last_action_time();
        let slot = packet.hotbar_slot;
        if slot >= 9 {
            return;
        }
        let previous_slot = player.inventory.get_selected_slot();
        if let Some(server) = player.world().server.upgrade() {
            let event = PlayerItemHeldEvent::new(player.clone(), previous_slot, slot);
            let event = server.plugin_manager.fire(event).await;
            if event.cancelled {
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
        let stack = inv.held_item().lock().await.clone();
        let equipment = &[(EquipmentSlot::MAIN_HAND, stack)];
        player.living_entity.send_equipment_changes(equipment);
    }
}

#[allow(clippy::too_many_lines)]
fn map_bedrock_container_slot(
    screen_handler: &dyn ScreenHandler,
    container_name: ContainerName,
    slot_id: u8,
) -> Option<usize> {
    let container_slots = screen_handler.get_behaviour().container_slots;
    let is_player_screen = screen_handler.window_type().is_none();

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
                (slot_id < 4).then(|| 1 + slot_id as usize)
            } else if screen_handler.window_type()
                == Some(pumpkin_data::screen::WindowType::Crafting)
            {
                (slot_id < 9).then(|| 1 + slot_id as usize)
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
    count: u8,
    stack_id: VarInt,
) {
    let final_stack_id = if count == 0 { VarInt(0) } else { stack_id };
    if let Some(existing) = updates
        .iter_mut()
        .find(|u| u.container_name == container_name && u.slot_id == slot_id)
    {
        existing.count = count;
        existing.stack_id = final_stack_id;
    } else {
        updates.push(SlotUpdate {
            container_name,
            slot_id,
            count,
            stack_id: final_stack_id,
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
        let cursor_lock = screen_handler.get_behaviour().cursor_stack.lock().await;
        return cursor_lock.clone();
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
