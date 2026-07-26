use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::player::statistics::StatisticCategory;
use crate::net::java::JavaClient;
use crate::plugin::player::player_move::PlayerMoveEvent;
use crate::plugin::player::player_toggle_sneak_event::PlayerToggleSneakEvent;
use crate::plugin::player::player_toggle_sprint_event::PlayerToggleSprintEvent;
use crate::server::Server;
use crate::world::World;
use crate::world::chunker;
use pumpkin_data::translation;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::bedrock::client::CMovePlayer;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::CEntityPositionSync;
use pumpkin_protocol::java::client::play::CHeadRot;
use pumpkin_protocol::java::client::play::CPlayerPosition;
use pumpkin_protocol::java::client::play::CSetCamera;
use pumpkin_protocol::java::client::play::CUpdateEntityPos;
use pumpkin_protocol::java::client::play::CUpdateEntityPosRot;
use pumpkin_protocol::java::client::play::CUpdateEntityRot;
use pumpkin_protocol::java::server::play::Action;
use pumpkin_protocol::java::server::play::FLAG_ON_GROUND;
use pumpkin_protocol::java::server::play::SConfirmTeleport;
use pumpkin_protocol::java::server::play::SMoveVehicle;
use pumpkin_protocol::java::server::play::SPaddleBoat;
use pumpkin_protocol::java::server::play::SPlayerCommand;
use pumpkin_protocol::java::server::play::SPlayerInput;
use pumpkin_protocol::java::server::play::SPlayerPosition;
use pumpkin_protocol::java::server::play::SPlayerPositionRotation;
use pumpkin_protocol::java::server::play::SPlayerRotation;
use pumpkin_protocol::java::server::play::SSetPlayerGround;
use pumpkin_protocol::java::server::play::STeleportToEntity;
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::debug;

/// Handles all Play packets sent by a real player.
/// NEVER TRUST THE CLIENT. HANDLE EVERY ERROR; UNWRAP/EXPECT ARE FORBIDDEN.
impl JavaClient {
    pub async fn handle_confirm_teleport(
        &self,
        player: &Player,
        confirm_teleport: SConfirmTeleport,
    ) {
        let mut awaiting_teleport = player.awaiting_teleport.lock().await;
        if let Some((id, position)) = awaiting_teleport.as_ref() {
            if id == &confirm_teleport.teleport_id {
                // We should set the position now to what we requested in the teleport packet.
                // This may fix issues when the client sends the position while being teleported.
                player.get_entity().set_pos(*position);

                *awaiting_teleport = None;
                drop(awaiting_teleport);
            } else {
                drop(awaiting_teleport);
                self.kick(TextComponent::text("Wrong teleport id")).await;
            }
        } else {
            drop(awaiting_teleport);
            self.kick(TextComponent::text(
                "Send Teleport confirm, but we did not teleport",
            ))
            .await;
        }
    }

    const fn clamp_horizontal(pos: f64) -> f64 {
        pos.clamp(-3.0E7, 3.0E7)
    }

    const fn clamp_vertical(pos: f64) -> f64 {
        pos.clamp(-2.0E7, 2.0E7)
    }

    pub fn handle_player_loaded(player: &Player) {
        player.set_client_loaded(true);
    }

    /// Returns whether syncing the position was needed
    fn sync_position(
        player: &Arc<Player>,
        world: &World,
        pos: Vector3<f64>,
        last_pos: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    ) -> bool {
        let delta = Vector3::new(pos.x - last_pos.x, pos.y - last_pos.y, pos.z - last_pos.z);
        let entity_id = player.entity_id();

        // Teleport when more than 8 blocks (-8..=7.999755859375)
        if delta.length_squared() < 64.0 {
            return false;
        }
        // Sync position with all other players.
        world.broadcast_packet_except(
            &[player.gameprofile.id],
            &CEntityPositionSync::new(
                entity_id.into(),
                pos,
                Vector3::new(0.0, 0.0, 0.0),
                yaw,
                pitch,
                on_ground,
            ),
        );
        true
    }

