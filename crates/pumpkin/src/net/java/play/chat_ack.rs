#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::java::server::play::SChatAck;

impl JavaClient {
    pub async fn handle_chat_ack(&self, player: &Arc<Player>, packet: &SChatAck) {
        let offset = packet.offset.0;
        if offset < 0 {
            warn!(
                "Failed to validate message acknowledgement offset from {}: negative offset {}",
                player.gameprofile.name, offset
            );
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_CHAT_VALIDATION_FAILED,
                translation::java::MULTIPLAYER_DISCONNECT_CHAT_VALIDATION_FAILED,
                [],
            ))
            .await;
            return;
        }

        let mut cache = player.signature_cache.lock().await;
        if let Err(err) = cache.last_seen_validator.apply_offset(offset as usize) {
            warn!(
                "Failed to validate message acknowledgement offset from {}: {}",
                player.gameprofile.name, err
            );
            drop(cache);
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_CHAT_VALIDATION_FAILED,
                translation::java::MULTIPLAYER_DISCONNECT_CHAT_VALIDATION_FAILED,
                [],
            ))
            .await;
            return;
        }

        trace!(
            "Player {} acknowledged {} chat messages",
            player.gameprofile.name, offset
        );
    }
}
