#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_select_trade(&self, player: &Arc<Player>, packet: SSelectTrade) {
        let screen_handler = player.current_screen_handler.lock().await;
        let mut screen_handler = screen_handler.lock().await;
        if let Some(merchant) = screen_handler
            .as_any_mut()
            .downcast_mut::<MerchantScreenHandler>()
        {
            merchant
                .set_selected_offer(packet.selected_slot.0 as usize)
                .await;
        }
    }
}
