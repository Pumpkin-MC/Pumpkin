use crate::block::entities::command_block::CommandBlockEntity;
use crate::block::entities::jigsaw_block::JigsawBlockEntity;
use crate::entity::player::ChatMode;
use crate::entity::player::Player;
use crate::net::PlayerConfig;
use crate::net::java::JavaClient;
use crate::plugin::player::changed_main_hand::PlayerChangedMainHandEvent;
use crate::plugin::player::player_toggle_flight_event::PlayerToggleFlightEvent;
use crate::server::Server;
use crate::world::chunker;
use pumpkin_data::Advancement;
use pumpkin_data::Block;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::CommandBlockLikeProperties;
use pumpkin_data::translation;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::java::client::play::CPingResponse;
use pumpkin_protocol::java::server::play::CommandBlockMode;
use pumpkin_protocol::java::server::play::SChangeGameMode;
use pumpkin_protocol::java::server::play::SChunkBatch;
use pumpkin_protocol::java::server::play::SClientCommand;
use pumpkin_protocol::java::server::play::SClientInformationPlay;
use pumpkin_protocol::java::server::play::SCookieResponse as SPCookieResponse;
use pumpkin_protocol::java::server::play::SJigsawGenerate;
use pumpkin_protocol::java::server::play::SKeepAlive;
use pumpkin_protocol::java::server::play::SPlayPingRequest;
use pumpkin_protocol::java::server::play::SPlayerAbilities;
use pumpkin_protocol::java::server::play::SSeenAdvancement;
use pumpkin_protocol::java::server::play::SSetCommandBlock;
use pumpkin_protocol::java::server::play::SSetJigsawBlock;
use pumpkin_protocol::java::server::play::SSetTestBlock;
use pumpkin_protocol::java::server::play::STestInstanceBlockAction;
use pumpkin_util::Hand;
use pumpkin_util::PermissionLvl;
use pumpkin_util::text::TextComponent;
use pumpkin_world::generation::structure::structures::jigsaw::JigsawJointType;
use pumpkin_world::world::BlockFlags;
use std::num::NonZeroU8;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;
use tracing::debug;
use tracing::trace;
use tracing::warn;

impl JavaClient {
    pub async fn handle_change_game_mode(
        &self,
        player: &Arc<Player>,
        change_game_mode: SChangeGameMode,
    ) {
        if player.permission_lvl.load() >= PermissionLvl::Two {
            player.set_gamemode(change_game_mode.game_mode).await;
            let gamemode_string = format!("gameMode.{}", change_game_mode.game_mode.name());
            player
                .send_system_message(&TextComponent::translate_cross(
                    translation::java::COMMANDS_GAMEMODE_SUCCESS_SELF,
                    translation::bedrock::COMMANDS_GAMEMODE_SUCCESS_SELF,
                    [TextComponent::translate_cross(
                        gamemode_string.clone(),
                        gamemode_string,
                        [],
                    )],
                ))
                .await;
        }
    }

