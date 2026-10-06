#[allow(clippy::wildcard_imports)]
use super::*;
use crate::entity::player::ClientMove;

impl JavaClient {
    pub fn handle_rotation(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        rotation: &SPlayerRotation,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        self.received_movement_this_tick
            .store(true, Ordering::Relaxed);
        player.apply_client_move(
            server,
            ClientMove {
                position: None,
                yaw: Some(rotation.yaw),
                pitch: Some(rotation.pitch),
                head_yaw: Some(rotation.yaw),
                on_ground: rotation.ground,
                horizontal_collision: rotation.horizontal_collision,
            },
        );
    }
}
