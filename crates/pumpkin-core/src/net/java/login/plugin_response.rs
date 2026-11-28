#[allow(clippy::wildcard_imports)]
use super::*;

impl PendingConnection {
    pub async fn handle_plugin_response(
        &mut self,
        server: &Arc<Server>,
        plugin_response: SLoginPluginResponse,
    ) -> Option<PacketHandlerResult> {
        debug!("Handling plugin");
        let proxy_config = &server.advanced_config.networking.proxy;
        // The request side is gated on this too, so forwarding must not stay
        // reachable through an unsolicited response while proxy mode is off.
        if !proxy_config.enabled {
            return None;
        }
        if proxy_config.vine.enabled {
            let expected_challenge = self.vine_challenge.take();
            match vine::receive_vine_plugin_response(
                self.address.port(),
                &proxy_config.vine,
                plugin_response,
                expected_challenge,
            ) {
                Ok((profile, new_address)) => {
                    self.gameprofile = Some(profile.clone());
                    self.address = new_address;
                    if !Self::evict_previous_session(server, &profile).await {
                        self.kick(TextComponent::text(
                            "Your previous session is still closing, please try again",
                        ))
                        .await;
                        return Some(PacketHandlerResult::Stop);
                    }
                    self.finish_login(server, &profile).await
                }
                Err(error) => {
                    self.kick(TextComponent::text(error.to_string())).await;
                    Some(PacketHandlerResult::Stop)
                }
            }
        } else if proxy_config.velocity.enabled {
            match velocity::receive_velocity_plugin_response(
                self.address.port(),
                &proxy_config.velocity,
                plugin_response,
                &mut self.velocity_message_id,
            ) {
                Ok((profile, new_address)) => {
                    self.gameprofile = Some(profile.clone());
                    self.address = new_address;
                    if !Self::evict_previous_session(server, &profile).await {
                        self.kick(TextComponent::text(
                            "Your previous session is still closing, please try again",
                        ))
                        .await;
                        return Some(PacketHandlerResult::Stop);
                    }
                    self.finish_login(server, &profile).await
                }
                Err(error) => {
                    self.kick(TextComponent::text(error.to_string())).await;
                    Some(PacketHandlerResult::Stop)
                }
            }
        } else {
            None
        }
    }
}

impl PendingConnection {
    /// How long a new login waits for the previous session of the same UUID to be removed.
    const SESSION_EVICTION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

    /// Kicks a still-registered session with this UUID and waits until it has been removed and
    /// saved, so the new session cannot load stale player data or run next to a ghost.
    ///
    /// Returns `false` when the old session is still there after the timeout (for example when a
    /// plugin cancelled the kick); the caller must then reject the new login.
    async fn evict_previous_session(server: &Arc<Server>, profile: &GameProfile) -> bool {
        let Some(old_player) = server.get_player_by_uuid(profile.id) else {
            return true;
        };
        tracing::warn!(
            "Disconnecting stale session for {} ({}) on new login",
            profile.name,
            profile.id
        );
        old_player.kick(
            crate::DisconnectReason::Kicked,
            &TextComponent::text("Logged in from another location"),
        );
        let deadline = tokio::time::Instant::now() + Self::SESSION_EVICTION_TIMEOUT;
        while server.get_player_by_uuid(profile.id).is_some() {
            if tokio::time::Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        true
    }
}
