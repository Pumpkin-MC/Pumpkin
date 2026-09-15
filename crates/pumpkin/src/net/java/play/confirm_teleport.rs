#[allow(clippy::wildcard_imports)]
use super::*;
use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
enum PendingTeleportConfirmation {
    Latest(Vector3<f64>),
    Superseded,
    Unknown,
}

fn confirm_pending_teleport(
    pending: &mut VecDeque<(VarInt, Vector3<f64>)>,
    confirmed_id: VarInt,
) -> PendingTeleportConfirmation {
    let Some(confirmed_index) = pending.iter().position(|(id, _)| *id == confirmed_id) else {
        return PendingTeleportConfirmation::Unknown;
    };
    let confirmed_latest = confirmed_index + 1 == pending.len();
    let confirmed_position = pending[confirmed_index].1;

    // Confirming a newer teleport also supersedes every older outstanding one.
    pending.drain(..=confirmed_index);

    if confirmed_latest {
        PendingTeleportConfirmation::Latest(confirmed_position)
    } else {
        PendingTeleportConfirmation::Superseded
    }
}

impl JavaClient {
    pub fn handle_confirm_teleport(&self, player: &Player, confirm_teleport: &SConfirmTeleport) {
        let mut awaiting_teleports = player
            .awaiting_teleports
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if awaiting_teleports.is_empty() {
            debug!(
                player = %player.gameprofile.name,
                teleport_id = confirm_teleport.teleport_id.0,
                "Ignoring teleport confirmation without a pending teleport"
            );
            return;
        }

        match confirm_pending_teleport(&mut awaiting_teleports, confirm_teleport.teleport_id) {
            PendingTeleportConfirmation::Latest(position) => {
                // Only the newest confirmed teleport controls the server position. An older
                // confirmation must not roll back a newer teleport that is still pending.
                player.get_entity().set_pos(position);
            }
            PendingTeleportConfirmation::Superseded => {}
            PendingTeleportConfirmation::Unknown => {
                let pending_ids: Vec<i32> = awaiting_teleports.iter().map(|(id, _)| id.0).collect();
                debug!(
                    player = %player.gameprofile.name,
                    teleport_id = confirm_teleport.teleport_id.0,
                    ?pending_ids,
                    "Ignoring unexpected teleport confirmation"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position(x: f64) -> Vector3<f64> {
        Vector3::new(x, 0.0, 0.0)
    }

    #[test]
    fn accepts_an_older_pending_teleport_without_clearing_newer_ones() {
        let mut pending = VecDeque::from([(VarInt(1), position(1.0)), (VarInt(2), position(2.0))]);

        assert_eq!(
            confirm_pending_teleport(&mut pending, VarInt(1)),
            PendingTeleportConfirmation::Superseded
        );
        assert_eq!(pending, VecDeque::from([(VarInt(2), position(2.0))]));
    }

    #[test]
    fn confirming_the_newest_teleport_supersedes_older_ones() {
        let mut pending = VecDeque::from([(VarInt(1), position(1.0)), (VarInt(2), position(2.0))]);

        assert_eq!(
            confirm_pending_teleport(&mut pending, VarInt(2)),
            PendingTeleportConfirmation::Latest(position(2.0))
        );
        assert!(pending.is_empty());
    }

    #[test]
    fn keeps_pending_teleports_after_an_unknown_confirmation() {
        let mut pending = VecDeque::from([(VarInt(2), position(2.0))]);

        assert_eq!(
            confirm_pending_teleport(&mut pending, VarInt(1)),
            PendingTeleportConfirmation::Unknown
        );
        assert_eq!(pending, VecDeque::from([(VarInt(2), position(2.0))]));
    }
}
