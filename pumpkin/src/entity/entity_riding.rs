use super::{Entity, EntityBase};
use pumpkin_data::BlockDirection;
use pumpkin_data::entity::EntityPose;
use pumpkin_data::fluid::Fluid;
use pumpkin_protocol::PositionFlag;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{CPlayerPosition, CSetPassengers};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

impl Entity {
    pub const LEASH_SNAP_DISTANCE: f64 = 12.0;
    pub const LEASH_ELASTIC_DISTANCE: f64 = 6.0;

    pub async fn leash_to(&self, holder: Arc<dyn EntityBase>) {
        let holder_entity = holder.get_entity();
        *self.leashed_to.lock().await = Some(holder.clone());

        let je_packet = pumpkin_protocol::java::client::play::CSetEntityLink::new(
            self.entity_id,
            holder_entity.entity_id,
        );
        let be_packet = pumpkin_protocol::bedrock::client::CSetActorLink {
            link: pumpkin_protocol::bedrock::client::common::EntityLink {
                ridden_unique_id: pumpkin_protocol::codec::var_long::VarLong(self.entity_id as i64),
                rider_unique_id: pumpkin_protocol::codec::var_long::VarLong(
                    holder_entity.entity_id as i64,
                ),
                link_type: 1, // Leash link
                immediate: true,
                rider_initiated: false,
                vehicle_angular_velocity: 0.0,
            },
        };

        self.world.load().broadcast_to_chunk_editioned_sync(
            self.chunk_pos.load(),
            &je_packet,
            &be_packet,
        );
    }

    pub async fn unleash(&self) {
        let old_holder = self.leashed_to.lock().await.take();
        if old_holder.is_none() {
            return;
        }

        let je_packet =
            pumpkin_protocol::java::client::play::CSetEntityLink::new(self.entity_id, -1);
        let be_packet = pumpkin_protocol::bedrock::client::CSetActorLink {
            link: pumpkin_protocol::bedrock::client::common::EntityLink {
                ridden_unique_id: pumpkin_protocol::codec::var_long::VarLong(self.entity_id as i64),
                rider_unique_id: pumpkin_protocol::codec::var_long::VarLong(-1),
                link_type: 0, // Unlink
                immediate: true,
                rider_initiated: false,
                vehicle_angular_velocity: 0.0,
            },
        };

        self.world.load().broadcast_to_chunk_editioned_sync(
            self.chunk_pos.load(),
            &je_packet,
            &be_packet,
        );
    }

    pub async fn tick_leash(&self) {
        let holder = {
            let guard = self.leashed_to.lock().await;
            guard.clone()
        };

        if let Some(holder) = holder {
            let holder_entity = holder.get_entity();

            // Drop leash if entity or holder is removed or dead
            if !self.is_alive() || !holder_entity.is_alive() {
                self.unleash().await;
                return;
            }

            let self_pos = self.pos.load();
            let holder_pos = holder_entity.pos.load();
            let diff = self_pos - holder_pos;
            let distance = diff.length();

            if distance > Self::LEASH_SNAP_DISTANCE {
                // Too far: snap/break leash and drop lead item
                self.unleash().await;
                let lead_item =
                    pumpkin_data::item_stack::ItemStack::new(1, &pumpkin_data::item::Item::LEAD);
                self.world
                    .load()
                    .drop_stack(&self.block_pos.load(), lead_item)
                    .await;
            } else if distance > Self::LEASH_ELASTIC_DISTANCE {
                // Elastic pull force towards leash holder
                let dir = (holder_pos - self_pos).normalize();
                let pull_strength = (distance - Self::LEASH_ELASTIC_DISTANCE) * 0.11;
                let current_vel = self.velocity.load();
                self.velocity.store(current_vel + dir * pull_strength);
                self.velocity_dirty.store(true, Relaxed);
            }
        }
    }

    pub async fn has_passengers(&self) -> bool {
        !self.passengers.lock().await.is_empty()
    }

    pub async fn has_vehicle(&self) -> bool {
        let vehicle = self.vehicle.lock().await;
        vehicle.is_some()
    }

    pub async fn add_passenger(
        &self,
        vehicle: Arc<dyn EntityBase>,
        passenger: Arc<dyn EntityBase>,
    ) {
        let passenger_entity = passenger.get_entity();
        *passenger_entity.vehicle.lock().await = Some(vehicle);

        let mut passengers = self.passengers.lock().await;
        passengers.push(passenger);

        let passenger_ids: Vec<VarInt> = passengers
            .iter()
            .map(|p| VarInt(p.get_entity().entity_id))
            .collect();

        let world = self.world.load();
        let chunk_pos = self.chunk_pos.load();
        world.broadcast_to_chunk(
            chunk_pos,
            &CSetPassengers::new(VarInt(self.entity_id), &passenger_ids),
        );
    }

