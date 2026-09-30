#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_player_command(
        &self,
        player: &Arc<Player>,
        command: &SPlayerCommand,
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
                    send_cancellable_blocking! {{
                        server;
                        PlayerToggleSprintEvent::new(player.clone(), true);
                        'after: {
                            player.set_sprinting(event.is_sprinting);
                            player.update_player_pose();
                        }
                    }}
                }
            }
            Action::StopSprinting => {
                if entity.is_sprinting() {
                    send_cancellable_blocking! {{
                        server;
                        PlayerToggleSprintEvent::new(player.clone(), false);
                        'after: {
                            player.set_sprinting(event.is_sprinting);
                            player.update_player_pose();
                        }
                    }}
                }
            }
            Action::LeaveBed => player.wake_up(),

            Action::StartHorseJump | Action::StopHorseJump | Action::OpenVehicleInventory => {
                debug!("todo");
            }
            Action::StartFlyingElytra => {
                if player.can_start_fall_flying() {
                    let mut event = crate::plugin::api::events::entity::entity_toggle_glide::EntityToggleGlideEvent::new(
                        entity.entity_id,
                        true,
                    );
                    server.plugin_manager.fire_blocking(server, &mut event);
                    if event.cancelled || !event.is_gliding {
                        player.stop_fall_flying();
                    } else {
                        player.start_fall_flying();
                    }
                } else {
                    player.stop_fall_flying();
                }
            }
            // <= 1.21.5
            Action::StartSneaking | Action::StopSneaking => {
                self.handle_player_input(
                    player,
                    &SPlayerInput {
                        input: SPlayerInput::SNEAK,
                    },
                    server,
                );
            }
        }
    }
}
