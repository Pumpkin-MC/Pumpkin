use super::Player;
use super::statistics;
use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::net::ClientPlatform;
use crate::plugin::player::player_change_world::PlayerChangeWorldEvent;
use crate::plugin::player::player_teleport::PlayerTeleportEvent;
use crate::world::World;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityPose;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CPlayerPosition;
use pumpkin_protocol::java::client::play::CRespawn;
use pumpkin_protocol::java::client::play::CSetSelectedSlot;
use pumpkin_protocol::java::client::play::CUnloadChunk;
use pumpkin_protocol::java::client::play::PlayerSpawnData;
use pumpkin_util::GameMode;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_world::biome;
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
use std::num::NonZeroU8;
use std::sync::Arc;
use std::sync::atomic::Ordering;

impl Player {
    pub async fn get_off_ground_speed(&self) -> f64 {
        let sprinting = self.get_entity().is_sprinting();

        if !self.get_entity().has_vehicle().await {
            let fly_speed = {
                let abilities = self.abilities.lock().await;

                abilities.flying.then_some(f64::from(abilities.fly_speed))
            };

            if let Some(flying) = fly_speed {
                return if sprinting { flying * 2.0 } else { flying };
            }
        }

        if sprinting { 0.025_999_999 } else { 0.02 }
    }

    pub async fn is_flying(&self) -> bool {
        let abilities = self.abilities.lock().await;
        abilities.flying
    }

    fn is_sleeping(&self) -> bool {
        // TODO: Track sleeping position state explicitly (vanilla checks sleepingPosition.isPresent()).
        self.sleeping_since.load().is_some()
    }

    async fn is_swimming(&self, flying: bool) -> bool {
        let entity = self.get_entity();
        let swim_height = self.living_entity.get_swim_height();

        // TODO: Replace this inferred check with vanilla-equivalent swimming state tracking
        // (LivingEntity#updateSwimming + entity swimming flag).
        entity.touching_water.load(Ordering::Relaxed)
            && entity.water_height.load() > swim_height
            && entity.is_sprinting()
            && !entity.on_ground.load(Ordering::Relaxed)
            && !flying
            && !entity.has_vehicle().await
    }

    const fn is_auto_spin_attack() -> bool {
        // TODO: Track active auto-spin/riptide state and return true while it is active.
        false
    }

    fn can_fit_pose(&self, pose: EntityPose) -> bool {
        let entity = self.get_entity();
        let dimensions = Entity::get_entity_dimensions(pose);
        let position = entity.pos.load();
        let aabb = BoundingBox::new_from_pos(position.x, position.y, position.z, &dimensions);
        entity
            .world
            .load()
            .is_space_empty(aabb.contract_all(1.0E-7))
    }

    pub async fn update_player_pose(&self) {
        let entity = self.get_entity();
        if !self.can_fit_pose(EntityPose::Swimming) {
            return;
        }

        let flying = self.is_flying().await;
        let desired_pose = if self.is_sleeping() {
            EntityPose::Sleeping
        } else if self.is_swimming(flying).await {
            EntityPose::Swimming
        } else if entity.is_fall_flying() {
            EntityPose::FallFlying
        } else if Self::is_auto_spin_attack() {
            EntityPose::SpinAttack
        } else if entity.is_sneaking() && !flying {
            EntityPose::Crouching
        } else {
            EntityPose::Standing
        };

        let new_pose = if self.gamemode.load() == GameMode::Spectator
            || entity.has_vehicle().await
            || self.can_fit_pose(desired_pose)
        {
            desired_pose
        } else if self.can_fit_pose(EntityPose::Crouching) {
            EntityPose::Crouching
        } else {
            EntityPose::Swimming
        };

        if entity.pose.load() != new_pose {
            entity.set_pose(new_pose);
        }
    }

    pub async fn jump(&self) {
        self.stats
            .lock()
            .await
            .increment_custom(statistics::CustomStatistic::Jump, 1);
        if self.living_entity.entity.is_sprinting() {
            self.add_exhaustion(0.2).await;
        } else {
            self.add_exhaustion(0.05).await;
        }
    }

    pub async fn progress_motion(&self, delta_pos: Vector3<f64>) {
        // TODO: Swimming, gliding...
        if self.living_entity.entity.on_ground.load(Ordering::Relaxed) {
            let delta = (delta_pos.horizontal_length() * 100.0).round() as f32;
            if delta > 0.0 {
                if self.living_entity.entity.is_sprinting() {
                    self.add_exhaustion(0.1 * delta * 0.01).await;
                } else {
                    self.add_exhaustion(0.0 * delta * 0.01).await;
                }
            }
        }
    }

