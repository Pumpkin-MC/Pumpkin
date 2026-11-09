#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_sign_update(&self, player: &Player, sign_data: &SUpdateSign<'_>) {
        // `ServerGamePacketListenerImpl#updateSignText` resets the action time before validating.
        player.update_last_action_time();

        let world = player.get_entity().world.load_full();
        // Vanilla checks the chunk before looking up the block entity. The reusable
        // `may_update_text` gate repeats this for the Bedrock path, which has no pre-check.
        if !world
            .level
            .is_chunk_loaded(&sign_data.location.chunk_position())
        {
            return;
        }

        let Some(block_entity) = world.get_block_entity(&sign_data.location) else {
            return;
        };
        let Some(sign_entity) =
            crate::block::entities::sign::SignEntityRef::from_block_entity(&*block_entity)
        else {
            return;
        };
        if !sign_entity.may_update_text(&world, player) {
            warn!(
                "Player {} just tried to change non-editable sign",
                player.gameprofile.name
            );
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

        text.set_client_messages(&new_messages);
        *sign_entity
            .currently_editing_player()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        world.update_block_entity(&block_entity);
    }
}
