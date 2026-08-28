#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_select_trade(&self, player: &Arc<Player>, packet: SSelectTrade) {
        let mut event = crate::plugin::api::events::inventory::trade_select::TradeSelectEvent::new(
            player.clone(),
            packet.selected_slot.0 as u8,
        );
        if let Some(server) = player.world().server.upgrade() {
            server.plugin_manager.fire(&server, &mut event).await;
        }
        if event.cancelled {
            return;
        }

        let screen_handler = player
            .current_screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let mut screen_handler = screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !screen_handler.can_use(player.as_ref()) {
            return;
        }
        if let Some(merchant) = screen_handler
            .as_any_mut()
            .downcast_mut::<MerchantScreenHandler>()
        {
            merchant.set_selected_offer(packet.selected_slot.0 as usize);
        }
    }
}
