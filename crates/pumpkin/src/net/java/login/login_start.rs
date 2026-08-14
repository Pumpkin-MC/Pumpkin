#[allow(clippy::wildcard_imports)]
use super::*;

impl PendingConnection {
    pub async fn handle_login_start(&mut self, server: &Server, login_start: SLoginStart) {
        debug!("login start");

        let max_players = server.advanced_config.networking.java.max_players;
        if max_players > 0 && server.get_player_count() >= max_players as usize {
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_SERVER_FULL,
                translation::bedrock::DISCONNECTIONSCREEN_SERVERFULL,
                [],
            ))
            .await;
            return;
        }

        if !is_valid_player_name(&login_start.name) {
            self.kick(TextComponent::text("Invalid characters in username"))
                .await;
            return;
        }

        let proxy = &server.advanced_config.networking.proxy;
        if proxy.enabled {
            if proxy.velocity.enabled {
                velocity::velocity_login(self).await;
            } else if proxy.bungeecord.enabled {
                match bungeecord::bungeecord_login(
                    &self.address,
                    &self.server_address,
                    login_start.name.into_string(),
                ) {
                    Ok((_ip, profile)) => {
                        self.finish_login(&profile).await;
                        self.gameprofile = Some(profile);
                    }
                    Err(error) => self.kick(TextComponent::text(error.to_string())).await,
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

            if server.advanced_config.networking.java.encryption {
                let verify_token: [u8; 4] = rand::random();
                self.send_packet_now(
                    &server
                        .encryption_request(
                            &verify_token,
                            server.advanced_config.networking.java.online_mode,
                        )
                        .await,
                )
                .await;
            } else {
                self.finish_login(&profile).await;
            }

            self.gameprofile = Some(profile);
        }
    }
}
