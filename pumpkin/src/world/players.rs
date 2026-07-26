use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::plugin::player::{
    player_change_world::PlayerChangeWorldEvent, player_leave::PlayerLeaveEvent,
    player_respawn::PlayerRespawnEvent,
};
use crate::world::World;
use pumpkin_data::translation;
use pumpkin_protocol::bedrock::client::player_list::{CPlayerList, PlayerListEntry, Skin};
use pumpkin_protocol::bedrock::client::remove_actor::CRemoveActor;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::java::client::play::{
    CChunkBatchEnd, CChunkBatchStart, CChunkData, CGameEvent, CPlayerSpawnPosition,
    CRemoveEntities, CRemovePlayerInfo, CRespawn, GameEvent, PlayerSpawnData,
};
use pumpkin_util::math::{
    boundingbox::BoundingBox, position::BlockPos, vector2::Vector2, vector3::Vector3,
};
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_world::biome;
use pumpkin_world::inventory::Clearable;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::{debug, info, warn};

impl World {
    #[allow(clippy::too_many_lines)]
    pub async fn respawn_player(self: &Arc<Self>, player: &Arc<Player>, alive: bool) {
        let last_pos = player.get_entity().last_pos.load();
        let death_dimension = ResourceLocation::from(player.world().dimension.minecraft_name);
        let death_location = BlockPos(Vector3::new(
            last_pos.x.round() as i32,
            last_pos.y.round() as i32,
            last_pos.z.round() as i32,
        ));

        let data_kept = u8::from(alive);

        // Copy spawn info from level_info to avoid holding lock across await
        let (spawn_x, spawn_z, spawn_yaw, spawn_pitch, keep_inventory) = {
            let info = self.level_info.load();
            (
                info.spawn_x,
                info.spawn_z,
                info.spawn_yaw,
                info.spawn_pitch,
                info.game_rules.keep_inventory,
            )
        };

        // Get respawn position and dimension
        let (position, yaw, pitch, respawn_dimension) =
            if let Some(respawn) = player.calculate_respawn_point().await {
                (
                    respawn.position,
                    respawn.yaw,
                    respawn.pitch,
                    respawn.dimension,
                )
            } else {
                // No valid respawn point - send notification and use world spawn
                player
                    .client
                    .send_packet_now(&CGameEvent::new(GameEvent::NoRespawnBlockAvailable, 0.0))
                    .await;

                // FIXME: This spawn position calculation is incorrect. Should use vanilla's
                // proper spawn position calculation (see #1381). The y-level calculation
                // needs to account for spawn radius and find a safe spawn position.
                let chunk_pos = Vector2::new(spawn_x >> 4, spawn_z >> 4);
                self.level.get_or_fetch_chunk(chunk_pos, |_| ()).await;
                let top = self.get_top_block(Vector2::new(spawn_x, spawn_z));

                (
                    Vector3::new(
                        f64::from(spawn_x) + 0.5,
                        (top + 1).into(),
                        f64::from(spawn_z) + 0.5,
                    ),
                    spawn_yaw,
                    spawn_pitch,
                    self.dimension.clone(),
                )
            };

        // Candidate destination world for a cross-dimension respawn.
        let candidate_world = if respawn_dimension == self.dimension {
            None
        } else {
            self.server.upgrade().map_or_else(
                || {
                    warn!("Could not get server for cross-dimension respawn");
                    None
                },
                |server| {
                    let worlds = server.worlds.load();
                    worlds
                        .iter()
                        .find(|w| w.dimension == respawn_dimension)
                        .cloned()
                },
            )
        };

        // Fire PlayerChangeWorldEvent (cancellable) before the transfer; it runs before
        // the non-cancellable PlayerRespawnEvent, which observes the resolved world.
        let (resolved_world, position, yaw, pitch) = if let Some(new_world) = candidate_world {
            if let Some(server) = self.server.upgrade() {
                let event = server
                    .plugin_manager
                    .fire(PlayerChangeWorldEvent {
                        player: player.clone(),
                        previous_world: self.clone(),
                        new_world: new_world.clone(),
                        position,
                        yaw,
                        pitch,
                        cancelled: false,
                    })
                    .await;

                if event.cancelled {
                    (None, position, yaw, pitch)
                } else {
                    let destination = event.new_world;
                    let position = event.position;
                    let yaw = event.yaw;
                    let pitch = event.pitch;

                    // Skip the transfer if redirected back to the current world.
                    if destination.uuid != self.uuid {
                        debug!(
                            "Cross-dimension respawn: {} -> {}",
                            self.dimension.minecraft_name, destination.dimension.minecraft_name
                        );

                        // Detach from the old world before publishing into the new one, so no
                        // observer sees the player in a world whose chunk manager doesn't match.
                        self.remove_player(player, false).await;
                        player.unload_watched_chunks(self).await;
                        player
                            .chunk_manager
                            .lock()
                            .await
                            .change_world(&self.level, destination.clone());
                        player.living_entity.entity.set_world(destination.clone());
                        destination.players.rcu(|current_list| {
                            let mut new_list = (**current_list).clone();
                            new_list.push(player.clone());
                            new_list
                        });
                    }

                    (Some(destination), position, yaw, pitch)
                }
            } else {
                warn!("Server dropped during cross-dimension respawn");
                (None, position, yaw, pitch)
            }
        } else {
            if respawn_dimension != self.dimension {
                warn!(
                    "Target world {:?} not found, using world spawn in {:?}",
                    respawn_dimension, self.dimension
                );
            }
            (None, position, yaw, pitch)
        };

        // Cancelled or unresolved cross-dimension respawns fall back to the current
        // world's spawn below; otherwise the resolved values from the event apply.
        let (target_world, position, yaw, pitch) = if let Some(ref new_world) = resolved_world {
            (new_world.clone(), position, yaw, pitch)
        } else if respawn_dimension != self.dimension {
            // FIXME: This spawn position calculation is incorrect. Should use vanilla's
            // proper spawn position calculation (see #1381).
            let chunk_pos = Vector2::new(spawn_x >> 4, spawn_z >> 4);
            self.level.get_or_fetch_chunk(chunk_pos, |_| ()).await;
            let top = self.get_top_block(Vector2::new(spawn_x, spawn_z));
            let fallback_pos = Vector3::new(
                f64::from(spawn_x) + 0.5,
                (top + 1).into(),
                f64::from(spawn_z) + 0.5,
            );
            (self.clone(), fallback_pos, spawn_yaw, spawn_pitch)
        } else {
            (self.clone(), position, yaw, pitch)
        };

        // Notify plugins that the player has respawned (non-cancellable).
        if let Some(server) = self.server.upgrade() {
            let _ = server
                .plugin_manager
                .fire(PlayerRespawnEvent::new(
                    player.clone(),
                    self.clone(),
                    target_world.clone(),
                    position,
                    yaw,
                    pitch,
                    alive,
                ))
                .await;
        }

        // Send respawn packet with target dimension (using send_packet_now to ensure proper order)
        player
            .client
            .send_packet_now(&CRespawn::new(
                PlayerSpawnData::new(
                    target_world.dimension.clone(),
                    biome::hash_seed(target_world.level.seed.0),
                    player.gamemode.load() as u8,
                    player.gamemode.load() as i8,
                    false,
                    false,
                    Some((death_dimension, death_location)),
                    VarInt(player.get_entity().portal_cooldown.load(Ordering::Relaxed) as i32),
                    target_world.sea_level.into(),
                ),
                data_kept,
            ))
            .await;

        // Inform the client of the default spawn position so the client doesn't
        // fall back to (0, 2, 0) while the world reloads (fixes rubberbanding).
        // This must be sent after the CRespawn packet for proper client positioning.
        let spawn_block_pos = BlockPos(Vector3::new(
            position.x.round() as i32,
            position.y.round() as i32,
            position.z.round() as i32,
        ));
        let bedrock_dimension = match target_world.dimension.minecraft_name {
            "minecraft:the_nether" => 1,
            "minecraft:the_end" => 2,
            _ => 0,
        };
        player
            .client
            .send_packet_now_editioned(
                &CPlayerSpawnPosition::new(
                    spawn_block_pos,
                    yaw,
                    pitch,
                    target_world.dimension.minecraft_name.to_string(),
                ),
                &pumpkin_protocol::bedrock::client::CSetSpawnPosition::new(
                    1, // World spawn
                    spawn_block_pos,
                    bedrock_dimension,
                    spawn_block_pos,
                ),
            )
            .await;

        player.living_entity.reset_state().await;

        player.send_permission_lvl_update();

        player.hunger_manager.restart();

        if !keep_inventory {
            player.set_experience(0, 0.0, 0).await;
            player.inventory.clear().await;
        }

        // Set entity position BEFORE loading chunks, so chunks load at the right location
        // This mirrors the initial spawn flow where update_position is called before teleport
        player.get_entity().set_pos(position);
        player.get_entity().set_rotation(yaw, pitch);
        player.get_entity().last_pos.store(position);

        // TODO: difficulty, exp bar, status effect

        // Load chunks and send world info FIRST (before teleport packet)
        target_world
            .send_world_info(player, position, yaw, pitch)
            .await;

        // Ensure at least the center chunk is sent synchronously before teleport.
        if let crate::net::ClientPlatform::Java(java_client) = player.client.as_ref() {
            let center_chunk = player.get_entity().chunk_pos.load();
            let chunk = target_world
                .level
                .get_or_fetch_chunk(center_chunk, std::clone::Clone::clone)
                .await;
            java_client.send_packet_now(&CChunkBatchStart).await;
            java_client.send_packet_now(&CChunkData(&chunk)).await;
            java_client
                .send_packet_now(&CChunkBatchEnd::new(1u16))
                .await;
        }

        // Send teleport packet after at least the center chunk was delivered
        player.request_teleport(position, yaw, pitch).await;
    }

