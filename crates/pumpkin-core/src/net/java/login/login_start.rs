#[allow(clippy::wildcard_imports)]
use super::*;

impl PendingConnection {
    pub async fn handle_login_start(
        &mut self,
        server: &Arc<Server>,
        login_start: SLoginStart,
    ) -> Option<PacketHandlerResult> {
        debug!("login start");

        // A second login start would replace an already authenticated profile
        // with a freshly claimed name and uuid.
        if self.login_started {
            self.kick(TextComponent::text("Unexpected login start"))
                .await;
            return Some(PacketHandlerResult::Stop);
        }
        self.login_started = true;

        let max_players = server.advanced_config.networking.java.max_players;
        if max_players > 0 && server.get_player_count() >= max_players as usize {
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_SERVER_FULL,
                translation::bedrock::DISCONNECTIONSCREEN_SERVERFULL,
                [],
            ))
            .await;
            return Some(PacketHandlerResult::Stop);
        }

        if !is_valid_player_name(&login_start.name) {
            self.kick(TextComponent::text("Invalid characters in username"))
                .await;
            return Some(PacketHandlerResult::Stop);
        }

        let proxy = &server.advanced_config.networking.proxy;
        if proxy.enabled && proxy.vine.enabled {
            vine::vine_login(self).await;
            None
        } else if proxy.enabled && proxy.velocity.enabled {
            velocity::velocity_login(self).await;
            None
        } else if proxy.enabled && proxy.bungeecord.enabled {
            match bungeecord::bungeecord_login(
                &self.address,
                &self.server_address,
                login_start.name.into_string(),
                &proxy.bungeecord.secret,
            ) {
                Ok((_ip, profile)) => {
                    self.gameprofile = Some(profile.clone());
                    self.finish_login(server, &profile).await
                }
                Err(error) => {
                    self.kick(TextComponent::text(error.to_string())).await;
                    Some(PacketHandlerResult::Stop)
                }
            }
        } else {
            let id = if server.advanced_config.networking.java.online_mode {
                login_start.uuid
            } else {
                offline_uuid(&login_start.name).unwrap_or_else(|_| uuid::Uuid::nil())
            };

            let profile = GameProfile {
                id,
                name: login_start.name.into_string(),
                properties: ArcSwap::new(Arc::new(vec![])),
                profile_actions: None,
            };

            if server.advanced_config.networking.java.compression.enabled {
                self.enable_compression(server).await;
            }

            self.gameprofile = Some(profile.clone());

            let online_mode = server.advanced_config.networking.java.online_mode;
            // Clients before 1.20.5 have no `should_authenticate` flag in `CEncryptionRequest`
            // and always contact Mojang's session server on encryption.
            if server.advanced_config.networking.java.encryption
                && (online_mode
                    || self.version.load() >= pumpkin_util::version::JavaMinecraftVersion::V_1_20_5)
            {
                let verify_token: [u8; 4] = rand::random();
                self.verify_token = Some(verify_token);
                self.send_packet_now(&server.encryption_request(&verify_token, online_mode).await)
                    .await;
                None
            } else {
                self.finish_login(server, &profile).await
            }
        }
    }
}
