use super::ChatError;
use crate::entity::EntityBase;
use crate::entity::player::ChatSession;
use crate::entity::player::Player;
use crate::error::PumpkinError;
use crate::log_at_level;
use crate::net::java::JavaClient;
use crate::plugin::player::player_chat::PlayerChatEvent;
use crate::plugin::player::player_command_send::PlayerCommandSendEvent;
use crate::server::Server;
use crate::server::seasonal_events;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::bedrock::server::text::SText;
use pumpkin_protocol::java::client::play::CCommandSuggestions;
use pumpkin_protocol::java::client::play::CPlayerInfoUpdate;
use pumpkin_protocol::java::client::play::CSystemChatMessage;
use pumpkin_protocol::java::client::play::InitChat;
use pumpkin_protocol::java::client::play::PlayerAction;
use pumpkin_protocol::java::server::play::SChatCommand;
use pumpkin_protocol::java::server::play::SChatMessage;
use pumpkin_protocol::java::server::play::SCommandSuggestion;
use pumpkin_protocol::java::server::play::SPlayerSession;
use pumpkin_util::math::polynomial_rolling_hash;
use pumpkin_util::text::TextComponent;
use rsa::pkcs1v15::Signature as RsaPkcs1v15Signature;
use rsa::pkcs1v15::VerifyingKey;
use rsa::signature::Verifier;
use sha1::Sha1;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tracing::info;

/// In secure chat mode, Player will be kicked if they send a chat message with a timestamp that is older than this (in ms)
/// Vanilla: 2 minutes
const CHAT_MESSAGE_MAX_AGE: i64 = 1000 * 60 * 2;

impl JavaClient {
    pub async fn handle_chat_command(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        command: &SChatCommand<'_>,
    ) {
        player.update_last_action_time();
        let player_clone = player.clone();
        let server_clone = server.clone();
        send_cancellable! {{
            server;
            PlayerCommandSendEvent {
                player: player.clone(),
                command: command.command.to_string(),
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

    pub async fn handle_chat_message(
        &self,
        server: &Server,
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

    pub async fn handle_command_suggestion(
        &self,
        player: &Arc<Player>,
        packet: SCommandSuggestion<'_>,
        server: &Arc<Server>,
    ) {
        let Some(cmd) = &packet.command.get(1..) else {
            return;
        };

        let Some((last_word_start, _)) = cmd.char_indices().rfind(|(_, c)| c.is_whitespace())
        else {
            return;
        };

        let suggestions = server
            .command_dispatcher
            .read()
            .await
            .suggest(cmd, &player.get_command_source(server).await)
            .await;

        let response = CCommandSuggestions::new(
            packet.id,
            ((last_word_start + 2) as i32).into(),
            ((cmd.len() - last_word_start - 1) as i32).into(),
            suggestions.into(),
        );

        self.enqueue_packet(&response).await;
    }
}
