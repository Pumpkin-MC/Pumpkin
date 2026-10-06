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
        if player.living_entity.dead.load(Ordering::Relaxed)
            || player.living_entity.health.load() <= 0.0
        {
            return;
        }
        if self.await_teleport(player, &packet) || !player.has_client_loaded() {
            return;
        }
        let entity = player.get_entity();
        // Like Geyser: the motion this tick started with was downward, the end motion can
        // already be cleared by the landing (e.g. gliding into the ground)
        let on_ground = packet.input_data.get(InputData::VerticalCollision as usize)
            && self.client_delta.load().y < 0.0
            && !entity.has_vehicle();
        entity.on_ground.store(on_ground, Ordering::Relaxed);
        // Velocity stays server owned like Java (vanilla `deltaMovement`). The client's
        // own motion is the position delta in `Entity::movement`, the same for both editions.
        self.input_tick.store(packet.tick.0, Ordering::Relaxed);
        self.client_delta.store(packet.delta.to_f64());

        let new_pos = player.feet_from_bedrock_pos(packet.position.to_f64());
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
                entity.set_rotation(new_yaw, new_pitch);
            }

            // TODO: use `pumpkin_util::math::pack_degrees`.
            let je_yaw = (new_yaw * 256.0 / 360.0).rem_euclid(256.0);
            let je_pitch = (new_pitch * 256.0 / 360.0).rem_euclid(256.0);

            let delta = new_pos - old_pos;

            let bedrock_move_packet = player.bedrock_move_packet(
                pumpkin_protocol::bedrock::client::CMovePlayer::MODE_NORMAL,
                on_ground,
                0,
            );

            if pos_changed && delta.length_squared() >= 64.0 {
                world.broadcast_packet_except(
                    &[player.gameprofile.id],
                    &pumpkin_protocol::java::client::play::CEntityPositionSync::new(
                        player.entity_id().into(),
                        new_pos,
                        pumpkin_util::math::vector3::Vector3::new(0.0, 0.0, 0.0),
                        je_yaw,
                        je_pitch,
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
                        je_yaw as u8,   // Use converted Java byte
                        je_pitch as u8, // Use converted Java byte
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
                        je_yaw as u8,   // Use converted Java byte
                        je_pitch as u8, // Use converted Java byte
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
                        je_yaw as u8,
                    ),
                );
            }

            if pos_changed {
                chunker::update_position(player);
                player.check_location_enchantments(new_pos, on_ground);
                player.progress_motion(delta);
            }
        }

        player.do_check_fall_damage(
            new_pos - old_pos,
            on_ground,
            packet
                .input_data
                .get(InputData::HorizontalCollision as usize),
        );

        let input_data = packet.input_data;

        if input_data.get(InputData::StartSprinting as usize) {
            player.set_sprinting(true);
        } else if input_data.get(InputData::StopSprinting as usize) {
            player.set_sprinting(false);
        }

        if input_data.get(InputData::StartSneaking as usize) {
            entity.set_sneaking(true);
        } else if input_data.get(InputData::StopSneaking as usize) {
            entity.set_sneaking(false);
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

        // Bedrock can stop in the same input it started, then it never glided
        if input_data.get(InputData::StartGliding as usize)
            && !input_data.get(InputData::StopGliding as usize)
        {
            Self::start_gliding(player, server);
        } else if input_data.get(InputData::StopGliding as usize) && entity.is_fall_flying() {
            entity.set_fall_flying(false);
        }

        if let Some(block_actions) = packet.block_actions {
            for action in &block_actions {
                self.handle_player_block_action(player, server, action);
            }
        }
    }
}

impl BedrockClient {
    /// The Bedrock client glides straight out of creative flight, which vanilla `canGlide`
    /// forbids, so flight ends first and comes back if the glide is refused.
    fn start_gliding(player: &Arc<Player>, server: &Arc<Server>) {
        if player.get_entity().is_fall_flying() {
            return;
        }
        let was_flying = player.is_flying();
        let set_flying = |flying: bool| {
            player
                .abilities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .flying = flying;
        };
        if was_flying {
            set_flying(false);
        }
        if !player.try_to_start_fall_flying(server) {
            if was_flying {
                set_flying(true);
            }
            player.get_entity().stop_fall_flying();
        }
        if was_flying {
            player.send_abilities_update();
        }
    }

    /// inputs are ignored until the client reaches a pending teleport, so stale
    /// positions (e.g. the death spot before a respawn) never count as movement or a fall.
    /// Returns true while still waiting.
    fn await_teleport(&self, player: &Arc<Player>, packet: &SPlayerAuthInput) -> bool {
        const TELEPORT_ERROR: f64 = 0.1;
        const RESEND_INPUTS: u32 = 20;

        let mut awaiting = player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some((_, target)) = *awaiting else {
            return false;
        };
        let position = player.feet_from_bedrock_pos(packet.position.to_f64());
        if (position.x - target.x).abs() < TELEPORT_ERROR
            && (position.y - target.y).abs() < TELEPORT_ERROR
            && (position.z - target.z).abs() < TELEPORT_ERROR
        {
            *awaiting = None;
            drop(awaiting);
            // Bedrock sends no `PlayerLoaded` after a respawn, reaching the spawn stands in
            player.set_client_loaded(true);
        } else if self
            .teleport_unconfirmed_inputs
            .fetch_add(1, Ordering::Relaxed)
            + 1
            >= RESEND_INPUTS
        {
            drop(awaiting);
            player.send_bedrock_teleport(self);
        }
        true
    }
}
