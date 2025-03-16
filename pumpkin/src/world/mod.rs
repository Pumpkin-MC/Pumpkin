use std::{
    collections::HashMap,
    sync::{Arc, atomic::Ordering},
};

pub mod chunker;
pub mod explosion;
pub mod time;

use crate::{
    PLUGIN_MANAGER, block,
    command::client_suggestions,
    entity::{Entity, EntityBase, EntityId, player::Player},
    error::PumpkinError,
    plugin::{
        block::block_break::BlockBreakEvent,
        player::{player_join::PlayerJoinEvent, player_leave::PlayerLeaveEvent},
        world::{chunk_load::ChunkLoad, chunk_save::ChunkSave, chunk_send::ChunkSend},
    },
    server::Server,
};
use border::Worldborder;
use explosion::Explosion;
use pumpkin_config::BasicConfiguration;
use pumpkin_data::{
    entity::{EntityStatus, EntityType},
    particle::Particle,
    sound::{Sound, SoundCategory},
    world::WorldEvent,
};
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::{
    ClientPacket,
    client::play::{
        CEntityStatus, CGameEvent, CLogin, CPlayerInfoUpdate, CRemoveEntities, CRemovePlayerInfo,
        CSpawnEntity, GameEvent, PlayerAction,
    },
};
use pumpkin_protocol::{client::play::CLevelEvent, codec::identifier::Identifier};
use pumpkin_protocol::{
    client::play::{
        CBlockUpdate, CDisguisedChatMessage, CExplosion, CRespawn, CSetBlockDestroyStage,
        CWorldEvent,
    },
    codec::var_int::VarInt,
};
use pumpkin_registry::DimensionType;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_world::level::Level;
use pumpkin_world::level::SyncChunk;
use pumpkin_world::{block::BlockDirection, chunk::ChunkData};
use pumpkin_world::{
    block::registry::{
        get_block_and_state_by_state_id, get_block_by_state_id, get_state_by_state_id,
    },
    coordinates::ChunkRelativeBlockCoordinates,
};
use rand::{Rng, thread_rng};
use scoreboard::Scoreboard;
use thiserror::Error;
use time::LevelTime;
use tokio::sync::{Mutex, mpsc::UnboundedReceiver};
use tokio::sync::{RwLock, mpsc};

pub mod border;
pub mod bossbar;
pub mod custom_bossbar;
pub mod scoreboard;
pub mod weather;

use weather::Weather;

#[derive(Debug, Error)]
pub enum GetBlockError {
    BlockOutOfWorldBounds,
    InvalidBlockId,
}

impl std::fmt::Display for GetBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PumpkinError for GetBlockError {
    fn is_kick(&self) -> bool {
        false
    }

    fn severity(&self) -> log::Level {
        log::Level::Warn
    }

    fn client_kick_reason(&self) -> Option<String> {
        None
    }
}

/// Represents a Minecraft world, containing entities, players, and the underlying level data.
///
/// Each dimension (Overworld, Nether, End) typically has its own `World`.
///
/// **Key Responsibilities:**
///
/// - Manages the `Level` instance for handling chunk-related operations.
/// - Stores and tracks active `Player` entities within the world.
/// - Provides a central hub for interacting with the world's entities and environment.
pub struct World {
    /// The underlying level, responsible for chunk management and terrain generation.
    pub level: Arc<Level>,
    /// A map of active players within the world, keyed by their unique UUID.
    pub players: Arc<RwLock<HashMap<uuid::Uuid, Arc<Player>>>>,
    /// A map of active entities within the world, keyed by their unique UUID.
    /// This does not include players.
    pub entities: Arc<RwLock<HashMap<uuid::Uuid, Arc<dyn EntityBase>>>>,
    /// The world's scoreboard, used for tracking scores, objectives, and display information.
    pub scoreboard: Mutex<Scoreboard>,
    /// The world's worldborder, defining the playable area and controlling its expansion or contraction.
    pub worldborder: Mutex<Worldborder>,
    /// The world's time, including counting ticks for weather, time cycles, and statistics.
    pub level_time: Mutex<LevelTime>,
    /// The type of dimension the world is in.
    pub dimension_type: DimensionType,
    /// The world's weather, including rain and thunder levels.
    pub weather: Mutex<Weather>,
    // TODO: entities
}

