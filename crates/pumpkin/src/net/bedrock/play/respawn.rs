#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_respawn(&self, player: &Arc<Player>, packet: SRespawn) {
        if packet.state != RespawnState::ClientReadyToSpawn
            || (!player.living_entity.dead.load(Ordering::Relaxed)
                && player.living_entity.health.load() > 0.0)
        {
            return;
        }

        let entity = player.get_entity();
        let position = entity.pos.load();
        self.enqueue_packet(&CRespawn::new(
            pumpkin_util::math::vector3::Vector3::new(
                position.x as f32,
                position.y as f32 + entity.entity_type.eye_height,
                position.z as f32,
            ),
            RespawnState::ReadyToSpawn,
            VarULong(player.entity_id() as u64),
        ))
        .await;
    }
}
