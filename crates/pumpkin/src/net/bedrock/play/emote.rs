#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_emote(&self, player: &Arc<Player>, _server: &Server, packet: SEmote<'_>) {
        if !player.has_client_loaded() {
            return;
        }

        tracing::info!(
            "handle_emote: player={} packet={:?}",
            player.gameprofile.name,
            packet
        );

        let entity = &player.living_entity.entity;
        let world = entity.world.load();

        let mut broadcast_packet = packet;
        broadcast_packet.runtime_entity_id = VarULong(entity.entity_id as u64);
        broadcast_packet.flags |= pumpkin_protocol::bedrock::server::emote::EMOTE_FLAG_SERVER_SIDE;

        world
            .broadcast_packet_except_editioned(
                &[player.gameprofile.id],
                &CEntityAnimation::new(
                    VarInt(entity.entity_id),
                    Animation::SwingMainArm, // Fallback for Java? Or just ignore
                ),
                &broadcast_packet,
            )
            .await;
    }
}