    pub async fn unload_watched_chunks(&self, world: &World) {
        let radial_chunks = self.watched_section.load().all_chunks_within();
        let level = &world.level;
        let chunks_to_clean = level.mark_chunks_as_not_watched(&radial_chunks).await;
        // level.clean_chunks(&chunks_to_clean).await;
        for chunk in chunks_to_clean {
            self.client
                .enqueue_packet(&CUnloadChunk::new(chunk.x, chunk.y))
                .await;
        }

        self.watched_section.store(Cylindrical::new(
            Vector2::new(0, 0),
            NonZeroU8::new(1).unwrap(),
        ));
    }

    /// Teleports the player to a different world or dimension with an optional position, yaw, and pitch.
    pub async fn teleport_world(
        self: &Arc<Self>,
        new_world: Arc<World>,
        position: Vector3<f64>,
        yaw: Option<f32>,
        pitch: Option<f32>,
    ) {
        let current_world = self.living_entity.entity.world.load_full();
        let yaw = yaw.unwrap_or(new_world.level_info.load().spawn_yaw);
        let pitch = pitch.unwrap_or(new_world.level_info.load().spawn_pitch);

        let server = new_world.server.upgrade().unwrap();

        send_cancellable! {{
            server;
            PlayerChangeWorldEvent {
                player: self.clone(),
                previous_world: current_world.clone(),
                new_world: new_world.clone(),
                position,
                yaw,
                pitch,
                cancelled: false,
            };

            'after: {
                // TODO: this is duplicate code from world
                let position = event.position;
                let yaw = event.yaw;
                let pitch = event.pitch;
                let new_world = event.new_world;

                self.set_client_loaded(false);
                let player = current_world.remove_player(self, false).await.unwrap();
               new_world.players.rcu(|current_list| {
                    let mut new_list = (**current_list).clone();
                    new_list.push(player.clone());
                    new_list
                });
                self.unload_watched_chunks(&current_world).await;

                self.chunk_manager.lock().await.change_world(&current_world.level, new_world.clone());
                self.living_entity.entity.set_world(new_world.clone());

                if new_world.dimension == pumpkin_data::dimension::Dimension::THE_NETHER {
                    self.trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::EnterDimension {
                        dimension: "the_nether".to_string(),
                    }).await;
                } else if new_world.dimension == pumpkin_data::dimension::Dimension::THE_END {
                    self.trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::EnterDimension {
                        dimension: "the_end".to_string(),
                    }).await;
                }

                let last_pos = self.living_entity.entity.last_pos.load();
                let death_dimension = ResourceLocation::from(self.world().dimension.minecraft_name);
                let death_location = BlockPos(Vector3::new(
                    last_pos.x.round() as i32,
                    last_pos.y.round() as i32,
                    last_pos.z.round() as i32,
                ));
                match self.client.as_ref() {
                    ClientPlatform::Java(java) => {
                        java.send_packet_now(&CRespawn::new(
                            PlayerSpawnData::new(
                                new_world.dimension.clone(),
                                biome::hash_seed(new_world.level.seed.0), // seed
                                self.gamemode.load() as u8,
                                self.previous_gamemode.load().unwrap_or(self.gamemode.load()) as i8,
                                false,
                                false,
                                Some((death_dimension, death_location)),
                                VarInt(self.get_entity().portal_cooldown.load(Ordering::Relaxed) as i32),
                                new_world.sea_level.into(),
                            ),
                            CRespawn::KEEP_ALL_DATA,
                        )).await;
                    }
                    ClientPlatform::Bedrock(bedrock) => {
                        let bedrock_dimension = if new_world.dimension == Dimension::OVERWORLD {
                            0
                        } else if new_world.dimension == Dimension::THE_NETHER {
                            1
                        } else if new_world.dimension == Dimension::THE_END {
                            2
                        } else {
                            0
                        };
                        let pos_f32 = Vector3::new(position.x as f32, position.y as f32, position.z as f32);
                        let change_dim_packet = pumpkin_protocol::bedrock::client::CChangeDimension::new(
                            bedrock_dimension,
                            pos_f32,
                            false,
                        );
                        bedrock.enqueue_packet(&change_dim_packet).await;
                    }
                }

                self.send_permission_lvl_update();

                player.clone().request_teleport(position, yaw, pitch).await;
                player.get_entity().last_pos.store(position);

                self.send_abilities_update().await;

                self.enqueue_set_held_item_packet(&CSetSelectedSlot::new(
                   self.get_inventory().get_selected_slot() as i8,
                )).await;

                self.on_screen_handler_opened(self.player_screen_handler.clone()).await;

                self.send_health().await;

                new_world.send_world_info(&player, position, yaw, pitch).await;
            }
        }}
    }

    /// `yaw` and `pitch` are in degrees.
    /// Rarly used, for example when waking up the player from a bed or their first time spawn. Otherwise, the `teleport` method should be used.
    /// The player should respond with the `SConfirmTeleport` packet.
    pub async fn request_teleport(self: &Arc<Self>, position: Vector3<f64>, yaw: f32, pitch: f32) {
        // This is the ultra special magic code used to create the teleport id
        // This returns the old value
        // This operation wraps around on overflow.
        let server = self.world().server.upgrade().unwrap();
        send_cancellable! {{
            server;
            PlayerTeleportEvent {
                player: self.clone(),
                from: self.living_entity.entity.pos.load(),
                to: position,
                cancelled: false,
            };

            'after: {
                let position = event.to;
                let i = self
                    .teleport_id_count
                    .fetch_add(1, Ordering::Relaxed);
                let teleport_id = i + 1;
                self.living_entity.entity.set_pos(position);
                let entity = &self.living_entity.entity;
                entity.set_rotation(yaw, pitch);
                *self.awaiting_teleport.lock().await = Some((teleport_id.into(), position));
                self.client
                    .send_packet_now(&CPlayerPosition::new(
                        teleport_id.into(),
                        position,
                        Vector3::new(0.0, 0.0, 0.0),
                        yaw,
                        pitch,
                        // TODO
                        Vec::new(),
                    )).await;
            }
        }}
    }

    pub fn block_interaction_range(&self) -> f64 {
        if self.gamemode.load() == GameMode::Creative {
            5.0
        } else {
            4.5
        }
    }

    pub fn entity_interaction_range(&self) -> f64 {
        self.living_entity
            .get_attribute_value(&Attributes::ENTITY_INTERACTION_RANGE)
    }

    pub fn is_within_entity_interaction_range(
        &self,
        bounding_box: &BoundingBox,
        additional_range: f64,
    ) -> bool {
        let range = self.entity_interaction_range() + additional_range;
        bounding_box.squared_magnitude(self.living_entity.entity.get_eye_pos()) < range * range
    }

    pub fn can_interact_with_block_at(&self, position: &BlockPos, additional_range: f64) -> bool {
        let d = self.block_interaction_range() + additional_range;
        let box_pos = BoundingBox::from_block(position);
        let entity_pos = self.living_entity.entity.pos.load();
        let eye_height = self.living_entity.entity.get_eye_height();
        box_pos.squared_magnitude(Vector3 {
            x: entity_pos.x,
            y: entity_pos.y + eye_height,
            z: entity_pos.z,
        }) < d * d
    }

    /// Returns the main non-air `BlockPos` underneath the player.
    pub fn get_supporting_block_pos(&self) -> Option<BlockPos> {
        let entity = self.get_entity();
        let entity_pos = entity.pos.load();
        let aabb = entity.bounding_box.load();
        let world = self.world();

        // Create the thin bounding box directly underneath the entity's feet
        let footprint = BoundingBox::new(
            Vector3::new(aabb.min.x, aabb.min.y - 1.0e-6, aabb.min.z),
            Vector3::new(aabb.max.x, aabb.min.y, aabb.max.z),
        );

        let min_pos = footprint.min_block_pos();
        let max_pos = footprint.max_block_pos();

        let mut closest_candidate = None;
        let mut min_dist_sq = f64::MAX;

        // Iterate through candidates
        for pos in BlockPos::iterate(min_pos, max_pos) {
            let (_, state) = world.get_block_and_state(&pos);

            // Only consider physical blocks
            if state.is_air() {
                continue;
            }

            // Calculate distance squared from the block's center to the entity's position
            let block_center_x = f64::from(pos.0.x) + 0.5;
            let block_center_y = f64::from(pos.0.y) + 0.5;
            let block_center_z = f64::from(pos.0.z) + 0.5;

            let dx = block_center_x - entity_pos.x;
            let dy = block_center_y - entity_pos.y;
            let dz = block_center_z - entity_pos.z;
            let dist_sq = dx * dx + dy * dy + dz * dz;

            // Pick the block with the smallest distance
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                closest_candidate = Some(pos);
            } else if (dist_sq - min_dist_sq).abs() < f64::EPSILON {
                // If the distance is the same, pick the block with the smallest y, then z, then x
                if let Some(best_pos) = closest_candidate {
                    let is_smaller = pos.0.y < best_pos.0.y
                        || (pos.0.y == best_pos.0.y && pos.0.z < best_pos.0.z)
                        || (pos.0.y == best_pos.0.y
                            && pos.0.z == best_pos.0.z
                            && pos.0.x < best_pos.0.x);

                    if is_smaller {
                        closest_candidate = Some(pos);
                    }
                }
            }
        }

        // Return the closest block if we found one
        if closest_candidate.is_some() {
            return closest_candidate;
        }

        // Fallback to the block directly underneath the player's position if no candidates were found
        let fallback_pos = BlockPos::new(
            entity_pos.x.floor() as i32,
            (entity_pos.y - 0.2).floor() as i32,
            entity_pos.z.floor() as i32,
        );

        let state = world.get_block_state(&fallback_pos);
        (!state.is_air()).then_some(fallback_pos)
    }
}
