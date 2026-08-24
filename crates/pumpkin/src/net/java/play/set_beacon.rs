#[allow(clippy::wildcard_imports)]
use super::*;
use crate::block::entities::beacon::BeaconBlockEntity;
use pumpkin_inventory::beacon_screen_handler::BeaconScreenHandler;
use pumpkin_protocol::java::server::play::SSetBeacon;
use pumpkin_world::inventory::Inventory;

impl JavaClient {
    pub async fn handle_set_beacon(&self, player: &Arc<Player>, packet: &SSetBeacon) {
        let screen_handler_lock = player.current_screen_handler.lock().await;
        let mut screen_handler = screen_handler_lock.lock().await;

        let Some(beacon_handler) = screen_handler
            .as_any_mut()
            .downcast_mut::<BeaconScreenHandler>()
        else {
            debug!(
                "Player {} interacted with invalid menu (expected beacon)",
                player.gameprofile.name
            );
            return;
        };

        let Some(beacon_entity) = beacon_handler
            .inventory
            .as_any()
            .downcast_ref::<BeaconBlockEntity>()
        else {
            return;
        };

        // Check if payment slot has an item
        if beacon_entity.payment.lock().await.is_empty() {
            return;
        }

        let levels = beacon_entity.levels.load(Ordering::Relaxed);
        let primary_id = packet.primary_effect.map(|v| v.0);
        let secondary_id = packet.secondary_effect.map(|v| v.0);

        if !BeaconBlockEntity::validate_effects(primary_id, secondary_id, levels) {
            warn!(
                "Player {} tried to set invalid beacon effects: primary {:?}, secondary {:?}",
                player.gameprofile.name, primary_id, secondary_id
            );
            self.kick(TextComponent::translate(
                "multiplayer.disconnect.generic",
                &[],
            ))
            .await;
            return;
        }

        beacon_entity
            .primary_effect
            .store(primary_id.unwrap_or(-1), Ordering::Relaxed);
        beacon_entity
            .secondary_effect
            .store(secondary_id.unwrap_or(-1), Ordering::Relaxed);

        // Remove 1 item from payment slot
        beacon_entity.remove_stack_specific(0, 1).await;
        beacon_entity.mark_dirty();

        screen_handler.sync_state().await;

        info!(
            "Player {} updated beacon effects: primary {:?}, secondary {:?}",
            player.gameprofile.name, primary_id, secondary_id
        );
    }
}