    #[expect(clippy::too_many_lines)]
    pub async fn handle_position(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: SPlayerPosition,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        if player.get_entity().has_vehicle().await {
            return;
        }
        // Ignore movement packets while awaiting a teleport confirmation (vanilla behavior)
        if player.awaiting_teleport.lock().await.is_some() {
            return;
        }
        // y = feet Y
        let position = packet.position;
        if position.x.is_nan() || position.y.is_nan() || position.z.is_nan() {
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                [],
            ))
            .await;
            return;
        }
        let position = Vector3::new(
            Self::clamp_horizontal(position.x),
            Self::clamp_vertical(position.y),
            Self::clamp_horizontal(position.z),
        );

        send_cancellable! {{
            server;
            PlayerMoveEvent {
                player: player.clone(),
                from: player.get_entity().pos.load(),
                to: position,
                cancelled: false,
            };

            'after: {
                let pos = event.to;
                let entity = &player.get_entity();
                let last_pos = entity.pos.load();
                player.get_entity().set_pos(pos);

                let distance = last_pos.squared_distance_to_vec(&pos).sqrt();
                let cm = (distance * 100.0) as i32;
                if cm > 0 {
                    let stat = player.get_movement_statistic().await;
                    player
                        .increment_stat(StatisticCategory::Custom, stat as i32, cm)
                        .await;
                }

                let height_difference = pos.y - last_pos.y;
                if entity.on_ground.load(Ordering::Relaxed) && packet.collision & FLAG_ON_GROUND == 0 && height_difference > 0.0 {
                    player.jump().await;
                }

                let new_on_ground = packet.collision & FLAG_ON_GROUND != 0;
                entity.on_ground.store(new_on_ground, Ordering::Relaxed);
                if new_on_ground && entity.is_fall_flying() {
                    entity.set_fall_flying(false).await;
                }
                let world = &player.world();

                // TODO: Warn when player moves to quickly
                if !Self::sync_position(player, world, pos, last_pos, entity.yaw.load(), entity.pitch.load(), packet.collision & FLAG_ON_GROUND != 0) {
                    // Send the new position to all other players.
                    world.broadcast_packet_except_editioned_sync(
                        &[player.gameprofile.id],
                        &CUpdateEntityPos::new(
                            player.entity_id().into(),
                            Vector3::new(
                                pos.x.mul_add(4096.0, -(last_pos.x * 4096.0)) as i16,
                                pos.y.mul_add(4096.0, -(last_pos.y * 4096.0)) as i16,
                                pos.z.mul_add(4096.0, -(last_pos.z * 4096.0)) as i16,
                            ),
                            packet.collision & FLAG_ON_GROUND != 0,
                        ),
                        &CMovePlayer::new(
                            VarULong(player.entity_id() as u64),
                            Vector3::new(pos.x as f32, pos.y as f32 + player.get_entity().entity_type.eye_height, pos.z as f32),
                            entity.pitch.load(),
                            entity.yaw.load(),
                            entity.yaw.load(),
                            CMovePlayer::MODE_NORMAL,
                            (packet.collision & FLAG_ON_GROUND) != 0,
                            VarULong(0),
                            0,
                            0,
                            VarULong(0),
                        ),
                    );
                }

                // Only process fall damage if player is alive
                if !player.abilities.lock().await.flying
                    && player.living_entity.health.load() > 0.0
                    && !player.living_entity.dead.load(Ordering::Relaxed)
                {
                    player.living_entity
                        .fall(
                            player.clone(),
                            height_difference,
                            packet.collision & FLAG_ON_GROUND != 0,
                            player.gamemode.load() == GameMode::Creative,
                        )
                        .await;
                }
                // ServerGamePacketListenerImpl resets accumulated fall distance
                // after every accepted upward player movement.
                if height_difference > 0.0 {
                    player.living_entity.fall_distance.store(0.0);
                }
                chunker::update_position(player).await;
                let delta = Vector3::new(
                    pos.x - last_pos.x,
                    pos.y - last_pos.y,
                    pos.z - last_pos.z,
                );
                // Only update idle timeout if there's actual movement (vanilla threshold)
                if delta.length_squared() > 1.0E-5 {
                    player.update_last_action_time();
                }
                player.progress_motion(delta).await;
            }

            'cancelled: {
                self.force_tp(player, player.get_entity().pos.load()).await;
            }
        }}
    }

    #[expect(clippy::too_many_lines)]
    pub async fn handle_position_rotation(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: SPlayerPositionRotation,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        if player.get_entity().has_vehicle().await {
            return;
        }
        // Ignore movement packets while awaiting a teleport confirmation (vanilla behavior)
        if player.awaiting_teleport.lock().await.is_some() {
            return;
        }
        // y = feet Y
        let position = packet.position;
        if !position.x.is_finite()
            || !position.y.is_finite()
            || !position.z.is_finite()
            || !packet.yaw.is_finite()
            || !packet.pitch.is_finite()
        {
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                [],
            ))
            .await;
            return;
        }

        let position = Vector3::new(
            Self::clamp_horizontal(position.x),
            Self::clamp_vertical(position.y),
            Self::clamp_horizontal(position.z),
        );

        send_cancellable! {{
            server;
            PlayerMoveEvent::new(
                player.clone(),
                player.get_entity().pos.load(),
                position,
            );

            'after: {
                let pos = event.to;
                let entity = &player.get_entity();
                let last_pos = entity.pos.load();
                player.get_entity().set_pos(pos);

                let distance = last_pos.squared_distance_to_vec(&pos).sqrt();
                let cm = (distance * 100.0) as i32;
                if cm > 0 {
                    let stat = player.get_movement_statistic().await;
                    player
                        .increment_stat(StatisticCategory::Custom, stat as i32, cm)
                        .await;
                }

                let height_difference = pos.y - last_pos.y;
                if entity.on_ground.load(Ordering::Relaxed)
                    && (packet.collision & FLAG_ON_GROUND) == 0
                    && height_difference > 0.0
                {
                    player.jump().await;
                }
                entity
                    .on_ground
                    .store((packet.collision & FLAG_ON_GROUND) != 0, Ordering::Relaxed);

                entity.set_rotation(wrap_degrees(packet.yaw) % 360.0, wrap_degrees(packet.pitch));

                let entity_id = entity.entity_id;

                let yaw = (entity.yaw.load() * 256.0 / 360.0).rem_euclid(256.0);
                let pitch = (entity.pitch.load() * 256.0 / 360.0).rem_euclid(256.0);
                // let head_yaw = (entity.head_yaw * 256.0 / 360.0).floor();
                let world = entity.world.load_full();

                // TODO: Warn when player moves to quickly
                if !Self::
                    sync_position(player, &world, pos, last_pos, yaw, pitch, (packet.collision & FLAG_ON_GROUND) != 0)
                {
                    // Send the new position to all other players.
                    world.broadcast_packet_except_editioned_sync(
                        &[player.gameprofile.id],
                        &CUpdateEntityPosRot::new(
                            entity_id.into(),
                            Vector3::new(
                                pos.x.mul_add(4096.0, -(last_pos.x * 4096.0)) as i16,
                                pos.y.mul_add(4096.0, -(last_pos.y * 4096.0)) as i16,
                                pos.z.mul_add(4096.0, -(last_pos.z * 4096.0)) as i16,
                            ),
                            yaw as u8,
                            pitch as u8,
                            (packet.collision & FLAG_ON_GROUND) != 0,
                        ),
                        &CMovePlayer::new(
                            VarULong(entity_id as u64),
                            Vector3::new(pos.x as f32, pos.y as f32 + player.get_entity().entity_type.eye_height, pos.z as f32),
                            entity.pitch.load(),
                            entity.yaw.load(),
                            entity.yaw.load(),
                            CMovePlayer::MODE_NORMAL,
                            (packet.collision & FLAG_ON_GROUND) != 0,
                            VarULong(0),
                            0,
                            0,
                            VarULong(0),
                        ),
                    );
                }

                world
                    .broadcast_packet_except(
                        &[player.gameprofile.id],
                        &CHeadRot::new(entity_id.into(), yaw as u8),
                    )
                   ;
                // Only process fall damage if player is alive
                if !player.abilities.lock().await.flying
                    && player.living_entity.health.load() > 0.0
                    && !player.living_entity.dead.load(Ordering::Relaxed)
                {
                    player.living_entity
                        .fall(
                            player.clone(),
                            height_difference,
                            (packet.collision & FLAG_ON_GROUND) != 0,
                            player.gamemode.load() == GameMode::Creative,
                        )
                        .await;
                }
                // ServerGamePacketListenerImpl resets accumulated fall distance
                // after every accepted upward player movement.
                if height_difference > 0.0 {
                    player.living_entity.fall_distance.store(0.0);
                }
                chunker::update_position(player).await;
                let delta = Vector3::new(
                    pos.x - last_pos.x,
                    pos.y - last_pos.y,
                    pos.z - last_pos.z,
                );
                // Only update idle timeout if there's actual movement (vanilla threshold)
                if delta.length_squared() > 1.0E-5 {
                    player.update_last_action_time();
                }
                player.progress_motion(delta).await;
            }

            'cancelled: {
                self.force_tp(player, position).await;
            }
        }}
    }

    pub async fn force_tp(&self, player: &Arc<Player>, position: Vector3<f64>) {
        let teleport_id = player.teleport_id_count.fetch_add(1, Ordering::Relaxed) + 1;
        *player.awaiting_teleport.lock().await = Some((teleport_id.into(), position));
        self.enqueue_packet(&CPlayerPosition::new(
            teleport_id.into(),
            player.get_entity().pos.load(),
            Vector3::new(0.0, 0.0, 0.0),
            player.get_entity().yaw.load(),
            player.get_entity().pitch.load(),
            Vec::new(),
        ))
        .await;
    }

    pub async fn handle_rotation(&self, player: &Arc<Player>, rotation: SPlayerRotation) {
        if !player.has_client_loaded() {
            return;
        }
        if player.get_entity().has_vehicle().await {
            return;
        }
        if player.awaiting_teleport.lock().await.is_some() {
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
        entity.set_rotation(
            wrap_degrees(rotation.yaw) % 360.0,
            wrap_degrees(rotation.pitch),
        );
        entity.on_ground.store(rotation.ground, Ordering::Relaxed);
        if rotation.ground
            && !player.abilities.lock().await.flying
            && player.living_entity.health.load() > 0.0
            && !player.living_entity.dead.load(Ordering::Relaxed)
        {
            // Rotation-only movement packets can also be the landing packet.
            player
                .living_entity
                .fall(
                    player.clone(),
                    0.0,
                    true,
                    player.gamemode.load() == GameMode::Creative,
                )
                .await;
        }
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

        world.broadcast_packet_except_editioned_sync(
            &[player.gameprofile.id],
            &je_packet,
            &be_packet,
        );

        let je_packet = CHeadRot::new(entity_id.into(), yaw as u8);
        world.broadcast_packet_except(&[player.gameprofile.id], &je_packet);
    }

    pub async fn handle_player_ground(&self, player: &Arc<Player>, ground: &SSetPlayerGround) {
        if !player.has_client_loaded()
            || player.get_entity().has_vehicle().await
            || player.awaiting_teleport.lock().await.is_some()
        {
            return;
        }

        player
            .living_entity
            .entity
            .on_ground
            .store(ground.on_ground, Ordering::Relaxed);

        if ground.on_ground
            && !player.abilities.lock().await.flying
            && player.living_entity.health.load() > 0.0
            && !player.living_entity.dead.load(Ordering::Relaxed)
        {
            // Status-only movement packets can be the landing packet. They must
            // perform the same fall-damage check as position-bearing packets.
            player
                .living_entity
                .fall(
                    player.clone(),
                    0.0,
                    true,
                    player.gamemode.load() == GameMode::Creative,
                )
                .await;
        }
    }

    pub async fn handle_player_command(
        &self,
        player: &Arc<Player>,
        command: SPlayerCommand,
        server: &Server,
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

    pub async fn handle_player_input(
        &self,
        player: &Arc<Player>,
        input: SPlayerInput,
        server: &Server,
    ) {
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

    pub async fn handle_move_vehicle(&self, player: &Arc<Player>, packet: SMoveVehicle) {
        let entity = player.get_entity();
        let pos = Vector3::new(packet.x, packet.y, packet.z);
        let vehicle = entity.vehicle.lock().await;
        if let Some(vehicle) = vehicle.as_ref() {
            let vehicle_entity = vehicle.get_entity();
            vehicle_entity.set_pos(pos);
            vehicle_entity.set_rotation(packet.yaw, packet.pitch);
        }
        drop(vehicle);
        entity.set_pos(pos);
        chunker::update_position(player).await;
    }

    pub async fn handle_paddle_boat(&self, player: &Arc<Player>, packet: SPaddleBoat) {
        let vehicle = player.get_entity().vehicle.lock().await.clone();
        if let Some(vehicle) = vehicle {
            vehicle
                .set_paddle_state(packet.left_paddle, packet.right_paddle)
                .await;
        }
    }

    pub async fn handle_teleport_to_entity(
        &self,
        player: &Arc<Player>,
        packet: STeleportToEntity,
        server: &Server,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        if player.gamemode.load() != GameMode::Spectator {
            return;
        }

        if let Some(target_player) = server.get_player_by_uuid(packet.target) {
            let target_pos = target_player.living_entity.entity.pos.load();
            let target_yaw = target_player.living_entity.entity.yaw.load();
            let target_pitch = target_player.living_entity.entity.pitch.load();

            let target_id = target_player.living_entity.entity.entity_id;
            player.camera_target_id.store(Some(target_id));
            player
                .client
                .send_packet_now(&CSetCamera::new(target_id.into()))
                .await;

            player
                .request_teleport(target_pos, target_yaw, target_pitch)
                .await;
        }
    }
}