impl World {
    #[must_use]
    pub fn load(level: Level, dimension_type: DimensionType) -> Self {
        Self {
            level: Arc::new(level),
            players: Arc::new(RwLock::new(HashMap::new())),
            entities: Arc::new(RwLock::new(HashMap::new())),
            scoreboard: Mutex::new(Scoreboard::new()),
            worldborder: Mutex::new(Worldborder::new(0.0, 0.0, 29_999_984.0, 0, 0, 0)),
            level_time: Mutex::new(LevelTime::new()),
            dimension_type,
            weather: Mutex::new(Weather::new()),
        }
    }

    pub async fn save(&self) {
        self.level.save().await;
    }

    pub async fn send_entity_status(&self, entity: &Entity, status: EntityStatus) {
        // TODO: only nearby
        self.broadcast_packet_all(&CEntityStatus::new(entity.entity_id, status as i8))
            .await;
    }

    /// Broadcasts a packet to all connected players within the world.
    ///
    /// Sends the specified packet to every player currently logged in to the world.
    ///
    /// **Note:** This function acquires a lock on the `current_players` map, ensuring thread safety.
    pub async fn broadcast_packet_all<P>(&self, packet: &P)
    where
        P: ClientPacket,
    {
        let current_players = self.players.read().await;
        for player in current_players.values() {
            player.client.send_packet(packet).await;
        }
    }

    pub async fn broadcast_message(
        &self,
        message: &TextComponent,
        sender_name: &TextComponent,
        chat_type: u32,
        target_name: Option<&TextComponent>,
    ) {
        self.broadcast_packet_all(&CDisguisedChatMessage::new(
            message,
            (chat_type + 1).into(),
            sender_name,
            target_name,
        ))
        .await;
    }

    /// Broadcasts a packet to all connected players within the world, excluding the specified players.
    ///
    /// Sends the specified packet to every player currently logged in to the world, excluding the players listed in the `except` parameter.
    ///
    /// **Note:** This function acquires a lock on the `current_players` map, ensuring thread safety.
    pub async fn broadcast_packet_except<P>(&self, except: &[uuid::Uuid], packet: &P)
    where
        P: ClientPacket,
    {
        let current_players = self.players.read().await;
        for (_, player) in current_players.iter().filter(|c| !except.contains(c.0)) {
            player.client.send_packet(packet).await;
        }
    }

    pub async fn spawn_particle(
        &self,
        position: Vector3<f64>,
        offset: Vector3<f32>,
        max_speed: f32,
        particle_count: i32,
        pariticle: Particle,
    ) {
        let players = self.players.read().await;
        for (_, player) in players.iter() {
            player
                .spawn_particle(position, offset, max_speed, particle_count, pariticle)
                .await;
        }
    }

    pub async fn play_sound(&self, sound: Sound, category: SoundCategory, position: &Vector3<f64>) {
        self.play_sound_raw(sound as u16, category, position, 1.0, 1.0)
            .await;
    }

    pub async fn play_sound_raw(
        &self,
        sound_id: u16,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
    ) {
        let seed = thread_rng().r#gen::<f64>();
        let players = self.players.read().await;
        for (_, player) in players.iter() {
            player
                .play_sound(sound_id, category, position, volume, pitch, seed)
                .await;
        }
    }

    pub async fn play_block_sound(
        &self,
        sound: Sound,
        category: SoundCategory,
        position: BlockPos,
    ) {
        let new_vec = Vector3::new(
            f64::from(position.0.x) + 0.5,
            f64::from(position.0.y) + 0.5,
            f64::from(position.0.z) + 0.5,
        );
        self.play_sound(sound, category, &new_vec).await;
    }

    pub async fn play_record(&self, record_id: i32, position: BlockPos) {
        self.broadcast_packet_all(&CLevelEvent::new(
            WorldEvent::JukeboxStartsPlaying as i32,
            position,
            record_id,
            false,
        ))
        .await;
    }

    pub async fn stop_record(&self, position: BlockPos) {
        self.broadcast_packet_all(&CLevelEvent::new(
            WorldEvent::JukeboxStopsPlaying as i32,
            position,
            0,
            false,
        ))
        .await;
    }

