#[allow(clippy::wildcard_imports)]
use super::*;
use crate::block::entities::crafter::CrafterBlockEntity;
use pumpkin_inventory::crafter_screen_handler::CrafterScreenHandler;
use pumpkin_protocol::java::server::play::SContainerSlotStateChanged;
use pumpkin_util::GameMode;

impl JavaClient {
    pub fn handle_container_slot_state_changed(
        &self,
        player: &Player,
        packet: &SContainerSlotStateChanged,
    ) {
        if player.gamemode.load() == GameMode::Spectator {
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

        if packet.container_id.0 != i32::from(screen_handler.sync_id()) {
            return;
        }

        let Some(crafter_handler) = screen_handler
            .as_any()
            .downcast_ref::<CrafterScreenHandler>()
        else {
            return;
        };

        let Ok(slot_id) = usize::try_from(packet.slot_id.0) else {
            return;
        };

        if let Some(crafter_entity) = crafter_handler
            .container
            .as_any()
            .downcast_ref::<CrafterBlockEntity>()
        {
            crafter_entity.set_slot_state(slot_id, packet.new_state);
            screen_handler.send_content_updates();
        }
    }
}
