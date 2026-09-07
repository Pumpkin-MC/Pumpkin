use pumpkin_protocol::bedrock::server::RespawnState;

#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub fn handle_respawn(&self, player: &Arc<Player>, packet: &SRespawn) {
        if packet.state != RespawnState::ClientReadyToSpawn
            || !player.bedrock_respawn_ack_pending.load(Ordering::Acquire)
        {
            return;
        }

        let entity = player.get_entity();
        let position = entity.pos.load();
        let Ok(response) = self.serialize_packet(&SRespawn {
            position: pumpkin_util::math::vector3::Vector3::new(
                position.x as f32,
                position.y as f32 + entity.entity_type.eye_height,
                position.z as f32,
            ),
            state: RespawnState::ReadyToSpawn,
            player_runtime_id: VarULong(player.entity_id() as u64),
        }) else {
            return;
        };
        // PlayerAction::Respawn may already have restored health. Acknowledge the
        // outstanding handshake exactly once, independently of that world update.
        if player
            .bedrock_respawn_ack_pending
            .swap(false, Ordering::AcqRel)
        {
            self.try_enqueue_packet(response);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::atomic::Ordering;

    use pumpkin_protocol::Packet;
    use pumpkin_protocol::bedrock::packet_decoder::BedrockBatchDecoder;
    use pumpkin_protocol::bedrock::server::respawn::{RespawnState, SRespawn};
    use pumpkin_protocol::codec::var_ulong::VarULong;
    use pumpkin_protocol::serial::PacketRead;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_util::text::TextComponent;

    use crate::test_support::TestServer;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn busy_encoder_preserves_pending_respawn_acknowledgement() {
        let fixture = TestServer::new().await;
        let player = fixture.new_bedrock_player().await;
        let client = player.client.bedrock().unwrap();
        player.living_entity.health.store(0.0);
        player.living_entity.dead.store(true, Ordering::Relaxed);
        player.handle_killed(&TextComponent::text("test death"));
        let _ = client.drain_outgoing_packets_for_test().await;
        let request = SRespawn {
            position: Vector3::default(),
            state: RespawnState::ClientReadyToSpawn,
            player_runtime_id: VarULong(player.entity_id() as u64),
        };

        let encoder = client.network_writer.write().await;
        client.handle_respawn(&player, &request);
        assert!(player.bedrock_respawn_ack_pending.load(Ordering::Acquire));
        assert!(client.drain_outgoing_packets_for_test().await.is_empty());
        drop(encoder);

        client.handle_respawn(&player, &request);
        assert!(!player.bedrock_respawn_ack_pending.load(Ordering::Acquire));
        let packets = client.drain_outgoing_packets_for_test().await;
        assert_eq!(packets.len(), 1);
        let mut decoder = BedrockBatchDecoder::new();
        let payload = decoder
            .get_packet_payload(packets[0].to_vec())
            .await
            .unwrap();
        let packet = decoder.get_game_packet(&mut Cursor::new(payload)).unwrap();
        assert_eq!(packet.id, SRespawn::PACKET_ID);
        let response = SRespawn::read(&mut packet.payload.as_ref()).unwrap();
        assert_eq!(response.state, RespawnState::ReadyToSpawn);

        drop(player);
        fixture.shutdown().await;
    }
}
