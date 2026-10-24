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

        debug!(
            "Bundle item selected: Slot ID {}, Selected Item Index {}",
            packet.slot_id.0, selected_item_index
        );
    }
}
