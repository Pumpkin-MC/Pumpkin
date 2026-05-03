use std::{
    num::{NonZero, NonZeroI32},
    sync::{Arc, atomic::Ordering},
};

use pumpkin_macros::send_cancellable;
use pumpkin_protocol::{
    bedrock::{
        client::{
            chunk_radius_update::CChunkRadiusUpdate,
            container_open::CContainerOpen,
            level_event::{CLevelEvent, LevelEvent},
        },
        server::{
            animate::{AnimateAction, SAnimate},
            command_request::SCommandRequest,
            container_close::SContainerClose,
            interaction::{Action, SInteraction},
            player_action::{Action as PlayerAction, SPlayerAction},
            player_auth_input::{InputData, SPlayerAuthInput},
            request_chunk_radius::SRequestChunkRadius,
            set_local_player_as_initialized::SSetLocalPlayerAsInitialized,
            text::SText,
        },
    },
    codec::{var_int::VarInt, var_long::VarLong, var_ulong::VarULong},
    java::client::play::{Animation, CEntityAnimation, CSetBlockDestroyStage, CSystemChatMessage},
};
use pumpkin_util::{
    GameMode,
    math::{position::BlockPos, vector3::Vector3},
    text::TextComponent,
};

use pumpkin_world::world::BlockFlags;

use crate::{
    entity::{EntityBase, player::Player},
    net::{DisconnectReason, bedrock::BedrockClient},
    plugin::player::{
        player_chat::PlayerChatEvent, player_command_send::PlayerCommandSendEvent,
        player_toggle_flight_event::PlayerToggleFlightEvent,
    },
    server::{Server, seasonal_events},
    world::chunker::{self},
};
use tracing::{debug, info};

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

        self.send_game_packet(&CChunkRadiusUpdate {
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

    pub async fn handle_player_auth_input(
        &self,
        player: &Arc<Player>,
        packet: SPlayerAuthInput,
        server: &Server,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        let new_pos = packet.position.to_f64();
        let old_pos = player.position();

        if new_pos != old_pos {
            player.living_entity.entity.set_pos(new_pos);
            chunker::update_position(player).await;
        }

        let input_data = packet.input_data;
        let entity = player.get_entity();

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

    pub async fn handle_animate(&self, player: &Arc<Player>, _server: &Server, packet: SAnimate) {
        if !player.has_client_loaded() {
            return;
        }

        let entity = &player.living_entity.entity;
        let world = entity.world.load();

        // Broadcast the animation to other players
        let java_animation = match packet.action {
            AnimateAction::SwingArm => Some(Animation::SwingMainArm),
            AnimateAction::WakeUp => Some(Animation::LeaveBed),
            AnimateAction::CriticalHit => Some(Animation::CriticalEffect),
            AnimateAction::MagicCriticalHit => Some(Animation::MagicCriticaleffect),
            _ => None,
        };

        // if let Some(animation) = java_animation {
        //     let je_packet = CEntityAnimation::new(VarInt(entity.entity_id), animation);
        //     let be_packet = SAnimate {
        //         action: packet.action,
        //         runtime_entity_id: VarULong(entity.entity_id as u64),
        //         boat_rowing_time: packet.boat_rowing_time,
        //     };
        //     world.broadcast_editioned(&je_packet, &be_packet).await;
        // }
    }

    pub async fn handle_interaction(&self, _player: &Arc<Player>, packet: SInteraction) {
        if matches!(packet.action, Action::OpenInventory) {
            self.send_game_packet(&CContainerOpen {
                container_id: 0,
                container_type: 0xff,
                position: BlockPos::ZERO,
                target_entity_id: VarLong(-1),
            })
            .await;
        }
    }

    pub async fn handle_container_close(&self, _player: &Arc<Player>, packet: SContainerClose) {
        if packet.container_id == 0 {
            self.send_game_packet(&SContainerClose {
                container_id: 0,
                container_type: 0xff,
                server_initiated: false,
            })
            .await;
        }
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

                let entity = &player.living_entity.entity;
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
            PlayerAction::StartBreak | PlayerAction::CreativePlayerDestroyBlock => {
                let location = packet.block_pos;
                if !player.can_interact_with_block_at(&location, 1.0) {
                    return;
                }

                let entity = &player.living_entity.entity;
                let world = entity.world.load_full();
                let (block, state) = world.get_block_and_state(&location).await;

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
                    // Broadcast that breaking started
                    world.set_block_breaking(entity, location, 0).await;

                    let speed = crate::block::calc_block_breaking(player, state, block).await;
                    if speed >= 1.0 {
                        let broken_state = world.get_block_state(&location).await;
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
            PlayerAction::CrackBreak => {
                // Don't do anything for this action. It is no longer used. Block
                // cracking is done fully server-side.
            }
            PlayerAction::AbortBreak | PlayerAction::StopBreak => {
                let location = packet.block_pos;
                let entity = &player.living_entity.entity;
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
}
