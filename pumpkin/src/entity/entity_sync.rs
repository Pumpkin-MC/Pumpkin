use super::Entity;
use pumpkin_data::entity::EntityType;
use pumpkin_protocol::bedrock::client::{
    move_actor_delta::{
        CMoveActorDelta, MOVE_ACTOR_DELTA_FLAG_HAS_HEAD_YAW, MOVE_ACTOR_DELTA_FLAG_HAS_PITCH,
        MOVE_ACTOR_DELTA_FLAG_HAS_X, MOVE_ACTOR_DELTA_FLAG_HAS_Y, MOVE_ACTOR_DELTA_FLAG_HAS_YAW,
        MOVE_ACTOR_DELTA_FLAG_HAS_Z, MOVE_ACTOR_DELTA_FLAG_ON_GROUND,
    },
    move_player::CMovePlayer,
};
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::{
    CHeadRot, CUpdateEntityPos, CUpdateEntityPosRot, CUpdateEntityRot,
};
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering::Relaxed;

impl Entity {
    pub fn send_rotation(&self) {
        let yaw = self.yaw.load();
        let pitch = self.pitch.load();
        let chunk_pos = self.chunk_pos.load();

        // Broadcast the update packet.

        let yaw = (yaw * 256.0 / 360.0).rem_euclid(256.0) as u8;
        let pitch = (pitch * 256.0 / 360.0).rem_euclid(256.0) as u8;

        if yaw == self.last_sent_yaw.load(Relaxed) && pitch == self.last_sent_pitch.load(Relaxed) {
            return;
        }

        self.last_sent_yaw.store(yaw, Relaxed);
        self.last_sent_pitch.store(pitch, Relaxed);

        self.world.load().broadcast_to_chunk(
            chunk_pos,
            &CUpdateEntityRot::new(
                self.entity_id.into(),
                yaw,
                pitch,
                self.on_ground.load(Relaxed),
            ),
        );

        self.send_head_rot(yaw);
    }

    pub fn send_head_rot(&self, head_yaw: u8) {
        let chunk_pos = self.chunk_pos.load();
        if head_yaw == self.last_sent_head_yaw.load(Relaxed) {
            return;
        }
        self.last_sent_head_yaw.store(head_yaw, Relaxed);

        self.world
            .load()
            .broadcast_to_chunk(chunk_pos, &CHeadRot::new(self.entity_id.into(), head_yaw));
    }

