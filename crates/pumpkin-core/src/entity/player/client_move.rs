use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::translation;
use pumpkin_protocol::bedrock::client::CMovePlayer;
use pumpkin_protocol::java::client::play::{
    CEntityPositionSync, CHeadRot, CPlayerPosition, CSetCamera, CUpdateEntityPos,
    CUpdateEntityPosRot, CUpdateEntityRot,
};
use pumpkin_protocol::java::server::play::SPlayerInput;
use pumpkin_util::GameMode;
use pumpkin_util::math::{vector3::Vector3, wrap_degrees};
use pumpkin_util::text::TextComponent;

use crate::entity::EntityBase;
use crate::net::{ClientPlatform, DisconnectReason};
use crate::plugin::player::player_input::PlayerInputEvent;
use crate::plugin::player::player_move::PlayerMoveEvent;
use crate::plugin::player::player_toggle_flight_event::PlayerToggleFlightEvent;
use crate::plugin::player::player_toggle_sneak_event::PlayerToggleSneakEvent;
use crate::plugin::player::player_toggle_sprint_event::PlayerToggleSprintEvent;
use crate::server::Server;
use crate::world::chunker;

use super::Player;
use super::statistics::StatisticCategory;

/// One client move this tick, from Java `ServerboundMovePlayerPacket` or Bedrock `PlayerAuthInput`.
#[derive(Clone, Copy)]
pub struct ClientMove {
    pub position: Option<Vector3<f64>>,
    pub yaw: Option<f32>,
    pub pitch: Option<f32>,
    pub head_yaw: Option<f32>,
    pub on_ground: bool,
    pub horizontal_collision: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientMoveOutcome {
    Applied,
    Rejected,
    Ignored,
}

impl Player {
    const fn clamp_horizontal(pos: f64) -> f64 {
        pos.clamp(-3.0E7, 3.0E7)
    }

    const fn clamp_vertical(pos: f64) -> f64 {
        pos.clamp(-2.0E7, 2.0E7)
    }

    fn packed_rot(degrees: f32) -> u8 {
        (degrees * 256.0 / 360.0).rem_euclid(256.0) as u8
    }

    /// Vanilla `ServerGamePacketListenerImpl.handlePlayerInput`
    pub fn apply_client_input(&self, server: &Arc<Server>, input: i8) {
        let Some(player) = self.world().get_player_by_uuid(self.gameprofile.id) else {
            return;
        };
        let mut input_event = PlayerInputEvent::new(player.clone(), format!("{input:b}"));
        server
            .plugin_manager
            .fire_blocking(server, &mut input_event);
        if input_event.cancelled {
            return;
        }

        self.last_input.store(input, Ordering::Relaxed);

        let sneak = input & SPlayerInput::SNEAK != 0;
        if sneak
            && self.gamemode.load() == GameMode::Spectator
            && self.camera_target_id.load().is_some()
        {
            self.camera_target_id.store(None);
            self.try_send_client_packet(&CSetCamera::new(self.entity_id().into()));
        }

        let entity = self.get_entity();
        if entity.is_sneaking() != sneak {
            send_cancellable_blocking! {{
                server;
                PlayerToggleSneakEvent::new(player, sneak);
                'after: {
                    entity.set_sneaking(event.is_sneaking);
                    if event.is_sneaking {
                        dismount_if_riding(entity);
                    }
                }
            }}
        } else if sneak {
            dismount_if_riding(entity);
        }
    }

