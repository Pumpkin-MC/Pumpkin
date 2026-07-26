use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::net::bedrock::BedrockClient;
use crate::plugin::player::player_chat::PlayerChatEvent;
use crate::plugin::player::player_command_send::PlayerCommandSendEvent;
use crate::server::Server;
use crate::server::seasonal_events;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::bedrock::server::command_request::SCommandRequest;
use pumpkin_protocol::bedrock::server::text::SText;
use pumpkin_protocol::java::client::play::CSystemChatMessage;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use tracing::info;

impl BedrockClient {
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
}