    pub async fn tick(&self, server: &Server) {
        // World ticks
        {
            let mut level_time = self.level_time.lock().await;
            level_time.tick_time();
            if level_time.world_age % 20 == 0 {
                level_time.send_time(self).await;
            }
        }

        {
            let mut weather = self.weather.lock().await;
            weather.tick_weather(self).await;
        };

        // Player ticks
        for player in self.players.read().await.values() {
            player.tick(server).await;
        }

        let entities_to_tick: Vec<_> = self.entities.read().await.values().cloned().collect();

        // Entity ticks
        for entity in entities_to_tick {
            entity.tick(server).await;
            // This boolean thing prevents deadlocks. Since we lock players, we can't broadcast packets.
            let mut collied_player = None;
            for player in self.players.read().await.values() {
                if player
                    .living_entity
                    .entity
                    .bounding_box
                    .load()
                    // This is vanilla, but TODO: change this when is in a vehicle
                    .expand(1.0, 0.5, 1.0)
                    .intersects(&entity.get_entity().bounding_box.load())
                {
                    collied_player = Some(player.clone());
                    break;
                }
            }
            if let Some(player) = collied_player {
                entity.on_player_collision(player).await;
            }
        }
    }

    /// Gets the y position of the first non air block from the top down
    pub async fn get_top_block(&self, position: Vector2<i32>) -> i32 {
        for y in (-64..=319).rev() {
            let pos = BlockPos(Vector3::new(position.x, y, position.z));
            let block = self.get_block_state(&pos).await;
            if let Ok(block) = block {
                if block.air {
                    continue;
                }
            }
            return y;
        }
        319
    }

    #[expect(clippy::too_many_lines)]
    pub async fn spawn_player(
        &self,
        base_config: &BasicConfiguration,
        player: Arc<Player>,
        server: &Server,
    ) {
        let dimensions: Vec<Identifier> =
            server.dimensions.iter().map(DimensionType::name).collect();

        // This code follows the vanilla packet order
        let entity_id = player.entity_id();
        let gamemode = player.gamemode.load();
        log::debug!(
            "spawning player {}, entity id {}",
            player.gameprofile.name,
            entity_id
        );

        // Send the login packet for our new player
        player
            .client
            .send_packet(&CLogin::new(
                entity_id,
                base_config.hardcore,
                &dimensions,
                base_config.max_players.into(),
                base_config.view_distance.get().into(), //  TODO: view distance
                base_config.simulation_distance.get().into(), // TODO: sim view dinstance
                false,
                true,
                false,
                (self.dimension_type as u8).into(),
                self.dimension_type.name(),
                0, // seed
                gamemode as u8,
                base_config.default_gamemode as i8,
                false,
                false,
                None,
                0.into(),
                0.into(),
                false,
            ))
            .await;
        // Permissions, i.e. the commands a player may use.
        player.send_permission_lvl_update().await;
        client_suggestions::send_c_commands_packet(&player, &server.command_dispatcher).await;
        // Teleport
        let info = &self.level.level_info;
        let mut position = Vector3::new(f64::from(info.spawn_x), 120.0, f64::from(info.spawn_z));
        let yaw = info.spawn_angle;
        let pitch = 10.0;

        let top = self
            .get_top_block(Vector2::new(position.x as i32, position.z as i32))
            .await;
        position.y = f64::from(top + 1);

        log::debug!("Sending player teleport to {}", player.gameprofile.name);
        player.request_teleport(position, yaw, pitch).await;

        player.living_entity.last_pos.store(position);

        let gameprofile = &player.gameprofile;
        // Firstly, send an info update to our new player, so they can see their skin
        // and also send their info to everyone else.
        log::debug!("Broadcasting player info for {}", player.gameprofile.name);
        self.broadcast_packet_all(&CPlayerInfoUpdate::new(
            0x01 | 0x04 | 0x08,
            &[pumpkin_protocol::client::play::Player {
                uuid: gameprofile.id,
                actions: vec![
                    PlayerAction::AddPlayer {
                        name: &gameprofile.name,
                        properties: &gameprofile.properties,
                    },
                    PlayerAction::UpdateListed(true),
                    PlayerAction::UpdateGameMode(VarInt(gamemode as i32)),
                ],
            }],
        ))
        .await;
        player.send_client_information().await;

        // Here, we send all the infos of players who already joined.
        let mut entries = Vec::new();
        {
            let current_players = self.players.read().await;
            for (_, playerr) in current_players
                .iter()
                .filter(|(c, _)| **c != player.gameprofile.id)
            {
                let gameprofile = &playerr.gameprofile;
                entries.push(pumpkin_protocol::client::play::Player {
                    uuid: gameprofile.id,
                    actions: vec![
                        PlayerAction::AddPlayer {
                            name: &gameprofile.name,
                            properties: &gameprofile.properties,
                        },
                        PlayerAction::UpdateListed(true),
                    ],
                });
            }
            log::debug!("Sending player info to {}", player.gameprofile.name);
            player
                .client
                .send_packet(&CPlayerInfoUpdate::new(0x01 | 0x08, &entries))
                .await;
        };

        let gameprofile = &player.gameprofile;

        log::debug!("Broadcasting player spawn for {}", player.gameprofile.name);
        // Spawn the player for every client.
        self.broadcast_packet_except(
            &[player.gameprofile.id],
            // TODO: add velo
            &CSpawnEntity::new(
                entity_id.into(),
                gameprofile.id,
                i32::from(EntityType::PLAYER.id).into(),
                position,
                pitch,
                yaw,
                yaw,
                0.into(),
                Vector3::new(0.0, 0.0, 0.0),
            ),
        )
        .await;
        // Spawn players for our client.
        let id = player.gameprofile.id;
        for (_, existing_player) in self.players.read().await.iter().filter(|c| c.0 != &id) {
            let entity = &existing_player.living_entity.entity;
            let pos = entity.pos.load();
            let gameprofile = &existing_player.gameprofile;
            log::debug!("Sending player entities to {}", player.gameprofile.name);
            player
                .client
                .send_packet(&CSpawnEntity::new(
                    existing_player.entity_id().into(),
                    gameprofile.id,
                    i32::from(EntityType::PLAYER.id).into(),
                    pos,
                    entity.yaw.load(),
                    entity.pitch.load(),
                    entity.head_yaw.load(),
                    0.into(),
                    Vector3::new(0.0, 0.0, 0.0),
                ))
                .await;
        }
        // Entity meta data
        // Set skin parts
        player.send_client_information().await;

        // Start waiting for level chunks. Sets the "Loading Terrain" screen
        log::debug!("Sending waiting chunks to {}", player.gameprofile.name);
        player
            .client
            .send_packet(&CGameEvent::new(GameEvent::StartWaitingChunks, 0.0))
            .await;

        self.worldborder
            .lock()
            .await
            .init_client(&player.client)
            .await;

        // Sends initial time
        player.send_time(self).await;

        // Send initial weather state
        let weather = self.weather.lock().await;
        if weather.raining {
            player
                .client
                .send_packet(&CGameEvent::new(GameEvent::BeginRaining, 0.0))
                .await;

            // Calculate rain and thunder levels directly from public fields
            let rain_level = weather.rain_level.clamp(0.0, 1.0);
            let thunder_level = weather.thunder_level.clamp(0.0, 1.0);

            player
                .client
                .send_packet(&CGameEvent::new(GameEvent::RainLevelChange, rain_level))
                .await;
            player
                .client
                .send_packet(&CGameEvent::new(
                    GameEvent::ThunderLevelChange,
                    thunder_level,
                ))
                .await;
        }

        // Spawn in initial chunks
        chunker::player_join(&player).await;

        // if let Some(bossbars) = self..lock().await.get_player_bars(&player.gameprofile.id) {
        //     for bossbar in bossbars {
        //         player.send_bossbar(bossbar).await;
        //     }
        // }

        player.send_mobs(self).await;
    }

