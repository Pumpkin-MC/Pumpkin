#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_set_held_item(&self, server: &Arc<Server>, player: &Player, held: &SSetHeldItem) {
        player.update_last_action_time();
        let slot = held.slot;
        if !(0..=8).contains(&slot) {
            self.try_kick(&TextComponent::text("Invalid held slot"));
            return;
        }
        let slot = slot as u8;
        let previous_slot = player.inventory.get_selected_slot();
        let Some(player_arc) = player.world().get_player_by_uuid(player.gameprofile.id) else {
            return;
        };
        let mut event = PlayerItemHeldEvent::new(player_arc, previous_slot, slot);
        server.plugin_manager.fire_blocking(server, &mut event);
        if event.cancelled {
            player.try_send_client_packet(&CSetSelectedSlot::new(previous_slot as i8));
            return;
        }

        let inv = player.inventory();
        inv.set_selected_slot(slot);
        let stack = inv.held_item();
        let equipment = &[(EquipmentSlot::MAIN_HAND, stack)];
        player.living_entity.send_equipment_changes(equipment);
    }
}