    #[allow(clippy::too_many_lines)]
    pub async fn remove_passenger(&self, passenger_id: i32) {
        let mut passengers = self.passengers.lock().await;
        let removed_passenger = if let Some(idx) = passengers
            .iter()
            .position(|p| p.get_entity().entity_id == passenger_id)
        {
            let passenger = passengers.remove(idx);
            *passenger.get_entity().vehicle.lock().await = None;
            Some(passenger)
        } else {
            None
        };

        let passenger_ids: Vec<VarInt> = passengers
            .iter()
            .map(|p| VarInt(p.get_entity().entity_id))
            .collect();
        drop(passengers);

        let chunk_pos = self.chunk_pos.load();

        if let Some(passenger) = removed_passenger {
            let vehicle_box = self.bounding_box.load();
            let passenger_entity = passenger.get_entity();

            // Pre-allocate teleport ID and block movement packets BEFORE sending
            // CSetPassengers. This prevents a race condition where the client receives
            // the dismount packet, sends stale position packets from the old riding
            // position, and the server processes them before the teleport arrives.
            let teleport_id = if let Some(player) = passenger.get_player() {
                let id = player
                    .teleport_id_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    + 1;
                // Use fallback position as placeholder — updated below with real position
                let placeholder =
                    Vector3::new(self.pos.load().x, vehicle_box.max.y, self.pos.load().z);
                *player.awaiting_teleport.lock().await = Some((id.into(), placeholder));
                Some(id)
            } else {
                None
            };

            // Vanilla: ridingCooldown = 60 (prevents immediate re-mount)
            passenger_entity.riding_cooldown.store(60, Relaxed);
            // TODO: world.emitGameEvent(passenger, GameEvent.ENTITY_DISMOUNT, vehicle.pos)

            // Now send CSetPassengers — client movement is already blocked.
            // Vanilla sends this directly to the dismounting player's connection,
            // then broadcasts to other players separately.
            let world = self.world.load();
            let passengers_packet = CSetPassengers::new(VarInt(self.entity_id), &passenger_ids);
            if let Some(player) = passenger.get_player() {
                player.client.enqueue_packet(&passengers_packet).await;
                world.broadcast_to_chunk_except(
                    chunk_pos,
                    &[player.get_entity().entity_uuid],
                    &passengers_packet,
                );
            } else {
                world.broadcast_to_chunk(chunk_pos, &passengers_packet);
            }

            // Calculate dismount directions and offsets (vanilla DismountHelper)
            let vehicle_yaw = self.yaw.load();
            // Wrap yaw to 0..360 range
            let wrapped_yaw = (vehicle_yaw % 360.0 + 360.0) % 360.0;
            let forward_dir = if !(45.0..315.0).contains(&wrapped_yaw) {
                BlockDirection::South
            } else if (45.0..135.0).contains(&wrapped_yaw) {
                BlockDirection::West
            } else if (135.0..225.0).contains(&wrapped_yaw) {
                BlockDirection::North
            } else {
                BlockDirection::East
            };

            let get_step = |dir: BlockDirection| -> (i32, i32) {
                match dir {
                    BlockDirection::North => (0, -1),
                    BlockDirection::South => (0, 1),
                    BlockDirection::East => (1, 0),
                    BlockDirection::West => (-1, 0),
                    _ => (0, 0),
                }
            };

            let get_clockwise = |dir: BlockDirection| -> BlockDirection {
                match dir {
                    BlockDirection::North => BlockDirection::East,
                    BlockDirection::East => BlockDirection::South,
                    BlockDirection::South => BlockDirection::West,
                    BlockDirection::West => BlockDirection::North,
                    other => other,
                }
            };

            let get_opposite = |dir: BlockDirection| -> BlockDirection {
                match dir {
                    BlockDirection::North => BlockDirection::South,
                    BlockDirection::South => BlockDirection::North,
                    BlockDirection::East => BlockDirection::West,
                    BlockDirection::West => BlockDirection::East,
                    other => other,
                }
            };

            let right_dir = get_clockwise(forward_dir);
            let left_dir = get_opposite(right_dir);
            let back_dir = get_opposite(forward_dir);

            let (fx, fz) = get_step(forward_dir);
            let (rx, rz) = get_step(right_dir);
            let (lx, lz) = get_step(left_dir);
            let (bx, bz) = get_step(back_dir);

            let offsets = [
                (rx, rz),
                (lx, lz),
                (bx + rx, bz + rz),
                (bx + lx, bz + lz),
                (fx + rx, fz + rz),
                (fx + lx, fz + lz),
                (bx, bz),
                (fx, fz),
            ];

            let target_block_y = vehicle_box.max.y.floor() as i32;
            let below_pos = BlockPos(Vector3::new(
                self.pos.load().x.floor() as i32,
                target_block_y - 1,
                self.pos.load().z.floor() as i32,
            ));

            let below_state_id = world.get_block_state_id(&below_pos);
            // Vanilla: isWater checks specifically for water fluid, not any fluid
            let is_water = Fluid::from_state_id(below_state_id)
                .is_some_and(|f| f.id == Fluid::WATER.id || f.id == Fluid::FLOWING_WATER.id);

            let fallback_pos =
                Vector3::new(self.pos.load().x, vehicle_box.max.y, self.pos.load().z);

            let dismount_pos = if is_water {
                fallback_pos
            } else {
                // Vanilla checks Standing, Crouching, Swimming poses and their respective height checks
                let poses_and_heights = [
                    (EntityPose::Standing, vec![0, 1, -1]),
                    (EntityPose::Crouching, vec![0, 1, -1]),
                    (EntityPose::Swimming, vec![0, 1]),
                ];

                let vehicle_block_pos = self.block_pos.load();
                let mut found = None;

                'search: for (pose, y_offsets) in poses_and_heights {
                    let dims = Self::get_entity_dimensions(pose);

                    for y_offset in y_offsets {
                        for &(ox, oz) in &offsets {
                            let target_block_x = vehicle_block_pos.0.x + ox;
                            let target_block_y = vehicle_block_pos.0.y + y_offset;
                            let target_block_z = vehicle_block_pos.0.z + oz;

                            let target_pos = BlockPos(Vector3::new(
                                target_block_x,
                                target_block_y,
                                target_block_z,
                            ));
                            let height = world.get_dismount_height(&target_pos);

                            if height.is_finite() && height < 1.0 {
                                let location = Vector3::new(
                                    f64::from(target_block_x) + 0.5,
                                    f64::from(target_block_y) + height,
                                    f64::from(target_block_z) + 0.5,
                                );

                                let bbox = BoundingBox::new_from_pos(
                                    location.x, location.y, location.z, &dims,
                                );
                                if world.is_space_empty(bbox) {
                                    found = Some((location, pose));
                                    break 'search;
                                }
                            }
                        }
                    }
                }

                if let Some((pos, pose)) = found {
                    if pose != EntityPose::Standing {
                        passenger_entity.set_pose(pose);
                    }
                    pos
                } else {
                    // Try dismounting directly on top of the vehicle as fallback
                    let mut found_fallback = None;
                    let vehicle_top = vehicle_box.max.y;

                    let poses = [
                        EntityPose::Standing,
                        EntityPose::Crouching,
                        EntityPose::Swimming,
                    ];

                    for pose in poses {
                        let dims = Self::get_entity_dimensions(pose);
                        let bbox = BoundingBox::new_from_pos(
                            self.pos.load().x,
                            vehicle_top,
                            self.pos.load().z,
                            &dims,
                        );
                        if world.is_space_empty(bbox) {
                            found_fallback = Some((
                                Vector3::new(self.pos.load().x, vehicle_top, self.pos.load().z),
                                pose,
                            ));
                            break;
                        }
                    }

                    if let Some((pos, pose)) = found_fallback {
                        if pose != EntityPose::Standing {
                            passenger_entity.set_pose(pose);
                        }
                        pos
                    } else {
                        fallback_pos
                    }
                }
            };

            if let Some(player) = passenger.get_player() {
                let id = teleport_id.unwrap();
                player.get_entity().set_pos(dismount_pos);
                // Update awaiting_teleport with the real dismount position
                *player.awaiting_teleport.lock().await = Some((id.into(), dismount_pos));
                // Use enqueue_packet (not send_packet_now) so the teleport goes through
                // the same packet queue as CSetPassengers, preserving send order.
                // Vanilla uses DELTA | ROT flags: position absolute, delta/rotation relative.
                // With rotation relative and yaw/pitch=0, the client preserves its current look.
                player
                    .client
                    .enqueue_packet(&CPlayerPosition::new(
                        id.into(),
                        dismount_pos,
                        Vector3::new(0.0, 0.0, 0.0),
                        0.0,
                        0.0,
                        vec![
                            PositionFlag::DeltaX,
                            PositionFlag::DeltaY,
                            PositionFlag::DeltaZ,
                            PositionFlag::YRot,
                            PositionFlag::XRot,
                        ],
                    ))
                    .await;
                // Vanilla: setSneaking(false) after dismount via sneak input
                if passenger_entity.sneaking.load(Relaxed) {
                    passenger_entity.set_sneaking(false).await;
                }
            } else {
                passenger_entity.set_pos(dismount_pos);
            }
        } else {
            // No passenger was removed, still need to broadcast the passenger list
            let world = self.world.load();
            world.broadcast_to_chunk(
                chunk_pos,
                &CSetPassengers::new(VarInt(self.entity_id), &passenger_ids),
            );
        }
    }
}
