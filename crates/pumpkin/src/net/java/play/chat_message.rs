#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_chat_message(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        chat_message: SChatMessage<'_>,
    ) {
        player.update_last_action_time();
        let gameprofile = &player.gameprofile;

        if let Err(err) = self
            .validate_chat_message(server, player, &chat_message)
            .await
        {
            log_at_level!(
                err.severity(),
                "{} (uuid {}) {}",
                gameprofile.name,
                gameprofile.id,
                err
            );
            if err.is_kick()
                && let Some(reason) = err.client_kick_reason()
            {
                self.kick(TextComponent::text(reason)).await;
            }
            return;
        }

        send_cancellable! {{
            server;
            PlayerChatEvent::new(player.clone(), chat_message.message.to_string(), vec![]);

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
                let world = entity.world.load_full();
                if server.basic_config.allow_chat_reports {
                    world.broadcast_secure_player_chat(player, &chat_message, &decorated_message).await;
                } else {
                    let je_packet = CSystemChatMessage::new(
                        &decorated_message,
                        false,
                    );
                    let be_packet = SText::new(
                        message, player.gameprofile.name.clone()
                    );

                    world.broadcast_editioned(&je_packet, &be_packet).await;
                }
            }
        }}
    }

    /// Runs all vanilla checks for a valid chat message
    pub async fn validate_chat_message(
        &self,
        server: &Server,
        player: &Arc<Player>,
        chat_message: &SChatMessage<'_>,
    ) -> Result<(), ChatError> {
        // Check for oversized messages
        // If we're able to find the 257th UTF-16 character, the message is too big.
        if chat_message.message.encode_utf16().nth(256).is_some() {
            return Err(ChatError::OversizedMessage);
        }
        // Check for illegal characters
        if chat_message
            .message
            .chars()
            .any(|c| c == '§' || c < ' ' || c == '\x7F')
        {
            return Err(ChatError::IllegalCharacters);
        }
        // These checks are only run in secure chat mode
        if server.basic_config.allow_chat_reports {
            // Check for unsigned chat
            if let Some(signature) = &chat_message.signature {
                if signature.len() != 256 {
                    return Err(ChatError::UnsignedChat); // Signature is the wrong length
                }
            } else {
                return Err(ChatError::UnsignedChat); // There is no signature
            }

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;

            // Verify message timestamp
            if chat_message.timestamp > now || chat_message.timestamp < (now - CHAT_MESSAGE_MAX_AGE)
            {
                return Err(ChatError::OutOfOrderChat);
            }

            // Verify session expiry
            if player.chat_session.lock().await.expires_at < now {
                return Err(ChatError::ExpiredPublicKey);
            }

            // Validate previous signature checksum (new in 1.21.5)
            // The client can bypass this check by sending 0
            if chat_message.checksum != 0 {
                let checksum =
                    polynomial_rolling_hash(player.signature_cache.lock().await.last_seen.as_ref());
                if checksum != chat_message.checksum {
                    return Err(ChatError::ChatValidationFailed);
                }
            }
        }
        Ok(())
    }

    pub async fn handle_chat_session_update(
        &self,
        player: &Arc<Player>,
        server: &Server,
        session: SPlayerSession,
    ) {
        // Keep the chat session default if we don't want reports
        if !server.basic_config.allow_chat_reports {
            return;
        }

        if let Err(err) = self.validate_chat_session(player, server, &session) {
            log_at_level!(
                err.severity(),
                "{} (uuid {}) {}",
                player.gameprofile.name,
                player.gameprofile.id,
                err
            );
            if err.is_kick()
                && let Some(reason) = err.client_kick_reason()
            {
                self.kick(TextComponent::text(reason)).await;
            }
            return;
        }

        // Update the chat session fields
        *player.chat_session.lock().await = ChatSession::new(
            session.session_id,
            session.expires_at,
            session.public_key.clone(),
            session.key_signature.clone(),
        );

        server.broadcast_packet_all(&CPlayerInfoUpdate::new(
            0x02,
            &[pumpkin_protocol::java::client::play::Player {
                uuid: player.gameprofile.id,
                actions: &[PlayerAction::InitializeChat(Some(InitChat {
                    session_id: session.session_id,
                    expires_at: session.expires_at,
                    public_key: session.public_key.clone(),
                    signature: session.key_signature.clone(),
                }))],
            }],
        ));
    }

    /// Runs vanilla checks for a valid player session
    pub fn validate_chat_session(
        &self,
        player: &Player,
        server: &Server,
        session: &SPlayerSession,
    ) -> Result<(), ChatError> {
        // Verify session expiry
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        if session.expires_at < now {
            return Err(ChatError::InvalidPublicKey);
        }

        let key_signature = RsaPkcs1v15Signature::try_from(session.key_signature.as_ref())
            .map_err(|_| ChatError::InvalidPublicKey)?;

        let mut signable = Vec::new();
        signable.extend_from_slice(player.gameprofile.id.as_bytes());
        signable.extend_from_slice(&session.expires_at.to_be_bytes());
        signable.extend_from_slice(&session.public_key);

        let public_keys_guard = server.mojang_public_keys.load();

        // Verify signature with RSA-SHA1
        let is_valid = public_keys_guard.iter().any(|key| {
            let verifying_key = VerifyingKey::<Sha1>::new(key.clone());
            verifying_key.verify(&signable, &key_signature).is_ok()
        });

        // Verify that the signable is valid for any one of Mojang's public keys
        if !is_valid {
            return Err(ChatError::InvalidPublicKey);
        }

        Ok(())
    }
}
