use std::{
    num::NonZero,
    sync::Arc,
};

use pumpkin_config::{BASIC_CONFIG, advanced_config};
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::{
    bedrock::{
        client::{
            chunk_radius_update::CChunkRadiusUpdate, container_open::CContainerOpen,
            network_chunk_publisher_update::CNetworkChunkPublisherUpdate,
        },
        server::{
            command_request::SCommandRequest,
            container_close::SContainerClose,
            interaction::{Action, SInteraction},
            player_auth_input::SPlayerAuthInput,
            request_chunk_radius::SRequestChunkRadius,
            text::SText,
        },
    },
    codec::{bedrock_block_pos::NetworkPos, var_long::VarLong},
    java::client::play::CSystemChatMessage,
};
use pumpkin_util::text::TextComponent;

use crate::{
    command::CommandSender,
    entity::{EntityBase, player::Player},
    net::{DisconnectReason, bedrock::BedrockClient},
    plugin::player::{player_chat::PlayerChatEvent, player_command_send::PlayerCommandSendEvent},
    server::{Server, seasonal_events},
    world::chunker,
};

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

        self.send_game_packet(&CChunkRadiusUpdate { chunk_radius })
            .await;

        let old_view_distance = {
            let mut config = player.config.write().await;
            let old_view_distance = config.view_distance;
            config.view_distance = NonZero::new(chunk_radius.0 as u8).unwrap();
            old_view_distance
        };

        if old_view_distance.get() != chunk_radius.0 as u8 {
            log::debug!(
                "Player {} updated their render distance: {} -> {}.",
                player.gameprofile.name,
                old_view_distance,
                chunk_radius.0
            );
            self.send_game_packet(&CNetworkChunkPublisherUpdate::new(
                player.get_entity().block_pos.load(),
                chunk_radius.0 as _,
            ))
            .await;
            chunker::be_update_position(player).await;
        }
    }

    pub async fn player_pos_update(&self, player: &Arc<Player>, packet: SPlayerAuthInput) {
        if !player.has_client_loaded() {
            return;
        }
        let pos = packet.position;
        player.living_entity.entity.set_pos(pos.to_f64());

        chunker::update_position(player).await;
        //self.send_game_packet(&CMovePlayer {
        //     player_runtime_id: VarULong(player.entity_id() as u64),
        //    position: packet.position + Vector3::new(10.0, 0.0, 0.0),
        //    pitch: packet.pitch,
        //    yaw: packet.yaw,
        //    y_head_rotation: packet.head_rotation,
        //    position_mode: 1,
        //    on_ground: false,
        //    riding_runtime_id: VarULong(0),
        //    tick: packet.client_tick,
        //})
        //.await;
    }

    pub async fn handle_interaction(&self, _player: &Arc<Player>, packet: SInteraction) {
        if matches!(packet.action, Action::OpenInventory) {
            self.send_game_packet(&CContainerOpen {
                container_id: 0,
                container_type: 0xff,
                position: NetworkPos(packet.position.to_block_pos()),
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

    pub async fn handle_chat_message(&self, player: &Arc<Player>, packet: SText) {
        let gameprofile = &player.gameprofile;

        send_cancellable! {{
            PlayerChatEvent::new(player.clone(), packet.message, vec![]);

            'after: {
                log::info!("<chat> {}: {}", gameprofile.name, event.message);

                let config = advanced_config();

                let message = match seasonal_events::modify_chat_message(&event.message) {
                    Some(m) => m,
                    None => event.message.clone(),
                };

                let decorated_message = &TextComponent::chat_decorated(
                    config.chat.format.clone(),
                    gameprofile.name.clone(),
                    message.clone(),
                );

                let entity = &player.living_entity.entity;
                let world = &entity.world.read().await;
                if BASIC_CONFIG.allow_chat_reports {
                    //TODO Alex help, what is this?
                    //world.broadcast_secure_player_chat(player, &message, decorated_message).await;
                } else {
                    let je_packet = CSystemChatMessage::new(
                        decorated_message,
                        false,
                    );

                    let be_packet = SText::new(
                        message, gameprofile.name.clone()
                    );

                    world.broadcast_editioned(&je_packet, &be_packet).await;
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
            PlayerCommandSendEvent {
                player: player.clone(),
                command: command.command.clone(),
                cancelled: false
            };

            'after: {
                let command = event.command;
                let command_clone = command.clone();
                // Some commands can take a long time to execute. If they do, they block packet processing for the player.
                // That's why we will spawn a task instead.
                server.spawn_task(async move {
                    let dispatcher = server_clone.command_dispatcher.read().await;
                    dispatcher
                        .handle_command(
                            &mut CommandSender::Player(player_clone),
                            &server_clone,
                            &command_clone,
                        )
                        .await;
                });

                if advanced_config().commands.log_console {
                    log::info!(
                        "Player ({}): executed command /{}",
                        player.gameprofile.name,
                        command
                    );
                }
            }
        }}
    }
}
