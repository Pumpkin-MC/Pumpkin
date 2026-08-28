#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_mob_equipment(
        &self,
        _server: &Arc<Server>,
        player: &Arc<Player>,
        packet: SMobEquipment,
    ) {
        player.update_last_action_time();
        let slot = packet.selected_slot;
        if slot >= 9 {
            return;
        }
        let previous_slot = player.inventory.get_selected_slot();
        let server = player.world().server.upgrade();
        if let Some(server) = server {
            let mut event = PlayerItemHeldEvent::new(player.clone(), previous_slot, slot);
            server.plugin_manager.fire(&server, &mut event).await;
            let is_cancelled = {
                use crate::plugin::Cancellable;
                event.cancelled()
            };
            if is_cancelled {
                self.enqueue_client_packet(&CPlayerHotbar {
                    selected_slot: VarUInt(previous_slot as u32),
                    container_id: 0,
                    should_select_slot: true,
                })
                .await;
                return;
            }
        }

        let inv = player.inventory();
        inv.set_selected_slot(slot);
        let stack = inv.held_item();
        let equipment = &[(EquipmentSlot::MAIN_HAND, stack)];
        player.living_entity.send_equipment_changes(equipment);
    }
}
