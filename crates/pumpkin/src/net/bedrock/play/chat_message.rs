#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
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
}
