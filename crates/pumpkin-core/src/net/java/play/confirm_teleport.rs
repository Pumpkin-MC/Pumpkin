#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_confirm_teleport(&self, player: &Player, confirm_teleport: &SConfirmTeleport) {
        enum TeleportResult {
            Success,
            NotTeleporting,
            InvalidLook,
        }

        let result = {
            let mut awaiting_teleport = player
                .awaiting_teleport
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            // A seat correction can be followed by a dismount before either
            // acknowledgement arrives. Only the newer teleport owns placement.
            let latest = awaiting_teleport.as_ref().map_or_else(
                || {
                    player
                        .teleport_id_count
                        .load(std::sync::atomic::Ordering::Acquire)
                },
                |(id, _)| id.0,
            );
            if confirm_teleport.teleport_id.0 != latest {
                return;
            }
            if let Some((id, position)) = awaiting_teleport.as_ref() {
                if id == &confirm_teleport.teleport_id {
                    // We should set the position now to what we requested in the teleport packet.
                    // This may fix issues when the client sends the position while being teleported.
                    let entity = player.get_entity();
                    let riding_look = player.riding_look_confirmation.swap(None) == Some(*id);
                    let position = *position;
                    *awaiting_teleport = None;
                    if riding_look
                        && (!confirm_teleport.yaw.is_finite()
                            || !confirm_teleport.pitch.is_finite())
                    {
                        TeleportResult::InvalidLook
                    } else {
                        entity.set_pos(position);
                        if riding_look {
                            use pumpkin_util::math::wrap_degrees;
                            entity.set_rotation(
                                wrap_degrees(confirm_teleport.yaw),
                                wrap_degrees(confirm_teleport.pitch),
                            );
                        }
                        TeleportResult::Success
                    }
                } else {
                    return;
                }
            } else {
                TeleportResult::NotTeleporting
            }
        };

        match result {
            TeleportResult::Success => {}
            TeleportResult::NotTeleporting => {
                self.try_kick(&TextComponent::text(
                    "Send Teleport confirm, but we did not teleport",
                ));
            }
            TeleportResult::InvalidLook => {
                self.try_kick(&TextComponent::translate(
                    "multiplayer.disconnect.invalid_player_movement",
                    [],
                ));
            }
        }
    }
}