    pub async fn send_world_info(
        &self,
        player: &Arc<Player>,
        position: Vector3<f64>,
        yaw: f32,
        pitch: f32,
    ) {
        self.worldborder
            .lock()
            .await
            .init_client(&player.client)
            .await;

        // TODO: World spawn (compass stuff)

        player
            .client
            .send_packet(&CGameEvent::new(GameEvent::StartWaitingChunks, 0.0))
            .await;

        let entity = &player.living_entity.entity;

        self.broadcast_packet_except(
            &[player.gameprofile.id],
            // TODO: add velo
            &CSpawnEntity::new(
                entity.entity_id.into(),
                player.gameprofile.id,
                i32::from(EntityType::PLAYER.id).into(),
                position,
                pitch,
                yaw,
                yaw,
                0.into(),
                Vector3::new(0.0, 0.0, 0.0),
            ),
        )
        .await;
        player.send_client_information().await;

        chunker::player_join(player).await;
        // Update commands

        player.set_health(20.0).await;
    }

    pub async fn explode(self: &Arc<Self>, server: &Server, position: Vector3<f64>, power: f32) {
        let explosion = Explosion::new(power, position);
        explosion.explode(server, self).await;
        let particle = if power < 2.0 {
            Particle::Explosion
        } else {
            Particle::ExplosionEmitter
        };
        let sound = pumpkin_protocol::IDOrSoundEvent {
            id: VarInt(Sound::EntityGenericExplode as i32 + 1),
            sound_event: None,
        };
        for (_, player) in self.players.read().await.iter() {
            if player.position().squared_distance_to_vec(position) > 4096.0 {
                continue;
            }
            player
                .client
                .send_packet(&CExplosion::new(
                    position,
                    None,
                    VarInt(particle as i32),
                    sound.clone(),
                ))
                .await;
        }
    }

