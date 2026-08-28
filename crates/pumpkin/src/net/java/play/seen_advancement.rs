#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_seen_advancement(&self, player: &Arc<Player>, packet: &SSeenAdvancement) {
        if let SSeenAdvancement::OpenTab(tab) = packet {
            let advancement = Advancement::from_minecraft_name(&tab.to_string());
            if advancement.is_some() {
                player
                    .advancements
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .set_selected_tab(advancement);
            }
        }
    }
}
