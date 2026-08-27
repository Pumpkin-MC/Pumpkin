#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_rotation(&self, player: &Player, rotation: SPlayerRotation) {
        if !player.has_client_loaded() {
            return;
        }
        if !rotation.yaw.is_finite() || !rotation.pitch.is_finite() {
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                [],
            ))
            .await;
            return;
        }
        let entity = &player.get_entity();
        entity.on_ground.store(rotation.ground, Ordering::Relaxed);
        entity.set_rotation(
            wrap_degrees(rotation.yaw) % 360.0,
            wrap_degrees(rotation.pitch),
        );
        // Send the new position to all other players.
        let entity_id = entity.entity_id;
        let yaw = (entity.yaw.load() * 256.0 / 360.0).rem_euclid(256.0);
        let pitch = (entity.pitch.load() * 256.0 / 360.0).rem_euclid(256.0);
        // let head_yaw = modulus(entity.head_yaw * 256.0 / 360.0, 256.0);

        let world = entity.world.load_full();
        let je_packet =
            CUpdateEntityRot::new(entity_id.into(), yaw as u8, pitch as u8, rotation.ground);

        let pos = entity.pos.load();

        let be_packet = CMovePlayer::new(
            VarULong(entity_id as u64),
            Vector3::new(
                pos.x as f32,
                pos.y as f32 + player.get_entity().entity_type.eye_height,
                pos.z as f32,
            ),
            entity.pitch.load(),
            entity.yaw.load(),
            entity.yaw.load(),
            CMovePlayer::MODE_ROTATION,
            rotation.ground,
            VarULong(0),
            0,
            0,
            VarULong(0),
        );

        world.broadcast_packet_except_editioned(&[player.gameprofile.id], &je_packet, &be_packet);

        let je_packet = CHeadRot::new(entity_id.into(), yaw as u8);
        world.broadcast_packet_except(&[player.gameprofile.id], &je_packet);
    }
}