    pub async fn respawn_player(&self, player: &Arc<Player>, alive: bool) {
        let last_pos = player.living_entity.last_pos.load();
        let death_dimension = player.world().await.dimension_type.name();
        let death_location = BlockPos(Vector3::new(
            last_pos.x.round() as i32,
            last_pos.y.round() as i32,
            last_pos.z.round() as i32,
        ));

        let data_kept = u8::from(alive);

        // TODO: switch world in player entity to new world

        player
            .client
            .send_packet(&CRespawn::new(
                (self.dimension_type as u8).into(),
                self.dimension_type.name(),
                0, // seed
                player.gamemode.load() as u8,
                player.gamemode.load() as i8,
                false,
                false,
                Some((death_dimension, death_location)),
                0.into(),
                0.into(),
                data_kept,
            ))
            .await;

        log::debug!("Sending player abilities to {}", player.gameprofile.name);
        player.send_abilities_update().await;

        player.send_permission_lvl_update().await;

        // Teleport
        let info = &self.level.level_info;
        let mut position = Vector3::new(f64::from(info.spawn_x), 120.0, f64::from(info.spawn_z));
        let yaw = info.spawn_angle;
        let pitch = 10.0;

        let top = self
            .get_top_block(Vector2::new(position.x as i32, position.z as i32))
            .await;
        position.y = f64::from(top + 1);

        log::debug!("Sending player teleport to {}", player.gameprofile.name);
        player.clone().request_teleport(position, yaw, pitch).await;

        player.living_entity.last_pos.store(position);

        // TODO: difficulty, exp bar, status effect

        self.send_world_info(player, position, yaw, pitch).await;
    }

    // NOTE: This function doesn't actually await on anything, it just spawns two tokio tasks
    /// IMPORTANT: Chunks have to be non-empty
    fn spawn_world_chunks(
        &self,
        player: Arc<Player>,
        chunks: Vec<Vector2<i32>>,
        center_chunk: Vector2<i32>,
    ) {
        if player
            .client
            .closed
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            log::info!("The connection has closed before world chunks were spawned");
            return;
        }
        #[cfg(debug_assertions)]
        let inst = std::time::Instant::now();

        // Sort such that the first chunks are closest to the center.
        let mut chunks = chunks;
        chunks.sort_unstable_by_key(|pos| {
            let rel_x = pos.x - center_chunk.x;
            let rel_z = pos.z - center_chunk.z;
            rel_x * rel_x + rel_z * rel_z
        });

        // We are loading a completely new world section; prioritize chunks the player is on top
        // of
        let new_spawn = chunks[0] == player.watched_section.load().center;
        let mut receiver = self.receive_chunks(chunks, new_spawn);
        let level = self.level.clone();

        // Only allow 128 chunk packets to be sent at a time to avoid overloading the client.
        // TODO: Bulk chunks?

        tokio::spawn(async move {
            'main: while let Some((chunk, first_load)) = receiver.recv().await {
                let position = chunk.read().await.position;

                #[cfg(debug_assertions)]
                if position == (0, 0).into() {
                    use pumpkin_protocol::client::play::CChunkData;
                    let binding = chunk.read().await;
                    let packet = CChunkData(&binding);
                    let mut test = bytes::BytesMut::new();
                    packet.write(&mut test);
                    let len = test.len();
                    log::debug!(
                        "Chunk packet size: {}B {}KB {}MB",
                        len,
                        len / 1024,
                        len / (1024 * 1024)
                    );
                }

                let (world, chunk) = if level.is_chunk_watched(&position) {
                    (player.world().await.clone(), chunk)
                } else {
                    send_cancellable! {{
                        ChunkSave {
                            world: player.world().await.clone(),
                            chunk,
                            cancelled: false,
                        };

                        'after: {
                            log::trace!(
                                "Received chunk {:?}, but it is no longer watched... cleaning",
                                &position
                            );
                            level.clean_chunk(&position).await;
                            continue 'main;
                        }
                    }};
                    (event.world, event.chunk)
                };

                let (world, chunk) = if first_load {
                    send_cancellable! {{
                        ChunkLoad {
                            world,
                            chunk,
                            cancelled: false,
                        };

                        'cancelled: {
                            continue 'main;
                        }
                    }}
                    (event.world, event.chunk)
                } else {
                    (world, chunk)
                };