    /// Gets a `Player` by an entity id
    pub fn get_player_by_id(&self, id: i32) -> Option<Arc<Player>> {
        for player in self.players.load().iter() {
            if player.entity_id() == id {
                return Some(player.clone());
            }
        }
        None
    }

    /// Gets a `Player` by a username
    pub fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        for player in self.players.load().iter() {
            if player.gameprofile.name.eq_ignore_ascii_case(name) {
                return Some(player.clone());
            }
        }
        None
    }

    // Gets all Player entities at a Box
    pub fn get_players_at_box(&self, aabb: &BoundingBox) -> Vec<Arc<Player>> {
        let players_guard = self.players.load();
        players_guard
            .iter()
            .filter(|player| player.get_entity().bounding_box.load().intersects(aabb))
            .cloned()
            .collect()
    }

    /// Retrieves a player by their unique UUID.
    ///
    /// This function searches the world's active player list for a player with the specified UUID.
    /// If found, it returns an `Arc<Player>` reference to the player. Otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `id`: The UUID of the player to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<Arc<Player>>` containing the player if found, or `None` if not.
    pub fn get_player_by_uuid(&self, id: uuid::Uuid) -> Option<Arc<Player>> {
        self.players
            .load()
            .iter()
            .find(|p| p.gameprofile.id == id)
            .cloned()
    }

    /// Gets a list of players whose location equals the given position in the world.
    ///
    /// It iterates through the players in the world and checks their location. If the player's location matches the
    /// given position, it will add this to a `Vec` which it later returns. If no
    /// player was found in that position, it will just return an empty `Vec`.
    ///
    /// # Arguments
    ///
    /// * `position`: The position the function will check.
    pub fn get_players_by_pos(&self, position: BlockPos) -> Vec<Arc<Player>> {
        self.players
            .load()
            .iter()
            .filter_map(|player| {
                let player_block_pos = player.get_entity().block_pos.load().0;
                (position.0.x == player_block_pos.x
                    && position.0.y == player_block_pos.y
                    && position.0.z == player_block_pos.z)
                    .then(|| Arc::clone(player))
            })
            .collect::<_>()
    }

    /// Gets the nearby players around a given world position.
    /// It "creates" a sphere and checks if whether players are inside
    /// and returns a `HashMap` where the UUID is the key and the `Player`
    /// object is the value.
    ///
    /// # Arguments
    /// * `pos`: The center of the sphere.
    /// * `radius`: The radius of the sphere. The higher the radius, the more area will be checked (in every direction).
    pub fn get_nearby_players(&self, pos: Vector3<f64>, radius: f64) -> Vec<Arc<Player>> {
        let radius_squared = radius.powi(2);

        self.players
            .load()
            .iter()
            .filter_map(|player| {
                let player_pos = player.get_entity().pos.load();
                (player_pos.squared_distance_to_vec(&pos) <= radius_squared).then(|| player.clone())
            })
            .collect()
    }

    pub fn get_closest_player(&self, pos: Vector3<f64>, radius: f64) -> Option<Arc<Player>> {
        let players = self.get_nearby_players(pos, radius);
        players
            .iter()
            .min_by(|a, b| {
                a.get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&pos)
                    .partial_cmp(&b.get_entity().pos.load().squared_distance_to_vec(&pos))
                    .unwrap()
            })
            .cloned()
    }

    /// Adds a player to the world and broadcasts a join message if enabled.
    ///
    /// This function takes a player's UUID and an `Arc<Player>` reference.
    /// It inserts the player into the world's `current_players` map using the UUID as the key.
    /// Additionally, it broadcasts a join message to all connected players in the world.
    ///
    /// # Arguments
    ///
    /// * `player`: An `Arc<Player>` reference to the player object.
    pub fn add_player(&self, player: &Arc<Player>) -> Result<(), String> {
        self.players.rcu(|current_list| {
            let mut new_list = (**current_list).clone();
            new_list.push(player.clone());
            new_list
        });
        Ok(())
    }

    /// Removes a player from the world and broadcasts a disconnect message if enabled.
    ///
    /// This function removes a player from the world based on their `Player` reference.
    /// It performs the following actions:
    ///
    /// 1. Removes the player from the `current_players` map using their UUID.
    /// 2. Broadcasts a `CRemovePlayerInfo` packet to all connected players to inform them about the player leaving.
    /// 3. Removes the player's entity from the world using its entity ID.
    /// 4. Optionally sends a disconnect message to all other players notifying them about the player leaving.
    ///
    /// # Arguments
    ///
    /// * `player`: A reference to the `Player` object to be removed.
    /// * `fire_event`: A boolean flag indicating whether to fire a `PlayerLeaveEvent` event.
    ///
    /// # Notes
    ///
    /// - This function assumes `broadcast_packet_expect` and `remove_entity` are defined elsewhere.
    /// - The disconnect message sending is currently optional. Consider making it a configurable option.
    pub async fn remove_player(
        &self,
        player: &Arc<Player>,
        fire_event: bool,
    ) -> Option<Arc<Player>> {
        let mut removed_player: Option<Arc<Player>> = None;

        self.players.rcu(|current_list| {
            let mut new_list = (**current_list).clone();
            // Find the player before we filter them out
            if let Some(pos) = new_list
                .iter()
                .position(|p| p.gameprofile.id == player.gameprofile.id)
            {
                removed_player = Some(new_list.remove(pos));
            }
            new_list
        });
        if let Some(ref player) = removed_player {
            let uuid = player.gameprofile.id;
            let entity_id = player.entity_id();

            let bedrock_remove_player = CPlayerList {
                action: CPlayerList::ACTION_REMOVE,
                entries: vec![PlayerListEntry {
                    uuid,
                    entity_unique_id: VarLong(entity_id as i64),
                    username: player.gameprofile.name.clone(),
                    xuid: String::new(),
                    platform_chat_id: String::new(),
                    build_platform: 0,
                    skin: Skin::steve(),
                    is_teacher: false,
                    is_host: false,
                    is_sub_client: false,
                    player_color: [0, 0, 0, 0],
                }],
            };

            self.broadcast_editioned(&CRemovePlayerInfo::new(&[uuid]), &bedrock_remove_player)
                .await;

            self.broadcast_editioned(
                &CRemoveEntities::new(&[entity_id.into()]),
                &CRemoveActor::new(VarLong(entity_id as i64)),
            )
            .await;

            if fire_event {
                let msg_comp = TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_PLAYER_LEFT,
                    translation::bedrock::MULTIPLAYER_PLAYER_LEFT,
                    [TextComponent::text(player.gameprofile.name.clone())],
                )
                .color_named(NamedColor::Yellow);
                let event = PlayerLeaveEvent::new(player.clone(), msg_comp);

                let event = self
                    .server
                    .upgrade()
                    .unwrap()
                    .plugin_manager
                    .fire(event)
                    .await;

                if !event.cancelled {
                    for player in self.players.load().iter() {
                        player.send_system_message(&event.leave_message).await;
                    }
                    info!("{}", event.leave_message.to_pretty_console());
                }
            }
        }
        removed_player
    }
}
