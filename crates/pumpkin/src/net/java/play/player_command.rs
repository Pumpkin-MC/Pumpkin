#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_player_command(
        &self,
        player: &Arc<Player>,
        command: SPlayerCommand,
        server: &Arc<Server>,
    ) {
        if command.entity_id != player.entity_id().into() {
            return;
        }
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        let entity = &player.get_entity();
        match command.action {
            Action::StartSprinting => {
                if !entity.is_sprinting() {
                    send_cancellable! {{
                        server;
                        PlayerToggleSprintEvent::new(player.clone(), true);
                        'after: {
                            player.get_entity().set_sprinting(event.is_sprinting).await;
                        }
                    }}
                }
            }
            Action::StopSprinting => {
                if entity.is_sprinting() {
                    send_cancellable! {{
                        server;
                        PlayerToggleSprintEvent::new(player.clone(), false);
                        'after: {
                            player.get_entity().set_sprinting(event.is_sprinting).await;
                        }
                    }}
                }
            }
            Action::LeaveBed => player.wake_up().await,

            Action::StartHorseJump | Action::StopHorseJump | Action::OpenVehicleInventory => {
                debug!("todo");
            }
            Action::StartFlyingElytra => {
                let fall_flying = entity.check_fall_flying();
                if entity.is_fall_flying() != fall_flying {
                    entity.set_fall_flying(fall_flying).await;
                }
            }
            // <= 1.21.5
            Action::StartSneaking | Action::StopSneaking => {
                self.handle_player_input(
                    player,
                    SPlayerInput {
                        input: SPlayerInput::SNEAK,
                    },
                    server,
                )
                .await;
            }
        }
    }
}
