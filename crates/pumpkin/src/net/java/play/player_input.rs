#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_player_input(
        &self,
        player: &Arc<Player>,
        input: SPlayerInput,
        server: &Arc<Server>,
    ) {
        let mut input_event =
            crate::plugin::api::events::player::player_input::PlayerInputEvent::new(
                player.clone(),
                format!("{:b}", input.input),
            );
        server.plugin_manager.fire(server, &mut input_event).await;
        if input_event.cancelled {
            return;
        }

        player.last_input.store(input.input, Ordering::Relaxed);

        let sneak = input.input & SPlayerInput::SNEAK != 0;
        if sneak
            && player.gamemode.load() == GameMode::Spectator
            && player.camera_target_id.load().is_some()
        {
            player.camera_target_id.store(None);
            player
                .client
                .send_packet_now(&CSetCamera::new(player.entity_id().into()))
                .await;
        }

        if player.get_entity().is_sneaking() != sneak {
            send_cancellable! {{
                server;
                PlayerToggleSneakEvent::new(player.clone(), sneak);
                'after: {
                    player.get_entity().set_sneaking(event.is_sneaking).await;
                    if event.is_sneaking {
                        let vehicle = player.get_entity().vehicle.lock().await.clone();
                        if let Some(vehicle) = vehicle {
                            vehicle
                                .get_entity()
                                .remove_passenger(player.entity_id())
                                .await;
                        }
                    }
                }
            }}
        } else if sneak {
            let vehicle = player.get_entity().vehicle.lock().await.clone();
            if let Some(vehicle) = vehicle {
                vehicle
                    .get_entity()
                    .remove_passenger(player.entity_id())
                    .await;
            }
        }
    }
}
