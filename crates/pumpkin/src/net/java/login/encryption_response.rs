#[allow(clippy::wildcard_imports)]
use super::*;

impl PendingConnection {
    async fn verify_encryption_token(
        &mut self,
        server: &Server,
        token: &[u8],
    ) -> Result<(), EncryptionError> {
        let Some(expected) = self.verify_token.take() else {
            return Err(EncryptionError::NoPendingVerifyToken);
        };

        let decrypted = server.decrypt(token).await?;
        if decrypted.as_slice() == expected.as_slice() {
            Ok(())
        } else {
            Err(EncryptionError::VerifyTokenMismatch)
        }
    }

    pub async fn handle_encryption_response(
        &mut self,
        server: &Arc<Server>,
        encryption_response: SEncryptionResponse,
    ) -> Option<PacketHandlerResult> {
        debug!("Handling encryption");
        if let Err(error) = self
            .verify_encryption_token(server, &encryption_response.verify_token)
            .await
        {
            debug!(
                "Rejecting encryption response from '{}': {error}",
                self.address
            );
            self.kick(TextComponent::text("Failed to verify encryption token"))
                .await;
            return Some(PacketHandlerResult::Stop);
        }

        let Ok(shared_secret) = server.decrypt(&encryption_response.shared_secret).await else {
            self.kick(TextComponent::text("Failed to decrypt shared secret"))
                .await;
            return Some(PacketHandlerResult::Stop);
        };

        if let Err(error) = self.set_encryption(&shared_secret) {
            self.kick(TextComponent::text(error.to_string())).await;
            return Some(PacketHandlerResult::Stop);
        }

        let profile_name = {
            let Some(profile) = self.gameprofile.as_ref() else {
                self.kick(TextComponent::text("No `GameProfile`")).await;
                return Some(PacketHandlerResult::Stop);
            };
            profile.name.clone()
        };

        if server.advanced_config.networking.java.online_mode {
            match self
                .authenticate(server, &shared_secret, &profile_name)
                .await
            {
                Ok(new_profile) => self.gameprofile = Some(new_profile),
                Err(error) => {
                    self.kick(match error {
                        AuthError::FailedResponse => TextComponent::translate_cross(
                            translation::java::MULTIPLAYER_DISCONNECT_AUTHSERVERS_DOWN,
                            translation::bedrock::DISCONNECT_LOGINFAILEDINFO_SERVERSUNAVAILABLE,
                            [],
                        ),
                        AuthError::UnverifiedUsername => TextComponent::translate_cross(
                            translation::java::MULTIPLAYER_DISCONNECT_UNVERIFIED_USERNAME,
                            translation::bedrock::DISCONNECT_LOGINFAILEDINFO_INVALIDSESSION,
                            [],
                        ),
                        e => TextComponent::text(e.to_string()),
                    })
                    .await;
                    return Some(PacketHandlerResult::Stop);
                }
            }
        }

        let Some(profile) = self.gameprofile.clone() else {
            return Some(PacketHandlerResult::Stop);
        };

        if let Some(online_player) = &server.get_player_by_uuid(profile.id) {
            debug!(
                "Player (IP '{}', username '{}') tried to log in with the same UUID ('{}') as an online player (username '{}')",
                &self.address, &profile.name, &profile.id, &online_player.gameprofile.name
            );
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_DUPLICATE_LOGIN,
                translation::bedrock::DISCONNECTIONSCREEN_LOGGEDINOTHERLOCATION,
                [],
            ))
            .await;
            return Some(PacketHandlerResult::Stop);
        }

        if let Some(online_player) = &server.get_player_by_name(&profile.name) {
            debug!(
                "A player (IP '{}', attempted username '{}') tried to log in with the same username as an online player (UUID '{}', username '{}')",
                &self.address, &profile.name, &profile.id, &online_player.gameprofile.name
            );
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_DUPLICATE_LOGIN,
                translation::bedrock::DISCONNECTIONSCREEN_LOGGEDINOTHERLOCATION,
                [],
            ))
            .await;
            return Some(PacketHandlerResult::Stop);
        }

        self.finish_login(server, &profile).await
    }

    pub(super) async fn enable_compression(&mut self, server: &Server) {
        let compression = server
            .advanced_config
            .networking
            .java
            .compression
            .info
            .clone();
        self.send_packet_now(&CSetCompression::new(
            pumpkin_protocol::codec::var_int::VarInt(compression.threshold as i32),
        ))
        .await;
        self.set_compression(&compression);
    }

    pub(super) async fn finish_login(
        &mut self,
        server: &Arc<Server>,
        profile: &GameProfile,
    ) -> Option<PacketHandlerResult> {
        let mut pre_login_event =
            crate::plugin::api::events::player::async_player_pre_login::AsyncPlayerPreLoginEvent {
                player_name: profile.name.clone(),
                player_uuid: profile.id,
                ip_address: self.address,
                kick_message: TextComponent::text("Disconnected"),
                cancelled: false,
            };
        server
            .plugin_manager
            .fire(server, &mut pre_login_event)
            .await;
        if pre_login_event.cancelled {
            self.kick(pre_login_event.kick_message).await;
            return Some(PacketHandlerResult::Stop);
        }

        let props = profile.properties.load();
        let packet = CLoginSuccess::new(
            &profile.id,
            &profile.name,
            &props,
            false,
            uuid::Uuid::new_v4(),
        );
        self.send_packet_now(&packet).await;
        None
    }

    async fn authenticate(
        &self,
        server: &Server,
        shared_secret: &[u8],
        username: &str,
    ) -> Result<GameProfile, AuthError> {
        let hash = server.digest_secret(shared_secret);
        let ip = self.address.ip();
        let profile = authentication::authenticate(
            username,
            &hash,
            &ip,
            &server.advanced_config.networking.java.authentication,
        )
        .await?;

        if let Some(actions) = &profile.profile_actions {
            if server
                .advanced_config
                .networking
                .java
                .authentication
                .player_profile
                .allow_banned_players
            {
                for allowed in &server
                    .advanced_config
                    .networking
                    .java
                    .authentication
                    .player_profile
                    .allowed_actions
                {
                    if !actions.contains(allowed) {
                        return Err(AuthError::DisallowedAction);
                    }
                }
                if !actions.is_empty() {
                    return Err(AuthError::Banned);
                }
            } else if !actions.is_empty() {
                return Err(AuthError::Banned);
            }
        }
        for property in profile.properties.load().iter() {
            authentication::validate_textures(
                property,
                &server
                    .advanced_config
                    .networking
                    .java
                    .authentication
                    .textures,
            )
            .map_err(AuthError::TextureError)?;
        }
        Ok(profile)
    }
}
