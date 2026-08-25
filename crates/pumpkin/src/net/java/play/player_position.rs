#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    const fn clamp_horizontal(pos: f64) -> f64 {
        pos.clamp(-3.0E7, 3.0E7)
    }

    const fn clamp_vertical(pos: f64) -> f64 {
        pos.clamp(-2.0E7, 2.0E7)
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
                    && (packet.collision & FLAG_ON_GROUND) != 0
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
        self.enqueue_client_packet(&CPlayerPosition::new(
            teleport_id.into(),
            player.get_entity().pos.load(),
            Vector3::new(0.0, 0.0, 0.0),
            player.get_entity().yaw.load(),
            player.get_entity().pitch.load(),
            Vec::new(),
        ))
        .await;
    }
}
