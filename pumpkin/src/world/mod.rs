use std::num::NonZeroU32;
use std::pin::Pin;
use std::sync::Weak;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{Arc, atomic::Ordering},
};

pub mod chunker;
pub mod explosion;
pub mod loot;
pub mod portal;
pub mod time;

use crate::world::loot::LootContextParameters;
use crate::{
    PLUGIN_MANAGER,
    block::{
        self,
        registry::BlockRegistry,
        {OnNeighborUpdateArgs, OnScheduledTickArgs},
    },
    command::client_suggestions,
    entity::{Entity, EntityBase, player::Player, r#type::from_type},
    error::PumpkinError,
    net::ClientPlatform,
    plugin::{
        block::block_break::BlockBreakEvent,
        player::{player_join::PlayerJoinEvent, player_leave::PlayerLeaveEvent},
    },
    server::Server,
};
use crate::{block::BlockEvent, entity::item::ItemEntity};
use border::Worldborder;
use crossbeam::queue::SegQueue;
use explosion::Explosion;
use pumpkin_config::BasicConfiguration;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::MobCategory;
use pumpkin_data::fluid::{Falling, FluidProperties, FluidState};
use pumpkin_data::{
    Block,
    entity::{EntityStatus, EntityType},
    fluid::Fluid,
    particle::Particle,
    sound::{Sound, SoundCategory},
    world::{RAW, WorldEvent},
};
use pumpkin_data::{BlockDirection, BlockState};
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_nbt::{compound::NbtCompound, to_bytes_unnamed};
use pumpkin_protocol::bedrock::client::chunk_radius_update::CChunkRadiusUpdate;
use pumpkin_protocol::bedrock::client::network_chunk_publisher_update::CNetworkChunkPublisherUpdate;
use pumpkin_protocol::bedrock::client::start_game::CStartGame;
use pumpkin_protocol::bedrock::frame_set::FrameSet;
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
        CBlockUpdate, CDisguisedChatMessage, CExplosion, CRespawn, CSetBlockDestroyStage,
        CWorldEvent,
    },
};
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_util::{
    Difficulty,
    math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3},
};
use pumpkin_util::{
    math::{position::chunk_section_from_pos, vector2::Vector2},
    random::{RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use pumpkin_world::chunk::palette::BlockPalette;
use pumpkin_world::world::{GetBlockError, WorldFuture};
use pumpkin_world::{
    BlockStateId, CURRENT_BEDROCK_MC_VERSION, GENERATION_SETTINGS, GeneratorSetting, biome,
    block::entities::BlockEntity, chunk::io::Dirtiable, inventory::Inventory, item::ItemStack,
    world::SimpleWorld,
};
use pumpkin_world::{chunk::ChunkData, world::BlockAccessor};
use pumpkin_world::{level::Level, tick::TickPriority};
use pumpkin_world::{world::BlockFlags, world_info::LevelData};
use rand::seq::SliceRandom;
use rand::{Rng, rng};
use scoreboard::Scoreboard;
use time::LevelTime;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

pub mod border;
pub mod bossbar;
pub mod custom_bossbar;
pub mod natural_spawner;
pub mod scoreboard;
pub mod weather;

use crate::world::natural_spawner::{SpawnState, spawn_for_chunk};
use pumpkin_data::effect::StatusEffect;
use pumpkin_world::chunk::ChunkHeightmapType::MotionBlocking;
use pumpkin_world::generation::settings::GenerationSettings;
use uuid::Uuid;
use weather::Weather;

type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;

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
    pub level_info: Arc<RwLock<LevelData>>,
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
    pub dimension_type: VanillaDimensionType,
    pub sea_level: i32,
    pub min_y: i32,
    /// The world's weather, including rain and thunder levels.
    pub weather: Mutex<Weather>,
    /// Block Behaviour
    pub block_registry: Arc<BlockRegistry>,
    pub server: Weak<Server>,
    decrease_block_light_queue: SegQueue<(BlockPos, u8)>,
    increase_block_light_queue: SegQueue<(BlockPos, u8)>,
    synced_block_event_queue: Mutex<Vec<BlockEvent>>,
    /// A map of unsent block changes, keyed by block position.
    unsent_block_changes: Mutex<HashMap<BlockPos, u16>>,
}

impl World {
    #[must_use]
    pub fn load(
        level: Arc<Level>,
        level_info: Arc<RwLock<LevelData>>,
        dimension_type: VanillaDimensionType,
        block_registry: Arc<BlockRegistry>,
        server: Weak<Server>,
    ) -> Self {
        // TODO
        let generation_settings = match dimension_type {
            VanillaDimensionType::Overworld => GENERATION_SETTINGS
                .get(&GeneratorSetting::Overworld)
                .unwrap(),
            VanillaDimensionType::OverworldCaves => todo!(),
            VanillaDimensionType::TheEnd => {
                GENERATION_SETTINGS.get(&GeneratorSetting::End).unwrap()
            }
            VanillaDimensionType::TheNether => {
                GENERATION_SETTINGS.get(&GeneratorSetting::Nether).unwrap()
            }
        };

        Self {
            level,
            level_info,
            players: Arc::new(RwLock::new(HashMap::new())),
            entities: Arc::new(RwLock::new(HashMap::new())),
            scoreboard: Mutex::new(Scoreboard::default()),
            worldborder: Mutex::new(Worldborder::new(0.0, 0.0, 30_000_000.0, 0, 0, 0)),
            level_time: Mutex::new(LevelTime::new()),
            dimension_type,
            weather: Mutex::new(Weather::new()),
            block_registry,
            sea_level: generation_settings.sea_level,
            min_y: i32::from(generation_settings.shape.min_y),
            synced_block_event_queue: Mutex::new(Vec::new()),
            unsent_block_changes: Mutex::new(HashMap::new()),
            decrease_block_light_queue: SegQueue::new(),
            increase_block_light_queue: SegQueue::new(),
            server,
        }
    }

    pub async fn shutdown(&self) {
        for (uuid, entity) in self.entities.read().await.iter() {
            self.save_entity(uuid, entity).await;
        }
        self.level.shutdown().await;
    }

    async fn save_entity(&self, uuid: &uuid::Uuid, entity: &Arc<dyn EntityBase>) {
        // First lets see if the entity was saved on an other chunk, and if the current chunk does not match we remove it
        // Otherwise we just update the nbt data
        let base_entity = entity.get_entity();
        let (current_chunk_coordinate, _) = base_entity
            .block_pos
            .load()
            .chunk_and_chunk_relative_position();
        let mut nbt = NbtCompound::new();
        entity.write_nbt(&mut nbt).await;
        if let Some(old_chunk) = base_entity.first_loaded_chunk_position.load() {
            let old_chunk = old_chunk.to_vec2_i32();
            let chunk = self.level.get_entity_chunk(old_chunk).await;
            let mut chunk = chunk.write().await;
            chunk.mark_dirty(true);
            if old_chunk == current_chunk_coordinate {
                chunk.data.insert(*uuid, nbt);
                return;
            }

            // The chunk has changed, lets remove the entity from the old chunk
            chunk.data.remove(uuid);
        }
        // We did not continue, so lets save data in a new chunk
        let chunk = self.level.get_entity_chunk(current_chunk_coordinate).await;
        let mut chunk = chunk.write().await;
        chunk.data.insert(*uuid, nbt);
        chunk.mark_dirty(true);
    }

    async fn remove_entity_data(&self, entity: &Entity) {
        let (current_chunk_coordinate, _) =
            entity.block_pos.load().chunk_and_chunk_relative_position();
        if let Some(old_chunk) = entity.first_loaded_chunk_position.load() {
            let old_chunk = old_chunk.to_vec2_i32();
            let chunk = self.level.get_entity_chunk(old_chunk).await;
            let mut chunk = chunk.write().await;
            chunk.mark_dirty(true);
            if old_chunk == current_chunk_coordinate {
                chunk.data.remove(&entity.entity_uuid);
                drop(chunk);
            } else {
                let chunk = self.level.get_entity_chunk(current_chunk_coordinate).await;
                let mut chunk = chunk.write().await;
                // The chunk has changed, lets remove the entity from the old chunk
                chunk.data.remove(&entity.entity_uuid);
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

    pub async fn set_difficulty(&self, difficulty: Difficulty) {
        let mut level_info = self.level_info.write().await;

        level_info.difficulty = difficulty;
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

    /// Broadcasts a packet to all connected players within the world.
    /// Please avoid this as we want to replace it with `broadcast_editioned`
    ///
    /// Sends the specified packet to every player currently logged in to the world.
    ///
    /// **Note:** This function acquires a lock on the `current_players` map, ensuring thread safety.
    pub async fn broadcast_packet_all<P: ClientPacket>(&self, packet: &P) {
        let current_players = self.players.read().await;

        for (_, player) in current_players.iter() {
            player.client.enqueue_packet(packet).await;
        }
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
        let current_players = self.players.read().await;

        for (_, player) in current_players.iter() {
            match &player.client {
                ClientPlatform::Java(client) => client.enqueue_packet(je_packet).await,
                ClientPlatform::Bedrock(client) => client.send_game_packet(be_packet).await,
            }
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

        for recipient in self.players.read().await.values() {
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
        let current_players = self.players.read().await;

        for (_, player) in current_players.iter().filter(|c| !except.contains(c.0)) {
            player.client.enqueue_packet(packet).await;
        }
    }

    pub async fn spawn_particle(
        &self,
        position: Vector3<f64>,
        offset: Vector3<f32>,
        max_speed: f32,
        particle_count: i32,
        particle: Particle,
    ) {
        let players = self.players.read().await;
        for player in players.values() {
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

        self.flush_block_updates().await;
        // tick block entities
        self.flush_synced_block_events().await;

        self.tick_environment().await;

        let chunk_start = tokio::time::Instant::now();
        // log::debug!("Ticking chunks");
        self.tick_chunks().await;
        let chunk_elapsed = chunk_start.elapsed();

        let players_to_tick: Vec<_> = self.players.read().await.values().cloned().collect();

        // log::debug!("Ticking players");
        // player ticks
        for player in players_to_tick {
            player.tick(server).await;
        }

        let entities_to_tick: Vec<_> = self.entities.read().await.values().cloned().collect();

        // log::debug!("Ticking entities");
        // Entity ticks
        for entity in entities_to_tick {
            entity.get_entity().age.fetch_add(1, Relaxed);

            entity.tick(entity.clone(), server).await;
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
                    entity.on_player_collision(player).await;
                    break;
                }
            }
        }

        self.level.chunk_loading.lock().unwrap().send_change();

        if start.elapsed().as_millis() > 50 {
            log::warn!(
                "Slow Tick: Total {:?} | Chunks {:?}",
                start.elapsed(),
                chunk_elapsed,
            );
        }
    }

    pub async fn flush_block_updates(&self) {
        let mut block_state_updates_by_chunk_section = HashMap::new();
        for (position, block_state_id) in self.unsent_block_changes.lock().await.drain() {
            let chunk_section = chunk_section_from_pos(&position);
            block_state_updates_by_chunk_section
                .entry(chunk_section)
                .or_insert(Vec::new())
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
                self.broadcast_packet_all(&CMultiBlockUpdate::new(chunk_section.clone()))
                    .await;
            }
        }
    }

    async fn tick_environment(&self) {
        let mut level_time = self.level_time.lock().await;
        level_time.tick_time();

        // Auto-save logic
        if level_time.world_age % 100 == 0 {
            self.level.should_unload.store(true, Relaxed);
            if level_time.world_age % 300 != 0 {
                self.level.level_channel.notify();
            }
        }
        if level_time.world_age % 300 == 0 {
            self.level.should_save.store(true, Relaxed);
            self.level.level_channel.notify();
        }

        let mut weather = self.weather.lock().await;
        weather.tick_weather(self).await;

        if self.should_skip_night().await {
            let time = level_time.time_of_day + 24000;
            level_time.set_time(time - time % 24000);
            level_time.send_time(self).await;

            for player in self.players.read().await.values() {
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
        let tick_data = self.level.get_tick_data().await;
        for scheduled_tick in tick_data.block_ticks {
            let block = self.get_block(&scheduled_tick.position).await;
            if let Some(pumpkin_block) = self.block_registry.get_pumpkin_block(block) {
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
            if let Some(pumpkin_fluid) = self.block_registry.get_pumpkin_fluid(fluid) {
                pumpkin_fluid
                    .on_scheduled_tick(self, fluid, &scheduled_tick.position)
                    .await;
            }
        }

        /* TODO: Fix this deadlock
        for scheduled_tick in tick_data.random_ticks {
            let block = self.get_block(&scheduled_tick.position).await;
            if let Some(pumpkin_block) = self.block_registry.get_pumpkin_block(block) {
                pumpkin_block
                    .random_tick(RandomTickArgs {
                        world: self,
                        block,
                        position: &scheduled_tick.position,
                    })
                    .await;
            }
        } */

        let spawn_entity_clock_start = tokio::time::Instant::now();

        let mut spawning_chunks_map = HashMap::new();
        // TODO use FixedPlayerDistanceChunkTracker

        for i in self.players.read().await.values() {
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

        let mut spawning_chunks: Vec<(Vector2<i32>, Arc<RwLock<ChunkData>>)> =
            spawning_chunks_map.into_iter().collect();

        let get_chunks_clock = spawn_entity_clock_start.elapsed();
        // log::debug!("spawning chunks size {}", spawning_chunks.len());

        let mut spawn_state =
            SpawnState::new(spawning_chunks.len() as i32, &self.entities, self).await; // TODO store it

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

        spawning_chunks.shuffle(&mut rng());

        // TODO i think it can be multithread
        for (pos, chunk) in &spawning_chunks {
            self.tick_spawning_chunk(pos, chunk, &spawn_list, &mut spawn_state)
                .await;
        }
        log::trace!(
            "Spawning entity took {:?}, getting chunks {:?}, spawning chunks: {}, avg {:?} per chunk",
            spawn_entity_clock_start.elapsed(),
            get_chunks_clock,
            spawning_chunks.len(),
            spawn_entity_clock_start
                .elapsed()
                .checked_div(spawning_chunks.len() as u32)
                .unwrap_or(Duration::new(0, 0))
        );

        for block_entity in tick_data.block_entities {
            let world: Arc<dyn SimpleWorld> = self.clone();
            block_entity.tick(world).await;
        }
    }

    pub async fn get_fluid_collisions(self: &Arc<Self>, bounding_box: BoundingBox) -> Vec<Fluid> {
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
                let block = Block::from_state_id(block_state_id);
                let block_state = BlockState::from_id(block_state_id);

                let blocks_movement = block_state.is_solid()
                    && block != &Block::COBWEB
                    && block != &Block::BAMBOO_SAPLING;

                if !blocks_movement {
                    let down_pos = pos.down();

                    let (down_fluid, down_state) = self.get_fluid_and_fluid_state(&down_pos).await;

                    if down_fluid.id == fluid0.id {
                        amplitude = f64::from(state0.height - down_state.height) + 0.888_888_9;
                    }
                }
            } else {
                if fluid.id != fluid0.id {
                    continue;
                }

                //let state = fluid.get_state(block_state_id);
                let state = fluid.states[0].clone();

                amplitude = f64::from(state0.height - state.height);
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

        if fluid.id == fluid0_id {
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
        'shapes: for shape in state.get_block_outline_shapes().unwrap() {
            let outline_shape = shape.at_pos(pos);

            if outline_shape.intersects(bounding_box) {
                inside = true;

                if !use_outline_shape {
                    break 'shapes;
                }

                using_outline_shape(&outline_shape.to_bounding_box());
            }
        }

        inside
    }

    pub fn check_collision<F>(
        bounding_box: &BoundingBox,
        pos: BlockPos,
        state: &BlockState,
        use_collision_shape: bool,
        mut using_collision_shape: F,
    ) -> bool
    where
        F: FnMut(&BoundingBox),
    {
        let mut collided = false;

        if !state.is_air() && state.is_solid() && !state.collision_shapes.is_empty() {
            for shape in state.get_block_collision_shapes() {
                let collision_shape = shape.at_pos(pos);

                if collision_shape.intersects(bounding_box) {
                    collided = true;

                    if !use_collision_shape {
                        break;
                    }

                    using_collision_shape(&collision_shape.to_bounding_box());
                }
            }
        }

        collided
    }

    // For adjusting movement
    pub async fn get_block_collisions(
        self: &Arc<Self>,
        bounding_box: BoundingBox,
    ) -> (Vec<BoundingBox>, Vec<(usize, BlockPos)>) {
        let mut collisions = Vec::new();

        let mut positions = Vec::new();

        // Include downwards for fences

        let min = BlockPos::floored_v(bounding_box.min.add_raw(0.0, -0.50001, 0.0));

        let max = bounding_box.max_block_pos();

        for x in min.0.x..=max.0.x {
            for y in min.0.y..=max.0.y {
                for z in min.0.z..=max.0.z {
                    let pos = BlockPos::new(x, y, z);

                    let state = self.get_block_state(&pos).await;

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

    pub async fn tick_spawning_chunk(
        self: &Arc<Self>,
        chunk_pos: &Vector2<i32>,
        chunk: &Arc<RwLock<ChunkData>>,
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
                chunk.read().await.heightmap.get_height(
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
                        Uuid::new_v4(),
                        self.clone(),
                        random_pos.to_f64(),
                        &EntityType::SKELETON_HORSE,
                        false,
                    );
                    self.spawn_entity(Arc::new(entity)).await;
                }
                let entity = Entity::new(
                    Uuid::new_v4(),
                    self.clone(),
                    random_pos.to_f64().add_raw(0.5, 0., 0.5),
                    &EntityType::LIGHTNING_BOLT,
                    false,
                );
                self.spawn_entity(Arc::new(entity)).await;
            }
        }

        if spawn_list.is_empty() {
            return;
        }
        // TODO this.level.canSpawnEntitiesInChunk(chunkPos)
        spawn_for_chunk(self, chunk_pos, chunk, spawn_state, spawn_list).await;
    }

    pub fn generation_settings(&self) -> &GenerationSettings {
        // TODO: this is bad
        match self.dimension_type {
            VanillaDimensionType::Overworld => GENERATION_SETTINGS
                .get(&GeneratorSetting::Overworld)
                .unwrap(),
            VanillaDimensionType::OverworldCaves => todo!(),
            VanillaDimensionType::TheEnd => {
                GENERATION_SETTINGS.get(&GeneratorSetting::End).unwrap()
            }
            VanillaDimensionType::TheNether => {
                GENERATION_SETTINGS.get(&GeneratorSetting::Nether).unwrap()
            }
        }
    }

    /// Gets the y position of the first non air block from the top down
    pub async fn get_top_block(&self, position: Vector2<i32>) -> i32 {
        let generation_settings = self.generation_settings();
        for y in (i32::from(generation_settings.shape.min_y)
            ..i32::from(generation_settings.shape.max_y()))
            .rev()
        {
            let pos = BlockPos::new(position.x, y, position.y);
            let block = self.get_block_state(&pos).await;
            if block.is_air() {
                continue;
            }
            return y;
        }
        i32::from(generation_settings.shape.min_y)
    }

    #[allow(clippy::too_many_lines)]
    pub async fn spawn_bedrock_player(
        &self,
        base_config: &BasicConfiguration,
        player: Arc<Player>,
        server: &Server,
    ) {
        let level_info = server.level_info.read().await;
        let weather = self.weather.lock().await;
        let runtime_id = player.entity_id() as u64;
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
                position: Vector3::new(0.0, 100.0, 0.0),
                pitch: 0.0,
                yaw: 0.0,
                level_settings,
                level_id: String::new(),
                level_name: "Pumpkin world".to_string(),
                premium_world_template_id: String::new(),
                is_trial: false,
                rewind_history_size: VarInt(40),
                server_authoritative_block_breaking: false,
                current_level_time: self.level_time.lock().await.world_age as _,
                enchantment_seed: VarInt(0),
                block_properties_size: VarUInt(0),
                // TODO Make this unique
                multiplayer_correlation_id: Uuid::default().to_string(),
                enable_itemstack_net_manager: false,
                // TODO Make this description better!
                // This gets send from the client to mojang for telemetry
                server_version: "Pumpkin Rust Server".to_string(),

                compound_id: 10,
                compound_len: VarUInt(0),
                compound_end: 0,

                block_registry_checksum: 0,
                world_template_id: Uuid::default(),
                // TODO The client needs extra biome data for this
                enable_clientside_generation: false,
                blocknetwork_ids_are_hashed: false,
                tick_death_system_enabled: false,
                server_auth_sounds: false,
            })
            .await;
        client
            .send_game_packet(&CChunkRadiusUpdate {
                chunk_radius: VarInt(player.config.read().await.view_distance.get().into()),
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

        let mut frame_set = FrameSet::default();
        client
            .write_game_packet_to_set(
                &CNetworkChunkPublisherUpdate::new(
                    BlockPos::new(0, 100, 0),
                    NonZeroU32::from(player.config.read().await.view_distance).into(),
                ),
                &mut frame_set,
            )
            .await;

        client
            .write_game_packet_to_set(
                &CUpdateAttributes {
                    runtime_id: VarULong(runtime_id),
                    attributes: vec![Attribute {
                        min_value: 0.0,
                        max_value: f32::MAX,
                        current_value: 0.1,
                        default_min_value: 0.0,
                        default_max_value: f32::MAX,
                        default_value: 0.1,
                        name: "minecraft:movement".to_string(),
                        modifiers_list_size: VarUInt(0),
                    }],
                    player_tick: VarULong(0),
                },
                &mut frame_set,
            )
            .await;

        client
            .write_game_packet_to_set(&CPlayStatus::PlayerSpawn, &mut frame_set)
            .await;
        client.send_frame_set(frame_set, 0x84).await;

        {
            let mut abilities = player.abilities.lock().await;
            abilities.set_for_gamemode(player.gamemode.load());
        };

        player.send_abilities_update().await;
    }

    #[expect(clippy::too_many_lines)]
    pub async fn spawn_java_player(
        &self,
        base_config: &BasicConfiguration,
        player: Arc<Player>,
        server: &Server,
    ) {
        let dimensions: Vec<ResourceLocation> = server
            .dimensions
            .iter()
            .map(VanillaDimensionType::resource_location)
            .collect();

        // This code follows the vanilla packet order
        let entity_id = player.entity_id();
        let gamemode = player.gamemode.load();
        log::debug!(
            "spawning player {}, entity id {}",
            player.gameprofile.name,
            entity_id
        );

        let client = player.client.java();
        // Send the login packet for our new player
        client
            .send_packet_now(&CLogin::new(
                entity_id,
                base_config.hardcore,
                &dimensions,
                base_config.max_players.try_into().unwrap(),
                base_config.view_distance.get().into(), //  TODO: view distance
                base_config.simulation_distance.get().into(), // TODO: sim view dinstance
                false,
                true,
                false,
                (self.dimension_type as u8).into(),
                self.dimension_type.resource_location(),
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
        server
            .tick_rate_manager
            .update_joining_player(&player)
            .await;

        // Permissions, i.e. the commands a player may use.
        player.send_permission_lvl_update().await;

        // Difficulty of the world
        player.send_difficulty_update().await;
        {
            let command_dispatcher = server.command_dispatcher.read().await;
            client_suggestions::send_c_commands_packet(&player, &command_dispatcher).await;
        };

        // Spawn in initial chunks
        // This is made before the player teleport so that the player doesn't glitch out when spawning
        chunker::update_position(&player).await;

        // Teleport
        let (position, yaw, pitch) = if player.has_played_before.load(Ordering::Relaxed) {
            let position = player.position();
            let yaw = player.living_entity.entity.yaw.load(); //info.spawn_angle;
            let pitch = player.living_entity.entity.pitch.load();

            (position, yaw, pitch)
        } else {
            let info = &self.level_info.read().await;
            let spawn_position = Vector2::new(info.spawn_x, info.spawn_z);
            let pos_y = self.get_top_block(spawn_position).await + 1; // +1 to spawn on top of the block

            let position = Vector3::new(
                f64::from(info.spawn_x) + 0.5,
                f64::from(pos_y),
                f64::from(info.spawn_z) + 0.5,
            );
            let yaw = info.spawn_angle;
            (position, yaw, 0.0) // Pitch
        };

        let velocity = player.living_entity.entity.velocity.load();

        log::debug!("Sending player teleport to {}", player.gameprofile.name);
        player.request_teleport(position, yaw, pitch).await;

        player.living_entity.entity.last_pos.store(position);

        let gameprofile = &player.gameprofile;
        // Firstly, send an info update to our new player, so they can see their skin
        // and also send their info to everyone else.
        log::debug!("Broadcasting player info for {}", player.gameprofile.name);
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
        let current_players = self.players.read().await;
        // Here, we send all the infos of players who already joined.
        {
            let mut current_player_data = Vec::new();

            for (_, player) in current_players
                .iter()
                .filter(|(c, _)| **c != player.gameprofile.id)
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

            log::debug!("Sending player info to {}", player.gameprofile.name);
            client
                .enqueue_packet(&CPlayerInfoUpdate::new(action_flags.bits(), &entries))
                .await;
        };

        let gameprofile = &player.gameprofile;

        log::debug!("Broadcasting player spawn for {}", player.gameprofile.name);
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
        for (_, existing_player) in current_players.iter().filter(|c| c.0 != &id) {
            let entity = &existing_player.living_entity.entity;
            let pos = entity.pos.load();
            let gameprofile = &existing_player.gameprofile;
            log::debug!("Sending player entities to {}", player.gameprofile.name);

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
                // TODO
                // let config = existing_player.config.read().await;
                // let mut buf = Vec::new();
                // for meta in [
                //     Metadata::new(
                //         DATA_PLAYER_MODE_CUSTOMISATION,
                //         MetaDataType::Byte,
                //         config.skin_parts,
                //     ),
                //     Metadata::new(
                //         DATA_PLAYER_MAIN_HAND,
                //         MetaDataType::Byte,
                //         config.main_hand as u8,
                //     ),
                // ] {
                //     let mut serializer_buf = Vec::new();

                //     let mut serializer = Serializer::new(&mut serializer_buf);
                //     meta.serialize(&mut serializer).unwrap();
                //     buf.extend(serializer_buf);
                // }
                // drop(config);
                // // END
                // buf.put_u8(255);
                // client
                //     .enqueue_packet(&CSetEntityMetadata::new(
                //         existing_player.get_entity().entity_id.into(),
                //         buf.into(),
                //     ))
                //     .await;
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
        player.send_client_information();

        // Sync selected slot
        player
            .enqueue_set_held_item_packet(&CSetSelectedSlot::new(
                player.get_inventory().get_selected_slot() as i8,
            ))
            .await;

        // Start waiting for level chunks. Sets the "Loading Terrain" screen
        log::debug!("Sending waiting chunks to {}", player.gameprofile.name);
        client
            .send_packet_now(&CGameEvent::new(GameEvent::StartWaitingChunks, 0.0))
            .await;

        self.worldborder.lock().await.init_client(client).await;

        // Sends initial time
        player.send_time(self).await;

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
        self.send_player_equipment(&player).await;
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
        player.send_client_information();

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
        for player in self.players.read().await.values() {
            if player.position().squared_distance_to_vec(position) > 4096.0 {
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

    pub async fn respawn_player(&self, player: &Arc<Player>, alive: bool) {
        let last_pos = player.living_entity.entity.last_pos.load();
        let death_dimension = player.world().dimension_type.resource_location();
        let death_location = BlockPos(Vector3::new(
            last_pos.x.round() as i32,
            last_pos.y.round() as i32,
            last_pos.z.round() as i32,
        ));

        let data_kept = u8::from(alive);

        // TODO: switch world in player entity to new world

        player
            .client
            .enqueue_packet(&CRespawn::new(
                (self.dimension_type as u8).into(),
                self.dimension_type.resource_location(),
                biome::hash_seed(self.level.seed.0), // seed
                player.gamemode.load() as u8,
                player.gamemode.load() as i8,
                false,
                false,
                Some((death_dimension, death_location)),
                VarInt(player.get_entity().portal_cooldown.load(Ordering::Relaxed) as i32),
                self.sea_level.into(),
                data_kept,
            ))
            .await;

        player.living_entity.reset_state().await;

        log::debug!("Sending player abilities to {}", player.gameprofile.name);
        player.send_abilities_update().await;

        player.send_permission_lvl_update().await;

        player.hunger_manager.restart();

        let info = &self.level_info.read().await;
        if !info.game_rules.keep_inventory {
            player.set_experience(0, 0.0, 0).await;
        }

        // Teleport
        let pitch = 0.0;
        let (position, yaw) = if let Some(respawn) = player.get_respawn_point().await {
            respawn
        } else {
            let top = self
                .get_top_block(Vector2::new(info.spawn_x, info.spawn_z))
                .await;

            (
                Vector3::new(
                    f64::from(info.spawn_x) + 0.5,
                    (top + 1).into(),
                    f64::from(info.spawn_z) + 0.5,
                ),
                info.spawn_angle,
            )
        };

        log::debug!("Sending player teleport to {}", player.gameprofile.name);
        player.request_teleport(position, yaw, pitch).await;
        player.living_entity.entity.last_pos.store(position);

        // TODO: difficulty, exp bar, status effect

        self.send_world_info(player, position, yaw, pitch).await;
    }

    /// Returns true if enough players are sleeping and we should skip the night.
    pub async fn should_skip_night(&self) -> bool {
        let players = self.players.read().await;

        let player_count = players.len();
        let sleeping_player_count = players
            .values()
            .filter(|player| {
                player
                    .sleeping_since
                    .load()
                    .is_some_and(|since| since >= 100)
            })
            .count();
        drop(players);

        // TODO: sleep ratio
        sleeping_player_count == player_count
    }

    // NOTE: This function doesn't actually await on anything, it just spawns two tokio tasks
    /// IMPORTANT: Chunks have to be non-empty
    #[allow(clippy::too_many_lines)]
    fn spawn_world_entity_chunks(
        self: &Arc<Self>,
        player: Arc<Player>,
        chunks: Vec<Vector2<i32>>,
        center_chunk: Vector2<i32>,
    ) {
        if player.client.closed() {
            log::info!("The connection has closed before world chunks were spawned");
            return;
        }
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
                        log::debug!("Canceling player packet processing");
                        None
                    },
                    recv_result = entity_receiver.recv() => {
                        recv_result
                    }
                };

                let Some((chunk, _first_load)) = recv_result else {
                    break;
                };
                let tmp_chunk = chunk.read().await;
                let position = Vector2::new(tmp_chunk.x, tmp_chunk.z);
                drop(tmp_chunk);

                let chunk = if level.is_chunk_watched(&position) {
                    chunk
                } else {
                    log::trace!(
                        "Received chunk {:?}, but it is no longer watched... cleaning",
                        &position
                    );
                    let mut ids = Vec::new();
                    // Remove all the entities from the world
                    let mut entities = world.entities.write().await;
                    for (uuid, entity_nbt) in &chunk.read().await.data {
                        let Some(id) = entity_nbt.get_string("id") else {
                            log::warn!("Entity has no ID");
                            continue;
                        };
                        let Some(entity_type) =
                            EntityType::from_name(id.strip_prefix("minecraft:").unwrap_or(id))
                        else {
                            log::warn!("Entity has no valid Entity Type {id}");
                            continue;
                        };
                        // Pos is zero since it will read from nbt
                        let entity =
                            from_type(entity_type, Vector3::new(0.0, 0.0, 0.0), &world, *uuid)
                                .await;
                        entity.read_nbt_non_mut(entity_nbt).await;
                        let base_entity = entity.get_entity();

                        entities.remove(&base_entity.entity_uuid);
                        ids.push(VarInt(base_entity.entity_id));

                        world.save_entity(uuid, &entity).await;
                    }
                    if !ids.is_empty() {
                        player
                            .client
                            .enqueue_packet(&CRemoveEntities::new(&ids))
                            .await;
                    }
                    level.clean_entity_chunk(&position).await;

                    continue 'main;
                };

                let entity_chunk = chunk.read().await;
                // Add all new Entities to the world
                let mut current_entities = world.entities.write().await;

                for (uuid, entity_nbt) in &entity_chunk.data {
                    let Some(id) = entity_nbt.get_string("id") else {
                        log::warn!("Entity has no ID");
                        continue;
                    };
                    let Some(entity_type) =
                        EntityType::from_name(id.strip_prefix("minecraft:").unwrap_or(id))
                    else {
                        log::warn!("Entity has no valid Entity Type {id}");
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
                    current_entities.insert(base_entity.entity_uuid, entity);
                }
            }

            #[cfg(debug_assertions)]
            log::debug!("Chunks queued after {}ms", inst.elapsed().as_millis());
        });
    }

    /// Gets a `Player` by an entity id
    pub async fn get_player_by_id(&self, id: i32) -> Option<Arc<Player>> {
        for player in self.players.read().await.values() {
            if player.entity_id() == id {
                return Some(player.clone());
            }
        }
        None
    }

    /// Gets an entity by an entity id
    pub async fn get_entity_by_id(&self, id: i32) -> Option<Arc<dyn EntityBase>> {
        for entity in self.entities.read().await.values() {
            if entity.get_entity().entity_id == id {
                return Some(entity.clone());
            }
        }
        for player in self.players.read().await.values() {
            if player.get_entity().entity_id == id {
                return Some(player.clone() as Arc<dyn EntityBase>);
            }
        }
        None
    }

    /// Gets a `Player` by a username
    pub async fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        for player in self.players.read().await.values() {
            if player.gameprofile.name.eq_ignore_ascii_case(name) {
                return Some(player.clone());
            }
        }
        None
    }

    pub async fn get_entities_at_box(&self, aabb: &BoundingBox) -> Vec<Arc<dyn EntityBase>> {
        let entities_guard = self.entities.read().await;
        entities_guard
            .values()
            .filter(|entity| entity.get_entity().bounding_box.load().intersects(aabb))
            .cloned()
            .collect()
    }
    pub async fn get_players_at_box(&self, aabb: &BoundingBox) -> Vec<Arc<Player>> {
        let players_guard = self.players.read().await;
        players_guard
            .values()
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
    pub async fn get_player_by_uuid(&self, id: uuid::Uuid) -> Option<Arc<Player>> {
        self.players.read().await.get(&id).cloned()
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
    /// * `radius`: The radius of the sphere. The higher the radius, the more area will be checked (in every direction).
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

    pub async fn get_nearby_entities(
        &self,
        pos: Vector3<f64>,
        radius: f64,
    ) -> HashMap<uuid::Uuid, Arc<dyn EntityBase>> {
        let radius_squared = radius.powi(2);

        self.entities
            .read()
            .await
            .iter()
            .filter_map(|(id, entity)| {
                let entity_pos = entity.get_entity().pos.load();
                (entity_pos.squared_distance_to_vec(pos) <= radius_squared)
                    .then(|| (*id, entity.clone()))
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
    pub async fn get_closest_entity(
        &self,
        pos: Vector3<f64>,
        radius: f64,
        entity_types: Option<&[&'static EntityType]>,
    ) -> Option<Arc<dyn EntityBase>> {
        // Get regular entities
        let entities = self.get_nearby_entities(pos, radius).await;

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
                    .squared_distance_to_vec(pos)
                    .partial_cmp(&b.1.get_entity().pos.load().squared_distance_to_vec(pos))
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
    pub async fn add_player(&self, uuid: uuid::Uuid, player: Arc<Player>) -> Result<(), String> {
        self.players.write().await.insert(uuid, player.clone());

        let current_players = self.players.clone();
        player.clone().spawn_task(async move {
            let msg_comp = TextComponent::translate(
                "multiplayer.player.joined",
                [TextComponent::text(player.gameprofile.name.clone())],
            )
            .color_named(NamedColor::Yellow);
            let event = PlayerJoinEvent::new(player.clone(), msg_comp);

            let event = PLUGIN_MANAGER.fire(event).await;

            if !event.cancelled {
                for player in current_players.read().await.values() {
                    player.send_system_message(&event.join_message).await;
                }
                log::info!("{}", event.join_message.to_pretty_console());
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
    pub async fn remove_player(&self, player: &Arc<Player>, fire_event: bool) {
        if self
            .players
            .write()
            .await
            .remove(&player.gameprofile.id)
            .is_none()
        {
            return;
        }
        let uuid = player.gameprofile.id;
        self.broadcast_packet_all(&CRemovePlayerInfo::new(&[uuid]))
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

            let event = PLUGIN_MANAGER.fire(event).await;

            if !event.cancelled {
                let players = self.players.read().await;
                for player in players.values() {
                    player.send_system_message(&event.leave_message).await;
                }
                drop(players);
                log::info!("{}", event.leave_message.to_pretty_console());
            }
        }
    }

    /// Adds an entity to the world.
    pub async fn spawn_entity(&self, entity: Arc<dyn EntityBase>) {
        let base_entity = entity.get_entity();
        self.broadcast_packet_all(&base_entity.create_spawn_packet())
            .await;
        entity.init_data_tracker().await;

        let (chunk_coordinate, _) = base_entity
            .block_pos
            .load()
            .chunk_and_chunk_relative_position();
        let chunk = self.level.get_entity_chunk(chunk_coordinate).await;
        let mut chunk = chunk.write().await;
        let mut nbt = NbtCompound::new();
        entity.write_nbt(&mut nbt).await;
        chunk.data.insert(base_entity.entity_uuid, nbt);
        chunk.mark_dirty(true);
        drop(chunk);

        let mut current_entities = self.entities.write().await;
        current_entities.insert(base_entity.entity_uuid, entity);
    }

    pub async fn remove_entity(&self, entity: &Entity) {
        self.entities.write().await.remove(&entity.entity_uuid);
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

    pub fn queue_block_light_decrease(self: &Arc<Self>, pos: BlockPos, level: u8) {
        self.decrease_block_light_queue.push((pos, level));
    }

    pub fn queue_block_light_increase(self: &Arc<Self>, pos: BlockPos, level: u8) {
        self.increase_block_light_queue.push((pos, level));
    }

    pub async fn perform_block_light_updates(self: &Arc<Self>) -> i32 {
        let mut updates = 0;

        updates += self.perform_block_light_decrease_updates().await;

        updates += self.perform_block_light_increase_updates().await;

        updates
    }

    async fn perform_block_light_decrease_updates(self: &Arc<Self>) -> i32 {
        let mut updates = 0;

        while let Some((pos, expected_light)) = self.decrease_block_light_queue.pop() {
            self.propagate_block_light_decrease(&pos, expected_light)
                .await;
            updates += 1;
        }

        updates
    }

    async fn perform_block_light_increase_updates(self: &Arc<Self>) -> i32 {
        let mut updates = 0;

        while let Some((pos, expected_light)) = self.increase_block_light_queue.pop() {
            self.propagate_block_light_increase(&pos, expected_light)
                .await;
            updates += 1;
        }

        updates
    }

    async fn propagate_block_light_increase(self: &Arc<Self>, pos: &BlockPos, light_level: u8) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            if let Some(neighbor_light) = self.get_block_light_level(&neighbor_pos).await {
                let neighbor_state = self.get_block_state(&neighbor_pos).await;
                let new_light = light_level.saturating_sub(neighbor_state.opacity.max(1));

                if new_light > neighbor_light {
                    // TODO: Add shape checking for non-trivial blocks

                    self.set_block_light_level(&neighbor_pos, new_light)
                        .await
                        .unwrap();

                    if new_light > 1 {
                        self.queue_block_light_increase(neighbor_pos, new_light);
                    }
                }
            }
        }
    }

    async fn propagate_block_light_decrease(
        self: &Arc<Self>,
        pos: &BlockPos,
        removed_light_level: u8,
    ) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            if let Some(neighbor_light) = self.get_block_light_level(&neighbor_pos).await {
                if neighbor_light == 0 {
                    continue; // Skip if already 0
                }

                let neighbor_state = self.get_block_state(&neighbor_pos).await;
                let opacity = neighbor_state.opacity.max(1);

                let expected_from_removed_source = removed_light_level.saturating_sub(opacity);

                if neighbor_light <= expected_from_removed_source {
                    let neighbor_luminance = neighbor_state.luminance;

                    self.set_block_light_level(&neighbor_pos, 0).await.unwrap();

                    if neighbor_luminance == 0 {
                        self.queue_block_light_decrease(neighbor_pos, neighbor_light);
                    } else {
                        self.set_block_light_level(&neighbor_pos, neighbor_luminance)
                            .await
                            .unwrap();
                        self.queue_block_light_increase(neighbor_pos, neighbor_luminance);
                    }
                } else {
                    self.queue_block_light_increase(neighbor_pos, neighbor_light);
                }
            }
        }
    }

    pub async fn check_block_light_updates(self: &Arc<Self>, pos: BlockPos) {
        let current_light = self.get_block_light_level(&pos).await.unwrap_or(0);
        let block_state = self.get_block_state(&pos).await;
        let expected_light = block_state.luminance;

        if expected_light < current_light {
            self.set_block_light_level(&pos, 0).await.unwrap();
            self.queue_block_light_decrease(pos, current_light);
        }

        if expected_light > 0 {
            self.set_block_light_level(&pos, expected_light)
                .await
                .unwrap();
            self.queue_block_light_increase(pos, expected_light);
        }

        //TODO check sky light updates

        self.check_neighbors_light_updates(pos, current_light).await;
    }

    pub async fn check_neighbors_light_updates(self: &Arc<Self>, pos: BlockPos, current_light: u8) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());
            if let Some(neighbor_light) = self.get_block_light_level(&neighbor_pos).await
                && neighbor_light > current_light + 1
            {
                self.queue_block_light_increase(neighbor_pos, neighbor_light);
            }
        }
        // TODO check sky light updates
    }

    pub async fn get_block_light_level(&self, position: &BlockPos) -> Option<u8> {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = self.level.get_chunk(chunk_coordinate).await;
        let Ok(chunk) = tokio::time::timeout(std::time::Duration::from_secs(1), chunk.read()).await
        else {
            panic!("Timed out while waiting to acquire chunk read lock")
        };
        let section_index = (relative.y - chunk.section.min_y) as usize / BlockPalette::SIZE;
        // +1 since block light has 1 section padding on both top and bottom
        if section_index + 1 >= chunk.light_engine.block_light.len() {
            return None;
        }
        Some(chunk.light_engine.block_light[section_index + 1].get(
            relative.x as usize,
            (relative.y - chunk.section.min_y) as usize % BlockPalette::SIZE,
            relative.z as usize,
        ))
    }

    pub async fn set_block_light_level(
        &self,
        position: &BlockPos,
        light_level: u8,
    ) -> Result<(), String> {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = self.level.get_chunk(chunk_coordinate).await;
        let Ok(mut chunk) =
            tokio::time::timeout(std::time::Duration::from_secs(1), chunk.write()).await
        else {
            panic!("Timed out while waiting to acquire chunk write lock")
        };
        let section_index = (relative.y - chunk.section.min_y) as usize / BlockPalette::SIZE;
        if section_index >= chunk.light_engine.block_light.len() {
            return Err("Invalid section index".to_string());
        }
        let relative_y = (relative.y - chunk.section.min_y) as usize % BlockPalette::SIZE;
        chunk.light_engine.block_light[section_index + 1].set(
            relative.x as usize,
            relative_y,
            relative.z as usize,
            light_level,
        );
        chunk.mark_dirty(true);
        Ok(())
    }

    /// Sets a block and returns the old block id
    #[allow(clippy::too_many_lines)]
    pub async fn set_block_state(
        self: &Arc<Self>,
        position: &BlockPos,
        block_state_id: BlockStateId,
        flags: BlockFlags,
    ) -> BlockStateId {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = self.level.get_chunk(chunk_coordinate).await;
        let Ok(mut chunk) =
            tokio::time::timeout(std::time::Duration::from_secs(1), chunk.write()).await
        else {
            panic!("Timed out while waiting to acquire chunk write lock")
        };
        let replaced_block_state_id = chunk.section.set_block_absolute_y(
            relative.x as usize,
            relative.y,
            relative.z as usize,
            block_state_id,
        );
        if replaced_block_state_id == block_state_id {
            return block_state_id;
        }
        chunk.mark_dirty(true);

        self.unsent_block_changes
            .lock()
            .await
            .insert(*position, block_state_id);
        drop(chunk);

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

        self.check_block_light_updates(*position).await;
        self.perform_block_light_updates().await;

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

    pub async fn break_block(
        self: &Arc<Self>,
        position: &BlockPos,
        cause: Option<Arc<Player>>,
        flags: BlockFlags,
    ) {
        let (broken_block, broken_block_state) = self.get_block_and_state_id(position).await;
        let event = BlockBreakEvent::new(cause.clone(), broken_block, *position, 0, false);

        let event = PLUGIN_MANAGER.fire::<BlockBreakEvent>(event).await;

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
        }
    }

    pub async fn drop_stack(self: &Arc<Self>, pos: &BlockPos, stack: ItemStack) {
        let height = EntityType::ITEM.dimension[1] / 2.0;
        let pos = Vector3::new(
            f64::from(pos.0.x) + 0.5 + rand::rng().random_range(-0.25..0.25),
            f64::from(pos.0.y) + 0.5 + rand::rng().random_range(-0.25..0.25) - f64::from(height),
            f64::from(pos.0.z) + 0.5 + rand::rng().random_range(-0.25..0.25),
        );

        let entity = Entity::new(Uuid::new_v4(), self.clone(), pos, &EntityType::ITEM, false);
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
        let x = x.floor() + rng.next_f64() * spawn_area + half_width;
        let y = y.floor() + rng.next_f64() * spawn_area;
        let z = z.floor() + rng.next_f64() * spawn_area + half_width;

        while !stack.is_empty() {
            let item = stack.split((rng.next_bounded_i32(21) + 10) as u8);
            let velocity = Vector3::new(
                rng.next_triangular(XZ_MODE, TRIANGULAR_DEVIATION),
                rng.next_triangular(Y_MODE, TRIANGULAR_DEVIATION),
                rng.next_triangular(XZ_MODE, TRIANGULAR_DEVIATION),
            );

            let entity = Entity::new(
                Uuid::new_v4(),
                self.clone(),
                Vector3::new(x, y, z),
                &EntityType::ITEM,
                false,
            );
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
    pub fn is_valid(dest: Vector3<f64>) -> bool {
        Self::is_valid_horizontally(dest) && Self::is_valid_vertically(dest.y)
    }
    #[must_use]
    pub fn is_valid_horizontally(dest: Vector3<f64>) -> bool {
        (-30_000_000.0..=30_000_000.0).contains(&dest.x)
            && (-30_000_000.0..=30_000_000.0).contains(&dest.z)
    }
    #[must_use]
    pub fn is_valid_vertically(y: f64) -> bool {
        (-20_000_000.0..=20_000_000.0).contains(&y)
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
            return fluid;
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

    pub async fn get_fluid_and_fluid_state(&self, position: &BlockPos) -> (Fluid, FluidState) {
        let id = self.get_block_state_id(position).await;

        let Some(fluid) = Fluid::from_state_id(id) else {
            let block = Block::from_state_id(id);
            if let Some(properties) = block.properties(id) {
                for (name, value) in properties.to_props() {
                    if name == "waterlogged" {
                        if value == "true" {
                            let fluid = Fluid::FLOWING_WATER;
                            let state = fluid.states[0].clone();
                            return (fluid, state);
                        }

                        break;
                    }
                }
            }

            let fluid = Fluid::EMPTY;
            let state = fluid.states[0].clone();

            return (fluid, state);
        };

        //let state = fluid.get_state(id);
        let state = fluid.states[0].clone();

        (fluid.clone(), state)
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
                self.block_registry.get_pumpkin_block(neighbor_block)
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
                self.block_registry.get_pumpkin_fluid(neighbor_fluid)
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

        if let Some(neighbor_pumpkin_block) = self.block_registry.get_pumpkin_block(neighbor_block)
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
            let flags = flags & !BlockFlags::SKIP_DROPS;
            if BlockState::from_id(new_state_id).is_air() {
                self.break_block(block_pos, None, flags).await;
            } else {
                self.set_block_state(block_pos, new_state_id, flags).await;
            }
        }
    }

    pub async fn get_block_entity(&self, block_pos: &BlockPos) -> Option<Arc<dyn BlockEntity>> {
        let chunk = self
            .level
            .get_chunk(block_pos.chunk_and_chunk_relative_position().0)
            .await;
        let chunk: tokio::sync::RwLockReadGuard<ChunkData> = chunk.read().await;

        chunk.block_entities.get(block_pos).cloned()
    }

    pub async fn add_block_entity(&self, block_entity: Arc<dyn BlockEntity>) {
        let block_pos = block_entity.get_position();
        let chunk = self
            .level
            .get_chunk(block_pos.chunk_and_chunk_relative_position().0)
            .await;
        let mut chunk: tokio::sync::RwLockWriteGuard<ChunkData> = chunk.write().await;
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

        chunk.block_entities.insert(block_pos, block_entity);
        chunk.mark_dirty(true);
    }

    pub async fn remove_block_entity(&self, block_pos: &BlockPos) {
        let chunk = self
            .level
            .get_chunk(block_pos.chunk_and_chunk_relative_position().0)
            .await;
        let mut chunk: tokio::sync::RwLockWriteGuard<ChunkData> = chunk.write().await;
        chunk.block_entities.remove(block_pos);
        chunk.mark_dirty(true);
    }

    pub async fn update_block_entity(&self, block_entity: &Arc<dyn BlockEntity>) {
        let block_pos = block_entity.get_position();
        let chunk = self
            .level
            .get_chunk(block_pos.chunk_and_chunk_relative_position().0)
            .await;
        let mut chunk: tokio::sync::RwLockWriteGuard<ChunkData> = chunk.write().await;
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
        self: &Arc<Self>,
        block_pos: &BlockPos,
        from: Vector3<f64>,
        to: Vector3<f64>,
    ) -> (bool, Option<BlockDirection>) {
        let state = self.get_block_state(block_pos).await;

        let Some(bounding_boxes) = state.get_block_outline_shapes() else {
            return (false, None);
        };

        if bounding_boxes.is_empty() {
            return (true, None);
        }

        for shape in &bounding_boxes {
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

    fn get_block_and_state<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = (&'static Block, &'static BlockState)> + Send + 'a>> {
        Box::pin(async move { self.get_block_and_state(position).await })
    }
}
