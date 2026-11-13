#[allow(clippy::wildcard_imports)]
use super::*;
use crate::entity::player::ClientMove;

impl JavaClient {
    pub fn handle_position(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &SPlayerPosition,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        // A movement packet was received this tick — tracked for SClientTickEnd zeroing.
        self.received_movement_this_tick
            .store(true, Ordering::Relaxed);
        player.apply_client_move(
            server,
            ClientMove {
                position: Some(packet.position),
                yaw: None,
                pitch: None,
                head_yaw: None,
                on_ground: packet.collision & FLAG_ON_GROUND != 0,
                horizontal_collision: packet.collision & FLAG_HORIZONTAL_COLLISION != 0,
            },
        );
    }

    pub fn handle_position_rotation(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &SPlayerPositionRotation,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        self.received_movement_this_tick
            .store(true, Ordering::Relaxed);
        player.apply_client_move(
            server,
            ClientMove {
                position: Some(packet.position),
                yaw: Some(packet.yaw),
                pitch: Some(packet.pitch),
                head_yaw: Some(packet.yaw),
                on_ground: packet.collision & FLAG_ON_GROUND != 0,
                horizontal_collision: packet.collision & FLAG_HORIZONTAL_COLLISION != 0,
            },
        );
    }
}
