#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_interaction(&self, player: &Arc<Player>, packet: SInteraction) {
        match packet.action {
            Action::OpenInventory => {
                if self.inventory_opened.load(Ordering::Relaxed) {
                    return;
                }
                self.inventory_opened.store(true, Ordering::Relaxed);
                self.enqueue_packet(&CContainerOpen {
                    container_id: 0,
                    container_type: 0xff,
                    position: BlockPos::ZERO,
                    target_entity_id: VarLong(-1),
                })
                .await;
            }
            // No longer used in newer versions
            Action::Attack => {
                let target_runtime_id = packet.target_runtime_id.0 as i32;
                let world = player.world();
                if let Some(target) = world.get_entity_by_id(target_runtime_id) {
                    player.attack(target).await;
                }
            }
            _ => {}
        }
    }
}
