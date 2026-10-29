#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub fn select_hotbar_slot(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        slot: u8,
        container_id: u8,
    ) {
        player.update_last_action_time();
        if slot >= 9 {
            return;
        }
        let previous_slot = player.inventory.get_selected_slot();
        let mut event = PlayerItemHeldEvent::new(player.clone(), previous_slot, slot);
        server.plugin_manager.fire_blocking(server, &mut event);
        if event.cancelled {
            self.try_enqueue_client_packet(&CPlayerHotbar {
                selected_slot: VarUInt(u32::from(previous_slot)),
                container_id,
                should_select_slot: true,
            });
            return;
        }

        let inv = player.inventory();
        inv.set_selected_slot(slot);
        let stack = inv.held_item();
        let equipment = &[(EquipmentSlot::MAIN_HAND, stack)];
        player.living_entity.send_equipment_changes(equipment);
        player.update_bedrock_shield_sneaking();
    }

    pub fn handle_mob_equipment(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        packet: &SMobEquipment,
    ) {
        self.select_hotbar_slot(server, player, packet.selected_slot, packet.container_id);
    }
}
