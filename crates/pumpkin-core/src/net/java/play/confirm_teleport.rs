#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::vector3::Vector3;

fn accept_teleport(
    pending: &mut Option<(VarInt, Vector3<f64>)>,
    id: VarInt,
    last_id: i32,
) -> Result<Option<Vector3<f64>>, &'static str> {
    if id.0 != last_id
        || pending
            .as_ref()
            .is_some_and(|(expected, _)| *expected != id)
    {
        return Ok(None);
    }
    pending
        .take()
        .map(|(_, position)| Some(position))
        .ok_or("Send Teleport confirm, but we did not teleport")
}

impl JavaClient {
    pub fn handle_confirm_teleport(&self, player: &Player, confirm_teleport: &SConfirmTeleport) {
        let result = {
            let mut pending = player
                .awaiting_teleport
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            accept_teleport(
                &mut pending,
                confirm_teleport.teleport_id,
                player
                    .teleport_id_count
                    .load(std::sync::atomic::Ordering::Relaxed),
            )
        };
        match result {
            Ok(Some(position)) => player.get_entity().set_pos(position),
            Ok(None) => {}
            Err(reason) => self.try_kick(&TextComponent::text(reason)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mismatched_acks_are_ignored_and_matching_acks_require_a_pending_teleport() {
        let destination = Vector3::new(5.5, 64.0, 0.5);
        let mut pending = Some((VarInt(3), destination));
        for id in [1, 2, 4] {
            assert_eq!(accept_teleport(&mut pending, VarInt(id), 3), Ok(None));
            assert_eq!(pending, Some((VarInt(3), destination)));
        }
        assert_eq!(
            accept_teleport(&mut pending, VarInt(3), 3),
            Ok(Some(destination))
        );
        assert!(pending.is_none());
        assert_eq!(accept_teleport(&mut pending, VarInt(2), 3), Ok(None));
        assert!(accept_teleport(&mut pending, VarInt(3), 3).is_err());
    }
}