                if !player.client.closed.load(Ordering::Relaxed) {
                    send_cancellable! {{
                        ChunkSend {
                            world,
                            chunk: chunk.clone(),
                            cancelled: false,
                        };

                        'after: {
                            let mut chunk_manager = player.chunk_manager.lock().await;
                            chunk_manager.push_chunk(position, chunk);
                        }
                    }};
                }
            }

            #[cfg(debug_assertions)]
            log::debug!("Chunks queued after {}ms", inst.elapsed().as_millis());
        });
    }

    /// Gets a `Player` by an entity id
    pub async fn get_player_by_id(&self, id: EntityId) -> Option<Arc<Player>> {
        for player in self.players.read().await.values() {
            if player.entity_id() == id {
                return Some(player.clone());
            }
        }
        None
    }

    /// Gets an entity by an entity id
    pub async fn get_entity_by_id(&self, id: EntityId) -> Option<Arc<dyn EntityBase>> {
        for entity in self.entities.read().await.values() {
            if entity.get_entity().entity_id == id {
                return Some(entity.clone());
            }
        }
        None
    }

    /// Gets a `Player` by a username
    pub async fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        for player in self.players.read().await.values() {
            if player.gameprofile.name.to_lowercase() == name.to_lowercase() {
                return Some(player.clone());
            }
        }
        None
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
    pub async fn get_player_by_uuid(&self, id: uuid::Uuid) -> Option<Arc<Player>> {
        return self.players.read().await.get(&id).cloned();
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
    pub async fn get_players_by_pos(&self, position: BlockPos) -> HashMap<uuid::Uuid, Arc<Player>> {
        self.players
            .read()
            .await
            .iter()
            .filter_map(|(uuid, player)| {
                let player_block_pos = player.living_entity.entity.block_pos.load().0;
                (position.0.x == player_block_pos.x
                    && position.0.y == player_block_pos.y
                    && position.0.z == player_block_pos.z)
                    .then(|| (*uuid, Arc::clone(player)))
            })
            .collect::<HashMap<uuid::Uuid, Arc<Player>>>()
    }

    /// Gets the nearby players around a given world position.
    /// It "creates" a sphere and checks if whether players are inside
    /// and returns a `HashMap` where the UUID is the key and the `Player`
    /// object is the value.
    ///
    /// # Arguments
    /// * `pos`: The center of the sphere.
    /// * `radius`: The radius of the sphere. The higher the radius,
    ///             the more area will be checked (in every direction).
    pub async fn get_nearby_players(
        &self,
        pos: Vector3<f64>,
        radius: f64,
    ) -> HashMap<uuid::Uuid, Arc<Player>> {
        let radius_squared = radius.powi(2);

        self.players
            .read()
            .await
            .iter()
            .filter_map(|(id, player)| {
                let player_pos = player.living_entity.entity.pos.load();
                (player_pos.squared_distance_to_vec(pos) <= radius_squared)
                    .then(|| (*id, player.clone()))
            })
            .collect()
    }

    pub async fn get_closest_player(&self, pos: Vector3<f64>, radius: f64) -> Option<Arc<Player>> {
        let players = self.get_nearby_players(pos, radius).await;
        players
            .iter()
            .min_by(|a, b| {
                a.1.living_entity
                    .entity
                    .pos
                    .load()
                    .squared_distance_to_vec(pos)
                    .partial_cmp(
                        &b.1.living_entity
                            .entity
                            .pos
                            .load()
                            .squared_distance_to_vec(pos),
                    )
                    .unwrap()
            })
            .map(|p| p.1.clone())
    }

    /// Adds a player to the world and broadcasts a join message if enabled.
    ///
    /// This function takes a player's UUID and an `Arc<Player>` reference.
    /// It inserts the player into the world's `current_players` map using the UUID as the key.
    /// Additionally, it broadcasts a join message to all connected players in the world.
    ///
    /// # Arguments
    ///
    /// * `uuid`: The unique UUID of the player to add.
    /// * `player`: An `Arc<Player>` reference to the player object.
    pub async fn add_player(&self, uuid: uuid::Uuid, player: Arc<Player>) {
        {
            let mut current_players = self.players.write().await;
            current_players.insert(uuid, player.clone())
        };

        let current_players = self.players.clone();
        tokio::spawn(async move {
            let msg_comp = TextComponent::translate(
                "multiplayer.player.joined",
                [TextComponent::text(player.gameprofile.name.clone())],
            )
            .color_named(NamedColor::Yellow);
            let event = PlayerJoinEvent::new(player.clone(), msg_comp);

            let event = PLUGIN_MANAGER
                .lock()
                .await
                .fire::<PlayerJoinEvent>(event)
                .await;

            if !event.cancelled {
                let current_players = current_players.clone();
                let players = current_players.read().await;
                for player in players.values() {
                    player.send_system_message(&event.join_message).await;
                }
                log::info!("{}", event.join_message.clone().to_pretty_console());
            }
        });
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
    pub async fn remove_player(&self, player: &Arc<Player>, fire_event: bool) {
        self.players
            .write()
            .await
            .remove(&player.gameprofile.id)
            .unwrap();
        let uuid = player.gameprofile.id;
        self.broadcast_packet_except(
            &[player.gameprofile.id],
            &CRemovePlayerInfo::new(1.into(), &[uuid]),
        )
        .await;
        self.broadcast_packet_all(&CRemoveEntities::new(&[player.entity_id().into()]))
            .await;

        if fire_event {
            let msg_comp = TextComponent::translate(
                "multiplayer.player.left",
                [TextComponent::text(player.gameprofile.name.clone())],
            )
            .color_named(NamedColor::Yellow);
            let event = PlayerLeaveEvent::new(player.clone(), msg_comp);

            let event = PLUGIN_MANAGER
                .lock()
                .await
                .fire::<PlayerLeaveEvent>(event)
                .await;

            if !event.cancelled {
                let players = self.players.read().await;
                for player in players.values() {
                    player.send_system_message(&event.leave_message).await;
                }
                log::info!("{}", event.leave_message.clone().to_pretty_console());
            }
        }
    }

    pub fn create_entity(
        self: &Arc<Self>,
        position: Vector3<f64>,
        entity_type: EntityType,
    ) -> Entity {
        let uuid = uuid::Uuid::new_v4();
        Entity::new(uuid, self.clone(), position, entity_type, false)
    }

    /// Adds an entity to the world.
    pub async fn spawn_entity(&self, entity: Arc<dyn EntityBase>) {
        let base_entity = entity.get_entity();
        self.broadcast_packet_all(&base_entity.create_spawn_packet())
            .await;
        let mut current_living_entities = self.entities.write().await;
        current_living_entities.insert(base_entity.entity_uuid, entity);
    }

    pub async fn remove_entity(&self, entity: &Entity) {
        self.entities.write().await.remove(&entity.entity_uuid);
        self.broadcast_packet_all(&CRemoveEntities::new(&[entity.entity_id.into()]))
            .await;
    }

    pub async fn set_block_breaking(&self, from: &Entity, location: BlockPos, progress: i32) {
        self.broadcast_packet_except(
            &[from.entity_uuid],
            &CSetBlockDestroyStage::new(from.entity_id.into(), location, progress as i8),
        )
        .await;
    }

    /// Sets a block.
    pub async fn set_block_state(&self, position: &BlockPos, block_state_id: u16) -> u16 {
        let (chunk_coordinate, relative_coordinates) = position.chunk_and_chunk_relative_position();

        // Since we divide by 16, remnant can never exceed `u8::MAX`
        let relative = ChunkRelativeBlockCoordinates::from(relative_coordinates);

        let chunk = self.receive_chunk(chunk_coordinate).await.0;
        let mut chunk = chunk.write().await;
        chunk.dirty = true;
        let replaced_block_state_id = chunk.blocks.get_block(relative).unwrap();
        chunk.blocks.set_block(relative, block_state_id);
        drop(chunk);

        self.broadcast_packet_all(&CBlockUpdate::new(
            position,
            i32::from(block_state_id).into(),
        ))
        .await;

        replaced_block_state_id
    }

    // Stream the chunks (don't collect them and then do stuff with them)
    /// Spawns a tokio task to stream chunks.
    /// Important: must be called from an async function (or changed to accept a tokio runtime
    /// handle)
    pub fn receive_chunks(
        &self,
        chunks: Vec<Vector2<i32>>,
        new_spawn: bool,
    ) -> UnboundedReceiver<(SyncChunk, bool)> {
        let (sender, receiver) = mpsc::unbounded_channel();
        // Put this in another thread so we aren't blocking on it
        let level = self.level.clone();
        tokio::spawn(async move {
            if new_spawn {
                if let Some((priority, rest)) = chunks.split_at_checked(9) {
                    // Ensure the client gets the 9 closest chunks first
                    level.fetch_chunks(priority, sender.clone()).await;
                    level.fetch_chunks(rest, sender).await;
                } else {
                    level.fetch_chunks(&chunks, sender).await;
                }
            } else {
                level.fetch_chunks(&chunks, sender).await;
            }
        });

        receiver
    }

    pub async fn receive_chunk(&self, chunk_pos: Vector2<i32>) -> (Arc<RwLock<ChunkData>>, bool) {
        let mut receiver = self.receive_chunks(vec![chunk_pos], false);

        receiver
            .recv()
            .await
            .expect("Channel closed for unknown reason")
    }

    /// If `server` is sent, it will do a block update.
    pub async fn break_block(
        self: &Arc<Self>,
        position: &BlockPos,
        cause: Option<Arc<Player>>,
        drop: bool,
        server: Option<&Server>,
    ) {
        let block = self.get_block(position).await.unwrap();
        let event = BlockBreakEvent::new(cause.clone(), block.clone(), 0, false);

        let event = PLUGIN_MANAGER
            .lock()
            .await
            .fire::<BlockBreakEvent>(event)
            .await;

        if !event.cancelled {
            let broken_block_state_id = self.set_block_state(position, 0).await;

            let particles_packet = CWorldEvent::new(
                WorldEvent::BlockBroken as i32,
                position,
                broken_block_state_id.into(),
                false,
            );

            if drop {
                block::drop_loot(self, &block, position, true, broken_block_state_id).await;
            }

            match cause {
                Some(player) => {
                    self.broadcast_packet_except(&[player.gameprofile.id], &particles_packet)
                        .await;
                }
                None => self.broadcast_packet_all(&particles_packet).await,
            }

            if let Some(server) = server {
                self.update_neighbors(server, position, None).await;
            }
        }
    }

    pub async fn get_block_state_id(&self, position: &BlockPos) -> Result<u16, GetBlockError> {
        let (chunk, relative) = position.chunk_and_chunk_relative_position();
        let relative = ChunkRelativeBlockCoordinates::from(relative);
        let chunk = self.receive_chunk(chunk).await.0;
        let chunk: tokio::sync::RwLockReadGuard<ChunkData> = chunk.read().await;

        let Some(id) = chunk.blocks.get_block(relative) else {
            return Err(GetBlockError::BlockOutOfWorldBounds);
        };

        Ok(id)
    }

    /// Gets a `Block` from the block registry. Returns `None` if the block was not found.
    pub async fn get_block(
        &self,
        position: &BlockPos,
    ) -> Result<pumpkin_data::block::Block, GetBlockError> {
        let id = self.get_block_state_id(position).await?;
        get_block_by_state_id(id).ok_or(GetBlockError::InvalidBlockId)
    }

    /// Gets the `BlockState` from the block registry. Returns `None` if the block state was not found.
    pub async fn get_block_state(
        &self,
        position: &BlockPos,
    ) -> Result<pumpkin_data::block::BlockState, GetBlockError> {
        let id = self.get_block_state_id(position).await?;
        get_state_by_state_id(id).ok_or(GetBlockError::InvalidBlockId)
    }

    /// Gets a `Block` + `BlockState` from the block registry. Returns `None` if the block state has not been found.
    pub async fn get_block_and_block_state(
        &self,
        position: &BlockPos,
    ) -> Result<(pumpkin_data::block::Block, pumpkin_data::block::BlockState), GetBlockError> {
        let id = self.get_block_state_id(position).await?;
        get_block_and_state_by_state_id(id).ok_or(GetBlockError::InvalidBlockId)
    }

    /// Updates neighboring blocks of a block.
    pub async fn update_neighbors(
        &self,
        server: &Server,
        block_pos: &BlockPos,
        except: Option<&BlockDirection>,
    ) {
        for direction in BlockDirection::update_order() {
            if Some(&direction) == except {
                continue;
            }
            let neighbor_pos = block_pos.offset(direction.to_offset());
            let neighbor_block = self.get_block(&neighbor_pos).await;
            if let Ok(neighbor_block) = neighbor_block {
                if let Some(neighbor_pumpkin_block) =
                    server.block_registry.get_pumpkin_block(&neighbor_block)
                {
                    neighbor_pumpkin_block
                        .on_neighbor_update(
                            server,
                            self,
                            &neighbor_block,
                            &neighbor_pos,
                            &direction,
                            block_pos,
                        )
                        .await;
                }
            }
        }
    }
}
