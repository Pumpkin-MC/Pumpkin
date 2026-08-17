#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_client_information(
        &self,
        server: &Arc<Server>,
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

            let (update_settings, update_watched, main_hand_changed, locale_changed) = {
                // 1. Load current snapshot
                let current_config = player.config.load();

                // 2. Calculate if settings changed before we overwrite
                let main_hand_changed = current_config.main_hand != main_hand;
                let locale_changed = current_config.locale != client_information.locale;
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
                let Some(new_view_distance) = NonZero::new(new_view_distance_raw) else {
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

                (
                    update_settings,
                    update_watched,
                    main_hand_changed,
                    locale_changed,
                )
            };

            if update_watched {
                chunker::update_position(player).await;
            }

            if main_hand_changed {
                let mut event = PlayerChangedMainHandEvent::new(player.clone(), main_hand);
                server.plugin_manager.fire(server, &mut event).await;
            }

            if locale_changed {
                let mut event = crate::plugin::api::events::player::player_locale_change::PlayerLocaleChangeEvent {
                    player: player.clone(),
                    new_locale: client_information.locale.to_string(),
                    cancelled: false,
                };
                server.plugin_manager.fire(server, &mut event).await;
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
}
