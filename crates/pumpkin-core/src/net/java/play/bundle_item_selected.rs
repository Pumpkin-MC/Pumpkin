#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_bundle_item_selected(&self, player: &Arc<Player>, packet: &SBundleItemSelected) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        let selected_item_index = packet.selected_item_index.0;
        if selected_item_index < 0 && selected_item_index != -1 {
            self.try_kick(&TextComponent::text("Invalid selected item index"));
            return;
        }

        let screen_handler_arc = player
            .current_screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let mut screen_handler = screen_handler_arc
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        screen_handler.set_selected_bundle_item_index(packet.slot_id.0, selected_item_index);
    }
}
