use std::pin::Pin;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Weak};
use std::{
    collections::{BTreeMap, HashMap},
    sync::atomic::Ordering,
};
use tracing::{debug, error, info, trace, warn};

pub mod chunker;
pub mod explosion;
pub mod loot;
pub mod portal;
pub mod time;

use crate::block::RandomTickArgs;
use crate::world::loot::LootContextParameters;
use crate::{
    block::BlockEvent, entity::experience_orb::ExperienceOrbEntity, entity::item::ItemEntity,
};
use crate::{
    block::{
        self,
        registry::BlockRegistry,
        {OnNeighborUpdateArgs, OnScheduledTickArgs},
    },
    command::client_suggestions,
    entity::{Entity, EntityBase, player::Player, r#type::from_type},
    error::PumpkinError,
    net::{ClientPlatform, java::JavaClient},
    plugin::{
        block::block_break::BlockBreakEvent,
        player::{player_join::PlayerJoinEvent, player_leave::PlayerLeaveEvent},
    },
    server::Server,
};
use arc_swap::ArcSwap;
use border::Worldborder;
use bytes::BufMut;
use explosion::Explosion;
use pumpkin_config::BasicConfiguration;
use pumpkin_data::block_properties::is_air;
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::MobCategory;
use pumpkin_data::fluid::{Falling, FluidProperties, FluidState};
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_data::{
    Block,
    entity::{EntityStatus, EntityType},
    fluid::Fluid,
    particle::Particle,
    sound::{Sound, SoundCategory},
    world::{RAW, WorldEvent},
};
use pumpkin_data::{BlockDirection, BlockState, translation};
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_nbt::{compound::NbtCompound, to_bytes_unnamed};
use pumpkin_protocol::bedrock::client::set_actor_data::{
    CSetActorData, EntityMetadata, MetadataValue, PropertySyncData, entity_data_flag,
    entity_data_key,
};
use pumpkin_protocol::bedrock::client::start_game::CStartGame;
use pumpkin_protocol::bedrock::frame_set::FrameSet;
use pumpkin_protocol::java::client::play::CPlayerSpawnPosition;
use pumpkin_protocol::java::client::play::{CSetEntityMetadata, Metadata};
use pumpkin_protocol::{
    BClientPacket, ClientPacket, IdOr, SoundEvent,
    bedrock::{
        client::{
            creative_content::{CreativeContent, Group},
            gamerules_changed::GameRules,
            play_status::CPlayStatus,
            start_game::{Experiments, GamePublishSetting, LevelSettings},
            update_artributes::{Attribute, CUpdateAttributes},
        },
        network_item::NetworkItemDescriptor,
        server::text::SText,
    },
    codec::{
        bedrock_block_pos::NetworkPos, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong,
    },
    java::{
        self,
        client::play::{
            CBlockEntityData, CEntityStatus, CGameEvent, CLogin, CMultiBlockUpdate,
            CPlayerChatMessage, CPlayerInfoUpdate, CRemoveEntities, CRemovePlayerInfo,
            CSetSelectedSlot, CSoundEffect, CSpawnEntity, FilterType, GameEvent, InitChat,
            PlayerAction, PlayerInfoFlags,
        },
        server::play::SChatMessage,
    },
};
use pumpkin_protocol::{
    codec::item_stack_seralizer::ItemStackSerializer,
    java::client::play::{CBlockEvent, CRemoveMobEffect, CSetEquipment},
};
use pumpkin_protocol::{
    codec::var_int::VarInt,
    java::client::play::{
        CBlockUpdate, CChunkBatchEnd, CChunkBatchStart, CChunkData, CDisguisedChatMessage,
        CExplosion, CRespawn, CSetBlockDestroyStage, CWorldEvent,
    },
};
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_util::version::MinecraftVersion;
use pumpkin_util::{
    Difficulty,
    math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3},
};
use pumpkin_util::{
    math::{position::chunk_section_from_pos, vector2::Vector2},
    random::{RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use pumpkin_world::inventory::Clearable;
use pumpkin_world::world::{GetBlockError, WorldFuture};
use pumpkin_world::{
    BlockStateId, CURRENT_BEDROCK_MC_VERSION, biome, block::entities::BlockEntity,
    chunk::io::Dirtiable, inventory::Inventory, item::ItemStack, world::SimpleWorld,
};
use pumpkin_world::{chunk::ChunkData, world::BlockAccessor};
use pumpkin_world::{level::Level, tick::TickPriority};
use pumpkin_world::{world::BlockFlags, world_info::LevelData};
use rand::seq::SliceRandom;
use rand::{RngExt, rng};
use scoreboard::Scoreboard;
use time::LevelTime;
use tokio::sync::Mutex;

pub mod border;
pub mod bossbar;
pub mod custom_bossbar;
pub mod natural_spawner;
pub mod scoreboard;
pub mod weather;

use crate::plugin::player::player_respawn::PlayerRespawnEvent;
use crate::world::natural_spawner::{SpawnState, spawn_for_chunk};
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::effect::StatusEffect;
use pumpkin_world::chunk::ChunkHeightmapType::MotionBlocking;
use uuid::Uuid;
use weather::Weather;

type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;

impl PumpkinError for GetBlockError {
    fn is_kick(&self) -> bool {
        false
    }

    fn severity(&self) -> tracing::Level {
        tracing::Level::WARN
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
    /// Represents the World's Unique Identifier
    pub uuid: Uuid,
    /// The underlying level, responsible for chunk management and terrain generation.
    pub level: Arc<Level>,
    pub level_info: Arc<ArcSwap<LevelData>>,
    /// A map of active players within the world, keyed by their unique UUID.
    pub players: ArcSwap<Vec<Arc<Player>>>,
    /// A map of active entities within the world, keyed by their unique UUID.
    /// This does not include players.
    pub entities: ArcSwap<Vec<Arc<dyn EntityBase>>>,
    /// The world's scoreboard, used for tracking scores, objectives, and display information.
    pub scoreboard: Mutex<Scoreboard>,
    /// The world's worldborder, defining the playable area and controlling its expansion or contraction.
    pub worldborder: Mutex<Worldborder>,
    /// The world's time, including counting ticks for weather, time cycles, and statistics.
    pub level_time: Mutex<LevelTime>,
    /// The type of dimension the world is in.
    pub dimension: Dimension,
    pub sea_level: i32,
    pub min_y: i32,
    /// The world's weather, including rain and thunder levels.
    pub weather: Mutex<Weather>,
    /// Block Behaviour
    pub block_registry: Arc<BlockRegistry>,
    pub server: Weak<Server>,
    synced_block_event_queue: Mutex<Vec<BlockEvent>>,
    /// A map of unsent block changes, keyed by block position.
    unsent_block_changes: Mutex<HashMap<BlockPos, u16>>,
    /// POI storage for fast portal lookups
    pub portal_poi: Mutex<portal::PortalPoiStorage>,
}

impl PartialEq for World {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for World {}

impl World {
    #[must_use]
    pub fn load(
        level: Arc<Level>,
        level_info: Arc<ArcSwap<LevelData>>,
        dimension: Dimension,
        block_registry: Arc<BlockRegistry>,
        server: Weak<Server>,
    ) -> Self {
        // TODO
        let generation_settings = GenerationSettings::from_dimension(&dimension);

        // Load portal POI from disk (PoiStorage::new automatically loads from disk if files exist)
        let portal_poi = portal::PortalPoiStorage::new(&level.level_folder.root_folder);

        Self {
            uuid: Uuid::new_v4(),
            level,
            level_info,
            players: ArcSwap::new(Arc::new(Vec::new())),
            entities: ArcSwap::new(Arc::new(Vec::new())),
            scoreboard: Mutex::new(Scoreboard::default()),
            worldborder: Mutex::new(Worldborder::new(0.0, 0.0, 5.999_996_8E7, 0, 5, 300)),
            level_time: Mutex::new(LevelTime::new()),
            dimension,
            weather: Mutex::new(Weather::new()),
            block_registry,
            sea_level: generation_settings.sea_level,
            min_y: i32::from(generation_settings.shape.min_y),
            synced_block_event_queue: Mutex::new(Vec::new()),
            unsent_block_changes: Mutex::new(HashMap::new()),
            portal_poi: Mutex::new(portal_poi),
            server,
        }
    }

    pub fn get_lighting_config(&self) -> LightingEngineConfig {
        self.server
            .upgrade()
            .map(|s| s.advanced_config.world.lighting)
            .unwrap_or_default()
    }

    /// Get the world folder name (e.g., `world`, `world_nether`, `world_the_end`).
    /// Falls back to "world" if the name cannot be determined.
    pub fn get_world_name(&self) -> &str {
        self.level
            .level_folder
            .root_folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("world")
    }

    pub async fn shutdown(&self) {
        for entity in self.entities.load().iter() {
            self.save_entity(entity).await;
        }

        // Save portal POI to disk
        let save_result = self.portal_poi.lock().await.save_all();
        if let Err(e) = save_result {
            error!("Failed to save portal POI: {e}");
        }

        self.level.shutdown().await;
    }

    async fn save_entity(&self, entity: &Arc<dyn EntityBase>) {
        // First lets see if the entity was saved on an other chunk, and if the current chunk does not match we remove it
        // Otherwise we just update the nbt data
        let base_entity = entity.get_entity();
        let uuid = base_entity.entity_uuid;
        let current_chunk_coordinate = base_entity.block_pos.load().chunk_position();
        let mut nbt = NbtCompound::new();
        entity.write_nbt(&mut nbt).await;
        if let Some(old_chunk) = base_entity.first_loaded_chunk_position.load() {
            let old_chunk = old_chunk.to_vec2_i32();
            let chunk = self.level.get_entity_chunk(old_chunk).await;
            chunk.mark_dirty(true);
            let mut data = chunk.data.lock().await;
            if old_chunk == current_chunk_coordinate {
                data.insert(uuid, nbt);
                return;
            }

            // The chunk has changed, lets remove the entity from the old chunk
            data.remove(&uuid);
        }
        // We did not continue, so lets save data in a new chunk
        let chunk = self.level.get_entity_chunk(current_chunk_coordinate).await;
        let mut data = chunk.data.lock().await;
        data.insert(uuid, nbt);
        chunk.mark_dirty(true);
    }

    async fn remove_entity_data(&self, entity: &Entity) {
        let current_chunk_coordinate = entity.block_pos.load().chunk_position();
        if let Some(old_chunk) = entity.first_loaded_chunk_position.load() {
            let old_chunk = old_chunk.to_vec2_i32();
            let chunk = self.level.get_entity_chunk(old_chunk).await;
            chunk.mark_dirty(true);
            if old_chunk == current_chunk_coordinate {
                chunk.data.lock().await.remove(&entity.entity_uuid);
            } else {
                let chunk = self.level.get_entity_chunk(current_chunk_coordinate).await;
                // The chunk has changed, lets remove the entity from the old chunk
                chunk.data.lock().await.remove(&entity.entity_uuid);
                chunk.mark_dirty(true);
            }
        }
    }

    pub async fn send_entity_status(&self, entity: &Entity, status: EntityStatus) {
        // TODO: only nearby
        self.broadcast_packet_all(&CEntityStatus::new(entity.entity_id, status as i8))
            .await;
    }

    pub async fn send_remove_mob_effect(
        &self,
        entity: &Entity,
        effect_type: &'static StatusEffect,
    ) {
        // TODO: only nearby
        self.broadcast_packet_all(&CRemoveMobEffect::new(
            entity.entity_id.into(),
            VarInt(i32::from(effect_type.id)),
        ))
        .await;
    }

    pub fn get_difficulty(&self, difficulty: Difficulty) {
        let current_info = self.level_info.load();
        let mut new_info = (**current_info).clone();
        new_info.difficulty = difficulty;
        self.level_info.store(Arc::new(new_info));
    }

    pub fn set_difficulty(&self, difficulty: Difficulty) {
        let current_info = self.level_info.load();
        let mut new_info = (**current_info).clone();
        new_info.difficulty = difficulty;
        self.level_info.store(Arc::new(new_info));
    }

    pub async fn add_synced_block_event(&self, pos: BlockPos, r#type: u8, data: u8) {
        let mut queue = self.synced_block_event_queue.lock().await;
        queue.push(BlockEvent { pos, r#type, data });
    }

    pub async fn flush_synced_block_events(self: &Arc<Self>) {
        let events;
        // THIS IS IMPORTANT
        // it prevents deadlocks and also removes the need to wait for a lock when adding a new synced block
        {
            let mut queue = self.synced_block_event_queue.lock().await;
            events = queue.clone();
            queue.clear();
        };

        for event in events {
            let block = self.get_block(&event.pos).await; // TODO
            if !self
                .block_registry
                .on_synced_block_event(block, self, &event.pos, event.r#type, event.data)
                .await
            {
                continue;
            }
            self.broadcast_packet_all(&CBlockEvent::new(
                event.pos,
                event.r#type,
                event.data,
                VarInt(i32::from(block.id)),
            ))
            .await;
        }
    }

    fn collect_java_recipients_by_version<'a>(
        players: impl Iterator<Item = &'a Arc<Player>>,
    ) -> BTreeMap<MinecraftVersion, Vec<&'a JavaClient>> {
        let mut recipients_by_version: BTreeMap<MinecraftVersion, Vec<&'a JavaClient>> =
            BTreeMap::new();
        for player in players {
            if let ClientPlatform::Java(java_client) = &player.client {
                recipients_by_version
                    .entry(java_client.version.load())
                    .or_default()
                    .push(java_client);
            }
        }
        recipients_by_version
    }

    async fn broadcast_java_grouped<P: ClientPacket>(
        packet: &P,
        recipients_by_version: BTreeMap<MinecraftVersion, Vec<&JavaClient>>,
    ) {
        for (version, recipients) in recipients_by_version {
            let packet_data = match JavaClient::serialize_packet_for_version(packet, version) {
                Ok(packet_data) => packet_data,
                Err(err) => {
                    error!(
                        "Failed to serialize packet {} for version {:?}: {}",
                        std::any::type_name::<P>(),
                        version,
                        err
                    );
                    continue;
                }
            };

            for recipient in recipients {
                recipient.enqueue_packet_data(packet_data.clone()).await;
            }
        }
    }

    /// Broadcasts a packet to all connected players within the world.
    /// Please avoid this as we want to replace it with `broadcast_editioned`
    ///
    /// Sends the specified packet to every player currently logged in to the world.
    ///
    /// **Note:** This function acquires a lock on the `current_players` map, ensuring thread safety.
    pub async fn broadcast_packet_all<P: ClientPacket>(&self, packet: &P) {
        let players = self.players.load();
        let recipients_by_version = Self::collect_java_recipients_by_version(players.iter());
        Self::broadcast_java_grouped(packet, recipients_by_version).await;
    }

    pub async fn broadcast_message(
        &self,
        message: &TextComponent,
        sender_name: &TextComponent,
        chat_type: u8,
        target_name: Option<&TextComponent>,
    ) {
        let be_packet = SText::new(message.clone().get_text(), sender_name.clone().get_text());
        let je_packet =
            CDisguisedChatMessage::new(message, (chat_type + 1).into(), sender_name, target_name);

        self.broadcast_editioned(&je_packet, &be_packet).await;
    }

    // This should replace broadcast_packet_all at some point
    pub async fn broadcast_editioned<J: ClientPacket, B: BClientPacket>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        let players = self.players.load();
        let je_recipients_by_version = Self::collect_java_recipients_by_version(players.iter());
        let mut be_recipients = Vec::new();

        for player in players.iter() {
            if let ClientPlatform::Bedrock(be_client) = &player.client {
                be_recipients.push(be_client.clone());
            }
        }

        Self::broadcast_java_grouped(je_packet, je_recipients_by_version).await;

        for recipient in be_recipients {
            recipient.send_game_packet(be_packet).await;
        }
    }

    pub async fn broadcast_secure_player_chat(
        &self,
        sender: &Arc<Player>,
        chat_message: &SChatMessage,
        decorated_message: &TextComponent,
    ) {
        let messages_sent: i32 = sender.chat_session.lock().await.messages_sent;
        let sender_last_seen = {
            let cache = sender.signature_cache.lock().await;
            cache.last_seen.clone()
        };

        for recipient in self.players.load().iter() {
            let messages_received: i32 = recipient.chat_session.lock().await.messages_received;
            let packet = &CPlayerChatMessage::new(
                VarInt(messages_received),
                sender.gameprofile.id,
                VarInt(messages_sent),
                chat_message.signature.clone(),
                chat_message.message.clone(),
                chat_message.timestamp,
                chat_message.salt,
                sender_last_seen.indexed_for(recipient).await,
                Some(decorated_message.clone()),
                FilterType::PassThrough,
                (RAW + 1).into(), // Custom registry chat_type with no sender name
                TextComponent::text(""), // Not needed since we're injecting the name in the message for custom formatting
                None,
            );
            recipient.client.enqueue_packet(packet).await;

            recipient
                .signature_cache
                .lock()
                .await
                .add_seen_signature(&chat_message.signature.clone().unwrap()); // Unwrap is safe because we check for None in validate_chat_message

            if recipient.gameprofile.id != sender.gameprofile.id {
                // Sender may update recipient on signatures recipient hasn't seen
                recipient
                    .signature_cache
                    .lock()
                    .await
                    .cache_signatures(sender_last_seen.as_ref());
            }
            recipient.chat_session.lock().await.messages_received += 1;
        }

        sender.chat_session.lock().await.messages_sent += 1;
    }

    /// Broadcasts a packet to all connected players within the world, excluding the specified players.
    ///
    /// Sends the specified packet to every player currently logged in to the world, excluding the players listed in the `except` parameter.
    ///
    /// **Note:** This function acquires a lock on the `current_players` map, ensuring thread safety.
    pub async fn broadcast_packet_except<P: ClientPacket>(
        &self,
        except: &[uuid::Uuid],
        packet: &P,
    ) {
        let players = self.players.load();
        let recipients_by_version = Self::collect_java_recipients_by_version(
            players
                .iter()
                .filter(|candidate| !except.contains(&candidate.gameprofile.id)),
        );
        Self::broadcast_java_grouped(packet, recipients_by_version).await;
    }

    pub async fn spawn_particle(
        &self,
        position: Vector3<f64>,
        offset: Vector3<f32>,
        max_speed: f32,
        particle_count: i32,
        particle: Particle,
    ) {
        for player in self.players.load().iter() {
            player
                .spawn_particle(position, offset, max_speed, particle_count, particle)
                .await;
        }
    }

    pub async fn play_sound(&self, sound: Sound, category: SoundCategory, position: &Vector3<f64>) {
        self.play_sound_raw(sound as u16, category, position, 1.0, 1.0)
            .await;
    }

    pub async fn play_sound_fine(
        &self,
        sound: Sound,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
    ) {
        self.play_sound_raw(sound as u16, category, position, volume, pitch)
            .await;
    }

    pub async fn play_sound_expect(
        &self,
        player: &Player,
        sound: Sound,
        category: SoundCategory,
        position: &Vector3<f64>,
    ) {
        self.play_sound_raw_expect(player, sound as u16, category, position, 1.0, 1.0)
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
        let seed = rng().random::<f64>();
        let packet = CSoundEffect::new(IdOr::Id(sound_id), category, position, volume, pitch, seed);
        self.broadcast_packet_all(&packet).await;
    }

    pub async fn play_sound_raw_expect(
        &self,
        player: &Player,
        sound_id: u16,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
    ) {
        let seed = rng().random::<f64>();
        let packet = CSoundEffect::new(IdOr::Id(sound_id), category, position, volume, pitch, seed);
        self.broadcast_packet_except(&[player.gameprofile.id], &packet)
            .await;
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

    pub async fn play_block_sound_expect(
        &self,
        player: &Player,
        sound: Sound,
        category: SoundCategory,
        position: BlockPos,
    ) {
        let new_vec = Vector3::new(
            f64::from(position.0.x) + 0.5,
            f64::from(position.0.y) + 0.5,
            f64::from(position.0.z) + 0.5,
        );
        self.play_sound_expect(player, sound, category, &new_vec)
            .await;
    }

    pub async fn tick(self: &Arc<Self>, server: &Server) {
        let start = tokio::time::Instant::now();

        // IMPORTANT: send flush_block_updates first to prevent issues with CAcknowledgeBlockChange
        self.flush_block_updates().await;
        self.flush_synced_block_events().await;
        self.tick_environment().await;

        let chunk_start = tokio::time::Instant::now();
        self.tick_chunks().await;
        let chunk_elapsed = chunk_start.elapsed();

        let player_start = tokio::time::Instant::now();
        let players = self.players.load();
        let player_count = players.len();
        for player in players.iter() {
            player.tick(server).await;
        }
        let player_elapsed = player_start.elapsed();

        let entity_start = tokio::time::Instant::now();
        let entities_to_tick = self.entities.load();
        let entity_count = entities_to_tick.len();

        for entity in entities_to_tick.iter() {
            entity.get_entity().age.fetch_add(1, Relaxed);
            entity.tick(entity.clone(), server).await;

            for player in players.iter() {
                if player
                    .living_entity
                    .entity
                    .bounding_box
                    .load()
                    .expand(1.0, 0.5, 1.0)
                    .intersects(&entity.get_entity().bounding_box.load())
                {
                    entity.on_player_collision(player).await;
                    break;
                }
            }
        }
        let entity_elapsed = entity_start.elapsed();

        //self.level.chunk_loading.lock().unwrap().send_change();

        let total_elapsed = start.elapsed();
        if total_elapsed.as_millis() > 50 {
            debug!(
                "Slow Tick [{}ms]: Chunks: {:?} | Players({}): {:?} | Entities({}): {:?}",
                total_elapsed.as_millis(),
                chunk_elapsed,
                player_count,
                player_elapsed,
                entity_count,
                entity_elapsed,
            );
        }
    }

    pub async fn flush_block_updates(&self) {
        let mut block_state_updates_by_chunk_section: HashMap<
            Vector3<i32>,
            Vec<(BlockPos, BlockStateId)>,
        > = HashMap::new();
        for (position, block_state_id) in self.unsent_block_changes.lock().await.drain() {
            let chunk_section = chunk_section_from_pos(&position);
            block_state_updates_by_chunk_section
                .entry(chunk_section)
                .or_default()
                .push((position, block_state_id));
        }

        // TODO: only send packet to players who have the chunks loaded
        // TODO: Send light updates to update the wire directly next to a broken block
        for chunk_section in block_state_updates_by_chunk_section.values() {
            if chunk_section.is_empty() {
                continue;
            }
            if chunk_section.len() == 1 {
                let (block_pos, block_state_id) = chunk_section[0];
                self.broadcast_packet_all(&CBlockUpdate::new(
                    block_pos,
                    i32::from(block_state_id).into(),
                ))
                .await;
            } else {
                self.broadcast_packet_all(&CMultiBlockUpdate::new(chunk_section))
                    .await;
            }
        }
    }

    async fn tick_environment(&self) {
        let mut level_time = self.level_time.lock().await;
        let (advance_time, advance_weather) = {
            let lock = self.level_info.load();
            (
                lock.game_rules.advance_time,
                lock.game_rules.advance_weather,
            )
        };
        level_time.tick_time(advance_time, advance_weather);

        // Auto-save logic
        if level_time.world_age % 100 == 0 {
            self.level.should_unload.store(true, Relaxed);
            // If autosave is configured and this tick will trigger an autosave, don't double notify
            if self.level.autosave_ticks == 0 {
                self.level.level_channel.notify();
            } else {
                let autosave = self.level.autosave_ticks as i64;
                if autosave == 0 || level_time.world_age % autosave != 0 {
                    self.level.level_channel.notify();
                }
            }
        }
        if self.level.autosave_ticks > 0 {
            let autosave = self.level.autosave_ticks as i64;
            if autosave > 0 && level_time.world_age % autosave == 0 {
                self.level.should_save.store(true, Relaxed);
                self.level.level_channel.notify();
            }
        }

        let mut weather = self.weather.lock().await;
        weather.tick_weather(self).await;

        if self.should_skip_night() && level_time.is_night() {
            let time = level_time.time_of_day + 24000;
            level_time.set_time(time - time % 24000);
            level_time.send_time(self).await;

            for player in self.players.load().iter() {
                player.wake_up().await;
            }

            if weather.weather_cycle_enabled && (weather.raining || weather.thundering) {
                weather.reset_weather_cycle(self).await;
            }
        } else if level_time.world_age % 20 == 0 {
            level_time.send_time(self).await;
        }
    }

    pub async fn tick_chunks(self: &Arc<Self>) {
        let tick_data = self.level.get_tick_data();
        for scheduled_tick in tick_data.block_ticks {
            let block = self.get_block(&scheduled_tick.position).await;
            if let Some(pumpkin_block) = self.block_registry.get_pumpkin_block(block.id) {
                pumpkin_block
                    .on_scheduled_tick(OnScheduledTickArgs {
                        world: self,
                        block,
                        position: &scheduled_tick.position,
                    })
                    .await;
            }
        }
        for scheduled_tick in tick_data.fluid_ticks {
            let fluid = self.get_fluid(&scheduled_tick.position).await;
            if let Some(pumpkin_fluid) = self.block_registry.get_pumpkin_fluid(fluid.id) {
                pumpkin_fluid
                    .on_scheduled_tick(self, fluid, &scheduled_tick.position)
                    .await;
            }
        }

        for scheduled_tick in tick_data.random_ticks {
            let block = self.get_block(&scheduled_tick.position).await;
            if let Some(pumpkin_block) = self.block_registry.get_pumpkin_block(block.id) {
                pumpkin_block
                    .random_tick(RandomTickArgs {
                        world: self,
                        block,
                        position: &scheduled_tick.position,
                    })
                    .await;
            }
        }

        let mut spawning_chunks_map = HashMap::new();
        // TODO use FixedPlayerDistanceChunkTracker

        for i in self.players.load().iter() {
            let center = i.living_entity.entity.chunk_pos.load();
            for dx in -8..=8 {
                for dy in -8..=8 {
                    // if dx.abs() <= 2 || dy.abs() <= 2 || dx.abs() >= 6 || dy.abs() >= 6 { // this is only for debug, spawning runs too slow
                    //     continue;
                    // }
                    let chunk_pos = center.add_raw(dx, dy);
                    if !spawning_chunks_map.contains_key(&chunk_pos)
                        && let Some(chunk) = self.level.try_get_chunk(&chunk_pos)
                    {
                        spawning_chunks_map.entry(chunk_pos).or_insert(chunk);
                    }
                }
            }
        }

        let mut spawn_state =
            SpawnState::new(spawning_chunks_map.len() as i32, &self.entities, self).await; // TODO store it

        // TODO gamerule this.spawnEnemies || this.spawnFriendlies
        let spawn_passives = self.level_time.lock().await.time_of_day % 400 == 0;
        let spawn_list: Vec<&'static MobCategory> =
            natural_spawner::get_filtered_spawning_categories(
                &spawn_state,
                true,
                true,
                spawn_passives,
            );

        // log::debug!("spawning list size {}", spawn_list.len());
        let mut spawning_chunks: Vec<(Vector2<i32>, Arc<ChunkData>)> =
            spawning_chunks_map.into_iter().collect();
        spawning_chunks.shuffle(&mut rng());

        // TODO i think it can be multithread
        for (pos, chunk) in spawning_chunks {
            self.tick_spawning_chunk(pos, &chunk, &spawn_list, &mut spawn_state)
                .await;
        }

        let world: Arc<dyn SimpleWorld> = self.clone();

        for block_entity in tick_data.block_entities {
            block_entity.tick(&world).await;
        }
    }

    pub async fn get_fluid_collisions(self: &Arc<Self>, bounding_box: BoundingBox) -> Vec<&Fluid> {
        let mut collisions = Vec::new();

        let min = bounding_box.min_block_pos();

        let max = bounding_box.max_block_pos();

        for x in min.0.x..=max.0.x {
            for y in min.0.y..=max.0.y {
                for z in min.0.z..=max.0.z {
                    let pos = BlockPos::new(x, y, z);

                    let (fluid, state) = self.get_fluid_and_fluid_state(&pos).await;

                    if fluid.id != Fluid::EMPTY.id {
                        let height = f64::from(state.height);

                        if height >= bounding_box.min.y {
                            collisions.push(fluid);
                        }
                    }
                }
            }
        }

        collisions
    }

    pub async fn check_fluid_collision(self: &Arc<Self>, bounding_box: BoundingBox) -> bool {
        let min = bounding_box.min_block_pos();

        let max = bounding_box.max_block_pos();

        for x in min.0.x..=max.0.x {
            for y in min.0.y..=max.0.y {
                for z in min.0.z..=max.0.z {
                    let pos = BlockPos::new(x, y, z);

                    let (fluid, state) = self.get_fluid_and_fluid_state(&pos).await;

                    if fluid.id != Fluid::EMPTY.id {
                        let height = f64::from(state.height);

                        if height >= bounding_box.min.y {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    // FlowableFluid.getVelocity()
    pub async fn get_fluid_velocity(
        &self,
        pos0: BlockPos,
        fluid0: &Fluid,
        state0: &FluidState,
    ) -> Vector3<f64> {
        let mut velo = Vector3::default();

        for dir in BlockDirection::horizontal() {
            let mut amplitude = 0.0;

            let offset = dir.to_offset();

            let pos = pos0.offset(offset);

            let block_state_id = self.get_block_state_id(&pos).await;

            let fluid = Fluid::from_state_id(block_state_id).unwrap_or(&Fluid::EMPTY);

            if fluid.id == Fluid::EMPTY.id {
                let block = Block::get_raw_id_from_state_id(block_state_id);
                let block_state = BlockState::from_id(block_state_id);

                let blocks_movement = block_state.is_solid()
                    && block != Block::COBWEB
                    && block != Block::BAMBOO_SAPLING;

                if !blocks_movement {
                    let down_pos = pos.down();

                    let (down_fluid, down_state) = self.get_fluid_and_fluid_state(&down_pos).await;

                    if down_fluid.matches_type(fluid0) {
                        amplitude = f64::from(state0.height - down_state.height) + 0.888_888_9;
                    }
                }
            } else {
                if !fluid.matches_type(fluid0) {
                    continue;
                }

                //let state = fluid.get_state(block_state_id);
                amplitude = f64::from(state0.height - fluid.states[0].height);
            }

            if amplitude == 0.0 {
                continue;
            }

            velo.x += f64::from(offset.x) * amplitude;

            velo.z += f64::from(offset.z) * amplitude;
        }

        // TODO: FALLING

        if state0.falling {
            for dir in BlockDirection::horizontal() {
                let pos = pos0.offset(dir.to_offset());

                if self.is_flow_blocked(fluid0.id, pos, dir).await
                    || self.is_flow_blocked(fluid0.id, pos.up(), dir).await
                {
                    if velo.length_squared() != 0.0 {
                        velo = velo.normalize();
                    }

                    velo.y -= 6.0;

                    break;
                }
            }
        }

        if velo.length_squared() == 0.0 {
            velo
        } else {
            velo.normalize()
        }
    }

    // FlowableFluid.isFlowBlocked()

    async fn is_flow_blocked(
        &self,
        fluid0_id: u16,
        pos: BlockPos,
        direction: BlockDirection,
    ) -> bool {
        let id = self.get_block_state_id(&pos).await;

        let fluid = Fluid::from_state_id(id).unwrap_or(&Fluid::EMPTY);

        if Fluid::same_fluid_type(fluid.id, fluid0_id) {
            return false;
        }

        if direction == BlockDirection::Up {
            return true;
        }

        let block = Block::from_state_id(id);
        let state = BlockState::from_id(id);

        // Doesn't count blue ice or packed ice

        if block == &Block::ICE || block == &Block::FROSTED_ICE {
            return false;
        }

        state.is_side_solid(direction)
    }

    pub fn check_outline<F>(
        bounding_box: &BoundingBox,
        pos: BlockPos,
        state: &BlockState,
        use_outline_shape: bool,
        mut using_outline_shape: F,
    ) -> bool
    where
        F: FnMut(&BoundingBox),
    {
        if state.outline_shapes.is_empty() {
            // Apparently we need this for air and moving pistons

            return true;
        }

        let mut inside = false;
        'shapes: for shape in state.get_block_outline_shapes() {
            let outline_shape = shape.at_pos(pos);

            if outline_shape.intersects(bounding_box) {
                inside = true;

                if !use_outline_shape {
                    break 'shapes;
                }

                using_outline_shape(&outline_shape);
            }
        }

        inside
    }

    pub fn check_collision<F>(
        bounding_box: &BoundingBox,
        pos: BlockPos,
        state: &BlockState,
        use_collision_shape: bool,
        mut on_collision: F,
    ) -> bool
    where
        F: FnMut(&BoundingBox),
    {
        if state.is_air() || !state.is_solid() {
            return false;
        }

        let mut shapes = state
            .get_block_collision_shapes()
            .map(|shape| shape.at_pos(pos));

        if use_collision_shape {
            let mut collided = false;
            for collision_shape in shapes {
                if collision_shape.intersects(bounding_box) {
                    collided = true;
                    // Convert to BB and trigger the callback
                    on_collision(&collision_shape);
                }
            }
            collided
        } else {
            shapes.any(|s| s.intersects(bounding_box))
        }
    }

    // For adjusting movement
    pub async fn get_block_collisions(
        self: &Arc<Self>,
        bounding_box: BoundingBox,
    ) -> (Vec<BoundingBox>, Vec<(usize, BlockPos)>) {
        let mut collisions = Vec::new();

        let mut positions = Vec::new();

        let min = BlockPos::floored_v(bounding_box.min.add_raw(0.0, -0.50001, 0.0));
        let max = bounding_box.max_block_pos();
        let pos_iter = BlockPos::iterate(min, max);

        for pos in pos_iter {
            let state = self.get_block_state(&pos).await;

            if state.is_air() {
                continue;
            }

            let collided = Self::check_collision(
                &bounding_box,
                pos,
                state,
                true,
                |collision_shape: &BoundingBox| {
                    collisions.push(*collision_shape);
                },
            );

            if collided {
                positions.push((collisions.len(), pos));
            }
        }

        (collisions, positions)
    }

    pub async fn is_space_empty(&self, bounding_box: BoundingBox) -> bool {
        let min = bounding_box.min_block_pos();
        let max = bounding_box.max_block_pos();

        for pos in BlockPos::iterate(min, max) {
            let state = self.get_block_state(&pos).await;
            let collided = Self::check_collision(&bounding_box, pos, state, false, |_| ());

            if collided {
                return false;
            }
        }
        true
    }

    /// Vanilla's `BlockView.getDismountHeight()`.
    /// Returns the Y surface height for dismounting at the given block position,
    /// or `f64::NEG_INFINITY` if no valid surface exists.
    pub async fn get_dismount_height(&self, pos: &BlockPos) -> f64 {
        let state = self.get_block_state(pos).await;
        let max_y = state
            .get_block_collision_shapes()
            .map(|s| s.max.y)
            .fold(f64::NEG_INFINITY, f64::max);
        if max_y != f64::NEG_INFINITY {
            return max_y;
        }
        // No collision at pos — check block below
        let below = BlockPos(Vector3::new(pos.0.x, pos.0.y - 1, pos.0.z));
        let below_state = self.get_block_state(&below).await;
        let below_max_y = below_state
            .get_block_collision_shapes()
            .map(|s| s.max.y)
            .fold(f64::NEG_INFINITY, f64::max);
        if below_max_y >= 1.0 {
            below_max_y - 1.0
        } else {
            f64::NEG_INFINITY
        }
    }

    pub async fn tick_spawning_chunk(
        self: &Arc<Self>,
        chunk_pos: Vector2<i32>,
        chunk: &Arc<ChunkData>,
        spawn_list: &Vec<&'static MobCategory>,
        spawn_state: &mut SpawnState,
    ) {
        // this.level.tickThunder(chunk);
        //TODO check in simulation distance
        let weather = self.weather.lock().await;
        if weather.raining && weather.thundering && rng().random_range(0..100_000) == 0 {
            let rand_value = rng().random::<i32>() >> 2;
            let delta = Vector3::new(rand_value & 15, rand_value >> 16 & 15, rand_value >> 8 & 15);
            let random_pos = Vector3::new(
                chunk_pos.x << 4,
                chunk.heightmap.lock().unwrap().get(
                    MotionBlocking,
                    chunk_pos.x << 4,
                    chunk_pos.y << 4,
                    self.min_y,
                ),
                chunk_pos.y << 4,
            )
            .add(&delta);
            // TODO this.getBrightness(LightLayer.SKY, blockPos) >= 15;
            // TODO heightmap

            // TODO findLightningRod(blockPos)
            // TODO encapsulatingFullBlocks
            if true {
                // TODO biome.getPrecipitationAt(pos, this.getSeaLevel()) == Biome.Precipitation.RAIN
                // TODO this.getCurrentDifficultyAt(blockPos);
                if rng().random::<f32>() < 0.0675
                    && self.get_block(&random_pos.to_block_pos().down()).await
                        != &Block::LIGHTNING_ROD
                {
                    let entity = Entity::new(
                        self.clone(),
                        random_pos.to_f64(),
                        &EntityType::SKELETON_HORSE,
                    );
                    self.spawn_entity(Arc::new(entity)).await;
                }
                let entity = Entity::new(
                    self.clone(),
                    random_pos.to_f64().add_raw(0.5, 0., 0.5),
                    &EntityType::LIGHTNING_BOLT,
                );
                self.spawn_entity(Arc::new(entity)).await;
            }
        }
        drop(weather);

        if spawn_list.is_empty() {
            return;
        }
        // TODO this.level.canSpawnEntitiesInChunk(chunkPos)
        spawn_for_chunk(self, chunk_pos, chunk, spawn_state, spawn_list).await;
    }

    /// Gets the y position of the first non air block from the top down
    pub async fn get_top_block(&self, position: Vector2<i32>) -> i32 {
        for y in (self.dimension.min_y..self.dimension.height).rev() {
            let pos = BlockPos::new(position.x, y, position.y);
            let block = self.get_block_state(&pos).await;
            if block.is_air() {
                continue;
            }
            return y;
        }
        self.dimension.min_y
    }

    /// Gets the `MOTION_BLOCKING` heightmap value for a given XZ position.
    pub async fn get_motion_blocking_height(&self, x: i32, z: i32) -> i32 {
        let chunk_pos = Vector2::new(x >> 4, z >> 4);
        let chunk = self.level.get_chunk(chunk_pos).await;
        chunk
            .heightmap
            .lock()
            .unwrap()
            .get(MotionBlocking, x, z, self.min_y)
    }

    #[allow(clippy::too_many_lines)]
    pub async fn spawn_bedrock_player(
        &self,
        base_config: &BasicConfiguration,
        player: Arc<Player>,
        server: &Server,
    ) {
        let level_info = server.level_info.load();
        let weather = self.weather.lock().await;
        let runtime_id = player.entity_id() as u64;
        let (position, yaw, pitch) = if player.has_played_before.load(Ordering::Relaxed) {
            let position = player.position();
            let yaw = player.living_entity.entity.yaw.load(); //info.spawn_angle;
            let pitch = player.living_entity.entity.pitch.load();

            (position, yaw, pitch)
        } else {
            let spawn_position = Vector2::new(level_info.spawn_x, level_info.spawn_z);
            let pos_y = self.get_top_block(spawn_position).await + 1; // +1 to spawn on top of the block

            let position = Vector3::new(
                f64::from(level_info.spawn_x) + 0.5,
                f64::from(pos_y),
                f64::from(level_info.spawn_z) + 0.5,
            );
            (position, level_info.spawn_yaw, level_info.spawn_pitch)
        };
        // Todo make the data less spread
        let level_settings = LevelSettings {
            seed: self.level.seed.0,
            spawn_biome_type: 0,
            custom_biome_name: String::new(),
            dimension: VarInt(0),
            generator_type: VarInt(1),
            world_gamemode: server.defaultgamemode.lock().await.gamemode,
            hardcore: base_config.hardcore,
            difficulty: VarInt(level_info.difficulty as i32),
            spawn_position: NetworkPos(BlockPos::new(
                level_info.spawn_x,
                level_info.spawn_y,
                level_info.spawn_z,
            )),
            has_achievements_disabled: false,
            editor_world_type: VarInt(0),
            is_created_in_editor: false,
            is_exported_from_editor: false,
            day_cycle_stop_time: VarInt(-1),
            education_edition_offer: VarInt(0),
            has_education_features_enabled: false,
            education_product_id: String::new(),
            rain_level: weather.rain_level,
            lightning_level: weather.thunder_level,
            has_confirmed_platform_locked_content: false,
            was_multiplayer_intended: true,
            was_lan_broadcasting_intended: true,
            xbox_live_broadcast_setting: GamePublishSetting::Public,
            platform_broadcast_setting: GamePublishSetting::Public,
            commands_enabled: level_info.allow_commands,
            is_texture_packs_required: false,
            rule_data: GameRules {
                list_size: VarUInt(0),
            },
            experiments: Experiments {
                names_size: 0,
                experiments_ever_toggled: false,
            },
            bonus_chest: false,
            has_start_with_map_enabled: false,
            // TODO Bedrock permission level are different
            permission_level: VarInt(2),
            server_simulation_distance: base_config.simulation_distance.get().into(),
            has_locked_behavior_pack: false,
            has_locked_resource_pack: false,
            is_from_locked_world_template: false,
            is_using_msa_gamertags_only: false,
            is_from_world_template: false,
            is_world_template_option_locked: false,
            is_only_spawning_v1_villagers: false,
            is_disabling_personas: false,
            is_disabling_custom_skins: false,
            emote_chat_muted: false,
            game_version: CURRENT_BEDROCK_MC_VERSION.into(),
            limited_world_width: 0,
            limited_world_height: 0,
            new_nether: true,
            edu_shared_uri_button_name: String::new(),
            edu_shared_uri_link_uri: String::new(),
            override_force_experimental_gameplay_has_value: false,
            chat_restriction_level: 0,
            disable_player_interactions: false,
            server_id: String::new(),
            world_id: String::new(),
            scenario_id: String::new(),
            owner_id: String::new(),
        };
        drop(level_info);
        drop(weather);

        let client = player.client.bedrock();

        client
            .send_game_packet(&CStartGame {
                entity_id: VarLong(runtime_id as _),
                runtime_entity_id: VarULong(runtime_id),
                player_gamemode: player.gamemode.load(),
                position: Vector3::new(position.x as f32, position.y as f32, position.z as f32),
                pitch,
                yaw,
                level_settings,
                level_id: String::new(),
                level_name: "Pumpkin world".to_string(),
                premium_world_template_id: String::new(),
                is_trial: false,
                rewind_history_size: VarInt(0),
                server_authoritative_block_breaking: true,
                current_level_time: self.level_time.lock().await.world_age as _,
                enchantment_seed: VarInt(0),
                block_properties_size: VarUInt(0),
                // TODO Make this unique
                multiplayer_correlation_id: Uuid::default().to_string(),
                enable_itemstack_net_manager: true,
                // TODO Make this description better!
                // This gets send from the client to mojang for telemetry
                server_version: "Pumpkin Rust Server".to_string(),

                compound_id: 10,
                compound_len: VarUInt(0),
                compound_end: 0,

                block_registry_checksum: 0,
                world_template_id: Uuid::nil(),
                // TODO The client needs extra biome data for this
                enable_clientside_generation: false,
                blocknetwork_ids_are_hashed: false,
                server_auth_sounds: false,
            })
            .await;
        chunker::update_position(&player).await;
        client
            .send_game_packet(&CreativeContent {
                groups: &[Group {
                    creative_category: 1,
                    name: String::new(),
                    icon_item: NetworkItemDescriptor::default(),
                }],
                entries: &[],
            })
            .await;

        {
            let mut abilities = player.abilities.lock().await;
            abilities.set_for_gamemode(player.gamemode.load());
        };
        let mut metadata = EntityMetadata::default();

        metadata.set(entity_data_key::WIDTH, MetadataValue::Float(0.6));
        metadata.set(entity_data_key::HEIGHT, MetadataValue::Float(1.8));

        // This is super important, otherwise the client will float by default
        metadata.set_flag(entity_data_flag::HAS_GRAVITY);

        // Prevents the client from showing air buddles on hud even when not in water
        metadata.set_flag(entity_data_flag::BREATHING);
        let actor_data = CSetActorData {
            actor_runtime_id: VarULong(runtime_id),
            metadata,
            synced_properties: PropertySyncData {
                int_properties: HashMap::new(),
                float_properties: HashMap::new(),
            },
            tick: VarULong(0),
        };
        client.send_game_packet(&actor_data).await;

        player.send_abilities_update().await;

        let mut frame_set = FrameSet::default();

        // https://github.com/pmmp/PocketMine-MP/blob/0b6d8f8cb2aaa05ffad0b6386bd88d73ef54b395/src/entity/AttributeFactory.php#L34
        client
            .write_game_packet_to_set(
                &CUpdateAttributes {
                    runtime_id: VarULong(runtime_id),
                    attributes: vec![
                        Attribute {
                            min_value: 0.0,
                            max_value: 3.402_823_5E38,
                            current_value: 0.1,
                            default_min_value: 0.0,
                            default_max_value: 3.402_823_5E38,
                            default_value: 0.1,
                            name: "minecraft:movement".to_string(),
                            modifiers_list_size: VarUInt(0),
                        },
                        Attribute {
                            min_value: 0.0,
                            max_value: 3.402_823_5E38,
                            current_value: 0.02,
                            default_min_value: 0.0,
                            default_max_value: 3.402_823_5E38,
                            default_value: 0.02,
                            name: "minecraft:underwater_movement".to_string(),
                            modifiers_list_size: VarUInt(0),
                        },
                        Attribute {
                            min_value: 0.0,
                            max_value: 1.0,
                            current_value: 0.08,
                            default_min_value: 0.0,
                            default_max_value: 1.0,
                            default_value: 0.08,
                            name: "minecraft:gravity".to_string(),
                            modifiers_list_size: VarUInt(0),
                        },
                        Attribute {
                            min_value: 0.0,
                            max_value: 400.0,
                            current_value: 400.0,
                            default_min_value: 0.0,
                            default_max_value: 400.0,
                            default_value: 400.0,
                            name: "minecraft:air".to_string(),
                            modifiers_list_size: VarUInt(0),
                        },
                        Attribute {
                            min_value: 0.0,
                            max_value: 20.0,
                            current_value: 20.0,
                            default_min_value: 0.0,
                            default_max_value: 20.0,
                            default_value: 20.0,
                            name: "minecraft:health".to_string(),
                            modifiers_list_size: VarUInt(0),
                        },
                        Attribute {
                            min_value: 0.0,
                            max_value: 20.0,
                            current_value: 20.0,
                            default_min_value: 0.0,
                            default_max_value: 20.0,
                            default_value: 20.0,
                            name: "minecraft:player.hunger".to_string(),
                            modifiers_list_size: VarUInt(0),
                        },
                    ],
                    player_tick: VarULong(0),
                },
                &mut frame_set,
            )
            .await;
        client
            .write_game_packet_to_set(&CPlayStatus::PlayerSpawn, &mut frame_set)
            .await;
        client.send_frame_set(frame_set, 0x84).await;
    }

    #[expect(clippy::too_many_lines)]
    pub async fn spawn_java_player(
        &self,
        base_config: &BasicConfiguration,
        player: &Arc<Player>,
        server: &Server,
    ) {
        let dimensions: Vec<ResourceLocation> = server
            .dimensions
            .iter()
            .map(|d| ResourceLocation::from(d.minecraft_name))
            .collect();

        // This code follows the vanilla packet order
        let entity_id = player.entity_id();
        let gamemode = player.gamemode.load();
        debug!(
            "spawning player {}, entity id {}",
            player.gameprofile.name, entity_id
        );

        let client = player.client.java();
        // Send the login packet for our new player
        client
            .send_packet_now(&CLogin::new(
                entity_id,
                base_config.hardcore,
                dimensions,
                base_config.max_players.try_into().unwrap(),
                base_config.view_distance.get().into(), //  TODO: view distance
                base_config.simulation_distance.get().into(), // TODO: sim view dinstance
                false,
                true,
                false,
                (self.dimension.id).into(),
                ResourceLocation::from(self.dimension.minecraft_name),
                biome::hash_seed(self.level.seed.0), // seed
                gamemode as u8,
                player
                    .previous_gamemode
                    .load()
                    .map_or(-1, |gamemode| gamemode as i8),
                false,
                false,
                None,
                VarInt(player.get_entity().portal_cooldown.load(Ordering::Relaxed) as i32),
                self.sea_level.into(),
                // This should stay true even when reports are disabled.
                // It prevents the annoying popup when joining the server.
                true,
            ))
            .await;

        // Send the current ticking state to the new player so they are in sync.
        server.tick_rate_manager.update_joining_player(player).await;

        // Permissions, i.e. the commands a player may use.
        player.send_permission_lvl_update().await;

        // Difficulty of the world
        player.send_difficulty_update().await;
        {
            let command_dispatcher = server.command_dispatcher.read().await;

            client_suggestions::send_c_commands_packet(player, server, &command_dispatcher).await;
        };

        // Spawn in initial chunks
        // This is made before the player teleport so that the player doesn't glitch out when spawning
        chunker::update_position(player).await;

        // Teleport
        let (position, yaw, pitch) = if player.has_played_before.load(Ordering::Relaxed) {
            let position = player.position();
            let yaw = player.living_entity.entity.yaw.load(); //info.spawn_angle;
            let pitch = player.living_entity.entity.pitch.load();

            (position, yaw, pitch)
        } else {
            let info = &self.level_info.load();
            let spawn_position = Vector2::new(info.spawn_x, info.spawn_z);
            let pos_y = self.get_top_block(spawn_position).await + 1; // +1 to spawn on top of the block

            let position = Vector3::new(
                f64::from(info.spawn_x) + 0.5,
                f64::from(pos_y),
                f64::from(info.spawn_z) + 0.5,
            );
            (position, info.spawn_yaw, info.spawn_pitch)
        };

        let velocity = player.living_entity.entity.velocity.load();

        debug!("Sending player teleport to {}", player.gameprofile.name);
        player.request_teleport(position, yaw, pitch).await;

        player.living_entity.entity.last_pos.store(position);

        let gameprofile = &player.gameprofile;
        // Firstly, send an info update to our new player, so they can see their skin
        // and also send their info to everyone else.
        debug!("Broadcasting player info for {}", player.gameprofile.name);
        self.broadcast_packet_all(&CPlayerInfoUpdate::new(
            (PlayerInfoFlags::ADD_PLAYER
                | PlayerInfoFlags::UPDATE_GAME_MODE
                | PlayerInfoFlags::UPDATE_LISTED)
                .bits(),
            &[pumpkin_protocol::java::client::play::Player {
                uuid: gameprofile.id,
                actions: &[
                    PlayerAction::AddPlayer {
                        name: &gameprofile.name,
                        properties: &gameprofile.properties,
                    },
                    PlayerAction::UpdateGameMode(VarInt(gamemode as i32)),
                    PlayerAction::UpdateListed(true),
                ],
            }],
        ))
        .await;
        // Here, we send all the infos of players who already joined.
        {
            let mut current_player_data = Vec::new();
            let players = self.players.load();
            for player in players
                .iter()
                .filter(|p| p.gameprofile.id != player.gameprofile.id)
            {
                let chat_session = player.chat_session.lock().await;

                let mut player_actions = vec![
                    PlayerAction::AddPlayer {
                        name: &player.gameprofile.name,
                        properties: &player.gameprofile.properties,
                    },
                    PlayerAction::UpdateListed(true),
                ];

                if base_config.allow_chat_reports {
                    player_actions.push(PlayerAction::InitializeChat(Some(InitChat {
                        session_id: chat_session.session_id,
                        expires_at: chat_session.expires_at,
                        public_key: chat_session.public_key.clone(),
                        signature: chat_session.signature.clone(),
                    })));
                }
                drop(chat_session);

                current_player_data.push((&player.gameprofile.id, player_actions));
            }

            let mut action_flags = PlayerInfoFlags::ADD_PLAYER | PlayerInfoFlags::UPDATE_LISTED;
            if base_config.allow_chat_reports {
                action_flags |= PlayerInfoFlags::INITIALIZE_CHAT;
            }

            let entries = current_player_data
                .iter()
                .map(|(id, actions)| java::client::play::Player {
                    uuid: **id,
                    actions,
                })
                .collect::<Vec<_>>();

            debug!("Sending player info to {}", player.gameprofile.name);
            client
                .enqueue_packet(&CPlayerInfoUpdate::new(action_flags.bits(), &entries))
                .await;
        };

        let gameprofile = &player.gameprofile;

        debug!("Broadcasting player spawn for {}", player.gameprofile.name);
        // Spawn the player for every client.
        self.broadcast_packet_except(
            &[player.gameprofile.id],
            &CSpawnEntity::new(
                entity_id.into(),
                gameprofile.id,
                i32::from(EntityType::PLAYER.id).into(),
                position,
                pitch,
                yaw,
                yaw,
                0.into(),
                velocity,
            ),
        )
        .await;

        // Spawn players for our client.
        let id = player.gameprofile.id;
        for existing_player in self
            .players
            .load()
            .iter()
            .filter(|c| c.gameprofile.id != id)
        {
            let entity = &existing_player.living_entity.entity;
            let pos = entity.pos.load();
            let gameprofile = &existing_player.gameprofile;
            debug!("Sending player entities to {}", player.gameprofile.name);

            client
                .enqueue_packet(&CSpawnEntity::new(
                    existing_player.entity_id().into(),
                    gameprofile.id,
                    i32::from(EntityType::PLAYER.id).into(),
                    pos,
                    entity.pitch.load(),
                    entity.yaw.load(),
                    entity.head_yaw.load(),
                    0.into(),
                    entity.velocity.load(),
                ))
                .await;
            {
                let config = existing_player.config.load();
                let mut buf = Vec::new();
                {
                    let meta = Metadata::new(
                        TrackedData::DATA_PLAYER_MODE_CUSTOMIZATION_ID,
                        MetaDataType::Byte,
                        config.skin_parts,
                    );
                    meta.write(&mut buf, &client.version.load()).unwrap();
                };
                drop(config);
                // END
                buf.put_u8(255);
                client
                    .enqueue_packet(&CSetEntityMetadata::new(
                        existing_player.get_entity().entity_id.into(),
                        buf.into(),
                    ))
                    .await;
            };

            {
                let mut equipment_list = Vec::new();

                equipment_list.push((
                    EquipmentSlot::MAIN_HAND.discriminant(),
                    existing_player.inventory.held_item().lock().await.clone(),
                ));

                for (slot, item_arc_mutex) in &existing_player
                    .inventory
                    .entity_equipment
                    .lock()
                    .await
                    .equipment
                {
                    let item_stack = item_arc_mutex.lock().await.clone();
                    equipment_list.push((slot.discriminant(), item_stack));
                }

                let equipment: Vec<(i8, ItemStackSerializer)> = equipment_list
                    .iter()
                    .map(|(slot, stack)| (*slot, ItemStackSerializer::from(stack.clone())))
                    .collect();

                client
                    .enqueue_packet(&CSetEquipment::new(
                        existing_player.entity_id().into(),
                        equipment,
                    ))
                    .await;
            }
        }
        player.send_client_information().await;

        player.send_abilities_update().await;

        // Sync selected slot
        player
            .enqueue_set_held_item_packet(&CSetSelectedSlot::new(
                player.get_inventory().get_selected_slot() as i8,
            ))
            .await;

        // Start waiting for level chunks. Sets the "Loading Terrain" screen
        debug!("Sending waiting chunks to {}", player.gameprofile.name);
        client
            .send_packet_now(&CGameEvent::new(GameEvent::StartWaitingChunks, 0.0))
            .await;

        self.worldborder.lock().await.init_client(client).await;

        // Sends initial time
        player.send_time(self).await;

        let (spawn_block_pos, yaw, pitch) = {
            let level_info_lock = self.level_info.load();
            (
                BlockPos::new(
                    level_info_lock.spawn_x,
                    level_info_lock.spawn_y,
                    level_info_lock.spawn_z,
                ),
                level_info_lock.spawn_yaw,
                level_info_lock.spawn_pitch,
            )
        };

        client
            .send_packet_now(&CPlayerSpawnPosition::new(
                spawn_block_pos,
                yaw,
                pitch,
                self.dimension.minecraft_name.to_owned(),
            ))
            .await;

        // Send initial weather state
        let weather = self.weather.lock().await;
        if weather.raining {
            client
                .enqueue_packet(&CGameEvent::new(GameEvent::BeginRaining, 0.0))
                .await;

            // Calculate rain and thunder levels directly from public fields
            let rain_level = weather.rain_level.clamp(0.0, 1.0);
            let thunder_level = weather.thunder_level.clamp(0.0, 1.0);
            drop(weather);

            client
                .enqueue_packet(&CGameEvent::new(GameEvent::RainLevelChange, rain_level))
                .await;
            client
                .enqueue_packet(&CGameEvent::new(
                    GameEvent::ThunderLevelChange,
                    thunder_level,
                ))
                .await;
        }

        // if let Some(bossbars) = self..lock().await.get_player_bars(&player.gameprofile.id) {
        //     for bossbar in bossbars {
        //         player.send_bossbar(bossbar).await;
        //     }
        // }

        player.has_played_before.store(true, Ordering::Relaxed);
        player
            .on_screen_handler_opened(player.player_screen_handler.clone())
            .await;

        player.send_active_effects().await;
        self.send_player_equipment(player).await;
    }

    async fn send_player_equipment(&self, from: &Player) {
        let mut equipment_list = Vec::new();

        equipment_list.push((
            EquipmentSlot::MAIN_HAND.discriminant(),
            from.inventory.held_item().lock().await.clone(),
        ));

        for (slot, item_arc_mutex) in &from.inventory.entity_equipment.lock().await.equipment {
            let item_stack = item_arc_mutex.lock().await.clone();
            equipment_list.push((slot.discriminant(), item_stack));
        }

        let equipment: Vec<(i8, ItemStackSerializer)> = equipment_list
            .iter()
            .map(|(slot, stack)| (*slot, ItemStackSerializer::from(stack.clone())))
            .collect();
        self.broadcast_packet_except(
            &[from.get_entity().entity_uuid],
            &CSetEquipment::new(from.entity_id().into(), equipment),
        )
        .await;
    }

    pub async fn send_world_info(
        &self,
        player: &Arc<Player>,
        position: Vector3<f64>,
        yaw: f32,
        pitch: f32,
    ) {
        if let ClientPlatform::Java(client) = &player.client {
            self.worldborder.lock().await.init_client(client).await;
        }

        // TODO: World spawn (compass stuff)

        player
            .client
            .enqueue_packet(&CGameEvent::new(GameEvent::StartWaitingChunks, 0.0))
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

        chunker::update_position(player).await;
        // Update commands

        player.set_health(20.0).await;
    }

    pub async fn explode(self: &Arc<Self>, position: Vector3<f64>, power: f32) {
        let explosion = Explosion::new(power, position);
        let block_count = explosion.explode(self).await;
        let particle = if power < 2.0 {
            Particle::Explosion
        } else {
            Particle::ExplosionEmitter
        };
        let sound = IdOr::<SoundEvent>::Id(Sound::EntityGenericExplode as u16);
        for player in self.players.load().iter() {
            if player.position().squared_distance_to_vec(&position) > 4096.0 {
                continue;
            }
            player
                .client
                .enqueue_packet(&CExplosion::new(
                    position,
                    power,
                    block_count as i32,
                    None,
                    VarInt(particle as i32),
                    sound.clone(),
                ))
                .await;
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn respawn_player(self: &Arc<Self>, player: &Arc<Player>, alive: bool) {
        let last_pos = player.living_entity.entity.last_pos.load();
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
                let top = self.get_top_block(Vector2::new(spawn_x, spawn_z)).await;

                (
                    Vector3::new(
                        f64::from(spawn_x) + 0.5,
                        (top + 1).into(),
                        f64::from(spawn_z) + 0.5,
                    ),
                    spawn_yaw,
                    spawn_pitch,
                    self.dimension,
                )
            };

        // Get target world (may be different from current world for cross-dimension respawn)
        let target_world = if respawn_dimension == self.dimension {
            None
        } else {
            // Cross-dimension respawn: get target world from server
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

        // Handle cross-dimension transfer if we found a different target world
        let (target_world, position) = if let Some(ref new_world) = target_world {
            debug!(
                "Cross-dimension respawn: {} -> {}",
                self.dimension.minecraft_name, new_world.dimension.minecraft_name
            );

            // Remove player from current world
            self.remove_player(player, false).await;
            new_world.players.rcu(|current_list| {
                let mut new_list = (**current_list).clone();
                new_list.push(player.clone());
                new_list
            });

            // Update chunk manager to target world
            player
                .chunk_manager
                .lock()
                .await
                .change_world(&self.level, new_world.clone());

            // Unload watched chunks from current world
            player.unload_watched_chunks(self).await;

            (new_world.as_ref(), position)
        } else if respawn_dimension != self.dimension {
            // Cross-dimension failed - fall back to current world's spawn
            warn!(
                "Target world {:?} not found, using world spawn in {:?}",
                respawn_dimension, self.dimension
            );
            // FIXME: This spawn position calculation is incorrect. Should use vanilla's
            // proper spawn position calculation (see #1381).
            let top = self.get_top_block(Vector2::new(spawn_x, spawn_z)).await;
            let fallback_pos = Vector3::new(
                f64::from(spawn_x) + 0.5,
                (top + 1).into(),
                f64::from(spawn_z) + 0.5,
            );
            (self.as_ref(), fallback_pos)
        } else {
            (self.as_ref(), position)
        };

        // Send respawn packet with target dimension (using send_packet_now to ensure proper order)
        player
            .client
            .send_packet_now(&CRespawn::new(
                (target_world.dimension.id).into(),
                ResourceLocation::from(target_world.dimension.minecraft_name),
                biome::hash_seed(target_world.level.seed.0),
                player.gamemode.load() as u8,
                player.gamemode.load() as i8,
                false,
                false,
                Some((death_dimension, death_location)),
                VarInt(player.get_entity().portal_cooldown.load(Ordering::Relaxed) as i32),
                target_world.sea_level.into(),
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
        player
            .client
            .send_packet_now(&CPlayerSpawnPosition::new(
                spawn_block_pos,
                yaw,
                pitch,
                target_world.dimension.minecraft_name.to_string(),
            ))
            .await;

        player.living_entity.reset_state().await;

        player.send_permission_lvl_update().await;

        player.hunger_manager.restart();

        if !keep_inventory {
            player.set_experience(0, 0.0, 0).await;
            player.inventory.clear().await;
        }

        // Set entity position BEFORE loading chunks, so chunks load at the right location
        // This mirrors the initial spawn flow where update_position is called before teleport
        player.living_entity.entity.set_pos(position);
        player.living_entity.entity.set_rotation(yaw, pitch);
        player.living_entity.entity.last_pos.store(position);

        // TODO: difficulty, exp bar, status effect

        // Load chunks and send world info FIRST (before teleport packet)
        target_world
            .send_world_info(player, position, yaw, pitch)
            .await;

        // Ensure at least the center chunk is sent synchronously before teleport.
        if let crate::net::ClientPlatform::Java(java_client) = &player.client {
            let center_chunk = player.living_entity.entity.chunk_pos.load();
            let chunk = target_world.level.get_chunk(center_chunk).await;
            java_client.send_packet_now(&CChunkBatchStart).await;
            java_client.send_packet_now(&CChunkData(&chunk)).await;
            java_client
                .send_packet_now(&CChunkBatchEnd::new(1u16))
                .await;
        }

        // Send teleport packet after at least the center chunk was delivered
        player.request_teleport(position, yaw, pitch).await;

        // Call spawn event
        if let Some(server) = self.server.upgrade() {
            let spawnevent = PlayerRespawnEvent::new(player.clone());
            server.plugin_manager.fire(spawnevent).await;
        }
    }

    /// Returns true if enough players are sleeping and we should skip the night.
    pub fn should_skip_night(&self) -> bool {
        let players = self.players.load();

        let player_count = players.len();
        let sleeping_player_count = players
            .iter()
            .filter(|player| {
                player
                    .sleeping_since
                    .load()
                    .is_some_and(|since| since >= 100)
            })
            .count();
        drop(players);

        if player_count == 0 {
            return false;
        }

        let sleep_percentage = self
            .level_info
            .load()
            .game_rules
            .players_sleeping_percentage
            .clamp(0, 100);
        let required_sleeping =
            ((player_count as f64 * sleep_percentage as f64) / 100.0).ceil() as usize;
        let required_sleeping = required_sleeping.max(1);

        sleeping_player_count >= required_sleeping
    }

    // NOTE: This function doesn't actually await on anything, it just spawns two tokio tasks
    /// IMPORTANT: Chunks have to be non-empty
    #[expect(clippy::too_many_lines)]
    fn spawn_world_entity_chunks(
        self: &Arc<Self>,
        player: Arc<Player>,
        chunks: Vec<Vector2<i32>>,
        center_chunk: Vector2<i32>,
    ) {
        #[cfg(debug_assertions)]
        let inst = std::time::Instant::now();

        // Sort such that the first chunks are closest to the center.
        let mut chunks = chunks;
        chunks.sort_unstable_by_key(|pos| {
            let rel_x = pos.x - center_chunk.x;
            let rel_z = pos.y - center_chunk.y;
            rel_x * rel_x + rel_z * rel_z
        });

        let mut entity_receiver = self.level.receive_entity_chunks(chunks);
        let level = self.level.clone();
        let world = self.clone();
        player.clone().spawn_task(async move {
            'main: loop {
                let recv_result = tokio::select! {
                    () = player.client.await_close_interrupt() => {
                        debug!("Canceling player packet processing");
                        None
                    },
                    recv_result = entity_receiver.recv() => {
                        recv_result
                    }
                };

                let Some((chunk, _first_load)) = recv_result else {
                    break;
                };
                let position = Vector2::new(chunk.x, chunk.z);

                let chunk = if level.is_chunk_watched(&position) {
                    chunk
                } else {
                    trace!(
                        "Received chunk {:?}, but it is no longer watched... cleaning",
                        &position
                    );
                    let mut ids_to_remove = Vec::new();

                    for (uuid, entity_nbt) in chunk.data.lock().await.iter() {
                        let Some(id) = entity_nbt.get_string("id") else {
                            warn!("Entity has no ID");
                            continue;
                        };
                        let Some(entity_type) =
                            EntityType::from_name(id.strip_prefix("minecraft:").unwrap_or(id))
                        else {
                            warn!("Entity has no valid Entity Type {id}");
                            continue;
                        };
                        // Pos is zero since it will read from nbt
                        let entity =
                            from_type(entity_type, Vector3::new(0.0, 0.0, 0.0), &world, *uuid)
                                .await;
                        entity.read_nbt_non_mut(entity_nbt).await;
                        let base_entity = entity.get_entity();

                        ids_to_remove.push(VarInt(base_entity.entity_id));

                        let mut nbt = NbtCompound::new();
                        entity.write_nbt(&mut nbt).await;
                        if let Some(old_chunk) = base_entity.first_loaded_chunk_position.load() {
                            let old_chunk = old_chunk.to_vec2_i32();
                            let chunk = world.level.get_entity_chunk(old_chunk).await;
                            chunk.mark_dirty(true);
                            let base_entity = entity.get_entity();
                            let current_chunk_coordinate =
                                base_entity.block_pos.load().chunk_position();

                            let mut data = chunk.data.lock().await;
                            if old_chunk == current_chunk_coordinate {
                                data.insert(*uuid, nbt);
                                return;
                            }

                            // The chunk has changed, lets remove the entity from the old chunk
                            data.remove(uuid);
                        }
                        chunk.data.lock().await.insert(*uuid, nbt);
                        chunk.mark_dirty(true);
                    }

                    if !ids_to_remove.is_empty() {
                        world.entities.rcu(|current_entities| {
                            let mut new_entities = (**current_entities).clone();
                            new_entities.retain(|e| {
                                !ids_to_remove.contains(&VarInt(e.get_entity().entity_id))
                            });
                            new_entities
                        });
                        player
                            .client
                            .enqueue_packet(&CRemoveEntities::new(&ids_to_remove))
                            .await;
                    }
                    level.clean_entity_chunk(&position);

                    continue 'main;
                };

                // Add all new Entities to the world
                let mut entities_to_add: Vec<Arc<dyn EntityBase>> = Vec::new();

                for (uuid, entity_nbt) in chunk.data.lock().await.iter() {
                    let Some(id) = entity_nbt.get_string("id") else {
                        debug!("Entity has no ID");
                        continue;
                    };
                    let Some(entity_type) =
                        EntityType::from_name(id.strip_prefix("minecraft:").unwrap_or(id))
                    else {
                        warn!("Entity has no valid Entity Type {id}");
                        continue;
                    };
                    // Pos is zero since it will read from nbt
                    let entity =
                        from_type(entity_type, Vector3::new(0.0, 0.0, 0.0), &world, *uuid).await;
                    entity.read_nbt_non_mut(entity_nbt).await;
                    let base_entity = entity.get_entity();
                    player
                        .client
                        .enqueue_packet(&base_entity.create_spawn_packet())
                        .await;
                    entity.init_data_tracker().await;

                    entities_to_add.push(entity);
                }
                if !entities_to_add.is_empty() {
                    world.entities.rcu(|current_entities| {
                        let mut new_entities = (**current_entities).clone();
                        new_entities.extend(entities_to_add.iter().cloned());
                        new_entities
                    });
                }
            }

            #[cfg(debug_assertions)]
            debug!("Chunks queued after {}ms", inst.elapsed().as_millis());
        });
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

    /// Gets an entity by an entity id
    pub fn get_entity_by_id(&self, id: i32) -> Option<Arc<dyn EntityBase>> {
        for entity in self.entities.load().iter() {
            if entity.get_entity().entity_id == id {
                return Some(entity.clone());
            }
        }
        for player in self.players.load().iter() {
            if player.get_entity().entity_id == id {
                return Some(player.clone() as Arc<dyn EntityBase>);
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

    // Gets all entities at a Box
    pub fn get_all_at_box(&self, aabb: &BoundingBox) -> Vec<Arc<dyn EntityBase>> {
        let entities_guard = self.entities.load();
        let players_guard = self.players.load();

        entities_guard
            .iter()
            .map(|e| e.clone() as Arc<dyn EntityBase>)
            .chain(
                players_guard
                    .iter()
                    .map(|p| p.clone() as Arc<dyn EntityBase>),
            )
            .filter(|entity| entity.get_entity().bounding_box.load().intersects(aabb))
            .collect()
    }

    // Gets all non Player entities at a Box
    pub fn get_entities_at_box(&self, aabb: &BoundingBox) -> Vec<Arc<dyn EntityBase>> {
        self.entities
            .load()
            .iter()
            .filter(|entity| entity.get_entity().bounding_box.load().intersects(aabb))
            .cloned()
            .collect()
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
                let player_block_pos = player.living_entity.entity.block_pos.load().0;
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
                let player_pos = player.living_entity.entity.pos.load();
                (player_pos.squared_distance_to_vec(&pos) <= radius_squared).then(|| player.clone())
            })
            .collect()
    }

    pub fn get_nearby_entities(
        &self,
        pos: Vector3<f64>,
        radius: f64,
    ) -> HashMap<uuid::Uuid, Arc<dyn EntityBase>> {
        let radius_squared = radius.powi(2);

        self.entities
            .load()
            .iter()
            .filter_map(|entity| {
                let entity_pos = entity.get_entity().pos.load();
                (entity_pos.squared_distance_to_vec(&pos) <= radius_squared)
                    .then(|| (entity.get_entity().entity_uuid, entity.clone()))
            })
            .collect()
    }

    pub fn get_closest_player(&self, pos: Vector3<f64>, radius: f64) -> Option<Arc<Player>> {
        let players = self.get_nearby_players(pos, radius);
        players
            .iter()
            .min_by(|a, b| {
                a.living_entity
                    .entity
                    .pos
                    .load()
                    .squared_distance_to_vec(&pos)
                    .partial_cmp(
                        &b.living_entity
                            .entity
                            .pos
                            .load()
                            .squared_distance_to_vec(&pos),
                    )
                    .unwrap()
            })
            .cloned()
    }

    /// Gets the closest entity to a position, with optional filtering by entity type.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position to search around.
    /// * `radius` - The radius to search within.
    /// * `entity_types` - Optional array of entity types to filter by. If None, all entity types are included.
    ///
    /// # Returns
    ///
    /// The closest entity that matches the filter criteria, or None if no entities are found.
    pub fn get_closest_entity(
        &self,
        pos: Vector3<f64>,
        radius: f64,
        entity_types: Option<&[&'static EntityType]>,
    ) -> Option<Arc<dyn EntityBase>> {
        // Get regular entities
        let entities = self.get_nearby_entities(pos, radius);

        // Filter by entity type if specified
        let filtered_entities = if let Some(types) = entity_types {
            entities
                .into_iter()
                .filter(|(_, entity)| {
                    let entity_type = entity.get_entity().entity_type;
                    types.contains(&entity_type)
                })
                .collect::<HashMap<_, _>>()
        } else {
            entities
        };

        // Find the closest entity
        filtered_entities
            .iter()
            .min_by(|a, b| {
                a.1.get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&pos)
                    .partial_cmp(&b.1.get_entity().pos.load().squared_distance_to_vec(&pos))
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
    pub fn add_player(&self, player: Arc<Player>) -> Result<(), String> {
        self.players.rcu(|current_list| {
            let mut new_list = (**current_list).clone();
            new_list.push(player.clone());
            new_list
        });

        let server = self.server.upgrade().unwrap();
        let current_players = self.players.load();
        player.clone().spawn_task(async move {
            let msg_comp = TextComponent::translate(
                translation::MULTIPLAYER_PLAYER_JOINED,
                [TextComponent::text(player.gameprofile.name.clone())],
            )
            .color_named(NamedColor::Yellow);
            let event = PlayerJoinEvent::new(player.clone(), msg_comp);

            let event = server.plugin_manager.fire(event).await;

            if !event.cancelled {
                for player in current_players.iter() {
                    player.send_system_message(&event.join_message).await;
                }
                // TODO: Switch to structured logging, e.g. info!(player = %name, "connected")
                info!("{}", event.join_message.to_pretty_console());
            }
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
            self.broadcast_packet_all(&CRemovePlayerInfo::new(&[uuid]))
                .await;
            self.broadcast_packet_all(&CRemoveEntities::new(&[player.entity_id().into()]))
                .await;

            if fire_event {
                let msg_comp = TextComponent::translate(
                    translation::MULTIPLAYER_PLAYER_LEFT,
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

    pub async fn spawn_entity(&self, entity: Arc<dyn EntityBase>) {
        let base_entity = entity.get_entity();
        self.broadcast_packet_all(&base_entity.create_spawn_packet())
            .await;
        entity.init_data_tracker().await;

        let chunk_coordinate = base_entity.block_pos.load().chunk_position();
        let chunk = self.level.get_entity_chunk(chunk_coordinate).await;
        {
            let mut nbt = NbtCompound::new();
            entity.write_nbt(&mut nbt).await;
            chunk.data.lock().await.insert(base_entity.entity_uuid, nbt);
            chunk.mark_dirty(true);
        };

        self.entities.rcu(|current_entities| {
            let mut new_entities = (**current_entities).clone();
            new_entities.push(entity.clone());
            new_entities
        });
    }

    pub async fn remove_entity(&self, entity: &Entity) {
        self.entities.rcu(|current_entities| {
            let mut new_entities = (**current_entities).clone();
            new_entities.retain(|e| e.get_entity().entity_uuid != entity.entity_uuid);
            new_entities
        });

        self.broadcast_packet_all(&CRemoveEntities::new(&[entity.entity_id.into()]))
            .await;

        self.remove_entity_data(entity).await;
    }

    pub async fn set_block_breaking(&self, from: &Entity, location: BlockPos, progress: i32) {
        self.broadcast_packet_except(
            &[from.entity_uuid],
            &CSetBlockDestroyStage::new(from.entity_id.into(), location, progress as i8),
        )
        .await;
    }

    /// Sets a block and returns the old block id
    #[expect(clippy::too_many_lines)]
    pub async fn set_block_state(
        self: &Arc<Self>,
        position: &BlockPos,
        block_state_id: BlockStateId,
        flags: BlockFlags,
    ) -> BlockStateId {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let level = &self.level;
        let chunk = level.get_chunk(chunk_coordinate).await;

        let replaced_block_state_id = chunk.section.set_block_absolute_y(
            relative.x as usize,
            relative.y,
            relative.z as usize,
            block_state_id,
        );
        if replaced_block_state_id == block_state_id {
            return block_state_id;
        }
        // Mark chunk dirty if it isn't already
        if !chunk.is_dirty() {
            chunk.mark_dirty(true);
        }
        drop(chunk);

        self.unsent_block_changes
            .lock()
            .await
            .insert(*position, block_state_id);

        let old_block = Block::from_state_id(replaced_block_state_id);
        let new_block = Block::from_state_id(block_state_id);

        let block_moved = flags.contains(BlockFlags::MOVED);

        let is_new_block = old_block != new_block;

        // WorldChunk.java line 305-314
        if is_new_block
            && old_block.default_state.block_entity_type != u16::MAX
            && let Some(entity) = self.get_block_entity(position).await
        {
            let world: Arc<dyn SimpleWorld> = self.clone();
            entity.on_block_replaced(world, *position).await;
            self.remove_block_entity(position).await;
        }

        // WorldChunk.java line 317
        if is_new_block && (flags.contains(BlockFlags::NOTIFY_NEIGHBORS) || block_moved) {
            self.block_registry
                .on_state_replaced(
                    self,
                    old_block,
                    position,
                    replaced_block_state_id,
                    block_moved,
                )
                .await;
        }

        // WorldChunk.java line 318
        if !flags.contains(BlockFlags::SKIP_BLOCK_ADDED_CALLBACK) && new_block != old_block {
            self.block_registry
                .on_placed(
                    self,
                    new_block,
                    block_state_id,
                    position,
                    replaced_block_state_id,
                    block_moved,
                )
                .await;
            let new_fluid = self.get_fluid(position).await;
            self.block_registry
                .on_placed_fluid(
                    self,
                    new_fluid,
                    block_state_id,
                    position,
                    replaced_block_state_id,
                    block_moved,
                )
                .await;
        }

        // Ig they do this cause it could be modified in chunkPos.setBlockState?
        if self.get_block_state_id(position).await == block_state_id {
            if flags.contains(BlockFlags::NOTIFY_LISTENERS) {
                // Mob AI update
            }

            if flags.contains(BlockFlags::NOTIFY_NEIGHBORS) {
                self.update_neighbors(position, None).await;
                // TODO: updateComparators
            }

            if !flags.contains(BlockFlags::FORCE_STATE) {
                let mut new_flags = flags;
                new_flags.remove(BlockFlags::NOTIFY_NEIGHBORS);
                new_flags.remove(BlockFlags::NOTIFY_LISTENERS);
                self.block_registry
                    .prepare(
                        self,
                        position,
                        Block::from_state_id(replaced_block_state_id),
                        replaced_block_state_id,
                        new_flags,
                    )
                    .await;
                self.block_registry
                    .update_neighbors(
                        self,
                        position,
                        Block::from_state_id(block_state_id),
                        new_flags,
                    )
                    .await;
                self.block_registry
                    .prepare(
                        self,
                        position,
                        Block::from_state_id(block_state_id),
                        block_state_id,
                        new_flags,
                    )
                    .await;
            }
        }

        let (_chunk_coordinate, _) = position.chunk_and_chunk_relative_position();

        level
            .light_engine
            .update_lighting_at(level, *position)
            .await;

        replaced_block_state_id
    }

    pub async fn schedule_block_tick(
        &self,
        block: &Block,
        block_pos: BlockPos,
        delay: u8,
        priority: TickPriority,
    ) {
        self.level
            .schedule_block_tick(block, block_pos, delay, priority)
            .await;
    }

    pub async fn schedule_fluid_tick(
        &self,
        fluid: &Fluid,
        block_pos: BlockPos,
        delay: u8,
        priority: TickPriority,
    ) {
        self.level
            .schedule_fluid_tick(fluid, block_pos, delay, priority)
            .await;
    }

    pub async fn is_block_tick_scheduled(&self, block_pos: &BlockPos, block: &Block) -> bool {
        self.level.is_block_tick_scheduled(block_pos, block).await
    }

    pub async fn is_fluid_tick_scheduled(&self, block_pos: &BlockPos, fluid: &Fluid) -> bool {
        self.level.is_fluid_tick_scheduled(block_pos, fluid).await
    }

    // Return new state
    pub async fn break_block(
        self: &Arc<Self>,
        position: &BlockPos,
        cause: Option<Arc<Player>>,
        flags: BlockFlags,
    ) -> Option<u16> {
        let (broken_block, broken_block_state) = self.get_block_and_state_id(position).await;
        let event = BlockBreakEvent::new(cause.clone(), broken_block, *position, 0, false);

        let event = self
            .server
            .upgrade()
            .unwrap()
            .plugin_manager
            .fire::<BlockBreakEvent>(event)
            .await;

        if !event.cancelled {
            let new_state_id = if broken_block
                .properties(broken_block_state)
                .and_then(|properties| {
                    properties
                        .to_props()
                        .into_iter()
                        .find(|p| p.0 == "waterlogged")
                        .map(|(_, value)| value == "true")
                })
                .unwrap_or(false)
            {
                let mut water_props = FlowingFluidProperties::default(&Fluid::FLOWING_WATER);
                water_props.level = pumpkin_data::fluid::Level::L8;
                water_props.falling = Falling::False;
                water_props.to_state_id(&Fluid::FLOWING_WATER)
            } else {
                0
            };

            let broken_state_id = self.set_block_state(position, new_state_id, flags).await;

            // Close container screens for any players viewing this block
            self.close_container_screens_at(position).await;

            if Block::from_state_id(broken_state_id) != &Block::FIRE {
                let particles_packet = CWorldEvent::new(
                    WorldEvent::BlockBroken as i32,
                    *position,
                    broken_state_id.into(),
                    false,
                );
                match cause {
                    Some(player) => {
                        self.broadcast_packet_except(&[player.gameprofile.id], &particles_packet)
                            .await;
                    }
                    None => self.broadcast_packet_all(&particles_packet).await,
                }
            }

            if !flags.contains(BlockFlags::SKIP_DROPS) {
                let params = LootContextParameters {
                    block_state: Some(BlockState::from_id(broken_state_id)),
                    ..Default::default()
                };
                block::drop_loot(self, broken_block, position, true, params).await;
            }
            return Some(new_state_id);
        }
        None
    }

    /// Close container screens for all players who have a container open at the given block position.
    pub async fn close_container_screens_at(&self, position: &BlockPos) {
        let players = self.players.load();
        for player in players.iter() {
            if player.open_container_pos.load() == Some(*position) {
                player.close_handled_screen().await;
            }
        }
    }

    pub async fn drop_stack(self: &Arc<Self>, pos: &BlockPos, stack: ItemStack) {
        let height = EntityType::ITEM.dimension[1] / 2.0;
        let spawn_pos = {
            let mut r = rand::rng();
            Vector3::new(
                f64::from(pos.0.x) + 0.5 + r.random_range(-0.25..0.25),
                f64::from(pos.0.y) + 0.5 + r.random_range(-0.25..0.25) - f64::from(height),
                f64::from(pos.0.z) + 0.5 + r.random_range(-0.25..0.25),
            )
        };

        let entity = Entity::new(self.clone(), spawn_pos, &EntityType::ITEM);
        let item_entity = Arc::new(ItemEntity::new(entity, stack).await);
        self.spawn_entity(item_entity).await;
    }

    /* ItemScatterer.java */
    pub async fn scatter_inventory(
        self: &Arc<Self>,
        position: &BlockPos,
        inventory: &Arc<dyn Inventory>,
    ) {
        for i in 0..inventory.size() {
            self.scatter_stack(
                f64::from(position.0.x),
                f64::from(position.0.y),
                f64::from(position.0.z),
                inventory.remove_stack(i).await,
            )
            .await;
        }
    }
    pub async fn scatter_stack(self: &Arc<Self>, x: f64, y: f64, z: f64, mut stack: ItemStack) {
        const TRIANGULAR_DEVIATION: f64 = 0.114_850_001_711_398_36;

        const XZ_MODE: f64 = 0.0;
        const Y_MODE: f64 = 0.2;

        let width = f64::from(EntityType::ITEM.dimension[0]);
        let half_width = width / 2.0;
        let spawn_area = 1.0 - width;

        let mut rng = Xoroshiro::from_seed(get_seed());

        // TODO: Use world random here: world.random.nextDouble()
        let x = rng.next_f64().mul_add(spawn_area, x.floor()) + half_width;
        let y = rng.next_f64().mul_add(spawn_area, y.floor());
        let z = rng.next_f64().mul_add(spawn_area, z.floor()) + half_width;

        while !stack.is_empty() {
            let item = stack.split((rng.next_bounded_i32(21) + 10) as u8);
            let velocity = Vector3::new(
                rng.next_triangular(XZ_MODE, TRIANGULAR_DEVIATION),
                rng.next_triangular(Y_MODE, TRIANGULAR_DEVIATION),
                rng.next_triangular(XZ_MODE, TRIANGULAR_DEVIATION),
            );

            let entity = Entity::new(self.clone(), Vector3::new(x, y, z), &EntityType::ITEM);
            let entity = Arc::new(ItemEntity::new_with_velocity(entity, item, velocity, 10).await);
            self.spawn_entity(entity).await;
        }
    }
    /* End ItemScatterer.java */

    pub async fn sync_world_event(&self, world_event: WorldEvent, position: BlockPos, data: i32) {
        self.broadcast_packet_all(&CWorldEvent::new(world_event as i32, position, data, false))
            .await;
    }
    #[must_use]
    pub fn is_valid(dest: BlockPos) -> bool {
        Self::is_valid_horizontally(dest) && Self::is_valid_vertically(dest.0.y)
    }
    #[must_use]
    pub fn is_valid_horizontally(dest: BlockPos) -> bool {
        // Note: 30_000_000 is not valid, but -30_000_000 is.
        (-30_000_000..30_000_000).contains(&dest.0.x)
            && (-30_000_000..30_000_000).contains(&dest.0.z)
    }
    #[must_use]
    pub fn is_valid_vertically(y: i32) -> bool {
        // Note: 20_000_000 is not valid, but -20_000_000 is.
        (-20_000_000..20_000_000).contains(&y)
    }
    #[must_use]
    pub fn is_in_build_limit(&self, dest: BlockPos) -> bool {
        self.is_in_height_limit(dest.0.y) && Self::is_valid_horizontally(dest)
    }
    #[must_use]
    pub fn is_in_height_limit(&self, y: i32) -> bool {
        (self.get_bottom_y()..=self.get_top_y()).contains(&y)
    }
    pub const fn get_bottom_y(&self) -> i32 {
        self.dimension.min_y
    }
    pub const fn get_top_y(&self) -> i32 {
        self.dimension.min_y + self.dimension.height - 1
    }
    /// Gets a `Block` from the block registry. Returns `Block::AIR` if the block was not found.
    pub async fn get_block(&self, position: &BlockPos) -> &'static Block {
        let id = self.get_block_state_id(position).await;
        Block::from_state_id(id)
    }

    pub async fn get_fluid(&self, position: &BlockPos) -> &'static pumpkin_data::fluid::Fluid {
        let id = self.get_block_state_id(position).await;
        let fluid = Fluid::from_state_id(id).ok_or(&Fluid::EMPTY);
        if let Ok(fluid) = fluid {
            return fluid.to_flowing();
        }
        let block = Block::from_state_id(id);
        block
            .properties(id)
            .and_then(|props| {
                props
                    .to_props()
                    .into_iter()
                    .find(|p| p.0 == "waterlogged")
                    .map(|(_, value)| {
                        if value == "true" {
                            &Fluid::FLOWING_WATER
                        } else {
                            &Fluid::EMPTY
                        }
                    })
            })
            .unwrap_or(&Fluid::EMPTY)
    }

    pub async fn get_block_and_fluid(
        &self,
        position: &BlockPos,
    ) -> (
        &'static pumpkin_data::Block,
        &'static pumpkin_data::fluid::Fluid,
    ) {
        let id = self.get_block_state_id(position).await;
        let block = Block::from_state_id(id);

        let fluid = Fluid::from_state_id(id)
            .map(Fluid::to_flowing)
            .ok_or(&Fluid::EMPTY)
            .unwrap_or_else(|_| {
                block
                    .properties(id)
                    .and_then(|props| {
                        props
                            .to_props()
                            .into_iter()
                            .find(|p| p.0 == "waterlogged")
                            .map(|(_, value)| {
                                if value == "true" {
                                    &Fluid::FLOWING_WATER
                                } else {
                                    &Fluid::EMPTY
                                }
                            })
                    })
                    .unwrap_or(&Fluid::EMPTY)
            });
        (block, fluid)
    }

    pub async fn get_fluid_and_fluid_state(
        &self,
        position: &BlockPos,
    ) -> (&'static Fluid, &'static FluidState) {
        let id = self.get_block_state_id(position).await;

        let Some(raw_fluid) = Fluid::from_state_id(id) else {
            let block = Block::from_state_id(id);
            if let Some(properties) = block.properties(id) {
                for (name, value) in properties.to_props() {
                    if name == "waterlogged" {
                        if value == "true" {
                            let state = &Fluid::FLOWING_WATER.states[0];
                            return (&Fluid::FLOWING_WATER, state);
                        }

                        break;
                    }
                }
            }

            let state = &Fluid::EMPTY.states[0];
            return (&Fluid::EMPTY, state);
        };

        let fluid = raw_fluid.to_flowing();
        let state = &fluid.states[0];

        (fluid, state)
    }

    pub async fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
        self.level.get_block_state(position).await.0
    }

    /// Gets the `BlockState` from the block registry. Returns Air if the block state was not found.
    pub async fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
        let id = self.get_block_state_id(position).await;
        BlockState::from_id(id)
    }

    /// Gets the Block + Block state from the Block Registry, Returns Air if the Block state has not been found
    pub async fn get_block_and_state(
        &self,
        position: &BlockPos,
    ) -> (&'static Block, &'static BlockState) {
        let id = self.get_block_state_id(position).await;
        BlockState::from_id_with_block(id)
    }

    /// Gets the Block + state id from the Block Registry, Returns Air if the Block state has not been found
    pub async fn get_block_and_state_id(&self, position: &BlockPos) -> (&'static Block, u16) {
        let id = self.get_block_state_id(position).await;
        (Block::from_state_id(id), id)
    }

    /// Updates neighboring blocks of a block
    pub async fn update_neighbors(
        self: &Arc<Self>,
        block_pos: &BlockPos,
        except: Option<BlockDirection>,
    ) {
        let source_block = self.get_block(block_pos).await;
        for direction in BlockDirection::update_order() {
            if except.is_some_and(|d| d == direction) {
                continue;
            }

            let neighbor_pos = block_pos.offset(direction.to_offset());
            let (neighbor_block, neighbor_fluid) = self.get_block_and_fluid(&neighbor_pos).await;

            if let Some(neighbor_pumpkin_block) =
                self.block_registry.get_pumpkin_block(neighbor_block.id)
            {
                neighbor_pumpkin_block
                    .on_neighbor_update(OnNeighborUpdateArgs {
                        world: self,
                        block: neighbor_block,
                        position: &neighbor_pos,
                        source_block,
                        notify: false,
                    })
                    .await;
            }

            if let Some(neighbor_pumpkin_fluid) =
                self.block_registry.get_pumpkin_fluid(neighbor_fluid.id)
            {
                neighbor_pumpkin_fluid
                    .on_neighbor_update(self, neighbor_fluid, &neighbor_pos, false)
                    .await;
            }
        }
    }

    pub async fn update_neighbor(
        self: &Arc<Self>,
        neighbor_block_pos: &BlockPos,
        source_block: &Block,
    ) {
        let neighbor_block = self.get_block(neighbor_block_pos).await;

        if let Some(neighbor_pumpkin_block) =
            self.block_registry.get_pumpkin_block(neighbor_block.id)
        {
            neighbor_pumpkin_block
                .on_neighbor_update(OnNeighborUpdateArgs {
                    world: self,
                    block: neighbor_block,
                    position: neighbor_block_pos,
                    source_block,
                    notify: false,
                })
                .await;
        }
    }

    pub async fn replace_with_state_for_neighbor_update(
        self: &Arc<Self>,
        block_pos: &BlockPos,
        direction: BlockDirection,
        flags: BlockFlags,
    ) {
        let (block, block_state_id) = self.get_block_and_state_id(block_pos).await;

        if flags.contains(BlockFlags::SKIP_REDSTONE_WIRE_STATE_REPLACEMENT)
            && *block == Block::REDSTONE_WIRE
        {
            return;
        }

        let neighbor_pos = block_pos.offset(direction.to_offset());
        let neighbor_state_id = self.get_block_state_id(&neighbor_pos).await;

        let new_state_id = self
            .block_registry
            .get_state_for_neighbor_update(
                self,
                block,
                block_state_id,
                block_pos,
                direction,
                &neighbor_pos,
                neighbor_state_id,
            )
            .await;

        if new_state_id != block_state_id {
            if is_air(new_state_id) {
                self.break_block(block_pos, None, flags).await;
            } else {
                self.set_block_state(block_pos, new_state_id, flags).await;
            }
        }
    }

    pub async fn get_block_entity(&self, block_pos: &BlockPos) -> Option<Arc<dyn BlockEntity>> {
        let chunk = self.level.get_chunk(block_pos.chunk_position()).await;
        chunk.block_entities.lock().unwrap().get(block_pos).cloned()
    }

    pub async fn add_block_entity(&self, block_entity: Arc<dyn BlockEntity>) {
        let block_pos = block_entity.get_position();
        let chunk = self.level.get_chunk(block_pos.chunk_position()).await;
        let block_entity_nbt = block_entity.chunk_data_nbt();

        if let Some(nbt) = &block_entity_nbt {
            let mut bytes = Vec::new();
            to_bytes_unnamed(nbt, &mut bytes).unwrap();
            self.broadcast_packet_all(&CBlockEntityData::new(
                block_entity.get_position(),
                VarInt(block_entity.get_id() as i32),
                bytes.into_boxed_slice(),
            ))
            .await;
        }

        chunk
            .block_entities
            .lock()
            .unwrap()
            .insert(block_pos, block_entity);
        chunk.mark_dirty(true);
    }

    pub async fn remove_block_entity(&self, block_pos: &BlockPos) {
        let chunk = self.level.get_chunk(block_pos.chunk_position()).await;
        if chunk
            .block_entities
            .lock()
            .unwrap()
            .remove(block_pos)
            .is_some()
        {
            chunk.mark_dirty(true);
        }
    }

    pub async fn update_block_entity(&self, block_entity: &Arc<dyn BlockEntity>) {
        let block_pos = block_entity.get_position();
        let chunk = self.level.get_chunk(block_pos.chunk_position()).await;
        let block_entity_nbt = block_entity.chunk_data_nbt();

        if let Some(nbt) = &block_entity_nbt {
            let mut bytes = Vec::new();
            to_bytes_unnamed(nbt, &mut bytes).unwrap();
            self.broadcast_packet_all(&CBlockEntityData::new(
                block_entity.get_position(),
                VarInt(block_entity.get_id() as i32),
                bytes.into_boxed_slice(),
            ))
            .await;
        }
        chunk.mark_dirty(true);
    }

    fn intersects_aabb_with_direction(
        from: Vector3<f64>,
        to: Vector3<f64>,
        min: Vector3<f64>,
        max: Vector3<f64>,
    ) -> Option<BlockDirection> {
        let dir = to.sub(&from);
        let mut tmin: f64 = 0.0;
        let mut tmax: f64 = 1.0;

        let mut hit_axis = None;
        let mut hit_is_min = false;

        macro_rules! check_axis {
            ($axis:ident, $dir_axis:ident, $min_axis:ident, $max_axis:ident, $direction_min:expr, $direction_max:expr) => {{
                if dir.$dir_axis.abs() < 1e-8 {
                    if from.$dir_axis < min.$min_axis || from.$dir_axis > max.$max_axis {
                        return None;
                    }
                } else {
                    let inv_d = 1.0 / dir.$dir_axis;
                    let t_near = (min.$min_axis - from.$dir_axis) * inv_d;
                    let t_far = (max.$max_axis - from.$dir_axis) * inv_d;

                    // Determine entry and exit points based on ray direction
                    let (t_entry, t_exit, is_min_face) = if inv_d >= 0.0 {
                        (t_near, t_far, true)
                    } else {
                        (t_far, t_near, false)
                    };

                    if t_entry > tmin {
                        tmin = t_entry;
                        hit_axis = Some(stringify!($axis));
                        hit_is_min = is_min_face;
                    }
                    tmax = tmax.min(t_exit);
                    if tmax < tmin {
                        return None;
                    }
                }
            }};
        }

        check_axis!(x, x, x, x, BlockDirection::West, BlockDirection::East);
        check_axis!(y, y, y, y, BlockDirection::Down, BlockDirection::Up);
        check_axis!(z, z, z, z, BlockDirection::North, BlockDirection::South);

        match (hit_axis, hit_is_min) {
            (Some("x"), true) => Some(BlockDirection::West),
            (Some("x"), false) => Some(BlockDirection::East),
            (Some("y"), true) => Some(BlockDirection::Down),
            (Some("y"), false) => Some(BlockDirection::Up),
            (Some("z"), true) => Some(BlockDirection::North),
            (Some("z"), false) => Some(BlockDirection::South),
            _ => None,
        }
    }

    async fn ray_outline_check(
        &self,
        block_pos: &BlockPos,
        from: Vector3<f64>,
        to: Vector3<f64>,
    ) -> (bool, Option<BlockDirection>) {
        let state = self.get_block_state(block_pos).await;

        if state.outline_shapes.is_empty() {
            return (true, None);
        }

        let bounding_boxes = state.get_block_outline_shapes();

        for shape in bounding_boxes {
            let world_min = shape.min.add(&block_pos.0.to_f64());
            let world_max = shape.max.add(&block_pos.0.to_f64());

            let direction = Self::intersects_aabb_with_direction(from, to, world_min, world_max);
            if direction.is_some() {
                return (true, direction);
            }
        }

        (false, None)
    }

    pub async fn raycast(
        self: &Arc<Self>,
        start_pos: Vector3<f64>,
        end_pos: Vector3<f64>,
        hit_check: impl AsyncFn(&BlockPos, &Arc<Self>) -> bool,
    ) -> Option<(BlockPos, BlockDirection)> {
        if start_pos == end_pos {
            return None;
        }

        let adjust = -1.0e-7f64;
        let to = end_pos.lerp(&start_pos, adjust);
        let from = start_pos.lerp(&end_pos, adjust);

        let mut block = BlockPos::floored(from.x, from.y, from.z);

        let (collision, direction) = self.ray_outline_check(&block, from, to).await;
        if let Some(dir) = direction
            && collision
        {
            return Some((block, dir));
        }

        let difference = to.sub(&from);

        let step = difference.sign();

        let delta = Vector3::new(
            if step.x == 0 {
                f64::MAX
            } else {
                (f64::from(step.x)) / difference.x
            },
            if step.y == 0 {
                f64::MAX
            } else {
                (f64::from(step.y)) / difference.y
            },
            if step.z == 0 {
                f64::MAX
            } else {
                (f64::from(step.z)) / difference.z
            },
        );

        let mut next = Vector3::new(
            delta.x
                * (if step.x > 0 {
                    1.0 - (from.x - from.x.floor())
                } else {
                    from.x - from.x.floor()
                }),
            delta.y
                * (if step.y > 0 {
                    1.0 - (from.y - from.y.floor())
                } else {
                    from.y - from.y.floor()
                }),
            delta.z
                * (if step.z > 0 {
                    1.0 - (from.z - from.z.floor())
                } else {
                    from.z - from.z.floor()
                }),
        );

        while next.x <= 1.0 || next.y <= 1.0 || next.z <= 1.0 {
            let block_direction = match (next.x, next.y, next.z) {
                (x, y, z) if x < y && x < z => {
                    block.0.x += step.x;
                    next.x += delta.x;
                    if step.x > 0 {
                        BlockDirection::West
                    } else {
                        BlockDirection::East
                    }
                }
                (_, y, z) if y < z => {
                    block.0.y += step.y;
                    next.y += delta.y;
                    if step.y > 0 {
                        BlockDirection::Down
                    } else {
                        BlockDirection::Up
                    }
                }
                _ => {
                    block.0.z += step.z;
                    next.z += delta.z;
                    if step.z > 0 {
                        BlockDirection::North
                    } else {
                        BlockDirection::South
                    }
                }
            };

            if hit_check(&block, self).await {
                let (collision, direction) = self.ray_outline_check(&block, from, to).await;
                if collision {
                    if let Some(dir) = direction {
                        return Some((block, dir));
                    }
                    return Some((block, block_direction));
                }
            }
        }

        None
    }
}

impl pumpkin_world::world::SimpleWorld for World {
    fn set_block_state(
        self: Arc<Self>,
        position: &BlockPos,
        block_state_id: BlockStateId,
        flags: BlockFlags,
    ) -> WorldFuture<'_, BlockStateId> {
        Box::pin(async move { Self::set_block_state(&self, position, block_state_id, flags).await })
    }

    fn update_neighbor<'a>(
        self: Arc<Self>,
        neighbor_block_pos: &'a BlockPos,
        source_block: &'a pumpkin_data::Block,
    ) -> WorldFuture<'a, ()> {
        Box::pin(async move {
            Self::update_neighbor(&self, neighbor_block_pos, source_block).await;
        })
    }

    fn update_neighbors(
        self: Arc<Self>,
        block_pos: &BlockPos,
        except: Option<BlockDirection>,
    ) -> WorldFuture<'_, ()> {
        Box::pin(async move {
            Self::update_neighbors(&self, block_pos, except).await;
        })
    }

    fn is_space_empty(&self, bounding_box: BoundingBox) -> WorldFuture<'_, bool> {
        Box::pin(async move { self.is_space_empty(bounding_box).await })
    }

    fn add_synced_block_event(&self, pos: BlockPos, r#type: u8, data: u8) -> WorldFuture<'_, ()> {
        Box::pin(async move {
            self.add_synced_block_event(pos, r#type, data).await;
        })
    }

    fn sync_world_event(
        &self,
        world_event: WorldEvent,
        position: BlockPos,
        data: i32,
    ) -> WorldFuture<'_, ()> {
        Box::pin(async move {
            self.sync_world_event(world_event, position, data).await;
        })
    }

    fn spawn_from_type(
        self: Arc<Self>,
        entity_type: &'static EntityType,
        position: Vector3<f64>,
    ) -> WorldFuture<'static, ()> {
        Box::pin(async move {
            let mob = from_type(entity_type, position, &self, Uuid::new_v4()).await;
            self.spawn_entity(mob).await;
        })
    }

    fn remove_block_entity<'a>(&'a self, block_pos: &'a BlockPos) -> WorldFuture<'a, ()> {
        Box::pin(async move {
            self.remove_block_entity(block_pos).await;
        })
    }

    fn get_block_entity<'a>(
        &'a self,
        block_pos: &'a BlockPos,
    ) -> WorldFuture<'a, Option<Arc<dyn BlockEntity>>> {
        Box::pin(async move { self.get_block_entity(block_pos).await })
    }

    fn get_world_age(&self) -> WorldFuture<'_, i64> {
        Box::pin(async move {
            // Note: MutexGuard must be released before returning the future's result.
            let level_time_guard = self.level_time.lock().await;
            level_time_guard.world_age
        })
    }

    fn play_sound<'a>(
        &'a self,
        sound: Sound,
        category: SoundCategory,
        position: &'a Vector3<f64>,
    ) -> WorldFuture<'a, ()> {
        Box::pin(async move {
            self.play_sound(sound, category, position).await;
        })
    }

    fn play_sound_fine<'a>(
        &'a self,
        sound: Sound,
        category: SoundCategory,
        position: &'a Vector3<f64>,
        volume: f32,
        pitch: f32,
    ) -> WorldFuture<'a, ()> {
        Box::pin(async move {
            self.play_sound_fine(sound, category, position, volume, pitch)
                .await;
        })
    }

    fn scatter_inventory<'a>(
        self: Arc<Self>,
        position: &'a BlockPos,
        inventory: &'a Arc<dyn Inventory>,
    ) -> WorldFuture<'a, ()> {
        Box::pin(async move {
            Self::scatter_inventory(&self, position, inventory).await;
        })
    }

    fn spawn_experience_orbs(
        self: Arc<Self>,
        position: Vector3<f64>,
        amount: u32,
    ) -> WorldFuture<'static, ()> {
        Box::pin(async move {
            ExperienceOrbEntity::spawn(&self, position, amount).await;
        })
    }

    fn update_from_neighbor_shapes(
        self: Arc<Self>,
        block_state_id: BlockStateId,
        position: &BlockPos,
    ) -> WorldFuture<'_, BlockStateId> {
        Box::pin(async move {
            let block = Block::from_state_id(block_state_id);
            let mut state_id = block_state_id;
            for direction in BlockDirection::update_order() {
                let neighbor_pos = position.offset(direction.to_offset());
                let neighbor_state_id = self.get_block_state_id(&neighbor_pos).await;
                state_id = self
                    .block_registry
                    .get_state_for_neighbor_update(
                        &self,
                        block,
                        state_id,
                        position,
                        direction,
                        &neighbor_pos,
                        neighbor_state_id,
                    )
                    .await;
            }
            state_id
        })
    }
}

impl BlockAccessor for World {
    fn get_block<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = &'static Block> + Send + 'a>> {
        Box::pin(async move { Self::get_block(self, position).await })
    }
    fn get_block_state<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = &'static BlockState> + Send + 'a>> {
        Box::pin(async move { Self::get_block_state(self, position).await })
    }

    fn get_block_state_id<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = BlockStateId> + Send + 'a>> {
        Box::pin(async move { Self::get_block_state_id(self, position).await })
    }

    fn get_block_and_state<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = (&'static Block, &'static BlockState)> + Send + 'a>> {
        Box::pin(async move { self.get_block_and_state(position).await })
    }
}
