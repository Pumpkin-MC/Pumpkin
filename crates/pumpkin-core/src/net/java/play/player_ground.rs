#[allow(clippy::wildcard_imports)]
use super::*;
use crate::entity::player::ClientMove;

impl JavaClient {
    pub fn handle_player_ground(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        ground: &SSetPlayerGround,
    ) {
        self.received_movement_this_tick
            .store(true, Ordering::Relaxed);
        player.apply_client_move(
            server,
            ClientMove {
                position: None,
                yaw: None,
                pitch: None,
                head_yaw: None,
                on_ground: ground.on_ground,
                horizontal_collision: ground.horizontal_collision,
            },
        );
    }
}
