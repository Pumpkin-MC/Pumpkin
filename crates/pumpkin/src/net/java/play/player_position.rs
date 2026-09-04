#[allow(clippy::wildcard_imports)]
use super::*;

struct ClientMovementSnapshot {
    chunk_send_epoch: u32,
    teleport_id: i32,
}

impl ClientMovementSnapshot {
    fn capture(player: &Player) -> Option<Self> {
        let _movement_guard = player.lock_client_movement_if_loaded()?;
        // Vanilla ignores movement while a teleport still awaits confirmation.
        if player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
        {
            return None;
        }
        Some(Self {
            chunk_send_epoch: player.chunk_send_epoch.load(Ordering::Acquire),
            teleport_id: player.teleport_id_count.load(Ordering::Relaxed),
        })
    }

    fn lock_if_current<'a>(&self, player: &'a Player) -> Option<std::sync::MutexGuard<'a, ()>> {
        let movement_guard = player.lock_client_movement_if_loaded()?;
        // A move-event plugin can complete a transfer or even confirm a same-world teleport
        // before returning. Loaded state and the pending teleport alone cannot detect that.
        (self.chunk_send_epoch == player.chunk_send_epoch.load(Ordering::Acquire)
            && self.teleport_id == player.teleport_id_count.load(Ordering::Relaxed)
            && player
                .awaiting_teleport
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_none())
        .then_some(movement_guard)
    }
}

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
    pub fn handle_position(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &SPlayerPosition,
    ) {
        let Some(movement) = ClientMovementSnapshot::capture(player) else {
            return;
        };
        if player.get_entity().has_vehicle() {
            return;
        }
        if player.is_movement_locked.load(Ordering::Relaxed) {
            let Some(_movement_guard) = movement.lock_if_current(player) else {
                return;
            };
            self.force_tp(player, player.get_entity().pos.load());
            return;
        }
        // y = feet Y
        let position = packet.position;
        if position.x.is_nan() || position.y.is_nan() || position.z.is_nan() {
            self.try_kick(&TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                [],
            ));
            return;
        }
        let position = Vector3::new(
            Self::clamp_horizontal(position.x),
            Self::clamp_vertical(position.y),
            Self::clamp_horizontal(position.z),
        );

        send_cancellable_blocking! {{
            server;
            PlayerMoveEvent {
                player: player.clone(),
                from: player.get_entity().pos.load(),
                to: position,
                cancelled: false,
            };

            'after: {
                let Some(movement_guard) = movement.lock_if_current(player) else {
                    return;
                };
                let pos = event.to;
                let entity = &player.get_entity();
                let last_pos = entity.pos.load();
                player.get_entity().set_pos(pos);

                let distance = last_pos.squared_distance_to_vec(&pos).sqrt();
                let cm = (distance * 100.0) as i32;
                if cm > 0 {
                    let stat = player.get_movement_statistic();
                    player.increment_stat(StatisticCategory::Custom, stat as i32, cm);
                }

                let height_difference = pos.y - last_pos.y;
                if entity.on_ground.load(Ordering::Relaxed) && packet.collision & FLAG_ON_GROUND == 0 && height_difference > 0.0 {
                    player.jump();
                }

                let new_on_ground = packet.collision & FLAG_ON_GROUND != 0;
                entity.on_ground.store(new_on_ground, Ordering::Relaxed);
                if new_on_ground && entity.is_fall_flying() {
                    entity.set_fall_flying(false);
                }
                let world = &player.world();

                // TODO: Warn when player moves to quickly
                if !Self::sync_position(player, world, pos, last_pos, entity.yaw.load(), entity.pitch.load(), packet.collision & FLAG_ON_GROUND != 0) {
                    // Send the new position to all other players.
                    let bedrock_move_packet = CMovePlayer::new(
                        VarULong(player.entity_id() as u64),
                        Vector3::new(
                            pos.x as f32,
                            pos.y as f32 + player.get_entity().entity_type.eye_height,
                            pos.z as f32,
                        ),
                        entity.pitch.load(),
                        entity.yaw.load(),
                        entity.yaw.load(),
                        CMovePlayer::MODE_NORMAL,
                        (packet.collision & FLAG_ON_GROUND) != 0,
                        VarULong(0),
                        0,
                        0,
                        VarULong(0),
                    );
                    world.broadcast_packet_except(
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
                    );
                    world.send_to_tracking_players_bedrock(entity, &bedrock_move_packet);
                }

                // Only process fall damage if player is alive
                if !player.abilities.lock().unwrap_or_else(std::sync::PoisonError::into_inner).flying
                    && player.living_entity.health.load() > 0.0
                    && !player.living_entity.dead.load(Ordering::Relaxed)
                {
                    player.living_entity.fall(
                        player.as_ref(),
                        height_difference,
                        packet.collision & FLAG_ON_GROUND != 0,
                        player.gamemode.load() == GameMode::Creative,
                    );
                }
                let delta = Vector3::new(
                    pos.x - last_pos.x,
                    pos.y - last_pos.y,
                    pos.z - last_pos.z,
                );
                // Only update idle timeout if there's actual movement (vanilla threshold)
                if delta.length_squared() > 1.0E-5 {
                    player.update_last_action_time();
                    player.check_location_enchantments(pos, packet.collision & FLAG_ON_GROUND != 0);
                }
                player.progress_motion(delta);
                drop(movement_guard);
                chunker::update_position(player);
            }

            'cancelled: {
                let Some(_movement_guard) = movement.lock_if_current(player) else {
                    return;
                };
                self.force_tp(player, player.get_entity().pos.load());
            }
        }}
    }

    #[expect(clippy::too_many_lines)]
    pub fn handle_position_rotation(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &SPlayerPositionRotation,
    ) {
        let Some(movement) = ClientMovementSnapshot::capture(player) else {
            return;
        };
        if player.get_entity().has_vehicle() {
            return;
        }
        if player.is_movement_locked.load(Ordering::Relaxed) {
            let Some(_movement_guard) = movement.lock_if_current(player) else {
                return;
            };
            let entity = player.get_entity();
            entity.set_rotation(packet.yaw, packet.pitch);
            self.force_tp(player, entity.pos.load());
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
            self.try_kick(&TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                [],
            ));
            return;
        }

        let position = Vector3::new(
            Self::clamp_horizontal(position.x),
            Self::clamp_vertical(position.y),
            Self::clamp_horizontal(position.z),
        );

        send_cancellable_blocking! {{
            server;
            PlayerMoveEvent::new(
                player.clone(),
                player.get_entity().pos.load(),
                position,
            );

            'after: {
                let Some(movement_guard) = movement.lock_if_current(player) else {
                    return;
                };
                let pos = event.to;
                let entity = &player.get_entity();
                let last_pos = entity.pos.load();
                player.get_entity().set_pos(pos);

                let distance = last_pos.squared_distance_to_vec(&pos).sqrt();
                let cm = (distance * 100.0) as i32;
                if cm > 0 {
                    let stat = player.get_movement_statistic();
                    player.increment_stat(StatisticCategory::Custom, stat as i32, cm);
                }

                let height_difference = pos.y - last_pos.y;
                if entity.on_ground.load(Ordering::Relaxed)
                    && (packet.collision & FLAG_ON_GROUND) != 0
                    && height_difference > 0.0
                {
                    player.jump();
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
                    let bedrock_move_packet = CMovePlayer::new(
                        VarULong(entity_id as u64),
                        Vector3::new(
                            pos.x as f32,
                            pos.y as f32 + player.get_entity().entity_type.eye_height,
                            pos.z as f32,
                        ),
                        entity.pitch.load(),
                        entity.yaw.load(),
                        entity.yaw.load(),
                        CMovePlayer::MODE_NORMAL,
                        (packet.collision & FLAG_ON_GROUND) != 0,
                        VarULong(0),
                        0,
                        0,
                        VarULong(0),
                    );
                    world.broadcast_packet_except(
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
                    );
                    world.send_to_tracking_players_bedrock(entity, &bedrock_move_packet);
                }

                world
                    .broadcast_packet_except(
                        &[player.gameprofile.id],
                        &CHeadRot::new(entity_id.into(), yaw as u8),
                    )
                   ;
                // Only process fall damage if player is alive
                if !player.abilities.lock().unwrap_or_else(std::sync::PoisonError::into_inner).flying
                    && player.living_entity.health.load() > 0.0
                    && !player.living_entity.dead.load(Ordering::Relaxed)
                {
                    player.living_entity.fall(
                        player.as_ref(),
                        height_difference,
                        (packet.collision & FLAG_ON_GROUND) != 0,
                        player.gamemode.load() == GameMode::Creative,
                    );
                }
                let delta = Vector3::new(
                    pos.x - last_pos.x,
                    pos.y - last_pos.y,
                    pos.z - last_pos.z,
                );
                // Only update idle timeout if there's actual movement (vanilla threshold)
                if delta.length_squared() > 1.0E-5 {
                    player.update_last_action_time();
                    player.check_location_enchantments(pos, (packet.collision & FLAG_ON_GROUND) != 0);
                }
                player.progress_motion(delta);
                drop(movement_guard);
                chunker::update_position(player);
            }

            'cancelled: {
                let Some(_movement_guard) = movement.lock_if_current(player) else {
                    return;
                };
                self.force_tp(player, position);
            }
        }}
    }

    pub fn force_tp(&self, player: &Arc<Player>, position: Vector3<f64>) {
        let teleport_id = player.teleport_id_count.fetch_add(1, Ordering::Relaxed) + 1;
        *player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some((teleport_id.into(), position));
        player.try_send_client_packet(&CPlayerPosition::new(
            teleport_id.into(),
            player.get_entity().pos.load(),
            Vector3::new(0.0, 0.0, 0.0),
            player.get_entity().yaw.load(),
            player.get_entity().pitch.load(),
            Vec::new(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, atomic::AtomicU32};

    use super::*;
    use crate::{
        net::ClientPlatform,
        plugin::{BoxFuture, EventHandler, EventPriority},
        test_support::TestServer,
    };

    const TELEPORT_POSITION: Vector3<f64> = Vector3::new(256.0, 120.0, -256.0);
    const EVENT_POSITION: Vector3<f64> = Vector3::new(3.0, 100.0, 4.0);

    #[derive(Clone, Copy)]
    enum MoveAction {
        Rewrite,
        Teleport { confirm: bool, advance_epoch: bool },
    }

    struct MovePlugin {
        action: Mutex<(MoveAction, bool)>,
        calls: AtomicU32,
    }

    impl EventHandler<PlayerMoveEvent> for MovePlugin {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PlayerMoveEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                self.calls.fetch_add(1, Ordering::Relaxed);
                let (action, cancelled) = *self.action.lock().unwrap();
                event.cancelled = cancelled;
                event.to = EVENT_POSITION;
                if let MoveAction::Teleport {
                    confirm,
                    advance_epoch,
                } = action
                {
                    if advance_epoch {
                        event.player.request_teleport(TELEPORT_POSITION, 90.0, 30.0);
                    } else {
                        event.player.request_teleport_in_current_chunk_epoch(
                            TELEPORT_POSITION,
                            90.0,
                            30.0,
                        );
                    }
                    if confirm {
                        confirm_teleport(&event.player);
                    }
                }
            })
        }
    }

    fn java_client(player: &Player) -> &JavaClient {
        let ClientPlatform::Java(client) = player.client.as_ref() else {
            panic!("expected a Java player");
        };
        client
    }

    fn confirm_teleport(player: &Player) {
        java_client(player).handle_confirm_teleport(
            player,
            &SConfirmTeleport {
                teleport_id: player.teleport_id_count.load(Ordering::Relaxed).into(),
            },
        );
    }

    fn send_movement(player: &Arc<Player>, server: &Arc<Server>, include_rotation: bool) {
        let position = Vector3::new(1.0, 100.0, 1.0);
        if include_rotation {
            java_client(player).handle_position_rotation(
                player,
                server,
                &SPlayerPositionRotation {
                    position,
                    yaw: 12.0,
                    pitch: 24.0,
                    collision: FLAG_ON_GROUND,
                },
            );
        } else {
            java_client(player).handle_position(
                player,
                server,
                &SPlayerPosition {
                    position,
                    collision: FLAG_ON_GROUND,
                },
            );
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn java_move_event_teleports_are_not_overwritten() {
        let fixture = TestServer::new().await;
        let player = fixture.new_java_player().await;
        let plugin = Arc::new(MovePlugin {
            action: Mutex::new((MoveAction::Rewrite, false)),
            calls: AtomicU32::new(0),
        });
        fixture
            .server
            .plugin_manager
            .register::<PlayerMoveEvent, _>(plugin.clone(), EventPriority::Normal, true);

        for include_rotation in [false, true] {
            for cancelled in [false, true] {
                // Cover a pending teleport, one already confirmed by the plugin, and a
                // confirmed teleport that deliberately leaves the chunk epoch unchanged.
                for (confirm, advance_epoch) in [(false, true), (true, true), (true, false)] {
                    *plugin.action.lock().unwrap() = (
                        MoveAction::Teleport {
                            confirm,
                            advance_epoch,
                        },
                        cancelled,
                    );
                    player.get_entity().set_pos(Vector3::new(0.0, 100.0, 0.0));
                    player.get_entity().set_rotation(0.0, 0.0);
                    let previous_id = player.teleport_id_count.load(Ordering::Relaxed);

                    send_movement(&player, &fixture.server, include_rotation);

                    assert_eq!(player.get_entity().pos.load(), TELEPORT_POSITION);
                    assert_eq!(player.get_entity().yaw.load(), 90.0);
                    assert_eq!(player.get_entity().pitch.load(), 30.0);
                    assert_eq!(
                        player.teleport_id_count.load(Ordering::Relaxed),
                        previous_id + 1,
                        "cancelling the move must not send a second correction teleport"
                    );
                    assert_eq!(player.awaiting_teleport.lock().unwrap().is_none(), confirm);
                    if !confirm {
                        confirm_teleport(&player);
                        assert_eq!(player.get_entity().pos.load(), TELEPORT_POSITION);
                    }
                }
            }

            // Rejecting stale events must not prevent a plugin from changing an ordinary
            // movement destination when no teleport or transfer occurred.
            *plugin.action.lock().unwrap() = (MoveAction::Rewrite, false);
            player.get_entity().set_pos(Vector3::new(0.0, 100.0, 0.0));
            send_movement(&player, &fixture.server, include_rotation);
            assert_eq!(player.get_entity().pos.load(), EVENT_POSITION);
        }
        assert_eq!(plugin.calls.load(Ordering::Relaxed), 14);
        assert!(!java_client(&player).is_closed());
        fixture.shutdown().await;
    }
}
