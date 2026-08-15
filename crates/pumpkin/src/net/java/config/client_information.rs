#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_client_information_config(
        &self,
        client_information: SClientInformationConfig<'_>,
    ) {
        debug!("Handling client settings");
        if client_information.view_distance <= 0 {
            self.kick(TextComponent::text(
                "Cannot have zero or negative view distance!",
            ))
            .await;
            return;
        }

        if let (Ok(main_hand), Ok(chat_mode)) = (
            Hand::try_from(client_information.main_hand.0),
            ChatMode::try_from(client_information.chat_mode.0),
        ) {
            self.config.store(Arc::new(PlayerConfig {
                locale: client_information.locale.to_string(),
                // client_information.view_distance was checked above to be > 0 so compiler should optimize this out.
                view_distance: NonZeroU8::new(client_information.view_distance as u8)
                    .unwrap_or(NonZeroU8::MIN),
                chat_mode,
                chat_colors: client_information.chat_colors,
                skin_parts: client_information.skin_parts,
                main_hand,
                text_filtering: client_information.text_filtering,
                server_listing: client_information.server_listing,
            }));
        } else {
            self.kick(TextComponent::text("Invalid hand or chat type"))
                .await;
        }
    }
}