    #[expect(clippy::too_many_lines)]
    pub fn send_pos_rot(&self) {
        let old = self.last_sent_pos.load();
        let new = self.pos.load();
        let chunk_pos = self.chunk_pos.load();

        let converted = Vector3::new(
            new.x.mul_add(4096.0, -(old.x * 4096.0)) as i16,
            new.y.mul_add(4096.0, -(old.y * 4096.0)) as i16,
            new.z.mul_add(4096.0, -(old.z * 4096.0)) as i16,
        );

        let yaw = self.yaw.load();

        let pitch = self.pitch.load();
        let yaw = (yaw * 256.0 / 360.0).rem_euclid(256.0) as u8;
        let pitch = (pitch * 256.0 / 360.0).rem_euclid(256.0) as u8;

        // Only broadcast when position or rotation has actually changed.
        let pos_changed = converted.x != 0 || converted.y != 0 || converted.z != 0;
        let rot_changed =
            yaw != self.last_sent_yaw.load(Relaxed) || pitch != self.last_sent_pitch.load(Relaxed);

        if !pos_changed && !rot_changed {
            return;
        }

        self.last_sent_pos.store(new);
        self.last_sent_yaw.store(yaw, Relaxed);
        self.last_sent_pitch.store(pitch, Relaxed);

        // Dynamically pick the most efficient packet
        if pos_changed && rot_changed {
            let je_packet = CUpdateEntityPosRot::new(
                self.entity_id.into(),
                Vector3::new(converted.x, converted.y, converted.z),
                yaw,
                pitch,
                self.on_ground.load(Relaxed),
            );
            if self.entity_type == &EntityType::PLAYER {
                self.world.load().broadcast_to_chunk_editioned_sync(
                    chunk_pos,
                    &je_packet,
                    &CMovePlayer::new(
                        VarULong(self.entity_id as u64),
                        Vector3::new(new.x as f32, new.y as f32, new.z as f32),
                        self.pitch.load(),
                        self.yaw.load(),
                        self.yaw.load(),
                        CMovePlayer::MODE_NORMAL,
                        self.on_ground.load(Relaxed),
                        VarULong(0),
                        0,
                        0,
                        VarULong(0),
                    ),
                );
            } else {
                let mut flags = MOVE_ACTOR_DELTA_FLAG_HAS_X
                    | MOVE_ACTOR_DELTA_FLAG_HAS_Y
                    | MOVE_ACTOR_DELTA_FLAG_HAS_Z
                    | MOVE_ACTOR_DELTA_FLAG_HAS_PITCH
                    | MOVE_ACTOR_DELTA_FLAG_HAS_YAW
                    | MOVE_ACTOR_DELTA_FLAG_HAS_HEAD_YAW;
                if self.on_ground.load(Relaxed) {
                    flags |= MOVE_ACTOR_DELTA_FLAG_ON_GROUND;
                }
                self.world.load().broadcast_to_chunk_editioned_sync(
                    chunk_pos,
                    &je_packet,
                    &CMoveActorDelta::new(
                        VarULong(self.entity_id as u64),
                        flags,
                        new.x as f32,
                        new.y as f32,
                        new.z as f32,
                        pitch,
                        yaw,
                        yaw,
                    ),
                );
            }
        } else if pos_changed {
            let je_packet = CUpdateEntityPos::new(
                self.entity_id.into(),
                Vector3::new(converted.x, converted.y, converted.z),
                self.on_ground.load(Relaxed),
            );
            if self.entity_type == &EntityType::PLAYER {
                self.world.load().broadcast_to_chunk_editioned_sync(
                    chunk_pos,
                    &je_packet,
                    &CMovePlayer::new(
                        VarULong(self.entity_id as u64),
                        Vector3::new(new.x as f32, new.y as f32, new.z as f32),
                        self.pitch.load(),
                        self.yaw.load(),
                        self.yaw.load(),
                        CMovePlayer::MODE_NORMAL,
                        self.on_ground.load(Relaxed),
                        VarULong(0),
                        0,
                        0,
                        VarULong(0),
                    ),
                );
            } else {
                let mut flags = MOVE_ACTOR_DELTA_FLAG_HAS_X
                    | MOVE_ACTOR_DELTA_FLAG_HAS_Y
                    | MOVE_ACTOR_DELTA_FLAG_HAS_Z;
                if self.on_ground.load(Relaxed) {
                    flags |= MOVE_ACTOR_DELTA_FLAG_ON_GROUND;
                }

                self.world.load().broadcast_to_chunk_editioned_sync(
                    chunk_pos,
                    &je_packet,
                    &CMoveActorDelta::new(
                        VarULong(self.entity_id as u64),
                        flags,
                        new.x as f32,
                        new.y as f32,
                        new.z as f32,
                        0,
                        0,
                        0,
                    ),
                );
            }
        } else if rot_changed {
            let je_packet = CUpdateEntityRot::new(
                self.entity_id.into(),
                yaw,
                pitch,
                self.on_ground.load(Relaxed),
            );
            if self.entity_type == &EntityType::PLAYER {
                self.world.load().broadcast_to_chunk_editioned_sync(
                    chunk_pos,
                    &je_packet,
                    &CMovePlayer::new(
                        VarULong(self.entity_id as u64),
                        Vector3::new(new.x as f32, new.y as f32, new.z as f32),
                        self.pitch.load(),
                        self.yaw.load(),
                        self.yaw.load(),
                        CMovePlayer::MODE_ROTATION,
                        self.on_ground.load(Relaxed),
                        VarULong(0),
                        0,
                        0,
                        VarULong(0),
                    ),
                );
            } else {
                let mut flags = MOVE_ACTOR_DELTA_FLAG_HAS_PITCH
                    | MOVE_ACTOR_DELTA_FLAG_HAS_YAW
                    | MOVE_ACTOR_DELTA_FLAG_HAS_HEAD_YAW;
                if self.on_ground.load(Relaxed) {
                    flags |= MOVE_ACTOR_DELTA_FLAG_ON_GROUND;
                }
                self.world.load().broadcast_to_chunk_editioned_sync(
                    chunk_pos,
                    &je_packet,
                    &CMoveActorDelta::new(
                        VarULong(self.entity_id as u64),
                        flags,
                        new.x as f32,
                        new.y as f32,
                        new.z as f32,
                        pitch,
                        yaw,
                        yaw,
                    ),
                );
            }
        }
        self.send_head_rot(yaw);
    }

    pub fn update_last_pos(&self) -> Vector3<f64> {
        let pos = self.pos.load();
        let old = self.last_pos.load();
        self.movement.store(pos - old);
        self.last_pos.store(pos);
        old
    }

    pub fn send_pos(&self) {
        let old = self.last_sent_pos.load();
        let new = self.pos.load();
        let chunk_pos = self.chunk_pos.load();

        let converted = Vector3::new(
            new.x.mul_add(4096.0, -(old.x * 4096.0)) as i16,
            new.y.mul_add(4096.0, -(old.y * 4096.0)) as i16,
            new.z.mul_add(4096.0, -(old.z * 4096.0)) as i16,
        );

        // Only broadcast when position has actually changed.
        if converted.x == 0 && converted.y == 0 && converted.z == 0 {
            return;
        }

        self.last_sent_pos.store(new);

        let je_packet = CUpdateEntityPos::new(
            self.entity_id.into(),
            Vector3::new(converted.x, converted.y, converted.z),
            self.on_ground.load(Relaxed),
        );

        if self.entity_type == &EntityType::PLAYER {
            self.world.load().broadcast_to_chunk_editioned_sync(
                chunk_pos,
                &je_packet,
                &CMovePlayer::new(
                    VarULong(self.entity_id as u64),
                    Vector3::new(new.x as f32, new.y as f32, new.z as f32),
                    self.pitch.load(),
                    self.yaw.load(),
                    self.yaw.load(),
                    CMovePlayer::MODE_NORMAL,
                    self.on_ground.load(Relaxed),
                    VarULong(0),
                    0,
                    0,
                    VarULong(0),
                ),
            );
        } else {
            let mut flags = MOVE_ACTOR_DELTA_FLAG_HAS_X
                | MOVE_ACTOR_DELTA_FLAG_HAS_Y
                | MOVE_ACTOR_DELTA_FLAG_HAS_Z;
            if self.on_ground.load(Relaxed) {
                flags |= MOVE_ACTOR_DELTA_FLAG_ON_GROUND;
            }

            self.world.load().broadcast_to_chunk_editioned_sync(
                chunk_pos,
                &je_packet,
                &CMoveActorDelta::new(
                    VarULong(self.entity_id as u64),
                    flags,
                    new.x as f32,
                    new.y as f32,
                    new.z as f32,
                    0,
                    0,
                    0,
                ),
            );
        }
    }
}