    /// Vanilla `ServerboundPlayerCommandPacket` start/stop sprinting
    pub fn apply_sprint_input(&self, server: &Arc<Server>, sprinting: bool) {
        if self.get_entity().is_sprinting() == sprinting {
            return;
        }
        let Some(player) = self.world().get_player_by_uuid(self.gameprofile.id) else {
            return;
        };
        send_cancellable_blocking! {{
            server;
            PlayerToggleSprintEvent::new(player, sprinting);
            'after: {
                self.set_sprinting(event.is_sprinting);
                self.update_player_pose();
            }
        }}
    }

    /// Vanilla `ServerboundPlayerAbilitiesPacket` flying bit
    pub fn apply_flight_input(&self, server: &Arc<Server>, flying: bool) {
        let (is_flying, allow_flying) = {
            let abilities = self
                .abilities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (abilities.flying, abilities.allow_flying)
        };
        let new_flying = flying && allow_flying;
        if is_flying == new_flying {
            return;
        }
        let Some(player) = self.world().get_player_by_uuid(self.gameprofile.id) else {
            return;
        };
        send_cancellable_blocking! {{
            server;
            PlayerToggleFlightEvent::new(player, new_flying);
            'after: {
                if event.is_flying {
                    self.living_entity.fall_distance.store(0.0);
                }
                self.abilities.lock().unwrap_or_else(std::sync::PoisonError::into_inner).flying = event.is_flying;
                self.send_abilities_update();
            }
            'cancelled: {
                self.send_abilities_update();
            }
        }}
    }

    /// Snap the client back. Vanilla `ServerGamePacketListenerImpl.teleport` used when a move is refused.
    pub fn reject_client_move(&self) {
        let entity = self.get_entity();
        let position = entity.pos.load();
        let teleport_id = self.teleport_id_count.fetch_add(1, Ordering::Relaxed) + 1;
        *self
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some((teleport_id.into(), position));
        match self.client.as_ref() {
            ClientPlatform::Java(_) => {
                self.try_send_client_packet(&CPlayerPosition::new(
                    teleport_id.into(),
                    position,
                    Vector3::new(0.0, 0.0, 0.0),
                    entity.yaw.load(),
                    entity.pitch.load(),
                    Vec::new(),
                ));
            }
            ClientPlatform::Bedrock(client) => {
                self.send_bedrock_teleport(client);
            }
        }
    }

    /// Vanilla `handlePlayerPositionChange`. Does not write `Entity.velocity`; that stays server
    /// owned (`deltaMovement`). Client motion this tick is `Entity.movement`.
    #[allow(clippy::too_many_lines)]
    pub fn apply_client_move(
        &self,
        server: &Arc<Server>,
        requested: ClientMove,
    ) -> ClientMoveOutcome {
        if !self.has_client_loaded() {
            return ClientMoveOutcome::Ignored;
        }

        if let (Some(x), Some(y), Some(z)) = (
            requested.position.map(|p| p.x),
            requested.position.map(|p| p.y),
            requested.position.map(|p| p.z),
        ) && !(x.is_finite() && y.is_finite() && z.is_finite())
        {
            self.kick_invalid_movement();
            return ClientMoveOutcome::Rejected;
        }
        if requested.yaw.is_some_and(|yaw| !yaw.is_finite())
            || requested.pitch.is_some_and(|pitch| !pitch.is_finite())
            || requested.head_yaw.is_some_and(|yaw| !yaw.is_finite())
        {
            self.kick_invalid_movement();
            return ClientMoveOutcome::Rejected;
        }

        // Vanilla `absSnapRotationTo` while awaiting a teleport: look only.
        if self
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
        {
            self.apply_and_broadcast_look(requested);
            return ClientMoveOutcome::Ignored;
        }

        let entity = self.get_entity();
        let last_pos = entity.pos.load();
        let mut target = requested.position.map(|pos| {
            Vector3::new(
                Self::clamp_horizontal(pos.x),
                Self::clamp_vertical(pos.y),
                Self::clamp_horizontal(pos.z),
            )
        });

        if self.is_movement_locked.load(Ordering::Relaxed) {
            self.apply_and_broadcast_look(requested);
            self.reject_client_move();
            return ClientMoveOutcome::Rejected;
        }

        if entity.has_vehicle() {
            // Vanilla passenger: rotation only, keep the vehicle's position.
            entity.on_ground.store(false, Ordering::Relaxed);
            entity.horizontal_collision.store(false, Ordering::Relaxed);
            self.apply_and_broadcast_look(requested);
            entity.movement.store(Vector3::default());
            return ClientMoveOutcome::Applied;
        }

        if self.sleeping_since.load().is_some()
            && let Some(pos) = target
            && (pos - last_pos).length_squared() > 1.0
        {
            self.reject_client_move();
            return ClientMoveOutcome::Rejected;
        }

        if let Some(pos) = target
            && pos != last_pos
        {
            let Some(player) = self.world().get_player_by_uuid(self.gameprofile.id) else {
                return ClientMoveOutcome::Ignored;
            };
            let mut cancelled = false;
            send_cancellable_blocking! {{
                server;
                PlayerMoveEvent::new(player, last_pos, pos);
                'after: {
                    target = Some(event.to);
                }
                'cancelled: {
                    cancelled = true;
                }
            }}
            if cancelled {
                self.reject_client_move();
                return ClientMoveOutcome::Rejected;
            }
        }

        let pos = target.unwrap_or(last_pos);
        let was_on_ground = entity.on_ground.load(Ordering::Relaxed);
        entity.set_pos(pos);

        let delta = Vector3::new(pos.x - last_pos.x, pos.y - last_pos.y, pos.z - last_pos.z);
        let distance = last_pos.squared_distance_to_vec(&pos).sqrt();
        let cm = (distance * 100.0) as i32;
        if cm > 0 {
            let stat = self.get_movement_statistic();
            self.increment_stat(StatisticCategory::Custom, stat as i32, cm);
        }

        // Vanilla `onGround && !isOnGround && movedUpwards` -> `jumpFromGround`
        if was_on_ground && !requested.on_ground && delta.y > 0.0 {
            self.jump();
        }

        entity
            .on_ground
            .store(requested.on_ground, Ordering::Relaxed);
        entity
            .horizontal_collision
            .store(requested.horizontal_collision, Ordering::Relaxed);
        if requested.on_ground && entity.is_fall_flying() && self.glide_stop_allowed() {
            entity.set_fall_flying(false);
        }

        let pos_changed = delta.length_squared() > 0.0;
        let rot_changed = self.look_changed(requested);
        self.apply_look(requested);

        if pos_changed || rot_changed {
            self.broadcast_client_move(
                last_pos,
                pos,
                requested.on_ground,
                pos_changed,
                rot_changed,
            );
        }

        self.do_check_fall_damage(delta, requested.on_ground, requested.horizontal_collision);

        if pos_changed && let Some(player) = self.world().get_player_by_uuid(self.gameprofile.id) {
            chunker::update_position(&player);
        }
        if delta.length_squared() > 1.0E-5 {
            self.update_last_action_time();
            self.check_location_enchantments(pos, requested.on_ground);
        }
        self.progress_motion(delta);
        // Vanilla `handlePlayerKnownMovement` / `setKnownMovement`
        entity.movement.store(delta);

        ClientMoveOutcome::Applied
    }

    fn look_changed(&self, requested: ClientMove) -> bool {
        let entity = self.get_entity();
        requested
            .yaw
            .is_some_and(|yaw| (wrap_degrees(yaw) % 360.0 - entity.yaw.load()).abs() > f32::EPSILON)
            || requested.pitch.is_some_and(|pitch| {
                (wrap_degrees(pitch).clamp(-90.0, 90.0) - entity.pitch.load()).abs() > f32::EPSILON
            })
            || requested.head_yaw.is_some_and(|head_yaw| {
                (wrap_degrees(head_yaw) % 360.0 - entity.head_yaw.load()).abs() > f32::EPSILON
            })
    }

    /// Look without position: teleport pending, movement locked or riding.
    fn apply_and_broadcast_look(&self, requested: ClientMove) {
        let rot_changed = self.look_changed(requested);
        self.apply_look(requested);
        if rot_changed {
            let entity = self.get_entity();
            let pos = entity.pos.load();
            let on_ground = entity.on_ground.load(Ordering::Relaxed);
            self.broadcast_client_move(pos, pos, on_ground, false, true);
        }
    }

    fn apply_look(&self, requested: ClientMove) {
        let entity = self.get_entity();
        match (requested.yaw, requested.pitch) {
            (Some(yaw), Some(pitch)) => {
                entity.set_rotation(wrap_degrees(yaw) % 360.0, wrap_degrees(pitch));
            }
            (Some(yaw), None) => {
                entity.set_rotation(wrap_degrees(yaw) % 360.0, entity.pitch.load());
            }
            (None, Some(pitch)) => {
                entity.set_rotation(entity.yaw.load(), wrap_degrees(pitch));
            }
            (None, None) => {}
        }
        if let Some(head_yaw) = requested.head_yaw {
            entity.head_yaw.store(wrap_degrees(head_yaw) % 360.0);
        }
    }

    fn kick_invalid_movement(&self) {
        self.client.try_kick(
            DisconnectReason::Kicked,
            &TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                [],
            ),
        );
    }

    fn broadcast_client_move(
        &self,
        last_pos: Vector3<f64>,
        pos: Vector3<f64>,
        on_ground: bool,
        pos_changed: bool,
        rot_changed: bool,
    ) {
        let entity = self.get_entity();
        let world = entity.world.load_full();
        let entity_id = entity.entity_id;
        let yaw = Self::packed_rot(entity.yaw.load());
        let pitch = Self::packed_rot(entity.pitch.load());
        let head = Self::packed_rot(entity.head_yaw.load());
        let delta = pos - last_pos;

        if pos_changed && delta.length_squared() >= 64.0 {
            world.send_to_tracking_players_editioned(
                entity,
                &CEntityPositionSync::new(
                    entity_id.into(),
                    pos,
                    Vector3::new(0.0, 0.0, 0.0),
                    entity.yaw.load(),
                    entity.pitch.load(),
                    on_ground,
                ),
                &self.bedrock_move_packet(CMovePlayer::MODE_TELEPORT, on_ground, 0),
            );
            world.send_to_tracking_players_bedrock(
                entity,
                &self.bedrock_move_packet(CMovePlayer::MODE_NORMAL, on_ground, 0),
            );
        } else if pos_changed && rot_changed {
            world.send_to_tracking_players_editioned(
                entity,
                &CUpdateEntityPosRot::new(
                    entity_id.into(),
                    packed_delta(last_pos, pos),
                    yaw,
                    pitch,
                    on_ground,
                ),
                &self.bedrock_move_packet(CMovePlayer::MODE_NORMAL, on_ground, 0),
            );
        } else if pos_changed {
            world.send_to_tracking_players_editioned(
                entity,
                &CUpdateEntityPos::new(entity_id.into(), packed_delta(last_pos, pos), on_ground),
                &self.bedrock_move_packet(CMovePlayer::MODE_NORMAL, on_ground, 0),
            );
        } else if rot_changed {
            world.send_to_tracking_players_editioned(
                entity,
                &CUpdateEntityRot::new(entity_id.into(), yaw, pitch, on_ground),
                &self.bedrock_move_packet(CMovePlayer::MODE_NORMAL, on_ground, 0),
            );
        }

        if rot_changed {
            world.send_to_tracking_players(entity, &CHeadRot::new(entity_id.into(), head));
        }
    }
}

fn packed_delta(last_pos: Vector3<f64>, pos: Vector3<f64>) -> Vector3<i16> {
    Vector3::new(
        pos.x.mul_add(4096.0, -(last_pos.x * 4096.0)) as i16,
        pos.y.mul_add(4096.0, -(last_pos.y * 4096.0)) as i16,
        pos.z.mul_add(4096.0, -(last_pos.z * 4096.0)) as i16,
    )
}

fn dismount_if_riding(entity: &crate::entity::Entity) {
    let vehicle = entity
        .vehicle
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    if let Some(vehicle) = vehicle {
        vehicle.get_entity().remove_passenger(entity.entity_id);
    }
}
