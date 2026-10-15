#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_data::entity::EntityPose;

impl BedrockClient {
    #[expect(clippy::too_many_lines)]
    pub fn handle_player_auth_input(
        &self,
        player: &Arc<Player>,
        packet: SPlayerAuthInput,
        server: &Arc<Server>,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        if player.living_entity.dead.load(Ordering::Relaxed)
            || player.living_entity.health.load() <= 0.0
        {
            return;
        }
        let entity = player.get_entity();
        let on_ground = packet.input_data.get(InputData::VerticalCollision as usize)
            && packet.delta.y < 0.0
            && !entity.has_vehicle();
        entity.on_ground.store(on_ground, Ordering::Relaxed);

        let new_pos = packet
            .position
            .add_raw(0.0, -entity.entity_type.eye_height, 0.0)
            .to_f64();
        let old_pos = player.position();

        let new_pitch = packet.pitch;
        let new_yaw = packet.yaw;

        let old_pitch = entity.pitch.load();
        let old_yaw = entity.yaw.load();

        let pos_changed = new_pos != old_pos;
        let rot_changed = new_pitch != old_pitch || new_yaw != old_yaw;

        if pos_changed || rot_changed {
            let world = player.world();

            if pos_changed {
                player.get_entity().set_pos(new_pos);
            }
            if rot_changed {
                entity.pitch.store(new_pitch);
                entity.yaw.store(new_yaw);
            }

            let byte_yaw = pumpkin_util::math::pack_degrees(new_yaw);
            let byte_pitch = pumpkin_util::math::pack_degrees(new_pitch);

            let delta = pumpkin_util::math::vector3::Vector3::new(
                new_pos.x - old_pos.x,
                new_pos.y - old_pos.y,
                new_pos.z - old_pos.z,
            );

            let bedrock_move_packet = pumpkin_protocol::bedrock::client::CMovePlayer::new(
                pumpkin_protocol::codec::var_ulong::VarULong(player.entity_id() as u64),
                pumpkin_util::math::vector3::Vector3::new(
                    new_pos.x as f32,
                    new_pos.y as f32 + entity.entity_type.eye_height,
                    new_pos.z as f32,
                ),
                new_pitch,
                new_yaw,
                new_yaw, // Head yaw
                pumpkin_protocol::bedrock::client::CMovePlayer::MODE_NORMAL,
                on_ground,
                pumpkin_protocol::codec::var_ulong::VarULong(0),
                0,
                0,
                pumpkin_protocol::codec::var_ulong::VarULong(0),
            );

            if pos_changed && delta.length_squared() >= 64.0 {
                world.broadcast_packet_except(
                    &[player.gameprofile.id],
                    &pumpkin_protocol::java::client::play::CEntityPositionSync::new(
                        player.entity_id().into(),
                        new_pos,
                        pumpkin_util::math::vector3::Vector3::new(0.0, 0.0, 0.0),
                        new_yaw,
                        new_pitch,
                        on_ground,
                    ),
                );
            } else if pos_changed && rot_changed {
                world.broadcast_packet_except_editioned(
                    &[player.gameprofile.id],
                    &pumpkin_protocol::java::client::play::CUpdateEntityPosRot::new(
                        player.entity_id().into(),
                        pumpkin_util::math::vector3::Vector3::new(
                            new_pos.x.mul_add(4096.0, -(old_pos.x * 4096.0)) as i16,
                            new_pos.y.mul_add(4096.0, -(old_pos.y * 4096.0)) as i16,
                            new_pos.z.mul_add(4096.0, -(old_pos.z * 4096.0)) as i16,
                        ),
                        byte_yaw,
                        byte_pitch,
                        on_ground,
                    ),
                    &bedrock_move_packet,
                );
            } else if pos_changed {
                world.broadcast_packet_except_editioned(
                    &[player.gameprofile.id],
                    &pumpkin_protocol::java::client::play::CUpdateEntityPos::new(
                        player.entity_id().into(),
                        pumpkin_util::math::vector3::Vector3::new(
                            new_pos.x.mul_add(4096.0, -(old_pos.x * 4096.0)) as i16,
                            new_pos.y.mul_add(4096.0, -(old_pos.y * 4096.0)) as i16,
                            new_pos.z.mul_add(4096.0, -(old_pos.z * 4096.0)) as i16,
                        ),
                        on_ground,
                    ),
                    &bedrock_move_packet,
                );
            } else if rot_changed {
                world.broadcast_packet_except_editioned(
                    &[player.gameprofile.id],
                    &pumpkin_protocol::java::client::play::CUpdateEntityRot::new(
                        player.entity_id().into(),
                        byte_yaw,
                        byte_pitch,
                        on_ground,
                    ),
                    &bedrock_move_packet,
                );
            }

            if rot_changed {
                world.broadcast_packet_except(
                    &[player.gameprofile.id],
                    // Adjust to `CHeadRot` if that is what your crate currently calls it
                    &pumpkin_protocol::java::client::play::CHeadRot::new(
                        player.entity_id().into(),
                        byte_yaw,
                    ),
                );
            }

            if pos_changed {
                chunker::update_position(player);
                player.check_location_enchantments(new_pos, on_ground);
                player.progress_motion(delta);
            }
        }

        let input_data = packet.input_data;

        if input_data.get(InputData::StartSprinting as usize) {
            if !player.get_entity().is_sprinting() {
                send_cancellable_blocking! {{
                    server;
                    PlayerToggleSprintEvent::new(player.clone(), true);
                    'after: {
                        player.set_sprinting(event.is_sprinting);
                        player.update_player_pose();
                    }
                }}
            }
        } else if input_data.get(InputData::StopSprinting as usize) {
            if player.get_entity().is_sprinting() {
                send_cancellable_blocking! {{
                    server;
                    PlayerToggleSprintEvent::new(player.clone(), false);
                    'after: {
                        player.set_sprinting(event.is_sprinting);
                        player.update_player_pose();
                    }
                }}
            }
        }

        if input_data.get(InputData::StartSneaking as usize) {
            if !entity.is_sneaking() {
                send_cancellable_blocking! {{
                    server;
                    PlayerToggleSneakEvent::new(player.clone(), true);
                    'after: {
                        entity.set_sneaking(event.is_sneaking);
                        if event.is_sneaking {
                            let vehicle = entity
                                .vehicle
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .clone();
                            if let Some(vehicle) = vehicle {
                                vehicle.get_entity().remove_passenger(player.entity_id());
                            }
                        }
                        player.update_player_pose();
                    }
                }}
            }
        } else if input_data.get(InputData::StopSneaking as usize) {
            if entity.is_sneaking() {
                send_cancellable_blocking! {{
                    server;
                    PlayerToggleSneakEvent::new(player.clone(), false);
                    'after: {
                        entity.set_sneaking(event.is_sneaking);
                        player.update_player_pose();
                    }
                }}
            }
        }

        if input_data.get(InputData::StartGliding as usize) {
            let fall_flying = entity.check_fall_flying();
            if entity.is_fall_flying() != fall_flying {
                let mut event = crate::plugin::api::events::entity::entity_toggle_glide::EntityToggleGlideEvent::new(
                    entity.entity_id,
                    fall_flying,
                );
                server.plugin_manager.fire_blocking(server, &mut event);
                if !event.cancelled {
                    entity.set_fall_flying(event.is_gliding);
                    player.update_player_pose();
                }
            }
        } else if input_data.get(InputData::StopGliding as usize) {
            if entity.is_fall_flying() {
                let mut event = crate::plugin::api::events::entity::entity_toggle_glide::EntityToggleGlideEvent::new(
                    entity.entity_id,
                    false,
                );
                server.plugin_manager.fire_blocking(server, &mut event);
                if !event.cancelled {
                    entity.set_fall_flying(false);
                    player.update_player_pose();
                }
            }
        }

        if input_data.get(InputData::StartSwimming as usize) {
            entity.set_swimming(true);
            player.update_player_pose();
        } else if input_data.get(InputData::StopSwimming as usize) {
            entity.set_swimming(false);
            player.update_player_pose();
        }

        if input_data.get(InputData::StartSpinAttack as usize) {
            player.living_entity.set_auto_spin_attack(true);
            player.update_player_pose();
        } else if input_data.get(InputData::StopSpinAttack as usize) {
            player.living_entity.set_auto_spin_attack(false);
            player.update_player_pose();
        }

        if input_data.get(InputData::StartJumping as usize) {
            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::Jump as i32,
                1,
            );
        }

        if input_data.get(InputData::StartCrawling as usize) {
            entity.set_pose(EntityPose::Swimming);
        } else if input_data.get(InputData::StopCrawling as usize) {
            player.update_player_pose();
        }

        if input_data.get(InputData::StartFlying as usize) {
            let flying = {
                player
                    .abilities
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .flying
            };
            if !flying {
                send_cancellable_blocking! {{
                    server;
                    PlayerToggleFlightEvent::new(player.clone(), true);
                    'after: {
                        player.living_entity.fall_distance.store(0.0);
                        {
                            player.abilities.lock().unwrap_or_else(std::sync::PoisonError::into_inner).flying = true;
                        };
                        player.send_abilities_update();
                    }
                    'cancelled: {
                        player.send_abilities_update();
                    }
                }}
            }
        } else if input_data.get(InputData::StopFlying as usize) {
            let flying = {
                player
                    .abilities
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .flying
            };
            if flying {
                send_cancellable_blocking! {{
                    server;
                    PlayerToggleFlightEvent::new(player.clone(), false);
                    'after: {
                        {
                            player.abilities.lock().unwrap_or_else(std::sync::PoisonError::into_inner).flying = false;
                        };
                        player.send_abilities_update();
                    }
                    'cancelled: {
                        player.send_abilities_update();
                    }
                }}
            }
        }

        if let Some(block_actions) = packet.block_actions {
            for action in &block_actions {
                self.handle_player_block_action(player, server, action);
            }
        }
    }
}
