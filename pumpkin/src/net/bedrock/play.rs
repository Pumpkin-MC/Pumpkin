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
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler};
use pumpkin_inventory::slot::Slot;
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
use pumpkin_util::{GameMode, Hand, math::position::BlockPos, text::TextComponent};

use pumpkin_world::inventory::Inventory;
use pumpkin_world::world::BlockFlags;

use crate::{
    block::{BlockHitResult, registry::BlockActionResult},
    entity::{EntityBase, player::Player},
    net::{DisconnectReason, bedrock::BedrockClient},
    plugin::player::{
        item_held::PlayerItemHeldEvent,
        player_chat::PlayerChatEvent,
        player_command_send::PlayerCommandSendEvent,
        player_interact_event::{InteractAction, PlayerInteractEvent},
        player_toggle_flight_event::PlayerToggleFlightEvent,
    },
    server::{Server, seasonal_events},
    world::chunker::{self},
};
use pumpkin_data::BlockDirection;
use tracing::{debug, info};

fn descriptor_to_stack(desc: &NetworkItemDescriptor, is_creative: bool) -> ItemStack {
    if desc.id.0 == 0 || desc.stack_size == 0 {
        ItemStack::EMPTY.clone()
    } else {
        let mut mapped_item = None;

        if is_creative {
            let index = (desc.id.0.saturating_sub(1)) as usize;
            if index < pumpkin_data::bedrock_creative::CREATIVE_ENTRIES.len() {
                let entry = pumpkin_data::bedrock_creative::CREATIVE_ENTRIES[index];
                if let Some(mapping) = pumpkin_data::item::JavaToBedrockItemMapping::from_bedrock(
                    entry.item_id,
                    entry.item_aux_value,
                ) {
                    mapped_item = Some(mapping.java_item);
                }
            }
        }

        if mapped_item.is_none()
            && let Some(mapping) = pumpkin_data::item::JavaToBedrockItemMapping::from_bedrock(
                desc.id.0 as i16,
                desc.aux_value.0,
            )
        {
            mapped_item = Some(mapping.java_item);
        }

        mapped_item.map_or_else(
            || {
                tracing::warn!(
                    "Failed to map bedrock item id {} and data {} to Java item",
                    desc.id.0,
                    desc.aux_value.0
                );
                ItemStack::EMPTY.clone()
            },
            |item| ItemStack::new(desc.stack_size as u8, item),
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
        let server = player.world().server.upgrade().unwrap();

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
            let flying = { player.abilities.lock().await.flying };
            // Reject free survival flight: only fly when the server granted may_fly.
            if !flying && player.abilities.lock().await.allow_flying {
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
            // Invalid action ids used to unwrap → panic hook shutdown.
            tracing::debug!(
                "Ignoring invalid bedrock block action {} from {}",
                packet.action.0,
                player.gameprofile.name
            );
            return;
        };
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

    pub async fn handle_emote(&self, player: &Arc<Player>, _server: &Server, packet: SEmote<'_>) {
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
        tracing::debug!("handle_inventory_action from {}", player.gameprofile.name);
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
                                0,
                                VarInt(0),
                            );
                            inventory_updated = true;
                        }
                    }
                }
            }
            player_screen_handler.send_content_updates().await;
        }

        let is_creative = player.gamemode.load() == GameMode::Creative;
        for action in &packet.actions {
            use pumpkin_protocol::bedrock::server::inventory_transaction::InventoryActionSource;
            let source_type = InventoryActionSource::from(action.source_type);
            if source_type == InventoryActionSource::World {
                // Only creative may spawn world-source drops from client descriptors.
                if !is_creative {
                    continue;
                }
                let old_stack = descriptor_to_stack(&action.old_item, is_creative);
                let new_stack = descriptor_to_stack(&action.new_item, is_creative);
                if old_stack.is_empty() && !new_stack.is_empty() {
                    player.drop_item(new_stack).await;
                }
            } else if let Some(window_id) = action.window_id {
                if let Some(screen_slot) =
                    map_bedrock_slot_to_screen_handler(window_id, action.inventory_slot)
                {
                    // Survival: never write arbitrary client item descriptors into slots.
                    // Creative may set slots from the creative catalog.
                    if !is_creative {
                        continue;
                    }
                    let item_stack = descriptor_to_stack(&action.new_item, is_creative);

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
                    // Click block. Never trust client item_in_hand to spawn items
                    // in survival — only creative may adopt the client descriptor.
                    let is_creative = player.gamemode.load() == GameMode::Creative;
                    let held_item = player.inventory.held_item();
                    if is_creative {
                        let client_stack = descriptor_to_stack(&data.item_in_hand, true);
                        if !client_stack.is_empty() {
                            let mut server_stack = held_item.lock().await;
                            if server_stack.is_empty()
                                || server_stack.item.id != client_stack.item.id
                            {
                                *server_stack = client_stack;
                            }
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

                    let mut stack = held_item.lock().await;
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
                    }
                } else if data.action_type.0 == 1 {
                    // Click air / Use item. Same rule as click-block: never adopt
                    // client item_in_hand in survival.
                    let is_creative = player.gamemode.load() == GameMode::Creative;
                    let held_item = player.inventory.held_item();
                    if is_creative {
                        let client_stack = descriptor_to_stack(&data.item_in_hand, true);
                        if !client_stack.is_empty() {
                            let mut server_stack = held_item.lock().await;
                            if server_stack.is_empty()
                                || server_stack.item.id != client_stack.item.id
                            {
                                *server_stack = client_stack;
                            }
                        }
                    }

                    let event = PlayerInteractEvent::new(
                        player,
                        InteractAction::RightClickAir,
                        &pumpkin_data::Block::AIR,
                        None,
                    );

                    let stack_for_use = held_item.lock().await.clone();

                    {
                        let mut held = held_item.lock().await;
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
                                if !inventory
                                    .is_already_equipped(&held_item, equippable.slot)
                                    .await
                                {
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
                        }
                    }

                    send_cancellable! {{
                        server;
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

    pub async fn handle_chat_message(
        &self,
        server: &Server,
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
            PlayerAction::DropItem => {
                player.drop_held_item(false).await;
            }
            // TODO
            _ => {}
        }
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
        server: &Server,
        packet: pumpkin_protocol::bedrock::server::modal_form_response::SModalFormResponse<'_>,
    ) {
        let event = crate::plugin::api::events::player::bedrock_form_response::BedrockFormResponseEvent::new(
            player.clone(),
            packet.form_id.0 as u32,
            packet.form_data.map(std::borrow::Cow::into_owned),
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

        const MAX_CRAFT_REPETITIONS: u8 = 64;

        let current_screen_handler = player.current_screen_handler.lock().await.clone();
        let mut screen_handler = current_screen_handler.lock().await;

        let mut responses = Vec::with_capacity(packet.requests.len());

        for request in packet.requests {
            let mut created_item: Option<ItemStack> = None;
            let mut updates = Vec::new();
            let mut result = 0u8; // 0 = Success, 1 = Error

            for action in request.actions {
                tracing::debug!("Processing ItemStackRequestAction");
                match action {
                    ItemStackRequestAction::CraftCreative {
                        creative_item_id,
                        repetitions,
                    } => {
                        // Creative catalog is creative-only. Survival must not spawn free items.
                        if player.gamemode.load() != GameMode::Creative {
                            result = 1;
                            break;
                        }
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
                        if source.container_name.container_name == ContainerName::CreatedOutput
                            && created_item.is_none()
                        {
                            tracing::debug!("CreatedOutput move without tracked craft");
                            result = 1;
                            break;
                        }
                        // A Cursor source backed by the created-item mirror spends the
                        // mirror; its remainder must not materialize in the physical cursor.
                        let source_uses_mirror =
                            is_cursor_mirror(&*screen_handler, &source, created_item.as_ref())
                                .await;
                        let mut source_stack =
                            get_slot_stack(&*screen_handler, &source, created_item.as_ref()).await;
                        if source_stack.is_empty() {
                            tracing::debug!("Source stack is empty in Take/Place");
                            result = 1;
                            break;
                        }
                        let count = count.min(source_stack.item_count);
                        if count > 0 {
                            // Destinations resolve to physical state only; the
                            // created-item mirror is never a merge base.
                            let mut dest_stack =
                                get_destination_slot_stack(&*screen_handler, &destination).await;
                            // Take-only slots (result/output) never accept moved-in stacks.
                            if !destination_accepts(&*screen_handler, &destination, &source_stack)
                                .await
                            {
                                tracing::debug!("Take/Place destination is take-only");
                                result = 1;
                                break;
                            }
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

                            let taken_stack = source_stack.copy_with_count(count);
                            // A take from a take-only slot is settled through the
                            // slot itself exactly once (grid use, trade cost, XP).
                            // Settle before consuming anything so a refused take
                            // leaves both the craft mirror and the grid untouched.
                            if !charge_take_only_source(
                                player.as_ref(),
                                &*screen_handler,
                                &source,
                                &taken_stack,
                            )
                            .await
                            {
                                tracing::debug!("Take/Place source slot refuses the take");
                                result = 1;
                                break;
                            }
                            source_stack.decrement(count);
                            consume_created_output_source(
                                source.container_name.container_name,
                                count,
                                &mut created_item,
                                &*screen_handler,
                            )
                            .await;
                            let source_stack = if source_stack.is_empty() || source_uses_mirror {
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
                        if (slot1.container_name.container_name == ContainerName::CreatedOutput
                            || slot2.container_name.container_name == ContainerName::CreatedOutput)
                            && created_item.is_none()
                        {
                            result = 1;
                            break;
                        }
                        let stack1 =
                            get_slot_stack(&*screen_handler, &slot1, created_item.as_ref()).await;
                        let stack2 =
                            get_slot_stack(&*screen_handler, &slot2, created_item.as_ref()).await;

                        // Take-only slots (result/output) never join a swap: the
                        // incoming half would be a free write into the result.
                        if !destination_accepts(&*screen_handler, &slot1, &stack2).await
                            || !destination_accepts(&*screen_handler, &slot2, &stack1).await
                        {
                            result = 1;
                            break;
                        }

                        // Mirror flags must be captured before any consumption.
                        let slot1_uses_mirror =
                            is_cursor_mirror(&*screen_handler, &slot1, created_item.as_ref()).await;
                        let slot2_uses_mirror =
                            is_cursor_mirror(&*screen_handler, &slot2, created_item.as_ref()).await;

                        // Moving CreatedOutput into inventory must spend the craft.
                        if slot1.container_name.container_name == ContainerName::CreatedOutput {
                            consume_created_item(&mut created_item, stack1.item_count);
                        }
                        if slot2.container_name.container_name == ContainerName::CreatedOutput {
                            consume_created_item(&mut created_item, stack2.item_count);
                        }
                        // A Cursor side backed by the created-item mirror spends the
                        // mirror exactly like a CreatedOutput source.
                        if slot1_uses_mirror {
                            consume_created_item(&mut created_item, stack1.item_count);
                        }
                        if slot2_uses_mirror {
                            consume_created_item(&mut created_item, stack2.item_count);
                        }

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
                        if source.container_name.container_name == ContainerName::CreatedOutput
                            && created_item.is_none()
                        {
                            result = 1;
                            break;
                        }
                        let source_uses_mirror =
                            is_cursor_mirror(&*screen_handler, &source, created_item.as_ref())
                                .await;
                        let mut source_stack =
                            get_slot_stack(&*screen_handler, &source, created_item.as_ref()).await;
                        if source_stack.is_empty() {
                            result = 1;
                            break;
                        }
                        let count = count.min(source_stack.item_count);
                        if count > 0 {
                            let dropped_stack = source_stack.copy_with_count(count);
                            // A take from a take-only slot is settled through the
                            // slot itself exactly once (grid use, trade cost, XP),
                            // and only a settled take may be dropped.
                            if !charge_take_only_source(
                                player.as_ref(),
                                &*screen_handler,
                                &source,
                                &dropped_stack,
                            )
                            .await
                            {
                                result = 1;
                                break;
                            }
                            player.drop_item(dropped_stack).await;

                            source_stack.decrement(count);
                            consume_created_output_source(
                                source.container_name.container_name,
                                count,
                                &mut created_item,
                                &*screen_handler,
                            )
                            .await;
                            let source_stack = if source_stack.is_empty() || source_uses_mirror {
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
                        if source.container_name.container_name == ContainerName::CreatedOutput
                            && created_item.is_none()
                        {
                            result = 1;
                            break;
                        }
                        let source_uses_mirror =
                            is_cursor_mirror(&*screen_handler, &source, created_item.as_ref())
                                .await;
                        let mut source_stack =
                            get_slot_stack(&*screen_handler, &source, created_item.as_ref()).await;
                        if source_stack.is_empty() {
                            result = 1;
                            break;
                        }
                        let count = count.min(source_stack.item_count);
                        if count > 0 {
                            let consumed_stack = source_stack.copy_with_count(count);
                            // A take from a take-only slot is settled through the
                            // slot itself exactly once (grid use, trade cost, XP).
                            if !charge_take_only_source(
                                player.as_ref(),
                                &*screen_handler,
                                &source,
                                &consumed_stack,
                            )
                            .await
                            {
                                result = 1;
                                break;
                            }
                            source_stack.decrement(count);
                            consume_created_output_source(
                                source.container_name.container_name,
                                count,
                                &mut created_item,
                                &*screen_handler,
                            )
                            .await;
                            let source_stack = if source_stack.is_empty() || source_uses_mirror {
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
                        // Only 2×2 player craft and 3×3 crafting table. Any other
                        // open window would treat slots[0] as "output" and mint
                        // free stacks from chest/etc contents.
                        let window_type = screen_handler.window_type();
                        if !window_allows_craft_recipe(window_type) {
                            tracing::debug!(
                                "Reject CraftRecipe on non-crafting window {:?}",
                                window_type
                            );
                            result = 1;
                            break;
                        }

                        if repetitions > 0 {
                            let reps = repetitions.min(MAX_CRAFT_REPETITIONS);
                            screen_handler.update_to_client().await;

                            let is_player = window_type.is_none();
                            let grid_size = if is_player { 4 } else { 9 };
                            let slots = &screen_handler.get_behaviour().slots;
                            // Output + full grid must exist.
                            if slots.len() <= grid_size {
                                result = 1;
                                break;
                            }

                            let output_slot = slots[0].clone();
                            // Refuse if slot 0 accepts inserts (not a craft output).
                            if Slot::can_insert(output_slot.as_ref(), ItemStack::EMPTY).await {
                                tracing::debug!("CraftRecipe output slot is not take-only");
                                result = 1;
                                break;
                            }

                            let output_stack = output_slot.get_cloned_stack().await;
                            if output_stack.is_empty() {
                                tracing::warn!("Client tried to craft, but output slot is empty!");
                                result = 1;
                                break;
                            }

                            let mut crafted_count: u16 = 0;
                            let expected_id = output_stack.item.id;
                            for _ in 0..reps {
                                let current = output_slot.get_cloned_stack().await;
                                if current.is_empty() || current.item.id != expected_id {
                                    break;
                                }
                                let batch = u16::from(current.item_count);
                                output_slot.on_take_item(player.as_ref(), &current).await;
                                crafted_count = crafted_count.saturating_add(batch);
                            }
                            if crafted_count == 0 {
                                result = 1;
                                break;
                            }
                            let mut total_crafted = output_stack.clone();
                            total_crafted.item_count = crafted_count.min(u16::from(u8::MAX)) as u8;
                            created_item = Some(total_crafted);

                            for i in 0..grid_size {
                                let grid_slot_index = 1 + i;
                                let grid_slot = slots[grid_slot_index].clone();
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

/// Only the 2×2 player craft and 3×3 crafting table may run `CraftRecipe`.
/// Any other open window would treat slots[0] as "output" and mint free
/// stacks from chest/etc contents.
const fn window_allows_craft_recipe(window_type: Option<pumpkin_data::screen::WindowType>) -> bool {
    matches!(
        window_type,
        None | Some(pumpkin_data::screen::WindowType::Crafting)
    )
}

/// True when a `Cursor` reference is served by the created-item mirror: the
/// physical cursor is empty while a crafted stack is still tracked. Mirror
/// reads are spent from `created_item`, and the remainder must never be
/// written back into the physical cursor — it lives in exactly one place.
async fn is_cursor_mirror(
    screen_handler: &dyn ScreenHandler,
    slot_info: &pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestSlotInfo,
    created_item: Option<&ItemStack>,
) -> bool {
    slot_info.container_name.container_name == ContainerName::Cursor
        && created_item.is_some()
        && screen_handler
            .get_behaviour()
            .cursor_stack
            .lock()
            .await
            .is_empty()
}

/// Destination resolution for moves. The created-item mirror is a virtual
/// craft remainder, never a merge base, so destinations read physical
/// state only.
async fn get_destination_slot_stack(
    screen_handler: &dyn ScreenHandler,
    slot_info: &pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestSlotInfo,
) -> ItemStack {
    get_slot_stack(screen_handler, slot_info, None).await
}

/// Whether a move destination may receive the given stack. Virtual or
/// unmapped containers accept the write-back as a no-op; a resolved slot
/// that refuses inserts (result/output slots) rejects the move so results
/// can never be written into for free.
async fn destination_accepts(
    screen_handler: &dyn ScreenHandler,
    slot_info: &pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestSlotInfo,
    stack: &ItemStack,
) -> bool {
    if let Some(screen_slot) = map_bedrock_container_slot(
        screen_handler,
        slot_info.container_name.container_name,
        slot_info.slot_id,
    ) {
        return screen_handler.get_behaviour().slots[screen_slot]
            .can_insert(stack)
            .await;
    }
    true
}

/// Settles a take from a resolved slot through the slot itself, exactly once
/// per verified take. A slot that refuses re-insertion of the stack taken
/// from it is take-only (result/output); its `on_take_item` performs the
/// consumption/charging (crafting grid use, trade cost, furnace XP).
/// Ordinary inventory slots accept the stack back and need no settlement.
/// Returns false when the slot currently refuses the take.
async fn charge_take_only_source(
    player: &dyn InventoryPlayer,
    screen_handler: &dyn ScreenHandler,
    slot_info: &pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestSlotInfo,
    taken_stack: &ItemStack,
) -> bool {
    let Some(screen_slot) = map_bedrock_container_slot(
        screen_handler,
        slot_info.container_name.container_name,
        slot_info.slot_id,
    ) else {
        return true;
    };
    let slot = &screen_handler.get_behaviour().slots[screen_slot];
    if slot.can_insert(taken_stack).await {
        return true;
    }
    if !slot.can_take_items(player).await {
        return false;
    }
    slot.on_take_item(player, taken_stack).await;
    true
}

fn consume_created_item(created_item: &mut Option<ItemStack>, count: u8) {
    if let Some(stack) = created_item.as_mut() {
        stack.decrement(count);
        if stack.is_empty() {
            *created_item = None;
        }
    }
}

async fn consume_created_output_source(
    source_name: ContainerName,
    count: u8,
    created_item: &mut Option<ItemStack>,
    screen_handler: &dyn ScreenHandler,
) {
    if source_name == ContainerName::CreatedOutput {
        consume_created_item(created_item, count);
    } else if source_name == ContainerName::Cursor {
        let cursor_is_empty = screen_handler
            .get_behaviour()
            .cursor_stack
            .lock()
            .await
            .is_empty();
        if cursor_is_empty {
            consume_created_item(created_item, count);
        }
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
    let slot_count = screen_handler.get_behaviour().slots.len();

    // Every mapped index is range-checked. No arm may return a raw usize.
    let in_range = |idx: usize| (idx < slot_count).then_some(idx);

    let raw: Option<usize> = match container_name {
        ContainerName::HotBar => {
            if slot_id >= 9 {
                None
            } else if is_player_screen {
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
        ContainerName::Armor => (slot_id < 4).then_some(5 + slot_id as usize),
        // Player screen only; anvil/merchant etc. have <46 slots.
        ContainerName::Offhand => (slot_id == 0 && is_player_screen).then_some(45),
        // Cursor and virtual craft output — never real screen indices.
        ContainerName::Cursor | ContainerName::CreatedOutput => None,
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
        // Previews resolve to the real result slot; takes from them are
        // settled through the slot's on_take_item, never served free.
        ContainerName::CraftingOutputPreview => (is_player_screen
            || screen_handler.window_type() == Some(pumpkin_data::screen::WindowType::Crafting))
        .then_some(0),
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
    };
    raw.and_then(in_range)
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
    let name = slot_info.container_name.container_name;
    // Virtual craft output only.
    if name == ContainerName::CreatedOutput {
        return created_item
            .cloned()
            .unwrap_or_else(|| ItemStack::EMPTY.clone());
    }
    // Cursor reads fall back to the created-item mirror when the physical
    // cursor is empty. Callers spending a mirror read must consume it (see
    // is_cursor_mirror); destination reads must use get_destination_slot_stack.
    if name == ContainerName::Cursor {
        let cursor_lock = screen_handler.get_behaviour().cursor_stack.lock().await;
        if cursor_lock.is_empty()
            && let Some(stack) = created_item
        {
            return stack.clone();
        }
        return cursor_lock.clone();
    }
    // Result previews resolve to the real result slot; takes from such a
    // slot are settled through charge_take_only_source, never served free.
    if let Some(screen_slot) = map_bedrock_container_slot(screen_handler, name, slot_info.slot_id) {
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
mod hardening_tests {
    //! Exploit-sequence tests for Bedrock item-move authority. A full packet
    //! round-trip needs a live `Server`/`Player`, so each test composes the
    //! exact helpers the request arms use, in the same order; `update_slot_stack`'s
    //! Cursor branch is a plain cursor write and is mirrored directly. Every
    //! sequence is traced from the reviews' exploit reports.
    use super::*;
    use pumpkin_data::item::Item;
    use pumpkin_data::screen::WindowType;
    use pumpkin_data::statistic::StatisticCategory;
    use pumpkin_inventory::crafting::crafting_inventory::CraftingInventory;
    use pumpkin_inventory::crafting::crafting_screen_handler::ResultSlot;
    use pumpkin_inventory::crafting::recipes::RecipeInputInventory;
    use pumpkin_inventory::entity_equipment::EntityEquipment;
    use pumpkin_inventory::player::player_inventory::PlayerInventory;
    use pumpkin_inventory::screen_handler::{
        ItemStackFuture, PlayerFuture, ScreenHandlerBehaviour,
    };
    use pumpkin_inventory::slot::{BoxFuture, NormalSlot, TakeOnlySlot};
    use pumpkin_protocol::bedrock::server::item_stack_request::ItemStackRequestSlotInfo;
    use pumpkin_protocol::java::client::play::{
        CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
        CSetPlayerInventory,
    };
    use pumpkin_world::inventory::SimpleInventory;
    use std::any::Any;
    use std::collections::HashMap;
    use std::sync::atomic::AtomicU8;
    use tokio::sync::Mutex;

    struct MockScreenHandler {
        behaviour: ScreenHandlerBehaviour,
    }

    impl MockScreenHandler {
        fn new(window_type: Option<WindowType>, container_slots: usize) -> Self {
            let mut behaviour = ScreenHandlerBehaviour::new(1, window_type);
            behaviour.container_slots = container_slots;
            Self { behaviour }
        }
    }

    impl ScreenHandler for MockScreenHandler {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
            &self.behaviour
        }
        fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
            &mut self.behaviour
        }
        fn quick_move<'a>(
            &'a mut self,
            _player: &'a dyn InventoryPlayer,
            _slot_index: i32,
        ) -> ItemStackFuture<'a> {
            Box::pin(async { ItemStack::EMPTY.clone() })
        }
    }

    struct MockInventoryPlayer {
        inventory: Arc<PlayerInventory>,
    }

    impl MockInventoryPlayer {
        fn new() -> Self {
            Self {
                inventory: Arc::new(PlayerInventory::new(
                    Arc::new(Mutex::new(EntityEquipment::new())),
                    Arc::new(HashMap::new()),
                )),
            }
        }
    }

    impl InventoryPlayer for MockInventoryPlayer {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn drop_item(&self, _item: ItemStack, _retain_ownership: bool) -> PlayerFuture<'_, ()> {
            Box::pin(async {})
        }
        fn get_inventory(&self) -> Arc<PlayerInventory> {
            self.inventory.clone()
        }
        fn has_infinite_materials(&self) -> bool {
            false
        }
        fn is_creative(&self) -> bool {
            false
        }
        fn experience_level(&self) -> i32 {
            0
        }
        fn add_experience_levels(&self, _levels: i32) -> PlayerFuture<'_, ()> {
            Box::pin(async {})
        }
        fn enchantment_seed(&self) -> i32 {
            0
        }
        fn set_enchantment_seed(&self, _seed: i32) -> PlayerFuture<'_, ()> {
            Box::pin(async {})
        }
        fn enqueue_inventory_packet<'a>(
            &'a self,
            _packet: &'a CSetContainerContent,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }
        fn enqueue_slot_packet<'a>(
            &'a self,
            _packet: &'a CSetContainerSlot,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }
        fn enqueue_cursor_packet<'a>(
            &'a self,
            _packet: &'a CSetCursorItem,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }
        fn enqueue_property_packet<'a>(
            &'a self,
            _packet: &'a CSetContainerProperty,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }
        fn enqueue_slot_set_packet<'a>(
            &'a self,
            _packet: &'a CSetPlayerInventory,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }
        fn enqueue_set_held_item_packet<'a>(
            &'a self,
            _packet: &'a CSetSelectedSlot,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }
        fn enqueue_equipment_change<'a>(
            &'a self,
            _slot: &'a EquipmentSlot,
            _stack: &'a ItemStack,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }
        fn award_experience(&self, _amount: i32) -> PlayerFuture<'_, ()> {
            Box::pin(async {})
        }
        fn increment_stat(
            &self,
            _category: StatisticCategory,
            _stat_id: i32,
            _amount: i32,
        ) -> PlayerFuture<'_, ()> {
            Box::pin(async {})
        }
    }

    /// Slot that records every `on_take_item` call; `accepts_insert` models
    /// normal vs take-only (result/output) slots.
    struct RecordingSlot {
        inventory: Arc<SimpleInventory>,
        index: usize,
        id: AtomicU8,
        accepts_insert: bool,
        can_take: bool,
        taken: Arc<Mutex<Vec<ItemStack>>>,
    }

    impl RecordingSlot {
        fn new(
            inventory: Arc<SimpleInventory>,
            index: usize,
            accepts_insert: bool,
            can_take: bool,
            taken: Arc<Mutex<Vec<ItemStack>>>,
        ) -> Self {
            Self {
                inventory,
                index,
                id: AtomicU8::new(0),
                accepts_insert,
                can_take,
                taken,
            }
        }
    }

    impl Slot for RecordingSlot {
        fn get_inventory(&self) -> Arc<dyn Inventory> {
            self.inventory.clone()
        }
        fn get_index(&self) -> usize {
            self.index
        }
        fn set_id(&self, index: usize) {
            self.id.store(index as u8, Ordering::Relaxed);
        }
        fn can_insert<'a>(&'a self, _stack: &'a ItemStack) -> BoxFuture<'a, bool> {
            let accepts = self.accepts_insert;
            Box::pin(async move { accepts })
        }
        fn can_take_items(&self, _player: &dyn InventoryPlayer) -> BoxFuture<'_, bool> {
            let can_take = self.can_take;
            Box::pin(async move { can_take })
        }
        fn on_take_item<'a>(
            &'a self,
            _player: &'a dyn InventoryPlayer,
            stack: &'a ItemStack,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                self.taken.lock().await.push(stack.clone());
            })
        }
        fn mark_dirty(&self) -> BoxFuture<'_, ()> {
            Box::pin(async {})
        }
    }

    fn make_slot_info(name: ContainerName, slot_id: u8) -> ItemStackRequestSlotInfo {
        ItemStackRequestSlotInfo {
            container_name: FullContainerName {
                container_name: name,
                dynamic_id: None,
            },
            slot_id,
            stack_id: VarInt(0),
        }
    }

    #[test]
    fn consume_created_item_clears_when_empty() {
        let mut created = Some(ItemStack::new(3, &Item::EMERALD));
        consume_created_item(&mut created, 2);
        assert_eq!(created.as_ref().unwrap().item_count, 1);
        consume_created_item(&mut created, 1);
        assert!(created.is_none());
        // No panic when already none
        consume_created_item(&mut created, 5);
        assert!(created.is_none());
    }

    #[test]
    fn craft_recipe_window_gate_rejects_non_crafting_windows() {
        assert!(window_allows_craft_recipe(None));
        assert!(window_allows_craft_recipe(Some(WindowType::Crafting)));
        assert!(!window_allows_craft_recipe(Some(WindowType::Generic9x3)));
        assert!(!window_allows_craft_recipe(Some(WindowType::Merchant)));
        assert!(!window_allows_craft_recipe(Some(WindowType::Anvil)));
    }

    /// Sequence: [CraftRecipe{1}, Take{src=CreatedOutput, dst=Cursor, count=4}]
    /// on a 4-output craft must leave cursor=4 and created=0, not cursor=8.
    #[tokio::test]
    async fn cursor_mirror_take_to_cursor_does_not_double() {
        let handler = MockScreenHandler::new(None, 0);
        let mut created = Some(ItemStack::new(4, &Item::STICK));
        let source = make_slot_info(ContainerName::CreatedOutput, 0);
        let destination = make_slot_info(ContainerName::Cursor, 0);

        // Arm flow: source read sees the tracked craft...
        let source_stack = get_slot_stack(&handler, &source, created.as_ref()).await;
        assert_eq!(source_stack.item_count, 4);
        // ...but the destination must resolve to physical state only.
        let mut dest_stack = get_destination_slot_stack(&handler, &destination).await;
        assert!(
            dest_stack.is_empty(),
            "created-item mirror must never be a destination merge base"
        );
        dest_stack = source_stack.copy_with_count(4);
        consume_created_output_source(ContainerName::CreatedOutput, 4, &mut created, &handler)
            .await;
        // update_slot_stack's Cursor branch is a plain physical write.
        *handler.get_behaviour().cursor_stack.lock().await = dest_stack;

        assert!(created.is_none(), "craft spent exactly once");
        let cursor = handler.get_behaviour().cursor_stack.lock().await;
        assert_eq!(cursor.item_count, 4, "no 4->8 cursor doubling");
        assert_eq!(cursor.item.id, Item::STICK.id);
    }

    /// Sequence: [CraftRecipe{1}, Swap{slot1=Cursor, slot2=Inventory}] — the
    /// mirror read must spend `created_item` exactly once.
    #[tokio::test]
    async fn cursor_mirror_swap_spends_created_once() {
        let inventory = Arc::new(SimpleInventory::new(36));
        inventory
            .set_stack(9, ItemStack::new(2, &Item::DIAMOND))
            .await;
        let mut handler = MockScreenHandler::new(None, 0);
        for i in 0..10 {
            handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), i)));
        }
        let mut created = Some(ItemStack::new(4, &Item::STICK));
        let slot1 = make_slot_info(ContainerName::Cursor, 0);
        let slot2 = make_slot_info(ContainerName::Inventory, 9);

        let stack1 = get_slot_stack(&handler, &slot1, created.as_ref()).await;
        let stack2 = get_slot_stack(&handler, &slot2, created.as_ref()).await;
        assert_eq!(stack1.item_count, 4, "Cursor source reads the mirror");
        assert_eq!(stack2.item_count, 2);
        assert!(is_cursor_mirror(&handler, &slot1, created.as_ref()).await);
        assert!(!is_cursor_mirror(&handler, &slot2, created.as_ref()).await);

        // Arm flow: a mirror-backed Cursor side spends the craft.
        consume_created_item(&mut created, stack1.item_count);
        *handler.get_behaviour().cursor_stack.lock().await = stack2;
        handler.get_behaviour().slots[9].set_stack(stack1).await;

        assert!(created.is_none(), "mirror consumed exactly once");
        assert_eq!(
            handler.get_behaviour().cursor_stack.lock().await.item_count,
            2
        );
        assert_eq!(
            handler.get_behaviour().slots[9]
                .get_cloned_stack()
                .await
                .item_count,
            4
        );
    }

    /// Sequence: [CraftRecipe{1}, Take{src=Cursor, dst=Inventory, count=2}]
    /// with the physical cursor empty — the remainder must stay tracked in
    /// exactly one place (`created_item`), not both mirror and cursor.
    #[tokio::test]
    async fn cursor_mirror_partial_take_leaves_single_remainder() {
        let handler = MockScreenHandler::new(None, 0);
        let mut created = Some(ItemStack::new(4, &Item::STICK));
        let source = make_slot_info(ContainerName::Cursor, 0);

        assert!(is_cursor_mirror(&handler, &source, created.as_ref()).await);
        let source_stack = get_slot_stack(&handler, &source, created.as_ref()).await;
        assert_eq!(source_stack.item_count, 4, "mirror read while cursor empty");

        // Arm flow: take 2, spend the mirror, write nothing back to the cursor.
        consume_created_output_source(ContainerName::Cursor, 2, &mut created, &handler).await;
        *handler.get_behaviour().cursor_stack.lock().await = ItemStack::EMPTY.clone();

        assert_eq!(
            created.as_ref().unwrap().item_count,
            2,
            "remainder stays in created_item only"
        );
        assert!(
            handler.get_behaviour().cursor_stack.lock().await.is_empty(),
            "remainder must not materialize in the physical cursor"
        );
        // A follow-up Cursor read still resolves through the mirror.
        let next = get_slot_stack(&handler, &source, created.as_ref()).await;
        assert_eq!(next.item_count, 2);
    }

    /// Sequence: Take{src=CraftingOutputPreview, dst=Inventory} on a crafting
    /// table — resolves to the real result slot, consumes the grid, and does
    /// not refill for free.
    #[tokio::test]
    async fn preview_take_consumes_grid_without_free_refill() {
        let player = MockInventoryPlayer::new();
        let crafting_inv = Arc::new(CraftingInventory::new(3, 3));
        // Stick recipe input: two planks in one column.
        crafting_inv
            .set_stack(0, ItemStack::new(1, &Item::OAK_PLANKS))
            .await;
        crafting_inv
            .set_stack(3, ItemStack::new(1, &Item::OAK_PLANKS))
            .await;
        let recipe_inv: Arc<dyn RecipeInputInventory> = crafting_inv.clone();
        let result = Arc::new(Mutex::new(ItemStack::new(4, &Item::STICK)));
        let result_slot = Arc::new(ResultSlot {
            inventory: recipe_inv,
            id: AtomicU8::new(0),
            result: result.clone(),
            recipe_provider: None,
        });
        let mut handler = MockScreenHandler::new(Some(WindowType::Crafting), 10);
        handler.add_slot(result_slot);

        let source = make_slot_info(ContainerName::CraftingOutputPreview, 0);
        // Previews resolve to the real result slot (previously rejected/empty).
        let source_stack = get_slot_stack(&handler, &source, None).await;
        assert_eq!(source_stack.item_count, 4);
        assert_eq!(source_stack.item.id, Item::STICK.id);
        // ...and the result slot refuses moves into it.
        assert!(!destination_accepts(&handler, &source, &source_stack).await);

        // Arm flow: the take is settled through the slot exactly once.
        let taken = source_stack.copy_with_count(4);
        assert!(charge_take_only_source(&player, &handler, &source, &taken).await);

        for i in 0..crafting_inv.size() {
            assert!(
                crafting_inv.get_stack(i).await.lock().await.is_empty(),
                "grid slot {i} must be consumed by the take"
            );
        }
        assert!(
            result.lock().await.is_empty(),
            "result re-derives to empty instead of refilling for free"
        );
    }

    /// Take-only sources are charged via `on_take_item` exactly once; ordinary
    /// slots are not charged; a slot refusing the take rejects the move.
    #[tokio::test]
    async fn take_only_source_charged_exactly_once() {
        let player = MockInventoryPlayer::new();
        let inventory = Arc::new(SimpleInventory::new(3));
        inventory
            .set_stack(2, ItemStack::new(1, &Item::DIAMOND))
            .await;
        let mut handler = MockScreenHandler::new(Some(WindowType::Anvil), 3);
        handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), 0)));
        handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), 1)));
        handler.add_slot(Arc::new(TakeOnlySlot::new(inventory.clone(), 2)));

        // Previews resolve to the real take-only result slot (previously empty).
        let source = make_slot_info(ContainerName::AnvilResultPreview, 0);
        let stack = get_slot_stack(&handler, &source, None).await;
        assert_eq!(
            stack.item.id,
            Item::DIAMOND.id,
            "preview resolves to result"
        );
        assert!(!destination_accepts(&handler, &source, &stack).await);
        assert!(charge_take_only_source(&player, &handler, &source, &stack).await);

        // A take-only slot that allows the take charges exactly once.
        let taken_log = Arc::new(Mutex::new(Vec::new()));
        let mut handler2 = MockScreenHandler::new(Some(WindowType::Anvil), 3);
        handler2.add_slot(Arc::new(NormalSlot::new(inventory.clone(), 0)));
        handler2.add_slot(Arc::new(NormalSlot::new(inventory.clone(), 1)));
        handler2.add_slot(Arc::new(RecordingSlot::new(
            inventory.clone(),
            2,
            false,
            true,
            taken_log.clone(),
        )));
        let source2 = make_slot_info(ContainerName::AnvilResultPreview, 0);
        let stack2 = get_slot_stack(&handler2, &source2, None).await;
        assert!(charge_take_only_source(&player, &handler2, &source2, &stack2).await);
        assert_eq!(taken_log.lock().await.len(), 1, "exactly one charge");

        // A take-only slot that refuses the take rejects the move, uncharged.
        let refused_log = Arc::new(Mutex::new(Vec::new()));
        let mut handler3 = MockScreenHandler::new(Some(WindowType::Anvil), 3);
        handler3.add_slot(Arc::new(RecordingSlot::new(
            inventory.clone(),
            0,
            false,
            false,
            refused_log.clone(),
        )));
        let source3 = make_slot_info(ContainerName::AnvilInput, 0);
        let stack3 = get_slot_stack(&handler3, &source3, None).await;
        assert!(!charge_take_only_source(&player, &handler3, &source3, &stack3).await);
        assert!(
            refused_log.lock().await.is_empty(),
            "refused take never charges"
        );

        // Ordinary inventory slots are not charged through the slot at all.
        let normal_log = Arc::new(Mutex::new(Vec::new()));
        let mut handler4 = MockScreenHandler::new(Some(WindowType::Anvil), 3);
        handler4.add_slot(Arc::new(RecordingSlot::new(
            inventory.clone(),
            0,
            true,
            true,
            normal_log.clone(),
        )));
        assert!(
            charge_take_only_source(&player, &handler4, &source3, &stack3).await,
            "ordinary slot allows the take"
        );
        assert!(
            normal_log.lock().await.is_empty(),
            "ordinary slot not charged"
        );
    }

    /// Misroute guard: with `container_slots == 0` an `Inventory` source slot 9
    /// resolves to screen slot 0; if that slot is take-only the take must be
    /// settled through `on_take_item` — never served free (belt-and-suspenders
    /// while per-screen `container_slots` values are fixed in pumpkin-inventory).
    #[tokio::test]
    async fn misrouted_take_only_source_is_charged_not_free() {
        let player = MockInventoryPlayer::new();
        let inventory = Arc::new(SimpleInventory::new(46));
        inventory
            .set_stack(0, ItemStack::new(4, &Item::STICK))
            .await;
        let taken_log = Arc::new(Mutex::new(Vec::new()));
        let mut handler = MockScreenHandler::new(Some(WindowType::Crafting), 0);
        handler.add_slot(Arc::new(RecordingSlot::new(
            inventory.clone(),
            0,
            false,
            true,
            taken_log.clone(),
        )));
        for i in 1..46 {
            handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), i)));
        }

        let source = make_slot_info(ContainerName::Inventory, 9);
        let stack = get_slot_stack(&handler, &source, None).await;
        assert_eq!(stack.item_count, 4, "misroute resolves to screen slot 0");
        assert!(!destination_accepts(&handler, &source, &stack).await);
        assert!(charge_take_only_source(&player, &handler, &source, &stack).await);
        assert_eq!(taken_log.lock().await.len(), 1, "settled through the slot");
    }

    /// `CreatedOutput` without a tracked craft must resolve empty (arms reject).
    #[tokio::test]
    async fn created_output_without_tracked_craft_is_empty() {
        let handler = MockScreenHandler::new(None, 0);
        let source = make_slot_info(ContainerName::CreatedOutput, 0);
        assert!(get_slot_stack(&handler, &source, None).await.is_empty());
    }

    /// Offhand/armor names on small screens must stay in range (no panic).
    #[tokio::test]
    async fn offhand_and_armor_on_small_screen_stay_in_range() {
        let inventory = Arc::new(SimpleInventory::new(5));
        let mut handler = MockScreenHandler::new(None, 0);
        for i in 0..5 {
            handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), i)));
        }
        let offhand = make_slot_info(ContainerName::Offhand, 0);
        assert!(get_slot_stack(&handler, &offhand, None).await.is_empty());
        let armor = make_slot_info(ContainerName::Armor, 3);
        assert!(get_slot_stack(&handler, &armor, None).await.is_empty());

        // On non-player screens Offhand never resolves at all.
        let anvil = MockScreenHandler::new(Some(WindowType::Anvil), 3);
        assert!(get_slot_stack(&anvil, &offhand, None).await.is_empty());
    }
}