    pub async fn handle_set_command_block(
        &self,
        player: &Arc<Player>,
        command: SSetCommandBlock<'_>,
    ) {
        if !player.is_creative() {
            return;
        }
        if player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }
        let pos = command.pos;
        if let Some(block_entity) = player.world().get_block_entity(&pos) {
            if block_entity.resource_location() != CommandBlockEntity::ID {
                warn!("Client tried to change Command block but not Command block entity found");
                return;
            }

            let Ok(command_block_mode) = CommandBlockMode::try_from(command.mode) else {
                self.kick(TextComponent::text("Invalid Command block mode"))
                    .await;
                return;
            };

            let block = player.world().get_block(&pos);
            let old_state_id = player.world().get_block_state_id(&pos);
            let mut props = CommandBlockLikeProperties::from_state_id(old_state_id, block);

            let block_type = match command_block_mode {
                CommandBlockMode::Chain => Block::CHAIN_COMMAND_BLOCK,
                CommandBlockMode::Repeating => Block::REPEATING_COMMAND_BLOCK,
                CommandBlockMode::Impulse => Block::COMMAND_BLOCK,
            };

            let Some(old_command_block) =
                block_entity.as_any().downcast_ref::<CommandBlockEntity>()
            else {
                return;
            };

            props.conditional = command.flags & 0x2 != 0;

            let new_state_id = props.to_state_id(&block_type);
            player
                .world()
                .set_block_state(
                    &command.pos,
                    new_state_id,
                    BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
                )
                .await;

            let mut cmd = command.command;
            if cmd.starts_with('/') {
                cmd = &cmd[1..];
            }

            let command_block = CommandBlockEntity {
                position: pos,
                powered: old_command_block.powered.load(Ordering::SeqCst).into(),
                condition_met: old_command_block
                    .condition_met
                    .load(Ordering::SeqCst)
                    .into(),
                auto: (command.flags & 0x4 != 0).into(),
                dirty: old_command_block.dirty.load(Ordering::SeqCst).into(),
                command: Mutex::new(cmd.to_string()),
                last_output: old_command_block.last_output.lock().await.clone().into(),
                track_output: (command.flags & 0x1 != 0).into(),
                success_count: AtomicU32::new(0),
            };
            player.world().add_block_entity(Arc::new(command_block));

            player
                .send_system_message(&TextComponent::text(format!(
                    "Command set: {}",
                    command.command
                )))
                .await;

            // The 0x4 flag means always active
            if command.flags & 0x4 != 0 && block_type != Block::CHAIN_COMMAND_BLOCK {
                player.world().schedule_block_tick(
                    &block_type,
                    pos,
                    1,
                    pumpkin_world::tick::TickPriority::Normal,
                );
            }
        }
    }

    pub async fn handle_set_jigsaw_block(&self, player: &Arc<Player>, jigsaw: SSetJigsawBlock<'_>) {
        if !player.is_creative() {
            return;
        }
        if player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }
        let pos = jigsaw.pos;
        if let Some(block_entity) = player.world().get_block_entity(&pos) {
            if block_entity.resource_location() != JigsawBlockEntity::ID {
                warn!("Client tried to change Jigsaw block but not Jigsaw block entity found");
                return;
            }

            let Some(jigsaw_block) = block_entity.as_any().downcast_ref::<JigsawBlockEntity>()
            else {
                return;
            };

            *jigsaw_block.name.lock().await = jigsaw.name.to_string();
            *jigsaw_block.target.lock().await = jigsaw.target.to_string();
            *jigsaw_block.pool.lock().await = jigsaw.pool.to_string();
            *jigsaw_block.final_state.lock().await = jigsaw.final_state.to_string();
            *jigsaw_block.joint.lock().await = JigsawJointType::from_str(jigsaw.joint);
            jigsaw_block
                .selection_priority
                .store(jigsaw.selection_priority.0, Ordering::SeqCst);
            jigsaw_block
                .placement_priority
                .store(jigsaw.placement_priority.0, Ordering::SeqCst);

            player.world().update_block_entity(&block_entity);
        }
    }

    pub async fn handle_jigsaw_generate(&self, player: &Arc<Player>, generate: SJigsawGenerate) {
        if !player.is_creative() {
            return;
        }
        if player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }
        let pos = generate.pos;
        if let Some(block_entity) = player.world().get_block_entity(&pos)
            && let Some(jigsaw_block) = block_entity.as_any().downcast_ref::<JigsawBlockEntity>()
        {
            jigsaw_block
                .generate(&player.world(), generate.levels.0, generate.keep_jigsaws)
                .await;
        }
    }

    pub async fn handle_client_information(
        &self,
        player: &Arc<Player>,
        client_information: SClientInformationPlay<'_>,
    ) {
        if let (Ok(main_hand), Ok(chat_mode)) = (
            Hand::try_from(client_information.main_hand.0),
            ChatMode::try_from(client_information.chat_mode.0),
        ) {
            if client_information.view_distance <= 0 {
                self.kick(TextComponent::text(
                    "Cannot have zero or negative view distance!",
                ))
                .await;
                return;
            }

            let (update_settings, update_watched, main_hand_changed) = {
                // 1. Load current snapshot
                let current_config = player.config.load();

                // 2. Calculate if settings changed before we overwrite
                let main_hand_changed = current_config.main_hand != main_hand;
                let update_settings =
                    main_hand_changed || current_config.skin_parts != client_information.skin_parts;

                let old_view_distance = current_config.view_distance;
                let new_view_distance_raw = client_information.view_distance as u8;

                let update_watched = if old_view_distance.get() == new_view_distance_raw {
                    false
                } else {
                    debug!(
                        "Player {} ({}) updated their render distance: {} -> {}.",
                        player.gameprofile.name, self.id, old_view_distance, new_view_distance_raw
                    );
                    true
                };

                // 3. Construct the new config
                // If view_distance is 0, we exit early (safe guard)
                let Some(new_view_distance) = NonZeroU8::new(new_view_distance_raw) else {
                    return;
                };

                let new_config = PlayerConfig {
                    locale: client_information.locale.to_string(),
                    view_distance: new_view_distance,
                    chat_mode,
                    chat_colors: client_information.chat_colors,
                    skin_parts: client_information.skin_parts,
                    main_hand,
                    text_filtering: client_information.text_filtering,
                    server_listing: client_information.server_listing,
                };

                // 4. Atomically swap the new config into the player
                player.config.store(std::sync::Arc::new(new_config));

                (update_settings, update_watched, main_hand_changed)
            };

            if update_watched {
                chunker::update_position(player).await;
            }

            if main_hand_changed && let Some(server) = player.world().server.upgrade() {
                let event = PlayerChangedMainHandEvent::new(player.clone(), main_hand);
                let _ = server.plugin_manager.fire(event).await;
            }

            if update_settings {
                debug!(
                    "Player {} ({}) updated their skin.",
                    player.gameprofile.name, self.id,
                );
                player.send_client_information();
            }
        } else {
            self.kick(TextComponent::text("Invalid hand or chat type"))
                .await;
        }
    }

    pub async fn handle_client_status(&self, player: &Arc<Player>, client_status: SClientCommand) {
        player.update_last_action_time();
        match client_status.action_id.0 {
            0 => {
                // Perform respawn
                if player.living_entity.health.load() > 0.0 {
                    return;
                }
                player.world().clone().respawn_player(player, false).await;

                {
                    let screen_handler = player.current_screen_handler.lock().await;
                    let mut screen_handler = screen_handler.lock().await;
                    screen_handler.sync_state().await;
                };

                // Restore abilities based on gamemode after respawn
                {
                    let mut abilities = player.abilities.lock().await;
                    abilities.set_for_gamemode(player.gamemode.load());
                };
                player.send_abilities_update().await;
            }
            1 => {
                // Request stats
                player.send_stats().await;
            }
            _ => {
                self.kick(TextComponent::text("Invalid client status"))
                    .await;
            }
        }
    }

    pub async fn handle_keep_alive(&self, player: &Player, keep_alive: SKeepAlive) {
        if self.wait_for_keep_alive.load(Ordering::Relaxed)
            && keep_alive.keep_alive_id == self.keep_alive_id.load()
        {
            let ping = self.last_keep_alive_time.load().elapsed();
            // Vanilla logic
            player.ping.store(
                (player.ping.load(Ordering::Relaxed) * 3 + ping.as_millis() as u32) / 4,
                Ordering::Relaxed,
            );
            self.wait_for_keep_alive.store(false, Ordering::Relaxed);
        } else {
            self.kick(TextComponent::translate(
                translation::java::DISCONNECT_TIMEOUT,
                [],
            ))
            .await;
        }
    }

    pub async fn handle_player_abilities(
        &self,
        player: &Arc<Player>,
        player_abilities: SPlayerAbilities,
        server: &Server,
    ) {
        let (flying, allow_flying) = {
            let abilities = player.abilities.lock().await;
            (abilities.flying, abilities.allow_flying)
        };

        // Set the flying ability
        let new_flying = player_abilities.flags & 0x02 != 0 && allow_flying;
        if flying != new_flying {
            send_cancellable! {{
                server;
                PlayerToggleFlightEvent::new(player.clone(), new_flying);
                'after: {
                    if event.is_flying {
                        player.living_entity.fall_distance.store(0.0);
                    }
                    player.abilities.lock().await.flying = event.is_flying;
                }
                'cancelled: {
                    player.send_abilities_update().await;
                }
            }}
        }
    }

    pub async fn handle_play_ping_request(&self, request: SPlayPingRequest) {
        self.enqueue_packet(&CPingResponse::new(request.payload))
            .await;
    }

    pub async fn handle_chunk_batch(&self, player: &Player, packet: SChunkBatch) {
        player
            .chunk_manager
            .lock()
            .await
            .handle_acknowledge(packet.chunks_per_tick);
        trace!(
            "Client requested {} chunks per tick",
            packet.chunks_per_tick
        );
    }

    pub fn handle_cookie_response(&self, packet: &SPCookieResponse<'_>) {
        // TODO: allow plugins to access this
        debug!(
            "Received cookie_response[play]: key: \"{}\", payload_length: \"{:?}\"",
            packet.key,
            packet.payload.as_ref().map(|p| p.len())
        );
    }

    pub async fn handle_seen_advancement(&self, player: &Arc<Player>, packet: SSeenAdvancement) {
        if let SSeenAdvancement::OpenTab(tab) = packet {
            let advancement = Advancement::from_minecraft_name(&tab.to_string());
            if advancement.is_some() {
                player
                    .advancements
                    .lock()
                    .await
                    .set_selected_tab(advancement)
                    .await;
            }
        }
    }

    pub fn handle_test_instance_block_action(
        &self,
        player: &Arc<Player>,
        packet: &STestInstanceBlockAction<'_>,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();
        debug!(
            "Test instance block action at {:?}: action={:?}",
            packet.pos, packet.action
        );
    }

    pub fn handle_set_test_block(&self, player: &Arc<Player>, packet: &SSetTestBlock<'_>) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();
        debug!(
            "Set test block at {:?}: mode={:?}, message={}",
            packet.position, packet.mode, packet.message
        );
    }
}
