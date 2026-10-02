#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_sign_update(&self, player: &Player, sign_data: &SUpdateSign<'_>) {
        let world = player.get_entity().world.load_full();
        let Some(block_entity) = world.get_block_entity(&sign_data.location) else {
            return;
        };
        let Some(sign_entity) =
            crate::block::entities::sign::SignEntityRef::from_block_entity(&*block_entity)
        else {
            return;
        };
        if sign_entity.is_waxed() {
            return;
        }

        let currently_editing = *sign_entity
            .currently_editing_player()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(editor_id) = currently_editing
            && editor_id != player.gameprofile.id
        {
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
                lines,
            );
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            if event.cancelled {
                return;
            }
        }

        let text = sign_entity.get_text(sign_data.is_front_text);

        let new_messages = [
            Box::<str>::from(sign_data.line_1),
            Box::<str>::from(sign_data.line_2),
            Box::<str>::from(sign_data.line_3),
            Box::<str>::from(sign_data.line_4),
        ];

        text.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone_from(&new_messages);
        // We do not filter anything, so the filtered copy has to track the raw one. Leaving it
        // empty makes it differ, which puts an empty `filtered_messages` on the wire and blanks
        // the sign for clients that render with text filtering enabled.
        *text
            .filtered_messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = new_messages;
        *sign_entity
            .currently_editing_player()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        world.update_block_entity(&block_entity);
    }
}
