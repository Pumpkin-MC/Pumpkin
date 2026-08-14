#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_paddle_boat(&self, player: &Arc<Player>, packet: SPaddleBoat) {
        let vehicle = player.get_entity().vehicle.lock().await.clone();
        if let Some(vehicle) = vehicle {
            vehicle
                .set_paddle_state(packet.left_paddle, packet.right_paddle)
                .await;
        }
    }
}
