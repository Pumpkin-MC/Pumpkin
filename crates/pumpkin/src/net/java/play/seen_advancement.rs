#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_seen_advancement(&self, player: &Arc<Player>, packet: SSeenAdvancement) {
        if let SSeenAdvancement::OpenTab(tab) = packet {
            let advancement = Advancement::from_minecraft_name(&tab.to_string());
            if advancement.is_some() {
                player
                    .advancements
                    .lock()
                    .await
                    .set_selected_tab(advancement)
                    .await;
            }
        }
    }
}
