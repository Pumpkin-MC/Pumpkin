#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_sign_update(&self, player: &Player, sign_data: SUpdateSign<'_>) {
        let world = player.get_entity().world.load_full();
        let Some(block_entity) = world.get_block_entity(&sign_data.location) else {
            return;
        };
        let Some(sign_entity) = block_entity.as_any().downcast_ref::<SignBlockEntity>() else {
            return;
        };
        if sign_entity.is_waxed.load(Ordering::Relaxed) {
            return;
        }

        let lines = vec![
            sign_data.line_1.to_string(),
            sign_data.line_2.to_string(),
            sign_data.line_3.to_string(),
            sign_data.line_4.to_string(),
        ];

        if let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id) {
            let mut event = crate::plugin::api::events::block::sign_change::SignChangeEvent::new(
                player_arc,
                sign_data.location,
                lines.clone(),
            );
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire(&server, &mut event).await;
            }
            if event.cancelled {
                return;
            }
        }

        let text = if sign_data.is_front_text {
            &sign_entity.front_text
        } else {
            &sign_entity.back_text
        };

        *text
            .messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = [
            sign_data.line_1.into(),
            sign_data.line_2.into(),
            sign_data.line_3.into(),
            sign_data.line_4.into(),
        ];
        *sign_entity.currently_editing_player.lock().await = None;
        world.update_block_entity(&block_entity);
    }
}
