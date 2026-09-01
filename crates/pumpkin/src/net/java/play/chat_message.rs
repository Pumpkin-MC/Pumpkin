#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_data::world::RAW;

impl JavaClient {
    pub async fn handle_chat_message(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        chat_message: SChatMessage<'_>,
    ) {
        player.update_last_action_time();

        if let Some(command) = chat_message.message.strip_prefix('/') {
            let command_packet = SChatCommand { command };
            self.handle_chat_command(player, server, &command_packet)
                .await;
            return;
        }

        let gameprofile = &player.gameprofile;

        if let Err(err) = self.validate_chat_message(server, player, &chat_message) {
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

        if player.check_chat_spam(server) {
            return;
        }

        send_cancellable! {{
            server;
            PlayerChatEvent::new(
                player.clone(),
                chat_message.message.to_string(),
                vec![],
                chat_message.signature.map(<[u8]>::to_vec),
            );

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
                    world.broadcast_secure_player_chat(player, &chat_message, &decorated_message);
                } else {
                    let outgoing = crate::net::chat::PlayerChatMessage::system(message).with_unsigned_content(decorated_message);
                    world.broadcast_chat_message(
                        &outgoing,
                        Player::is_text_filtering_enabled,
                        Some(player),
                        (RAW + 1).into(),
                        &TextComponent::empty(),
                        None,
                    );
                }
            }
        }}
    }

    /// Runs all vanilla checks for a valid chat message
    pub fn validate_chat_message(
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
            if player
                .chat_session
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .expires_at
                < now
            {
                return Err(ChatError::ExpiredPublicKey);
            }

            let offset = chat_message.message_count.0;
            if offset < 0 {
                return Err(ChatError::ChatValidationFailed);
            }

            {
                let mut cache = player
                    .signature_cache
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if !chat_message.acknowledged.is_empty() {
                    if cache
                        .last_seen_validator
                        .apply_update(offset as usize, chat_message.acknowledged)
                        .is_err()
                    {
                        return Err(ChatError::ChatValidationFailed);
                    }
                } else if cache
                    .last_seen_validator
                    .apply_offset(offset as usize)
                    .is_err()
                {
                    return Err(ChatError::ChatValidationFailed);
                }

                if cache.last_seen_validator.tracked_messages_count() > 4096 {
                    return Err(ChatError::TooManyPendingChats);
                }
            }

            // Validate previous signature checksum (new in 1.21.5)
            // The client can bypass this check by sending 0
            if chat_message.checksum != 0 {
                let checksum = polynomial_rolling_hash(
                    player
                        .signature_cache
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .last_seen
                        .as_ref(),
                );
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

        if let Err(err) = self.validate_chat_session(player, server, &session).await {
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
        *player
            .chat_session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = ChatSession::new(
            session.session_id,
            session.expires_at,
            session.public_key.clone(),
            session.key_signature.clone(),
        );

        server.broadcast_packet_all(&CPlayerInfoUpdate::new(
            PlayerInfoFlags::INITIALIZE_CHAT.bits(),
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
    pub async fn validate_chat_session(
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

        let public_keys = server.mojang_public_keys.load_full();

        let (tx, rx) = tokio::sync::oneshot::channel();
        rayon::spawn(move || {
            let is_valid = public_keys.iter().any(|key| {
                let verifying_key = VerifyingKey::<Sha1>::new(key.clone());
                verifying_key.verify(&signable, &key_signature).is_ok()
            });
            let _ = tx.send(is_valid);
        });
        let is_valid = rx.await.unwrap_or(false);

        // Verify that the signable is valid for any one of Mojang's public keys
        if !is_valid {
            return Err(ChatError::InvalidPublicKey);
        }

        Ok(())
    }
}
