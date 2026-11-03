#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub fn handle_interaction(&self, player: &Arc<Player>, packet: &SInteract) {
        match packet.action {
            Action::OpenInventory => {
                if player.open_vehicle_inventory() {
                    return;
                }
                if self.inventory_opened.load(Ordering::Relaxed) {
                    return;
                }
                self.inventory_opened.store(true, Ordering::Relaxed);
                self.try_enqueue_client_packet(&CContainerOpen {
                    container_id: 0,
                    container_type: 0xff,
                    position: BlockPos::ZERO,
                    target_entity_id: VarLong(-1),
                });
            }
            Action::StopRiding => {
                let vehicle = player
                    .get_entity()
                    .vehicle
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                if let Some(vehicle) = vehicle {
                    vehicle.get_entity().remove_passenger(player.entity_id());
                }
            }
            _ => {}
        }
    }
}
