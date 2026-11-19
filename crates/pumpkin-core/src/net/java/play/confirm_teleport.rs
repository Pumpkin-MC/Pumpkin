#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::vector3::Vector3;

fn confirm_position_correction(
    pending: &mut Option<(VarInt, Vector3<f64>)>,
    id: VarInt,
    current_position: Vector3<f64>,
) -> Result<bool, &'static str> {
    let Some((expected, position)) = pending.as_ref() else {
        return Err("Send Teleport confirm, but we did not teleport");
    };
    if *expected != id {
        return Err("Wrong teleport id");
    }
    let moved = current_position != *position;
    *pending = None;
    Ok(moved)
}

impl JavaClient {
    pub fn handle_confirm_teleport(&self, player: &Player, confirm_teleport: &SConfirmTeleport) {
        let result = {
            let mut awaiting_teleport = player
                .awaiting_teleport
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            confirm_position_correction(
                &mut awaiting_teleport,
                confirm_teleport.teleport_id,
                player.get_entity().pos.load(),
            )
        };

        match result {
            Ok(true) => player.sync_position_after_external_move(),
            Ok(false) => {}
            Err(reason) => self.try_kick(&TextComponent::text(reason)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acknowledgement_keeps_external_motion_and_requests_the_latest_position() {
        let sent = Vector3::new(1.0, 64.0, 1.0);
        let current = Vector3::new(1.25, 64.0, 1.0);
        let mut pending = Some((VarInt(12), sent));
        assert_eq!(
            confirm_position_correction(&mut pending, VarInt(11), current),
            Err("Wrong teleport id")
        );
        assert_eq!(pending, Some((VarInt(12), sent)));
        assert_eq!(
            confirm_position_correction(&mut pending, VarInt(12), current),
            Ok(true)
        );
        assert!(pending.is_none());
        pending = Some((VarInt(13), current));
        assert_eq!(
            confirm_position_correction(&mut pending, VarInt(13), current),
            Ok(false)
        );
    }
}
