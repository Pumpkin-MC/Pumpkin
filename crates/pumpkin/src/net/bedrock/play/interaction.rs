#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_interaction(
        &self,
        _player: &Arc<Player>,
        packet: SInteract,
        _server: &Arc<Server>,
    ) {
        if matches!(packet.action, Action::OpenInventory) {
            if self.inventory_opened.load(Ordering::Relaxed) {
                return;
            }
            self.inventory_opened.store(true, Ordering::Relaxed);
            self.enqueue_client_packet(&CContainerOpen {
                container_id: 0,
                container_type: 0xff,
                position: BlockPos::ZERO,
                target_entity_id: VarLong(-1),
            })
            .await;
        }
    }
}
