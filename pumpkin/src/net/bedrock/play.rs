use std::{
    num::{NonZero, NonZeroI32},
    sync::Arc,
};

use pumpkin_macros::send_cancellable;
use pumpkin_protocol::{
    bedrock::{
        client::{chunk_radius_update::CChunkRadiusUpdate, container_open::CContainerOpen},
        server::{
            command_request::SCommandRequest,
            container_close::SContainerClose,
            interaction::{Action, SInteraction},
            player_auth_input::{InputData, SPlayerAuthInput},
            request_chunk_radius::SRequestChunkRadius,
            text::SText,
        },
    },
    codec::{bedrock_block_pos::NetworkPos, var_int::VarInt, var_long::VarLong},
    java::client::play::CSystemChatMessage,
};
use pumpkin_util::{math::position::BlockPos, text::TextComponent};

use crate::{
    command::CommandSender,
    entity::{EntityBase, player::Player},
    net::{DisconnectReason, bedrock::BedrockClient},
    plugin::player::{
        player_chat::PlayerChatEvent, player_command_preprocess::PlayerCommandPreprocessEvent,
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

        if old_view_distance.get() != view_distance as u8 {
            debug!(
                "Player {} updated their render distance: {} -> {}.",
                player.gameprofile.name, old_view_distance, view_distance
            );
            chunker::update_position(player).await;
        }
    }

    pub async fn player_pos_update(&self, player: &Arc<Player>, packet: SPlayerAuthInput) {
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

        if input_data.get(InputData::StartSprinting) {
            entity.set_sprinting(true).await;
        } else if input_data.get(InputData::StopSprinting) {
            entity.set_sprinting(false).await;
        }

        if input_data.get(InputData::StartFlying) {
            player.abilities.lock().await.flying = true;
            player.send_abilities_update().await;
        } else if input_data.get(InputData::StopFlying) {
            player.abilities.lock().await.flying = false;
            player.send_abilities_update().await;
        }

        if input_data.get(InputData::StartSneaking) {
            entity.set_sneaking(true).await;
        } else if input_data.get(InputData::StopSneaking) {
            entity.set_sneaking(false).await;
        }
    }

    pub async fn handle_interaction(&self, _player: &Arc<Player>, packet: SInteraction) {
        if matches!(packet.action, Action::OpenInventory) {
            self.send_game_packet(&CContainerOpen {
                container_id: 0,
                container_type: 0xff,
                position: NetworkPos(BlockPos::ZERO),
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

    pub async fn handle_chat_command(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        command: SCommandRequest,
    ) {
        let player_clone = player.clone();
        let server_clone: Arc<Server> = server.clone();
        send_cancellable! {{
            server;
            PlayerCommandPreprocessEvent {
                player: player.clone(),
                command: format!("/{}", command.command),
                cancelled: false
            };

            'after: {
                let command = event.command;
                let command_stripped = command.strip_prefix('/').unwrap_or(&command).to_string();
                let command_clone = command_stripped.clone();
                // Some commands can take a long time to execute. If they do, they block packet processing for the player.
                // That's why we will spawn a task instead.
                server.spawn_task(async move {
                    let dispatcher = server_clone.command_dispatcher.read().await;
                    dispatcher
                        .handle_command(
                            &CommandSender::Player(player_clone),
                            &server_clone,
                            &command_clone,
                        )
                        .await;
                });

                if server.advanced_config.commands.log_console {
                    info!(
                        "Player ({}): executed command /{}",
                        player.gameprofile.name,
                        command_stripped
                    );
                }
            }
        }}
    }
}
