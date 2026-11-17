use pumpkin_protocol::bedrock::server::RespawnState;

#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub fn handle_respawn(&self, player: &Arc<Player>, packet: &SRespawn) {
        if packet.state != RespawnState::ClientReadyToSpawn
            || (!player.living_entity.dead.load(Ordering::Relaxed)
                && player.living_entity.health.load() > 0.0)
        {
            return;
        }

        self.try_enqueue_client_packet(&SRespawn {
            position: player.bedrock_pos().to_f32_lossy(),
            state: RespawnState::ReadyToSpawn,
            player_runtime_id: VarULong(player.entity_id() as u64),
        });
    }
}
