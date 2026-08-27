use crate::block::entities::{BlockEntity, block_entity_from_nbt};
use dashmap::DashMap;
use pumpkin_data::chunk::Biome;
use pumpkin_data::item::{BedrockItem, BedrockItemVersion};
use pumpkin_protocol::bedrock::client::item_registry::{CItemRegistry, ItemData};
use pumpkin_protocol::bedrock::client::level_event::{CLevelEvent, LevelEvent};
use pumpkin_protocol::bedrock::client::{CBiomeDefinitionList, block_actor_data::CBlockActorData};
use pumpkin_protocol::bedrock::network_item::{NetworkItemDescriptor, NetworkItemStackDescriptor};
use pumpkin_protocol::codec::data_component::data_to_proto_sound;
use pumpkin_world::generation::proto_chunk::GenerationCache;
use rayon::prelude::*;
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
pub mod map;
pub mod portal;
pub mod raid;
pub mod time;
pub mod villager_poi;

use crate::block::RandomTickArgs;
use crate::world::chunker::get_view_distance;
use crate::world::chunker::is_within_view_distance;
use crate::{block::BlockEvent, entity::item::ItemEntity};
use crate::{
    block::{
        registry::BlockRegistry,
        {OnNeighborUpdateArgs, OnScheduledTickArgs},
    },
    command::client_suggestions,
    entity::{Entity, EntityBase, RemovalReason, player::Player, r#type::from_type},
    error::PumpkinError,
    net::{ClientPlatform, bedrock::BedrockClient, java::JavaClient},
    plugin::player::{
        player_change_world::PlayerChangeWorldEvent, player_join::PlayerJoinEvent,
        player_leave::PlayerLeaveEvent, player_respawn::PlayerRespawnEvent,
    },
    server::Server,
};
use arc_swap::ArcSwap;
use border::Worldborder;
use bytes::BufMut;
pub use explosion::{
    BlockInteraction, DefaultExplosionDamageCalculator, Explosion, ExplosionDamageCalculator,
    ExplosionInteraction, SimpleExplosionDamageCalculator,
};
use pumpkin_config::BasicConfiguration;
use pumpkin_data::block_properties::{blocks_movement, is_air};
use pumpkin_data::block_rotation::{Mirror, Rotation};
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::MobCategory;
use pumpkin_data::fluid::FluidState;
use pumpkin_data::game_rules::{GameRule, GameRuleValue};
use pumpkin_data::{
    Block, BlockStateId,
    entity::{EntityStatus, EntityType},
    fluid::Fluid,
    item_stack::ItemStack,
    packet::CURRENT_MC_VERSION,
    particle::Particle,
    sound::{Sound, SoundCategory},
    sound_id_remap::remap_sound_id_for_version,
    world::{RAW, WorldEvent},
};
use pumpkin_data::{
    BlockDirection, BlockState, HorizontalFacingExt,
    block_properties::{BlockProperties, ChestLikeProperties, ChestType},
    tag::Taggable,
    translation,
};
use pumpkin_inventory::crafting::recipe_provider::RecipeProvider;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::client::set_actor_data::{CSetActorData, PropertySyncData};
use pumpkin_protocol::bedrock::client::start_game::{CStartGame, ServerTelemetryData};
use pumpkin_protocol::java::client::play::{
    CBlockUpdate, CDisguisedChatMessage, CExplosion, CRespawn, CSetBlockDestroyStage, CWorldEvent,
    PlayerSpawnData,
};
use pumpkin_protocol::java::client::play::{
    CPlayerSpawnPosition, CRecipeBookAdd, CRecipeBookSettings, CSystemChatMessage,
};
use pumpkin_protocol::java::client::play::{CSetEntityMetadata, Metadata};
use pumpkin_protocol::{
    BClientPacket, ClientPacket, IdOr, SoundEvent,
    bedrock::{
        client::{
            add_player::CAddPlayer,
            block_event::CBlockEvent as CBedrockBlockEvent,
            common::BuildPlatform,
            creative_content::{
                CCreativeContent, CreativeCategory, CreativeGroupInfoPayload,
                CreativeItemEntryPayload,
            },
            level_sound_event::CLevelSoundEvent,
            player_list::{CPlayerList, PlayerListEntry, Skin},
            remove_actor::CRemoveActor,
            start_game::{Experiments, GamePublishSetting, LevelSettings},
            update_attributes::{AttributeData, CUpdateAttributes},
        },
        server::{
            actor_event::{ActorEventID, SActorEvent},
            text::SText,
        },
    },
    codec::{var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong},
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
    java::client::play::{CBlockEvent, CRemoveMobEffect, CSetEquipment, CUpdateMobEffect},
};
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_util::{
    Difficulty,
    math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3},
};
use pumpkin_util::{
    math::{get_section_cord, position::chunk_section_from_pos, vector2::Vector2},
    random::{RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use pumpkin_world::inventory::Clearable;
use pumpkin_world::world::{GetBlockError, WorldPortalExt};
use pumpkin_world::{
    CURRENT_BEDROCK_MC_VERSION, biome,
    chunk::{io::Dirtiable, palette::bedrock_water_state},
    inventory::Inventory,
};
use pumpkin_world::{chunk::ChunkData, world::BlockAccessor};
use pumpkin_world::{level::Level, tick::TickPriority};
pub use pumpkin_world::{world::BlockFlags, world_info::LevelData};
use rand::seq::SliceRandom;
use rand::{RngExt, rng};
use scoreboard::Scoreboard;
use time::LevelTime;

pub mod block_placer;
pub mod border;
pub mod bossbar;
pub mod custom_bossbar;
pub mod dragon_fight;
pub mod end_podium;
pub mod natural_spawner;
pub mod scoreboard;
pub mod weather;

use crate::world::natural_spawner::{SpawnState, spawn_for_chunk};
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::effect::StatusEffect;
use pumpkin_world::chunk::ChunkHeightmapType::{self, MotionBlocking};
use uuid::Uuid;
use weather::Weather;

const MAX_LIGHT_LEVEL: u8 = 15;

fn bedrock_chest_block_actor(state_id: BlockStateId, position: BlockPos) -> Option<NbtCompound> {
    let (block, _) = BlockState::from_id_with_block(state_id);
    if !block.has_tag(&pumpkin_data::tag::Block::C_CHESTS_WOODEN)
        && !block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_COPPER_CHESTS)
    {
        return None;
    }

    // Block actor tags describe the chest itself. Container contents are synchronized
    // through inventory packets and must not be exposed in chunk data.
    let mut nbt = NbtCompound::new();
    nbt.put_string("id", "Chest".to_string());
    nbt.put_int("x", position.0.x);
    nbt.put_int("y", position.0.y);
    nbt.put_int("z", position.0.z);
    nbt.put_bool("isMovable", true);

    let properties = ChestLikeProperties::from_state_id(state_id, block);
    if properties.r#type != ChestType::Single {
        let direction = if properties.r#type == ChestType::Left {
            properties.facing.rotate_clockwise()
        } else {
            properties.facing.rotate_counter_clockwise()
        };
        let pair = position.offset(direction.to_offset());
        nbt.put_int("pairx", pair.0.x);
        nbt.put_int("pairz", pair.0.z);
        if properties.r#type == ChestType::Right {
            nbt.put_bool("pairlead", true);
        }
    }

    Some(nbt)
}

use rustc_hash::{FxHashMap, FxHashSet};

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
    pub scoreboard: std::sync::Mutex<Scoreboard>,
    /// The world's worldborder, defining the playable area and controlling its expansion or contraction.
    pub worldborder: std::sync::Mutex<Worldborder>,
    /// The world's time, including counting ticks for weather, time cycles, and statistics.
    pub level_time: std::sync::Mutex<LevelTime>,
    /// The type of dimension the world is in.
    pub dimension: Dimension,
    pub sea_level: i32,
    pub min_y: i32,
    /// The world's weather, including rain and thunder levels.
    pub weather: std::sync::Mutex<Weather>,
    /// Block Behaviour
    pub block_registry: Arc<BlockRegistry>,
    pub server: Weak<Server>,
    synced_block_event_queue: std::sync::Mutex<Vec<BlockEvent>>,
    /// A map of unsent block changes, keyed by block position.
    unsent_block_changes: std::sync::Mutex<HashMap<BlockPos, BlockStateId>>,
    /// Persisted vanilla POI storage for portal and villager lookups.
    pub portal_poi: std::sync::Mutex<portal::PortalPoiStorage>,
    /// Villager job sites and their current owners.
    pub villager_poi: std::sync::Mutex<villager_poi::VillagerPoiStorage>,
    /// Active raids in this world.
    pub raids: std::sync::Mutex<raid::Raids>,
    /// End Dragon fight manager (only present in `THE_END` dimension).
    pub dragon_fight: Option<std::sync::Mutex<dragon_fight::DragonFight>>,
    pub spawn_state: ArcSwap<SpawnState>,
    pub active_chunks: ArcSwap<FxHashSet<Vector2<i32>>>,
    pub forced_chunks: std::sync::Mutex<FxHashSet<Vector2<i32>>>,
    /// Block entities indexed by chunk, so ticking only visits the currently
    /// active chunks instead of scanning every loaded block entity each tick.
    pub block_entities: DashMap<Vector2<i32>, FxHashMap<BlockPos, Arc<dyn BlockEntity>>>,
    /// Persistent custom data for the world (matching Bukkit's `PersistentDataHolder`)
    pub custom_data: std::sync::Mutex<NbtCompound>,
    /// Persistent custom data for block entities at specific positions
    pub custom_block_entity_data: DashMap<BlockPos, NbtCompound>,
}

#[derive(Clone, Copy)]
pub(crate) enum BlockBreakingProgress {
    Start { stage: i32, speed: f32 },
    Update { stage: i32, speed: Option<f32> },
    Stop,
}

impl PartialEq for World {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for World {}

impl World {
    pub async fn get_block_state_id_async(&self, position: &BlockPos) -> BlockStateId {
        if !self.is_in_build_limit(*position) {
            return Block::AIR.default_state.id;
        }

        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        self.level
            .get_or_fetch_chunk(chunk_coordinate, |chunk| {
                chunk
                    .section
                    .get_block_absolute_y(relative.x as usize, relative.y, relative.z as usize)
                    .unwrap_or(Block::AIR.default_state.id)
            })
            .await
    }

    pub async fn get_block_state_async(&self, position: &BlockPos) -> &'static BlockState {
        let id = self.get_block_state_id_async(position).await;
        BlockState::from_id(id)
    }

    pub async fn get_heightmap_height_async(
        &self,
        height_map: ChunkHeightmapType,
        x: i32,
        z: i32,
    ) -> i32 {
        let chunk_pos = Vector2::new(x >> 4, z >> 4);
        self.level
            .get_or_fetch_chunk(chunk_pos, |chunk| {
                chunk
                    .heightmap
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .get(height_map, x, z, self.min_y)
            })
            .await
    }

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
        let portal_poi = portal::PortalPoiStorage::new(level.level_folder.poi_folder.clone());
        let dragon_fight = (dimension.minecraft_name == Dimension::THE_END.minecraft_name)
            .then(|| std::sync::Mutex::new(dragon_fight::DragonFight::new()));

        let custom_data_path = level
            .level_folder
            .root_folder
            .join("pumpkin_custom_data.nbt");
        let custom_data = if custom_data_path.exists()
            && let Ok(bytes) = std::fs::read(&custom_data_path)
            && let Ok(nbt) = pumpkin_nbt::Nbt::read_unnamed(
                &mut pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut std::io::Cursor::new(
                    bytes,
                )),
            ) {
            nbt.root_tag
        } else {
            NbtCompound::new()
        };

        Self {
            uuid: Uuid::new_v4(),
            level,
            level_info,
            players: ArcSwap::new(Arc::new(Vec::new())),
            entities: ArcSwap::new(Arc::new(Vec::new())),
            scoreboard: std::sync::Mutex::new(Scoreboard::default()),
            worldborder: std::sync::Mutex::new(Worldborder::new(
                0.0,
                0.0,
                5.999_996_8E7,
                0,
                5,
                300,
            )),
            level_time: std::sync::Mutex::new(LevelTime::new()),
            dimension,
            weather: std::sync::Mutex::new(Weather::new()),
            block_registry,
            sea_level: generation_settings.sea_level,
            min_y: i32::from(generation_settings.shape.min_y),
            synced_block_event_queue: std::sync::Mutex::new(Vec::new()),
            unsent_block_changes: std::sync::Mutex::new(HashMap::new()),
            portal_poi: std::sync::Mutex::new(portal_poi),
            villager_poi: std::sync::Mutex::new(villager_poi::VillagerPoiStorage::default()),
            raids: std::sync::Mutex::new(raid::Raids::default()),
            dragon_fight,
            spawn_state: ArcSwap::new(Arc::new(SpawnState::empty())),
            active_chunks: ArcSwap::new(Arc::new(FxHashSet::default())),
            forced_chunks: std::sync::Mutex::new(FxHashSet::default()),
            server,
            block_entities: DashMap::new(),
            custom_data: std::sync::Mutex::new(custom_data),
            custom_block_entity_data: DashMap::new(),
        }
    }

    pub fn update_active_chunks(self: &Arc<Self>) {
        let mut active_chunks = FxHashSet::default();
        let sim_dist = self.server.upgrade().map_or(10, |s| {
            s.advanced_config.networking.java.simulation_distance.get()
        }) as i32;
        for player in self.players.load().iter() {
            let center = player.get_entity().chunk_pos.load();
            for dx in -sim_dist..=sim_dist {
                for dy in -sim_dist..=sim_dist {
                    active_chunks.insert(center.add_raw(dx, dy));
                }
            }
        }
        if let Ok(forced) = self.forced_chunks.lock() {
            active_chunks.extend(forced.iter().copied());
        }

        let mut spawnable_chunks = 0;
        for pos in &active_chunks {
            if self.level.is_chunk_loaded(pos) {
                spawnable_chunks += 1;
                self.migrate_pending_block_entities(*pos);
            }
        }

        self.active_chunks.store(Arc::new(active_chunks));

        self.spawn_state.store(Arc::new(SpawnState::new(
            spawnable_chunks,
            &self.entities,
            self,
        )));
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

    /// Returns the configured shared world spawn block position and rotation.
    #[must_use]
    pub fn get_spawn_location(&self) -> (BlockPos, f32, f32) {
        let level_info = self.level_info.load();
        (
            BlockPos::new(level_info.spawn_x, level_info.spawn_y, level_info.spawn_z),
            level_info.spawn_yaw,
            level_info.spawn_pitch,
        )
    }

    pub async fn shutdown(&self) {
        for entity in self.entities.load().iter() {
            self.save_entity(entity).await;
        }

        let chunks: Vec<Vector2<i32>> = self
            .block_entities
            .iter()
            .map(|chunk_block_entities| *chunk_block_entities.key())
            .collect();
        for chunk_pos in chunks {
            self.save_block_entities(chunk_pos);
        }

        // Save portal POI to disk
        let save_result = self
            .portal_poi
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .save_all();
        if let Err(e) = save_result {
            error!("Failed to save portal POI: {e}");
        }

        self.level.shutdown().await;
    }

    /// Serializes a live entity into its current chunk's entity data. The live
    /// entity list is the source of truth while a chunk is loaded (its saved NBT
    /// is consumed on load), so this simply appends the entity to the chunk it is
    /// currently in; the chunk is rewritten from scratch every unload cycle, so
    /// there is nothing stale to deduplicate.
    async fn save_entity(&self, entity: &Arc<dyn EntityBase>) {
        let base_entity = entity.get_entity();
        if base_entity.is_removed() {
            return;
        }
        let current_chunk = base_entity.block_pos.load().chunk_position();
        let mut nbt = NbtCompound::new();
        entity.write_nbt(&mut nbt);
        let chunk = self.level.get_entity_chunk(current_chunk).await;
        chunk
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(nbt);
        chunk.mark_dirty(true);
    }

    /// Serializes the live block entities of a chunk back into that chunk's block
    /// entity data. The live map is the source of truth while a chunk is loaded -
    /// `get_block_entity` takes the saved NBT out of the chunk when it wakes an
    /// entity up - so this has to run before the chunk is dropped, or everything
    /// the entity did since it was loaded is lost.
    fn save_block_entities(&self, chunk_pos: Vector2<i32>) {
        let Some(block_entities) = self
            .block_entities
            .get(&chunk_pos)
            .map(|chunk_block_entities| chunk_block_entities.values().cloned().collect::<Vec<_>>())
        else {
            return;
        };

        for block_entity in block_entities {
            let mut nbt = NbtCompound::new();
            block_entity.write_internal(&mut nbt);
            if let Some(custom_data) = self
                .custom_block_entity_data
                .get(&block_entity.get_position())
                && !custom_data.is_empty()
            {
                nbt.put_compound("PumpkinCustomData", custom_data.clone());
            }
            self.add_block_entity_nbt(block_entity.get_position(), &nbt);
        }
    }

    /// Sends an entity status update to all players tracking the specified entity.
    pub fn send_entity_status(
        &self,
        entity: &Entity,
        java_status: EntityStatus,
        bedrock_status: Option<ActorEventID>,
    ) {
        let chunk_pos = entity.chunk_pos.load();
        let je_packet = CEntityStatus::new(entity.entity_id, java_status as i8);
        if let Some(be_event) = bedrock_status {
            let be_packet = SActorEvent {
                target_runtime_id: VarULong(entity.entity_id as u64),
                event_id: be_event,
                data: VarInt(0),
                fire_at_position: None,
            };
            self.broadcast_to_chunk_editioned(chunk_pos, &je_packet, &be_packet);
        } else {
            self.broadcast_to_chunk(chunk_pos, &je_packet);
        }
    }

    pub fn send_remove_mob_effect(&self, entity: &Entity, effect_type: &'static StatusEffect) {
        let chunk_pos = entity.chunk_pos.load();
        let je_packet =
            CRemoveMobEffect::new(entity.entity_id.into(), VarInt(i32::from(effect_type.id)));

        let be_packet = pumpkin_protocol::bedrock::client::CMobEffect {
            target_runtime_id: VarULong(entity.entity_id as u64),
            event_id: pumpkin_protocol::bedrock::client::CMobEffect::EVENT_REMOVE,
            effect_id: VarInt(effect_type.to_bedrock_id()),
            effect_amplifier: VarInt(0),
            show_particles: false,
            effect_duration_ticks: VarInt(0),
            tick: VarULong(0),
            ambient: false,
        };
        self.broadcast_to_chunk_editioned(chunk_pos, &je_packet, &be_packet);
    }

    pub fn send_add_mob_effect(&self, entity: &Entity, effect: &pumpkin_data::potion::Effect) {
        let chunk_pos = entity.chunk_pos.load();
        let mut flags: i8 = 0;
        if effect.ambient {
            flags |= 0x01;
        }
        if effect.show_particles {
            flags |= 0x02;
        }
        if effect.show_icon {
            flags |= 0x04;
        }

        let je_packet = CUpdateMobEffect::new(
            VarInt(entity.entity_id),
            VarInt(i32::from(effect.effect_type.id)),
            VarInt(i32::from(effect.amplifier)),
            VarInt(effect.duration),
            flags,
        );

        let be_packet = pumpkin_protocol::bedrock::client::CMobEffect {
            target_runtime_id: VarULong(entity.entity_id as u64),
            event_id: pumpkin_protocol::bedrock::client::CMobEffect::EVENT_ADD,
            effect_id: VarInt(effect.effect_type.to_bedrock_id()),
            effect_amplifier: VarInt(i32::from(effect.amplifier)),
            show_particles: effect.show_particles,
            effect_duration_ticks: VarInt(effect.duration),
            tick: VarULong(0),
            ambient: effect.ambient,
        };

        self.broadcast_to_chunk_editioned(chunk_pos, &je_packet, &be_packet);
    }

    pub fn set_difficulty(&self, difficulty: Difficulty) {
        let current_info = self.level_info.load();
        let mut new_info = (**current_info).clone();
        new_info.difficulty = difficulty;
        self.level_info.store(Arc::new(new_info));
    }

    pub fn get_game_rule(&self, rule: &GameRule) -> GameRuleValue<i64, bool> {
        let level_info = self.level_info.load();
        match level_info.game_rules.get(rule) {
            GameRuleValue::Int(v) => GameRuleValue::Int(*v),
            GameRuleValue::Bool(v) => GameRuleValue::Bool(*v),
        }
    }

    pub fn set_game_rule(&self, rule: &GameRule, value: GameRuleValue<i64, bool>) {
        let current_info = self.level_info.load();
        let mut new_info = (**current_info).clone();
        match (new_info.game_rules.get_mut(rule), value) {
            (GameRuleValue::Int(target), GameRuleValue::Int(val)) => {
                *target = val;
            }
            (GameRuleValue::Bool(target), GameRuleValue::Bool(val)) => {
                *target = val;
            }
            _ => {}
        }
        self.level_info.store(Arc::new(new_info));
    }

    pub fn add_synced_block_event(&self, pos: BlockPos, r#type: u8, data: u8) {
        let mut queue = self
            .synced_block_event_queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        queue.push(BlockEvent { pos, r#type, data });
    }

    pub fn flush_synced_block_events(self: &Arc<Self>) {
        // THIS IS IMPORTANT
        // it prevents deadlocks and also removes the need to wait for a lock when adding a new synced block
        let events = {
            let mut queue = self
                .synced_block_event_queue
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *queue)
        };

        for event in events {
            let block = self.get_block(&event.pos);
            if !self.block_registry.on_synced_block_event(
                block,
                self,
                &event.pos,
                event.r#type,
                event.data,
            ) {
                continue;
            }
            let chunk_pos = event.pos.chunk_position();
            self.broadcast_to_chunk_editioned(
                chunk_pos,
                &CBlockEvent::new(
                    event.pos,
                    event.r#type,
                    event.data,
                    VarInt(block.id.as_u16() as i32),
                ),
                &CBedrockBlockEvent {
                    block_position: event.pos,
                    event_type: event.r#type.into(),
                    event_value: event.data.into(),
                },
            );
        }
    }

    pub(crate) fn collect_java_recipients_by_version<'a>(
        players: impl Iterator<Item = &'a Arc<Player>>,
    ) -> BTreeMap<JavaMinecraftVersion, Vec<&'a JavaClient>> {
        let mut recipients_by_version: BTreeMap<JavaMinecraftVersion, Vec<&'a JavaClient>> =
            BTreeMap::new();
        for player in players {
            if let ClientPlatform::Java(java_client) = player.client.as_ref() {
                recipients_by_version
                    .entry(java_client.version.load())
                    .or_default()
                    .push(java_client);
            }
        }
        recipients_by_version
    }

    pub fn broadcast_java_clients<'a, P: ClientPacket>(
        packet: &P,
        recipients: impl Iterator<Item = &'a JavaClient>,
    ) {
        let mut recipients_by_version: BTreeMap<JavaMinecraftVersion, Vec<&JavaClient>> =
            BTreeMap::new();
        for client in recipients {
            recipients_by_version
                .entry(client.version.load())
                .or_default()
                .push(client);
        }
        Self::broadcast_java_grouped(packet, recipients_by_version);
    }

    fn broadcast_java_grouped<P: ClientPacket>(
        packet: &P,
        recipients_by_version: BTreeMap<JavaMinecraftVersion, Vec<&JavaClient>>,
    ) {
        for (version, recipients) in recipients_by_version {
            let packet_data = match JavaClient::serialize_packet_for_version(packet, version) {
                Ok(packet_data) => packet_data,
                Err(pumpkin_protocol::ser::WritingError::UnsupportedVersion(_)) => {
                    continue;
                }
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
                recipient.try_enqueue_packet(packet_data.clone());
            }
        }
    }

    fn broadcast_bedrock_grouped<'a, P: BClientPacket>(
        packet: &P,
        recipients: impl Iterator<Item = &'a Arc<BedrockClient>>,
    ) {
        for recipient in recipients {
            match recipient.serialize_packet(packet) {
                Ok(packet_data) => recipient.try_enqueue_packet(packet_data),
                Err(err) => {
                    error!(
                        "Failed to serialize bedrock packet {}: {}",
                        std::any::type_name::<P>(),
                        err
                    );
                }
            }
        }
    }

    /// Broadcasts a packet to all connected players within the world.
    /// Please avoid this as we want to replace it with `broadcast_editioned`
    ///
    /// Sends the specified packet to every player currently logged in to the world.
    ///
    /// **Note:** This function acquires a lock on the `current_players` map, ensuring thread safety.
    pub fn broadcast_packet_all<P: ClientPacket>(&self, packet: &P) {
        let players = self.players.load();
        let recipients_by_version = Self::collect_java_recipients_by_version(players.iter());
        Self::broadcast_java_grouped(packet, recipients_by_version);
    }

    pub fn broadcast_system_message(&self, message: &TextComponent, overlay: bool) {
        let je_packet = CSystemChatMessage::new(message, overlay);
        let be_packet = Self::component_to_bedrock_text(message);
        self.broadcast_editioned(&je_packet, &be_packet);
    }

    fn component_to_bedrock_text(message: &TextComponent) -> SText<'static> {
        match &*message.0.content {
            pumpkin_util::text::TextContent::Translate {
                translate,
                bedrock_translate,
                with,
            } => {
                let key = bedrock_translate.as_deref().unwrap_or(translate.as_ref());
                let parameters = with
                    .iter()
                    .map(pumpkin_util::text::TextComponentBase::to_bedrock_string)
                    .collect();
                SText::translation(key.to_string(), parameters)
            }
            _ => SText::system_message(
                message
                    .0
                    .to_bedrock_legacy(pumpkin_util::translation::Locale::EnUs),
            ),
        }
    }

    pub fn broadcast_message(
        &self,
        message: &TextComponent,
        sender_name: &TextComponent,
        chat_type: u8,
        target_name: Option<&TextComponent>,
    ) {
        let be_packet = SText::new(message.clone().get_text(), sender_name.clone().get_text());
        let je_packet =
            CDisguisedChatMessage::new(message, (chat_type + 1).into(), sender_name, target_name);

        self.broadcast_editioned(&je_packet, &be_packet);
    }

    // This should replace broadcast_packet_all at some point
    pub fn broadcast_editioned<J: ClientPacket, B: BClientPacket>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        let players = self.players.load();
        let je_recipients_by_version = Self::collect_java_recipients_by_version(players.iter());

        Self::broadcast_java_grouped(je_packet, je_recipients_by_version);
        Self::broadcast_bedrock_grouped(
            be_packet,
            players.iter().filter_map(|p| match p.client.as_ref() {
                ClientPlatform::Bedrock(be) => Some(be),
                ClientPlatform::Java(_) => None,
            }),
        );
    }

    pub async fn broadcast_secure_player_chat(
        &self,
        sender: &Arc<Player>,
        chat_message: &SChatMessage<'_>,
        decorated_message: &TextComponent,
    ) {
        let messages_sent: i32 = sender
            .chat_session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .messages_sent;
        let sender_last_seen = {
            let cache = sender
                .signature_cache
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            cache.last_seen.clone()
        };

        for recipient in self.players.load().iter() {
            let messages_received: i32 = recipient
                .chat_session
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .messages_received;
            let packet = &CPlayerChatMessage::new(
                VarInt(messages_received),
                sender.gameprofile.id,
                VarInt(messages_sent),
                chat_message.signature.map(std::convert::Into::into),
                chat_message.message.into(),
                chat_message.timestamp,
                chat_message.salt,
                sender_last_seen.indexed_for(recipient),
                Some(decorated_message.clone()),
                FilterType::PassThrough,
                (RAW + 1).into(), // Custom registry chat_type with no sender name
                TextComponent::empty(), // Not needed since we're injecting the name in the message for custom formatting
                None,
            );
            let packet_data = recipient.client.java().map_or_else(
                || {
                    JavaClient::serialize_packet_for_version(
                        packet,
                        recipient.client.java_version(),
                    )
                },
                |j| j.serialize_packet(packet),
            );
            if let Ok(data) = packet_data {
                recipient.client.enqueue_packet(data).await;
            }

            if let Some(signature) = chat_message.signature {
                let mut cache = recipient
                    .signature_cache
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                cache.add_seen_signature(signature);
                cache.last_seen_validator.add_pending(signature);
                let tracked_count = cache.last_seen_validator.tracked_messages_count();
                drop(cache);

                if tracked_count > 4096 {
                    recipient.kick(
                        crate::net::DisconnectReason::Kicked,
                        &TextComponent::translate_cross(
                            pumpkin_data::translation::java::MULTIPLAYER_DISCONNECT_TOO_MANY_PENDING_CHATS,
                            pumpkin_data::translation::java::MULTIPLAYER_DISCONNECT_TOO_MANY_PENDING_CHATS,
                            [],
                        ),
                    );
                }
            }

            if recipient.gameprofile.id != sender.gameprofile.id {
                // Sender may update recipient on signatures recipient hasn't seen
                recipient
                    .signature_cache
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .cache_signatures(sender_last_seen.as_ref());
            }
            recipient
                .chat_session
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .messages_received += 1;
        }

        sender
            .chat_session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .messages_sent += 1;
    }

    pub fn broadcast_packet_except_editioned<J: ClientPacket, B: BClientPacket>(
        &self,
        except: &[uuid::Uuid],
        je_packet: &J,
        be_packet: &B,
    ) {
        let players = self.players.load();
        let mut java_recipients = Vec::new();
        let mut bedrock_recipients = Vec::new();

        for p in players.iter() {
            if except.contains(&p.gameprofile.id) {
                continue;
            }
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => bedrock_recipients.push(be_client),
            }
        }

        let recipients_by_version =
            Self::collect_java_recipients_by_version(java_recipients.into_iter());
        Self::broadcast_java_grouped(je_packet, recipients_by_version);
        Self::broadcast_bedrock_grouped(be_packet, bedrock_recipients.into_iter());
    }

    /// Broadcasts the skin layers of a player, encoding the metadata for each Java client's own
    /// protocol version since the tracked data index differs between versions.
    fn broadcast_skin_parts<B: BClientPacket>(
        &self,
        except: &[uuid::Uuid],
        entity_id: i32,
        skin_parts: u8,
        be_packet: &B,
    ) {
        let players = self.players.load();
        let mut java_recipients = Vec::new();
        let mut bedrock_recipients = Vec::new();

        for p in players.iter() {
            if except.contains(&p.gameprofile.id) {
                continue;
            }
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => bedrock_recipients.push(be_client),
            }
        }

        let recipients_by_version =
            Self::collect_java_recipients_by_version(java_recipients.into_iter());

        for (version, recipients) in recipients_by_version {
            if version < CURRENT_MC_VERSION {
                continue;
            }
            let mut buf = Vec::new();
            for meta in [
                Metadata::new(
                    pumpkin_data::tracked_data::player::PLAYER_MODE_CUSTOMISATION,
                    skin_parts,
                ),
                Metadata::new(
                    pumpkin_data::tracked_data::player::PLAYER_MODE_CUSTOMIZATION_ID,
                    skin_parts,
                ),
            ] {
                let _ = meta.write(&mut buf, &version);
            }
            buf.put_u8(255);
            let packet = CSetEntityMetadata::new(entity_id.into(), buf.into());
            if let Ok(packet_data) = JavaClient::serialize_packet_for_version(&packet, version) {
                for recipient in recipients {
                    recipient.try_enqueue_packet(packet_data.clone());
                }
            }
        }

        Self::broadcast_bedrock_grouped(be_packet, bedrock_recipients.into_iter());
    }

    /// Broadcasts a packet to all connected players within the world, excluding the specified players.
    ///
    /// Sends the specified packet to every player currently logged in to the world, excluding the players listed in the `except` parameter.
    ///
    /// **Note:** This function acquires a lock on the `current_players` map, ensuring thread safety.
    pub fn broadcast_packet_except<P: ClientPacket>(&self, except: &[uuid::Uuid], packet: &P) {
        let players = self.players.load();
        let recipients_by_version = Self::collect_java_recipients_by_version(
            players
                .iter()
                .filter(|candidate| !except.contains(&candidate.gameprofile.id)),
        );
        Self::broadcast_java_grouped(packet, recipients_by_version);
    }

    pub fn spawn_particle(
        &self,
        position: Vector3<f64>,
        offset: Vector3<f32>,
        max_speed: f32,
        particle_count: i32,
        particle: Particle,
    ) {
        for player in self.players.load().iter() {
            player.spawn_particle(position, offset, max_speed, particle_count, particle);
        }
    }

    pub fn play_sound(&self, sound: Sound, category: SoundCategory, position: &Vector3<f64>) {
        self.play_sound_raw(sound as u16, category, position, 1.0, 1.0);
    }

    pub fn play_sound_event(
        &self,
        sound: &pumpkin_data::data_component_impl::IdOr<
            pumpkin_data::data_component_impl::SoundEvent,
        >,
        category: SoundCategory,
        position: &Vector3<f64>,
    ) {
        let seed = rng().random::<f64>();
        let packet = CSoundEffect::new(
            data_to_proto_sound(sound),
            category,
            position,
            1.0,
            1.0,
            seed,
        );
        self.broadcast_packet_all(&packet);
    }

    pub fn play_sound_fine(
        &self,
        sound: Sound,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
    ) {
        self.play_sound_raw(sound as u16, category, position, volume, pitch);
    }

    /// Plays a Bedrock level sound for players close enough to hear it.
    pub fn play_bedrock_level_sound(
        &self,
        sound_id: &str,
        position: &Vector3<f64>,
        extra_data: i32,
    ) {
        let packet = CLevelSoundEvent {
            sound_event: sound_id.to_string(),
            position: Vector3::new(position.x as f32, position.y as f32, position.z as f32),
            data: VarInt(extra_data),
            actor_identifier: String::new(),
            is_baby: false,
            is_global: false,
            actor_unique_id: 0,
            fire_at_position: None,
        };
        let chunk_pos = BlockPos::floored_v(*position).chunk_position();

        for player in self.players.load().iter() {
            if is_within_view_distance(chunk_pos, player.get_entity().chunk_pos.load(), 1)
                && let ClientPlatform::Bedrock(client) = player.client.as_ref()
                && let Ok(data) = client.serialize_packet(&packet)
            {
                client.try_enqueue_packet(data);
            }
        }
    }

    pub fn play_sound_expect(
        &self,
        player: &Player,
        sound: Sound,
        category: SoundCategory,
        position: &Vector3<f64>,
    ) {
        self.play_sound_raw_expect(player, sound as u16, category, position, 1.0, 1.0);
    }

    pub fn play_sound_raw(
        &self,
        sound_id: u16,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
    ) {
        let seed = rand::rng().random::<f64>();
        let packet = CSoundEffect::new(IdOr::Id(sound_id), category, position, volume, pitch, seed);

        // Calculate the number of chunks the sound can be heard from based on its volume.
        let audible_chunks = f64::from(volume.max(1.0)).ceil() as i32;
        let chunk_pos = BlockPos::floored_v(*position).chunk_position();

        let players = self.players.load();
        let recipients = players.iter().filter(|p| {
            let center = p.get_entity().chunk_pos.load();
            // If the sound reaches their chunk, send it!
            is_within_view_distance(chunk_pos, center, audible_chunks)
        });

        let recipients_by_version = Self::collect_java_recipients_by_version(recipients);
        Self::broadcast_java_grouped(&packet, recipients_by_version);
    }

    pub fn play_sound_raw_expect(
        &self,
        player: &Player,
        sound_id: u16,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
    ) {
        let seed = rand::rng().random::<f64>();
        let packet = CSoundEffect::new(IdOr::Id(sound_id), category, position, volume, pitch, seed);

        let audible_chunks = f64::from(volume.max(1.0)).ceil() as i32;
        let chunk_pos = BlockPos::floored_v(*position).chunk_position();

        let players = self.players.load();
        let recipients = players.iter().filter(|p| {
            // Skip the expected player
            if p.gameprofile.id == player.gameprofile.id {
                return false;
            }

            let center = p.get_entity().chunk_pos.load();
            is_within_view_distance(chunk_pos, center, audible_chunks)
        });

        let recipients_by_version = Self::collect_java_recipients_by_version(recipients);
        Self::broadcast_java_grouped(&packet, recipients_by_version);
    }

    pub fn play_block_sound(&self, sound: Sound, category: SoundCategory, position: BlockPos) {
        let new_vec = Vector3::new(
            f64::from(position.0.x) + 0.5,
            f64::from(position.0.y) + 0.5,
            f64::from(position.0.z) + 0.5,
        );
        self.play_sound(sound, category, &new_vec);
    }

    pub fn play_block_sound_expect(
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
        self.play_sound_expect(player, sound, category, &new_vec);
    }

    #[expect(clippy::too_many_lines)]
    pub fn tick(self: &Arc<Self>, server: &Arc<Server>) {
        const ENTITY_TICK_BATCH_SIZE: usize = 16;

        let start = std::time::Instant::now();

        self.flush_block_updates();
        self.flush_synced_block_events();
        self.update_active_chunks();
        self.tick_environment();
        let mut raids = {
            let mut guard = self
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *guard)
        };
        raids.tick(self);
        {
            let mut guard = self
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (id, raid) in guard.raid_map.drain() {
                raids.raid_map.insert(id, raid);
            }
            raids.next_id = raids.next_id.max(guard.next_id);
            *guard = raids;
        };

        let t_chunks = std::time::Instant::now();
        self.tick_chunks(server);
        let chunk_elapsed = t_chunks.elapsed();

        let handle = server.runtime.clone();

        let players = self.players.load();
        let player_count = players.len();
        let players_cache = Arc::new(
            players
                .par_iter()
                .map(|player| {
                    let entity = player.get_entity();
                    let pos = entity.pos.load();
                    let bb = entity.bounding_box.load().expand(1.0, 0.5, 1.0);
                    let chunk_pos = Vector2::new(
                        get_section_cord(pos.x.floor() as i32),
                        get_section_cord(pos.z.floor() as i32),
                    );
                    (player.clone(), pos, bb, chunk_pos)
                })
                .collect::<Vec<_>>(),
        );

        let t_players = std::time::Instant::now();
        let player_handle = handle.clone();
        players.par_iter().for_each(|player| {
            let _guard = player_handle.enter();
            player.tick(server);
        });
        let player_elapsed = t_players.elapsed();

        let entities_to_tick = self.entities.load();
        let entity_count = entities_to_tick.len();
        let server_for_entities = (*server).clone();
        let active_chunks = self.active_chunks.load();
        let level_for_entities = self.level.clone();
        let entity_handle = handle.clone();

        let t_entities = std::time::Instant::now();
        let tickable: Vec<_> = entities_to_tick
            .par_iter()
            .filter_map(|entity| {
                let entity_pos = entity.get_entity().pos.load();
                let entity_chunk = Vector2::new(
                    get_section_cord(entity_pos.x.floor() as i32),
                    get_section_cord(entity_pos.z.floor() as i32),
                );
                if !active_chunks.contains(&entity_chunk) {
                    return None;
                }
                if !level_for_entities.is_chunk_loaded(&entity_chunk) {
                    return None;
                }
                Some((entity.clone(), entity_chunk))
            })
            .collect();

        tickable
            .par_chunks(ENTITY_TICK_BATCH_SIZE)
            .for_each(|batch| {
                let _guard = entity_handle.enter();
                let s_clone = server_for_entities.clone();
                let p_cache = players_cache.clone();

                for (entity, entity_chunk) in batch {
                    entity.get_entity().age.fetch_add(1, Relaxed);
                    entity.tick(entity, &s_clone);

                    let entity_inner = entity.get_entity();
                    let entity_pos = entity_inner.pos.load();
                    let entity_bb = entity_inner.bounding_box.load();

                    for (player, player_pos, player_bb, player_chunk) in p_cache.iter() {
                        if (player_chunk.x - entity_chunk.x).abs() <= 1
                            && (player_chunk.y - entity_chunk.y).abs() <= 1
                            && (player_pos.x - entity_pos.x).abs() < 5.0
                            && (player_pos.y - entity_pos.y).abs() < 5.0
                            && (player_pos.z - entity_pos.z).abs() < 5.0
                            && player_bb.intersects(&entity_bb)
                        {
                            entity.on_player_collision(player);
                            break;
                        }
                    }
                }
            });
        let entity_elapsed = t_entities.elapsed();

        let mut block_entities: Vec<Arc<dyn BlockEntity>> = Vec::new();
        for chunk_pos in active_chunks.iter() {
            if let Some(chunk_block_entities) = self.block_entities.get(chunk_pos) {
                block_entities.extend(chunk_block_entities.values().cloned());
            }
        }
        let block_entity_count = block_entities.len();

        let t_be = std::time::Instant::now();
        let world_for_be = self.clone();
        let be_handle = handle;
        block_entities.par_chunks(16).for_each(|batch| {
            let _guard = be_handle.enter();
            let w_clone = world_for_be.clone();
            for be in batch {
                be.tick(&w_clone);
            }
        });
        let block_entity_elapsed = t_be.elapsed();

        self.level
            .chunk_loading
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .send_change();

        if let Some(ref fight_mutex) = self.dragon_fight {
            dragon_fight::DragonFight::tick(fight_mutex, self);
        }

        let total_elapsed = start.elapsed();
        if total_elapsed.as_millis() > 50 {
            debug!(
                "Slow Tick [{}ms]: Chunks: {:?} | Players({}): {:?} | Entities({}): {:?} | Block Entities({}): {:?}",
                total_elapsed.as_millis(),
                chunk_elapsed,
                player_count,
                player_elapsed,
                entity_count,
                entity_elapsed,
                block_entity_count,
                block_entity_elapsed,
            );
        }
    }

    pub fn register_block_change(&self, position: BlockPos, block_state_id: BlockStateId) {
        self.unsent_block_changes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(position, block_state_id);
    }

    /// Queues block state changes for broadcast to nearby players.
    ///
    /// Call [`flush_block_updates`](Self::flush_block_updates) afterward to send the packets.
    pub fn queue_block_updates(&self, changes: &[(BlockPos, BlockStateId)]) {
        let mut guard = self
            .unsent_block_changes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (pos, state_id) in changes {
            guard.insert(*pos, *state_id);
        }
    }

    #[expect(clippy::too_many_lines)]
    pub fn flush_block_updates(&self) {
        let mut block_state_updates_by_chunk_section: HashMap<
            Vector3<i32>,
            Vec<(BlockPos, BlockStateId)>,
        > = HashMap::new();
        let changes = {
            let mut guard = self
                .unsent_block_changes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *guard)
        };
        for (position, block_state_id) in changes {
            let chunk_section = chunk_section_from_pos(&position);
            block_state_updates_by_chunk_section
                .entry(chunk_section)
                .or_default()
                .push((position, block_state_id));
        }

        // TODO: only send packet to players who have the chunks loaded
        // TODO: Send light updates to update the wire directly next to a broken block
        for (chunk_section, updates) in block_state_updates_by_chunk_section {
            if updates.is_empty() {
                continue;
            }
            let chunk_pos = Vector2::new(chunk_section.x, chunk_section.z);
            if updates.len() == 1 {
                let (block_pos, block_state_id) = updates[0];
                let be_block_id = BlockState::to_be_network_id(block_state_id);
                self.broadcast_to_chunk_editioned(
                    chunk_pos,
                    &CBlockUpdate::new(block_pos, i32::from(block_state_id.as_u16()).into()),
                    &pumpkin_protocol::bedrock::client::CUpdateBlock::new(
                        block_pos,
                        be_block_id as u32,
                    ),
                );
                if let Some(block_entity) = self.get_block_entity(&block_pos)
                    && let Some(nbt) = block_entity.chunk_data_nbt()
                {
                    let bytes = pumpkin_nbt::Nbt::from(nbt).write_unnamed();
                    self.broadcast_to_chunk(
                        chunk_pos,
                        &CBlockEntityData::new(
                            block_pos,
                            VarInt(block_entity.get_id() as i32),
                            bytes.as_ref().into(),
                        ),
                    );
                }
                if let Some(data) = self.bedrock_block_entity_data(block_state_id, block_pos) {
                    self.broadcast_to_chunk_bedrock(
                        chunk_pos,
                        &CBlockActorData::new(block_pos, data),
                    );
                }
            } else {
                let players = self.players.load();
                let mut java_recipients = Vec::new();

                let recipients = players.iter().filter(|p| {
                    let center = p.get_entity().chunk_pos.load();
                    let view_distance = get_view_distance(p).get() as i32;
                    is_within_view_distance(chunk_pos, center, view_distance)
                });

                let mut bedrock_packets = Vec::new();
                for (block_pos, block_state_id) in &updates {
                    let be_block_id = BlockState::to_be_network_id(*block_state_id);
                    let update_packet = pumpkin_protocol::bedrock::client::CUpdateBlock::new(
                        *block_pos,
                        be_block_id as u32,
                    );
                    let actor_packet = self
                        .bedrock_block_entity_data(*block_state_id, *block_pos)
                        .map(|data| CBlockActorData::new(*block_pos, data));
                    bedrock_packets.push((update_packet, actor_packet));
                }

                let mut bedrock_recipients = Vec::new();
                for p in recipients {
                    match p.client.as_ref() {
                        ClientPlatform::Java(_) => java_recipients.push(p),
                        ClientPlatform::Bedrock(be_client) => {
                            bedrock_recipients.push(be_client);
                        }
                    }
                }

                for be_client in bedrock_recipients {
                    for (update_packet, actor_packet) in &bedrock_packets {
                        if let Ok(data) = be_client.serialize_packet(update_packet) {
                            be_client.try_enqueue_packet(data);
                        }
                        if let Some(actor_packet) = actor_packet
                            && let Ok(data) = be_client.serialize_packet(actor_packet)
                        {
                            be_client.try_enqueue_packet(data);
                        }
                    }
                }

                let recipients_by_version =
                    Self::collect_java_recipients_by_version(java_recipients.into_iter());
                Self::broadcast_java_grouped(
                    &CMultiBlockUpdate::new(&updates),
                    recipients_by_version,
                );

                for (block_pos, _) in &updates {
                    if let Some(block_entity) = self.get_block_entity(block_pos)
                        && let Some(nbt) = block_entity.chunk_data_nbt()
                    {
                        let bytes = pumpkin_nbt::Nbt::from(nbt).write_unnamed();
                        self.broadcast_to_chunk(
                            chunk_pos,
                            &CBlockEntityData::new(
                                *block_pos,
                                VarInt(block_entity.get_id() as i32),
                                bytes.as_ref().into(),
                            ),
                        );
                    }
                }
            }

            let mut bedrock_water_packets = Vec::new();
            for (block_pos, block_state_id) in &updates {
                let water_state = bedrock_water_state(*block_state_id);
                let packet = pumpkin_protocol::bedrock::client::CUpdateBlock::with_layer(
                    *block_pos,
                    u32::from(BlockState::to_be_network_id(water_state)),
                    1,
                );
                bedrock_water_packets.push(packet);
            }

            if !bedrock_water_packets.is_empty() {
                let players = self.players.load();
                let recipients = players.iter().filter(|player| {
                    let center = player.get_entity().chunk_pos.load();
                    let view_distance = get_view_distance(player).get() as i32;
                    is_within_view_distance(chunk_pos, center, view_distance)
                });
                for player in recipients {
                    if let ClientPlatform::Bedrock(client) = player.client.as_ref() {
                        for packet in &bedrock_water_packets {
                            if let Ok(data) = client.serialize_packet(packet) {
                                client.try_enqueue_packet(data);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn tick_environment(self: &Arc<Self>) {
        let (world_age, is_night, time_of_day) = {
            let mut level_time = self
                .level_time
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let advance_time = self.level_info.load().game_rules.advance_time;
            level_time.tick(advance_time);

            // Auto-save logic
            if level_time.world_age % 100 == 0 {
                self.level.should_unload.store(true, Relaxed);
                let cleaned_chunks = self.level.clean_memory();
                if !cleaned_chunks.is_empty() {
                    let world_clone = self.clone();
                    tokio::spawn(async move {
                        world_clone.remove_entities_in_chunks(&cleaned_chunks).await;
                        world_clone.level.clean_entity_chunks(&cleaned_chunks);
                    });
                }
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
            if self.level.autosave_ticks > 0 && self.level.save_enabled.load(Relaxed) {
                let autosave = self.level.autosave_ticks as i64;
                if autosave > 0 && level_time.world_age % autosave == 0 {
                    self.level.should_save.store(true, Relaxed);
                    self.level.level_channel.notify();
                }
            }
            (
                level_time.world_age,
                level_time.is_night(),
                level_time.time_of_day,
            )
        };

        let (should_reset_weather, weather_cycle_enabled) = {
            let mut weather = self
                .weather
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            weather.tick_weather(self);
            (
                weather.raining || weather.thundering,
                weather.weather_cycle_enabled,
            )
        };

        if self.should_skip_night() && is_night {
            let level_time = {
                let mut guard = self
                    .level_time
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let time = time_of_day + 24000;
                guard.set_time(time - time % 24000);
                guard.clone()
            };
            level_time.send_time(self);

            for player in self.players.load().iter() {
                player.wake_up();
            }

            if weather_cycle_enabled && should_reset_weather {
                let mut weather = self
                    .weather
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                weather.reset_weather_cycle(self);
            }
        } else if world_age % 20 == 0 {
            let level_time = self
                .level_time
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            level_time.send_time(self);
        }
    }

    #[expect(clippy::too_many_lines)]
    pub fn tick_chunks(self: &Arc<Self>, server: &Arc<Server>) {
        const BATCH_SIZE: usize = 32;

        let active_chunks = self.active_chunks.load();
        let tick_data = self.level.get_tick_data(&active_chunks);
        let handle = server.runtime.clone();

        // 1. Parallel Block Ticks via Rayon
        let world = self.clone();
        let block_handle = handle.clone();
        tick_data
            .block_ticks
            .par_chunks(BATCH_SIZE)
            .for_each(|batch| {
                let _guard = block_handle.enter();
                let world = world.clone();
                for scheduled_tick in batch {
                    let pos = scheduled_tick.position;
                    let block = world.get_block(&pos);
                    if let Some(pumpkin_block) = world.block_registry.get_pumpkin_block(block.id) {
                        pumpkin_block.on_scheduled_tick(OnScheduledTickArgs {
                            world: &world,
                            block,
                            position: &pos,
                        });
                    }
                }
            });

        // 2. Parallel Fluid Ticks via Rayon
        let world = self.clone();
        let fluid_handle = handle.clone();
        tick_data
            .fluid_ticks
            .par_chunks(BATCH_SIZE)
            .for_each(|batch| {
                let _guard = fluid_handle.enter();
                let world = world.clone();
                for scheduled_tick in batch {
                    let pos = scheduled_tick.position;
                    let fluid = world.get_fluid(&pos);
                    if let Some(pumpkin_fluid) = world.block_registry.get_pumpkin_fluid(fluid.id) {
                        pumpkin_fluid.on_scheduled_tick(&world, fluid, &pos);
                    }
                }
            });

        // 3. Parallel Random Ticks via Rayon
        let world = self.clone();
        let random_handle = handle.clone();
        tick_data
            .random_ticks
            .par_chunks(BATCH_SIZE)
            .for_each(|batch| {
                let _guard = random_handle.enter();
                let world = world.clone();
                for scheduled_tick in batch {
                    let pos = scheduled_tick.position;
                    let (block, fluid) =
                        match (scheduled_tick.tick_block, scheduled_tick.tick_fluid) {
                            (true, true) => {
                                let (b, f) = world.get_block_and_fluid(&pos);
                                (Some(b), Some(f))
                            }
                            (true, false) => (Some(world.get_block(&pos)), None),
                            (false, true) => (None, Some(world.get_fluid(&pos))),
                            (false, false) => (None, None),
                        };

                    if let Some(block) = block
                        && let Some(pumpkin_block) =
                            world.block_registry.get_pumpkin_block(block.id)
                    {
                        pumpkin_block.random_tick(RandomTickArgs {
                            world: &world,
                            block,
                            position: &pos,
                        });
                    }

                    if let Some(fluid) = fluid
                        && let Some(pumpkin_fluid) =
                            world.block_registry.get_pumpkin_fluid(fluid.id)
                    {
                        pumpkin_fluid.random_tick(fluid, &world, &pos);
                    }
                }
            });

        // 4. Calculate Spawn List (Sequential setup)
        let spawn_state = self.spawn_state.load();
        let (spawn_mobs, spawn_monsters, peaceful) = {
            let lock = self.level_info.load();
            (
                lock.game_rules.spawn_mobs,
                lock.game_rules.spawn_monsters,
                lock.difficulty == Difficulty::Peaceful,
            )
        };
        let spawn_passives = self.get_time_of_day() % 400 == 0;
        let spawn_enemies = !peaceful && spawn_monsters && spawn_mobs;
        let spawn_passives = spawn_passives && spawn_mobs;

        let spawn_list = Arc::new(natural_spawner::get_filtered_spawning_categories(
            &spawn_state,
            spawn_mobs,
            spawn_enemies,
            spawn_passives,
        ));

        // 5. Parallel Chunk Spawners via Rayon
        if !spawn_list.is_empty() {
            let mut spawning_chunks = Vec::new();
            for pos in active_chunks.iter() {
                if let Some(chunk) = self.level.read_chunk_sync(pos, std::clone::Clone::clone) {
                    spawning_chunks.push((*pos, chunk));
                }
            }

            spawning_chunks.shuffle(&mut rng());

            let world = self.clone();
            let spawn_handle = handle;
            spawning_chunks.par_chunks(8).for_each(|batch| {
                let _guard = spawn_handle.enter();
                let world = world.clone();
                let s_list = spawn_list.clone();
                let s_state = spawn_state.clone();
                for (pos, chunk) in batch {
                    world.tick_spawning_chunk(*pos, chunk, &s_list, &s_state);
                }
            });
        }

        // Update chunk inhabited time for active chunks in parallel with Rayon
        let loaded_chunks = self.level.loaded_chunks.clone();
        let active_chunks_vec: Vec<_> = active_chunks.iter().copied().collect();
        active_chunks_vec.par_iter().for_each(|pos| {
            if let Some(chunk) = loaded_chunks.get(pos) {
                chunk.inhabited_time.fetch_add(1, Relaxed);
            }
        });
    }

    pub fn check_fluid_collision(self: &Arc<Self>, bounding_box: BoundingBox) -> bool {
        let min = bounding_box.min_block_pos();

        let max = bounding_box.max_block_pos();

        for x in min.0.x..=max.0.x {
            for y in min.0.y..=max.0.y {
                for z in min.0.z..=max.0.z {
                    let pos = BlockPos::new(x, y, z);

                    let (fluid, state) = self.get_fluid_and_fluid_state(&pos);

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

    // FlowingFluid.getFlow()
    pub fn get_fluid_velocity(
        &self,
        pos0: BlockPos,
        fluid0: &Fluid,
        state0: &FluidState,
    ) -> Vector3<f64> {
        let mut velo = Vector3::default();

        for dir in BlockDirection::horizontal() {
            let offset = dir.to_offset();
            let pos = pos0.offset(offset);

            let (neighbor_fluid, neighbor_state) = self.get_fluid_and_fluid_state(&pos);

            if neighbor_fluid.matches_type(fluid0) {
                let mut neighbor_height = neighbor_state.height;
                let mut amplitude = 0.0;

                if neighbor_height == 0.0 {
                    let state_id = self.get_block_state_id(&pos);
                    let block_id = state_id.to_block_id();
                    let block_state = state_id.to_state();

                    let blocks_movement = blocks_movement(block_state, block_id);

                    if !blocks_movement {
                        let down_pos = pos.down();
                        let (down_fluid, down_state) = self.get_fluid_and_fluid_state(&down_pos);

                        if down_fluid.matches_type(fluid0) {
                            neighbor_height = down_state.height;
                            if neighbor_height > 0.0 {
                                amplitude = f64::from(state0.height)
                                    - (f64::from(neighbor_height) - 0.888_888_9);
                            }
                        }
                    }
                } else if neighbor_height > 0.0 {
                    amplitude = f64::from(state0.height) - f64::from(neighbor_height);
                }

                if amplitude != 0.0 {
                    velo.x += f64::from(offset.x) * amplitude;
                    velo.z += f64::from(offset.z) * amplitude;
                }
            }
        }

        if state0.falling {
            for dir in BlockDirection::horizontal() {
                let pos = pos0.offset(dir.to_offset());

                if self.is_solid_face(fluid0.id, pos, dir.to_block_direction())
                    || self.is_solid_face(fluid0.id, pos.up(), dir.to_block_direction())
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

    // FlowingFluid.isSolidFace()
    fn is_solid_face(&self, fluid0_id: u16, pos: BlockPos, direction: BlockDirection) -> bool {
        let id = self.get_block_state_id(&pos);

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
        'shapes: for shape in state.get_block_outline_shapes_at(&pos) {
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
            .get_block_collision_shapes_at(&pos)
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
    pub fn get_block_collisions(
        &self,
        bounding_box: BoundingBox,
        entity: &dyn EntityBase,
    ) -> (Vec<BoundingBox>, Vec<(usize, BlockPos)>) {
        let mut collisions = Vec::new();

        let mut positions = Vec::new();

        let min = BlockPos::floored_v(bounding_box.min.add_raw(0.0, -0.50001, 0.0));
        let max = bounding_box.max_block_pos();
        let pos_iter = BlockPos::iterate(min, max);

        for pos in pos_iter {
            let state = self.get_block_state(&pos);

            if state.is_air() {
                continue;
            }

            let block = Block::from_state_id(state.id);
            let mut collided = false;

            if block == &Block::POWDER_SNOW {
                if let Some(shape) =
                    crate::block::blocks::powder_snow::collision_shape_for_entity(entity, &pos)
                {
                    let shape = shape.at_pos(pos);
                    if shape.intersects(&bounding_box) {
                        collided = true;
                        collisions.push(shape);
                    }
                }
            } else {
                for shape in state.get_block_collision_shapes_at(&pos) {
                    let shape = shape.at_pos(pos);
                    if shape.intersects(&bounding_box) {
                        collided = true;
                        collisions.push(shape);
                    }
                }
            }

            if collided {
                positions.push((collisions.len(), pos));
            }
        }

        (collisions, positions)
    }

    pub fn is_space_empty(&self, bounding_box: BoundingBox) -> bool {
        let min = bounding_box.min_block_pos();
        let max = bounding_box.max_block_pos();

        for pos in BlockPos::iterate(min, max) {
            let state = self.get_block_state(&pos);
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
    pub fn get_dismount_height(&self, pos: &BlockPos) -> f64 {
        let state = self.get_block_state(pos);
        let max_y = state
            .get_block_collision_shapes_at(pos)
            .map(|s| s.max.y)
            .fold(f64::NEG_INFINITY, f64::max);
        if max_y != f64::NEG_INFINITY {
            return max_y;
        }
        // No collision at pos — check block below
        let below = BlockPos(Vector3::new(pos.0.x, pos.0.y - 1, pos.0.z));
        let below_state = self.get_block_state(&below);
        let below_max_y = below_state
            .get_block_collision_shapes_at(&below)
            .map(|s| s.max.y)
            .fold(f64::NEG_INFINITY, f64::max);
        if below_max_y >= 1.0 {
            below_max_y - 1.0
        } else {
            f64::NEG_INFINITY
        }
    }

    pub fn tick_spawning_chunk(
        self: &Arc<Self>,
        chunk_pos: Vector2<i32>,
        chunk: &Arc<ChunkData>,
        spawn_list: &Vec<&'static MobCategory>,
        spawn_state: &Arc<SpawnState>,
    ) {
        // this.level.tickThunder(chunk);
        //TODO check in simulation distance
        let (is_raining, is_thundering) = (self.is_raining(), self.is_thundering());

        if is_raining && is_thundering && rng().random_range(0..100_000) == 0 {
            let rand_value = rng().random::<i32>() >> 2;
            let delta = Vector3::new(rand_value & 15, rand_value >> 16 & 15, rand_value >> 8 & 15);
            let random_pos = Vector3::new(
                chunk_pos.x << 4,
                chunk
                    .heightmap
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .get(
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
                    && self.get_block(&random_pos.to_block_pos().down()) != &Block::LIGHTNING_ROD
                {
                    let entity = Entity::new(
                        self.clone(),
                        random_pos.to_f64(),
                        &EntityType::SKELETON_HORSE,
                    );
                    self.spawn_entity_non_save(Arc::new(entity));
                }
                let entity = Entity::new(
                    self.clone(),
                    random_pos.to_f64().add_raw(0.5, 0., 0.5),
                    &EntityType::LIGHTNING_BOLT,
                );
                self.spawn_entity_non_save(Arc::new(entity));
            }
        }

        if spawn_list.is_empty() {
            return;
        }
        // TODO this.level.canSpawnEntitiesInChunk(chunkPos)
        let entities = spawn_for_chunk(
            self,
            chunk_pos,
            chunk,
            spawn_state,
            spawn_list,
            is_thundering,
        );
        for entity in entities {
            self.spawn_entity_non_save(entity);
        }
    }

    pub fn get_world_age(&self) -> i64 {
        self.level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .world_age
    }

    pub fn get_time_of_day(&self) -> i64 {
        self.level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .time_of_day
    }

    pub fn set_time_of_day(&self, time: i64) {
        let level_time = {
            let mut guard = self
                .level_time
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            guard.set_time(time);
            guard.clone()
        };
        level_time.send_time(self);
    }

    pub fn is_raining(&self) -> bool {
        self.weather
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .raining
    }

    pub fn is_raining_at(&self, pos: &BlockPos) -> bool {
        if !self.is_raining() {
            return false;
        }
        if self.get_heightmap_height(MotionBlocking, pos.0.x, pos.0.z) + 1 > pos.0.y {
            return false;
        }
        self.can_see_sky(pos)
            && self
                .get_biome(pos)
                .weather
                .is_rain_at(pos.0.x, pos.0.y, pos.0.z, self.sea_level)
    }

    pub fn set_raining(&self, raining: bool) {
        if let Some(server) = self.server.upgrade() {
            let world_arc = server.get_world_from_dimension(&self.dimension);
            let mut event =
                crate::plugin::api::events::world::weather_change::WeatherChangeEvent::new(
                    world_arc, raining,
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
        }
        let mut weather = self
            .weather
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if weather.raining != raining {
            let thunder = weather.thundering;
            weather.set_weather_parameters(self, 0, 0, raining, thunder);
        }
    }

    pub fn is_thundering(&self) -> bool {
        self.weather
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .thundering
    }

    pub fn set_thundering(&self, thundering: bool) {
        if let Some(server) = self.server.upgrade() {
            let world_arc = server.get_world_from_dimension(&self.dimension);
            let mut event =
                crate::plugin::api::events::world::weather_change::ThunderChangeEvent::new(
                    world_arc, thundering,
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
        }
        let mut weather = self
            .weather
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if weather.thundering != thundering {
            let raining = weather.raining;
            weather.set_weather_parameters(self, 0, 0, raining, thundering);
        }
    }

    /// Gets the y position of the first non air block from the top down
    pub fn get_top_block(&self, position: Vector2<i32>) -> i32 {
        let chunk_pos = Vector2::new(position.x >> 4, position.y >> 4);
        let relative_x = (position.x & 15) as usize;
        let relative_z = (position.y & 15) as usize;

        self.level
            .read_chunk_sync(&chunk_pos, |chunk| {
                let height = chunk
                    .heightmap
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .get(
                        ChunkHeightmapType::WorldSurface,
                        position.x,
                        position.y,
                        self.dimension.min_y,
                    );

                if height >= self.dimension.min_y {
                    return height;
                }

                for y in (self.dimension.min_y..self.dimension.min_y + self.dimension.height).rev()
                {
                    if let Some(block_id) = chunk
                        .section
                        .get_block_absolute_y(relative_x, y, relative_z)
                        && !is_air(block_id)
                    {
                        return y;
                    }
                }
                self.dimension.min_y
            })
            .unwrap_or(self.dimension.min_y)
    }

    pub fn get_heightmap_height(&self, height_map: ChunkHeightmapType, x: i32, z: i32) -> i32 {
        let chunk_pos = Vector2::new(x >> 4, z >> 4);
        self.level
            .read_chunk_sync(&chunk_pos, |chunk| {
                chunk
                    .heightmap
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .get(height_map, x, z, self.min_y)
            })
            .unwrap_or(self.min_y)
    }

    #[allow(clippy::too_many_lines)]
    pub async fn spawn_bedrock_player(
        &self,
        base_config: &BasicConfiguration,
        player: Arc<Player>,
        server: &Arc<Server>,
    ) {
        static CREATIVE_CONTENT: std::sync::OnceLock<(
            Vec<CreativeGroupInfoPayload>,
            Vec<CreativeItemEntryPayload>,
        )> = std::sync::OnceLock::new();

        static BEDROCK_CRAFTING_DATA: std::sync::OnceLock<
            Vec<pumpkin_protocol::bedrock::client::BedrockRecipe>,
        > = std::sync::OnceLock::new();

        let level_info = server.level_info.load();
        let (rain_level, lightning_level) = {
            let weather = self
                .weather
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (weather.rain_level, weather.thunder_level)
        };
        let runtime_id = player.entity_id() as u64;
        let (position, yaw, pitch) = if player.has_played_before.load(Ordering::Relaxed) {
            let position = player.position();
            let yaw = player.get_entity().yaw.load(); //info.spawn_angle;
            let pitch = player.get_entity().pitch.load();

            (position, yaw, pitch)
        } else {
            let spawn_position = Vector2::new(level_info.spawn_x, level_info.spawn_z);
            let chunk_pos = Vector2::new(level_info.spawn_x >> 4, level_info.spawn_z >> 4);
            self.level.get_or_fetch_chunk(chunk_pos, |_| ()).await;
            let top = self.get_top_block(spawn_position);
            let pos_y = if top > self.dimension.min_y {
                top + 1
            } else {
                level_info.spawn_y
            };

            let position = Vector3::new(
                f64::from(level_info.spawn_x) + 0.5,
                f64::from(pos_y),
                f64::from(level_info.spawn_z) + 0.5,
            );
            (position, level_info.spawn_yaw, level_info.spawn_pitch)
        };

        // Keep the server-side transform aligned with the StartGame position. In
        // particular, this ensures an early disconnect persists the real spawn.
        player.living_entity.entity.set_pos(position);
        player.living_entity.entity.set_rotation(yaw, pitch);
        player.living_entity.entity.last_pos.store(position);

        // Todo make the data less spread
        let level_settings = LevelSettings {
            seed: self.level.seed.0,
            spawn_biome_type: 0,
            custom_biome_name: String::new(),
            dimension: VarInt(0),
            generator_type: VarInt(1),
            world_gamemode: server
                .defaultgamemode
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .gamemode
                .into(),
            hardcore: base_config.hardcore,
            difficulty: VarInt(level_info.difficulty as i32),
            spawn_position: BlockPos::new(
                level_info.spawn_x,
                level_info.spawn_y,
                level_info.spawn_z,
            ),
            has_achievements_disabled: false,
            editor_world_type: VarInt(0),
            is_created_in_editor: false,
            is_exported_from_editor: false,
            day_cycle_stop_time: VarInt(-1),
            education_edition_offer: VarUInt(0),
            has_education_features_enabled: false,
            education_product_id: String::new(),
            rain_level,
            lightning_level,
            has_confirmed_platform_locked_content: false,
            was_multiplayer_intended: true,
            was_lan_broadcasting_intended: true,
            xbox_live_broadcast_setting: GamePublishSetting::Public,
            platform_broadcast_setting: GamePublishSetting::Public,
            commands_enabled: level_info.allow_commands,
            is_texture_packs_required: false,
            rule_data: Vec::new(),
            experiments: Experiments {
                toggles: Vec::new(),
                experiments_ever_toggled: false,
            },
            bonus_chest: false,
            has_start_with_map_enabled: false,
            // TODO Bedrock permission level are different
            permission_level: 2,
            server_simulation_distance: server
                .advanced_config
                .networking
                .bedrock
                .simulation_distance
                .get()
                .into(),
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
            server_editor_connection_policy: VarInt(0),
            allow_anonymous_block_drops_in_editor_worlds: false,
        };
        drop(level_info);

        let Some(client) = player.client.bedrock() else {
            return;
        };

        let start_game = CStartGame {
            entity_id: VarLong(runtime_id as _),
            runtime_entity_id: VarULong(runtime_id),
            player_gamemode: player.gamemode.load().into(),
            // Bedrock represents the local player at eye height; Pumpkin stores feet position.
            position: Vector3::new(
                position.x as f32,
                position.y as f32 + player.get_entity().entity_type.eye_height,
                position.z as f32,
            ),
            pitch,
            yaw,
            level_settings,
            level_id: String::new(),
            level_name: "Pumpkin world".to_string(),
            premium_world_template_id: String::new(),
            is_trial: false,
            rewind_history_size: VarInt(0),
            server_authoritative_block_breaking: true,
            current_level_time: self.get_world_age() as _,
            enchantment_seed: VarInt(0),
            block_properties_size: VarUInt(0),
            // TODO Make this unique
            multiplayer_correlation_id: Uuid::default().to_string(),
            enable_itemstack_net_manager: true,
            server_version: "Pumpkin Rust Server".to_string(),
            compound_id: 10,
            compound_len: VarUInt(0),
            compound_end: 0,
            block_registry_checksum: 0,
            world_template_id: Uuid::nil(),
            enable_clientside_generation: false,
            blocknetwork_ids_are_hashed: false,
            server_auth_sounds: true,
            server_join_information: None,
            telemetry: ServerTelemetryData {
                server_id: String::new(),
                scenario_id: String::new(),
                world_id: String::new(),
                owner_id: String::new(),
            },
        };
        if let Ok(data) = client.serialize_packet(&start_game) {
            client.send_game_packet(data).await;
        }

        if let Ok(data) = client.serialize_packet(&CBiomeDefinitionList) {
            client.send_game_packet(data).await;
        }

        let item_registry = CItemRegistry {
            items: BedrockItem::ALL_BEDROCK_ITEMS
                .iter()
                .map(|b| ItemData {
                    item_name: b.registry_key.into(),
                    item_id: b.id,
                    is_component_based: b.component_based,
                    item_version: VarInt::from(match b.version {
                        BedrockItemVersion::Legacy => 0,
                        BedrockItemVersion::DataDriven => 1,
                        BedrockItemVersion::None => 2,
                    }),
                    component_data: b.definition_components.into(),
                })
                .collect::<Vec<_>>(),
        };
        if let Ok(data) = client.serialize_packet(&item_registry) {
            client.send_game_packet(data).await;
        }

        let (groups, entries) = CREATIVE_CONTENT.get_or_init(|| {
            let groups = pumpkin_data::bedrock_creative::CREATIVE_GROUPS
                .iter()
                .map(|g| {
                    let creative_category = match g.category {
                        1 => CreativeCategory::Construction,
                        2 => CreativeCategory::Nature,
                        3 => CreativeCategory::Equipment,
                        4 => CreativeCategory::Items,
                        5 => CreativeCategory::ItemCommandOnly,
                        _ => CreativeCategory::Undefined,
                    };
                    let icon_item = if g.icon_item_id != 0 {
                        NetworkItemDescriptor {
                            id: VarInt::from(g.icon_item_id),
                            stack_size: 1,
                            aux_value: VarUInt(g.icon_item_aux_value),
                            block_runtime_id: VarInt(0),
                            nbt_data: pumpkin_nbt::Nbt::default(),
                            place_on_blocks: Vec::new(),
                            destroy_blocks: Vec::new(),
                            shield_blocking_tick: 0,
                        }
                    } else {
                        NetworkItemDescriptor::default()
                    };

                    CreativeGroupInfoPayload {
                        creative_category,
                        name: g.name.to_string(),
                        group_icon_item: icon_item,
                    }
                })
                .collect::<Vec<_>>();

            let entries = pumpkin_data::bedrock_creative::CREATIVE_ENTRIES
                .iter()
                .enumerate()
                .map(|(i, e)| CreativeItemEntryPayload {
                    id: VarUInt((i + 1) as u32),
                    item: NetworkItemDescriptor {
                        id: VarInt::from(e.item_id),
                        stack_size: 1,
                        aux_value: VarUInt(e.item_aux_value),
                        block_runtime_id: VarInt(0),
                        nbt_data: pumpkin_nbt::Nbt::default(),
                        place_on_blocks: Vec::new(),
                        destroy_blocks: Vec::new(),
                        shield_blocking_tick: 0,
                    },
                    group_index: VarUInt(e.group_index),
                })
                .collect::<Vec<_>>();

            (groups, entries)
        });
        let creative_content = CCreativeContent { groups, entries };
        if let Ok(data) = client.serialize_packet(&creative_content) {
            client.send_game_packet(data).await;
        }

        let bedrock_recipes = BEDROCK_CRAFTING_DATA.get_or_init(|| {
            use pumpkin_data::item::{Item, JavaToBedrockItemMapping};
            use pumpkin_data::recipes::{CraftingRecipeTypes, RecipeIngredientTypes};
            use pumpkin_protocol::bedrock::client::{
                BedrockRecipe, BedrockShapedRecipe, BedrockShapelessRecipe, ItemDescriptorCount,
                RecipeUnlockRequirement,
            };
            use pumpkin_protocol::bedrock::network_item::NetworkItemDescriptor;
            use pumpkin_protocol::codec::{var_int::VarInt, var_uint::VarUInt};

            let mut mapped_recipes = Vec::new();
            let mut network_id_counter = 1u32;

            for recipe in pumpkin_data::recipes::RECIPES_CRAFTING {
                let map_ingredient = |ing: &RecipeIngredientTypes| -> ItemDescriptorCount {
                    let item_key = match ing {
                        RecipeIngredientTypes::Simple(name) => Some(*name),
                        RecipeIngredientTypes::Tagged(tag) => {
                            let tag_name = tag.strip_prefix('#').unwrap_or(tag);
                            pumpkin_data::tag::get_tag_ids(
                                pumpkin_data::tag::RegistryKey::Item,
                                tag_name,
                            )
                            .and_then(|ids| {
                                ids.first().and_then(|&first_id| {
                                    Item::from_id(first_id).map(|item| item.registry_key)
                                })
                            })
                        }
                        RecipeIngredientTypes::OneOf(names) => names.first().copied(),
                    };

                    if let Some(key) = item_key {
                        let registry_key = key.strip_prefix("minecraft:").unwrap_or(key);
                        if let Some(item) = Item::from_registry_key(registry_key)
                            && let Some(mapping) =
                                JavaToBedrockItemMapping::from_java_item_id(item.id)
                        {
                            return ItemDescriptorCount {
                                item_identifier: mapping.bedrock_item.registry_key.to_string(),
                                metadata_value: mapping.bedrock_data as i32,
                                count: 1,
                            };
                        }
                    }

                    ItemDescriptorCount {
                        item_identifier: String::new(),
                        metadata_value: 0,
                        count: 0,
                    }
                };

                match recipe {
                    CraftingRecipeTypes::CraftingShaped {
                        category: _,
                        group: _,
                        show_notification: _,
                        key,
                        pattern,
                        result,
                    } => {
                        let height = pattern.len() as i32;
                        let width = pattern.iter().map(|s| s.len()).max().unwrap_or(0) as i32;

                        let mut input = Vec::new();
                        for r in 0..height {
                            let pattern_row = pattern[r as usize];
                            for c in 0..width {
                                let ch = pattern_row.chars().nth(c as usize).unwrap_or(' ');
                                if ch == ' ' {
                                    input.push(ItemDescriptorCount {
                                        item_identifier: String::new(),
                                        metadata_value: 0,
                                        count: 0,
                                    });
                                } else {
                                    let mut ingredient = None;
                                    for &(key_ch, ref ing) in *key {
                                        if key_ch == ch {
                                            ingredient = Some(ing);
                                            break;
                                        }
                                    }
                                    if let Some(ing) = ingredient {
                                        input.push(map_ingredient(ing));
                                    } else {
                                        input.push(ItemDescriptorCount {
                                            item_identifier: String::new(),
                                            metadata_value: 0,
                                            count: 0,
                                        });
                                    }
                                }
                            }
                        }

                        let output_item = Item::from_registry_key(result.id);
                        if let Some(item) = output_item
                            && let Some(mapping) =
                                JavaToBedrockItemMapping::from_java_item_id(item.id)
                        {
                            let output_descriptor = NetworkItemDescriptor {
                                id: VarInt::from(mapping.bedrock_item.id),
                                stack_size: result.count as u16,
                                aux_value: VarUInt(mapping.bedrock_data),
                                block_runtime_id: VarInt::from(mapping.bedrock_block_state),
                                nbt_data: pumpkin_nbt::Nbt::default(),
                                place_on_blocks: Vec::new(),
                                destroy_blocks: Vec::new(),
                                shield_blocking_tick: 0,
                            };

                            mapped_recipes.push(BedrockRecipe::Shaped(BedrockShapedRecipe {
                                recipe_id: format!("pumpkin:recipe_{network_id_counter}"),
                                width: VarInt(width),
                                height: VarInt(height),
                                input,
                                output: vec![output_descriptor],
                                uuid: Uuid::nil(),
                                block: "crafting_table".to_string(),
                                priority: VarInt(1),
                                assume_symmetry: true,
                                unlock_requirement: RecipeUnlockRequirement { context: 1 },
                                recipe_network_id: VarUInt(network_id_counter),
                            }));
                            network_id_counter += 1;
                        }
                    }
                    CraftingRecipeTypes::CraftingShapeless {
                        category: _,
                        group: _,
                        ingredients,
                        result,
                    } => {
                        let input = ingredients.iter().map(map_ingredient).collect::<Vec<_>>();

                        let output_item = Item::from_registry_key(result.id);
                        if let Some(item) = output_item
                            && let Some(mapping) =
                                JavaToBedrockItemMapping::from_java_item_id(item.id)
                        {
                            let output_descriptor = NetworkItemDescriptor {
                                id: VarInt::from(mapping.bedrock_item.id),
                                stack_size: result.count as u16,
                                aux_value: VarUInt(mapping.bedrock_data),
                                block_runtime_id: VarInt::from(mapping.bedrock_block_state),
                                nbt_data: pumpkin_nbt::Nbt::default(),
                                place_on_blocks: Vec::new(),
                                destroy_blocks: Vec::new(),
                                shield_blocking_tick: 0,
                            };

                            mapped_recipes.push(BedrockRecipe::Shapeless(BedrockShapelessRecipe {
                                recipe_id: format!("pumpkin:recipe_{network_id_counter}"),
                                input,
                                output: vec![output_descriptor],
                                uuid: Uuid::nil(),
                                block: "crafting_table".to_string(),
                                priority: VarInt(1),
                                unlock_requirement: RecipeUnlockRequirement { context: 1 },
                                recipe_network_id: VarUInt(network_id_counter),
                            }));
                            network_id_counter += 1;
                        }
                    }
                    _ => {}
                }
            }
            mapped_recipes
        });

        let crafting_data = pumpkin_protocol::bedrock::client::CCraftingData {
            recipes: bedrock_recipes.clone(),
            clean_recipes: false,
        };
        if let Ok(data) = client.serialize_packet(&crafting_data) {
            client.send_game_packet(data).await;
        }

        player.on_screen_handler_opened(&player.player_screen_handler);

        {
            let mut abilities = player
                .abilities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            abilities.set_for_gamemode(player.gamemode.load());
        };

        let entity = &player.get_entity();
        let metadata = entity.bedrock_metadata();

        let actor_data = CSetActorData {
            target_runtime_id: VarULong(runtime_id),
            actor_data: metadata,
            synced_properties: PropertySyncData {
                int_entries_list: HashMap::new(),
                float_entries_list: HashMap::new(),
            },
            tick: VarULong(0),
        };
        if let Ok(data) = client.serialize_packet(&actor_data) {
            client.send_game_packet(data).await;
        }
        player.send_abilities_update();

        {
            let command_dispatcher = server.command_dispatcher.load();
            client_suggestions::send_bedrock_commands_packet(&player, server, &command_dispatcher);
        };

        client
            .enqueue_client_packet(&CUpdateAttributes {
                target_runtime_id: VarULong(runtime_id),
                attribute_list: vec![
                    AttributeData {
                        min_value: 0.0,
                        max_value: 3.402_823_5E38,
                        current_value: 0.1,
                        default_min_value: 0.0,
                        default_max_value: 3.402_823_5E38,
                        default_value: 0.1,
                        name: "minecraft:movement".to_string(),
                        modifiers: Vec::new(),
                    },
                    AttributeData {
                        min_value: 0.0,
                        max_value: 3.402_823_5E38,
                        current_value: 0.02,
                        default_min_value: 0.0,
                        default_max_value: 3.402_823_5E38,
                        default_value: 0.02,
                        name: "minecraft:underwater_movement".to_string(),
                        modifiers: Vec::new(),
                    },
                    AttributeData {
                        min_value: 0.0,
                        max_value: 1.0,
                        current_value: 0.08,
                        default_min_value: 0.0,
                        default_max_value: 1.0,
                        default_value: 0.08,
                        name: "minecraft:gravity".to_string(),
                        modifiers: Vec::new(),
                    },
                    AttributeData {
                        min_value: 0.0,
                        max_value: 400.0,
                        current_value: 400.0,
                        default_min_value: 0.0,
                        default_max_value: 400.0,
                        default_value: 400.0,
                        name: "minecraft:air".to_string(),
                        modifiers: Vec::new(),
                    },
                    AttributeData {
                        min_value: 0.0,
                        max_value: 20.0,
                        current_value: player.living_entity.health.load(),
                        default_min_value: 0.0,
                        default_max_value: 20.0,
                        default_value: 20.0,
                        name: "minecraft:health".to_string(),
                        modifiers: Vec::new(),
                    },
                    AttributeData {
                        min_value: 0.0,
                        max_value: 20.0,
                        current_value: player.hunger_manager.level.load().into(),
                        default_min_value: 0.0,
                        default_max_value: 20.0,
                        default_value: 20.0,
                        name: "minecraft:player.hunger".to_string(),
                        modifiers: Vec::new(),
                    },
                ],
                tick: VarULong(0),
            })
            .await;

        // --- MULTIPLAYER BROADCASTING ---

        let gameprofile = &player.gameprofile;
        let velocity = player.get_entity().velocity.load();

        // 1. Broadcast the new Bedrock player to everyone else (Java + Bedrock)
        let bedrock_player_list = CPlayerList {
            action: CPlayerList::ACTION_ADD,
            entries: vec![PlayerListEntry {
                uuid: gameprofile.id,
                entity_unique_id: VarLong(runtime_id as i64),
                username: gameprofile.name.clone(),
                xuid: String::new(),
                platform_chat_id: String::new(),
                build_platform: BuildPlatform::Unknown,
                skin: (**player.bedrock_skin.load()).clone(),
                is_teacher: false,
                is_host: false,
                is_sub_client: false,
                player_color: [0, 0, 0, 0],
            }],
        };

        let gamemode = player.gamemode.load();
        self.broadcast_packet_except_editioned(
            &[gameprofile.id],
            &CPlayerInfoUpdate::new(
                (PlayerInfoFlags::ADD_PLAYER
                    | PlayerInfoFlags::UPDATE_GAME_MODE
                    | PlayerInfoFlags::UPDATE_LISTED
                    | PlayerInfoFlags::UPDATE_LATENCY
                    | PlayerInfoFlags::UPDATE_LIST_PRIORITY
                    | PlayerInfoFlags::UPDATE_HAT)
                    .bits(),
                &[pumpkin_protocol::java::client::play::Player {
                    uuid: gameprofile.id,
                    actions: &[
                        PlayerAction::AddPlayer {
                            name: &gameprofile.name,
                            properties: &gameprofile.properties.load(),
                        },
                        PlayerAction::UpdateGameMode(VarInt(gamemode as i32)),
                        PlayerAction::UpdateListed(true),
                        PlayerAction::UpdateLatency(VarInt(0)),
                        PlayerAction::UpdateListOrder(VarInt(0)),
                        PlayerAction::UpdateHat(true),
                    ],
                }],
            ),
            &bedrock_player_list,
        );

        let bedrock_add_player = CAddPlayer {
            uuid: gameprofile.id,
            player_name: gameprofile.name.clone(),
            target_runtime_id: VarULong(runtime_id),
            platform_chat_id: String::new(),
            position: Vector3::new(position.x as f32, position.y as f32, position.z as f32),
            velocity: Vector3::new(velocity.x as f32, velocity.y as f32, velocity.z as f32),
            rotation: Vector2::new(pitch, yaw),
            y_head_rotation: yaw,
            carried_item: NetworkItemStackDescriptor::default(),
            player_game_type: player.gamemode.load().into(),
            entity_data: entity.bedrock_metadata(),
            synced_properties: PropertySyncData::default(),
            abilities_data: pumpkin_protocol::bedrock::client::SerializedAbilitiesData {
                target_player_raw_id: runtime_id as i64,
                player_permissions:
                    pumpkin_protocol::bedrock::client::PlayerPermissionLevel::Visitor,
                command_permissions: pumpkin_protocol::bedrock::client::CommandPermissionLevel::Any,
                layers: vec![
                    pumpkin_protocol::bedrock::client::SerializedAbilitiesDataSerializedLayer {
                        serialized_layer: 0,
                        abilities_set: 0,
                        ability_value: 0,
                        fly_speed: 0.05,
                        vertical_fly_speed: 0.05,
                        walk_speed: 0.1,
                    },
                ],
            },
            actor_links: Vec::new(),
            device_id: String::new(),
            build_platform: BuildPlatform::Unknown,
        };

        self.broadcast_packet_except_editioned(
            &[gameprofile.id],
            &CSpawnEntity::new(
                (runtime_id as i32).into(),
                gameprofile.id,
                i32::from(EntityType::PLAYER.id).into(),
                position,
                pitch,
                yaw,
                yaw,
                0.into(),
                velocity,
            ),
            &bedrock_add_player,
        );

        self.send_player_equipment(&player);

        // Broadcast metadata to Java players so they can correctly interact with the new player
        let skin_parts = player.config.load().skin_parts;

        self.broadcast_skin_parts(
            &[gameprofile.id],
            runtime_id as i32,
            skin_parts,
            &actor_data,
        );

        // 2. Spawn existing players for our new Bedrock client
        let players = self.players.load();

        for existing_player in players
            .iter()
            .filter(|p| p.gameprofile.id != gameprofile.id)
        {
            let ex_profile = &existing_player.gameprofile;
            let ex_entity = &existing_player.get_entity();
            let ex_pos = ex_entity.pos.load();
            let ex_vel = ex_entity.velocity.load();

            let ex_player_list = CPlayerList {
                action: CPlayerList::ACTION_ADD,
                entries: vec![PlayerListEntry {
                    uuid: ex_profile.id,
                    entity_unique_id: VarLong(existing_player.entity_id() as i64),
                    username: ex_profile.name.clone(),
                    xuid: String::new(),
                    platform_chat_id: String::new(),
                    build_platform: BuildPlatform::Unknown,
                    skin: (**existing_player.bedrock_skin.load()).clone(),
                    is_teacher: false,
                    is_host: false,
                    is_sub_client: false,
                    player_color: [0, 0, 0, 0],
                }],
            };
            // Send PlayerList FIRST
            client.send_packet(&ex_player_list).await;

            let ex_add_player = CAddPlayer {
                uuid: ex_profile.id,
                player_name: ex_profile.name.clone(),
                target_runtime_id: VarULong(existing_player.entity_id() as u64),
                platform_chat_id: String::new(),
                position: Vector3::new(ex_pos.x as f32, ex_pos.y as f32, ex_pos.z as f32),
                velocity: Vector3::new(ex_vel.x as f32, ex_vel.y as f32, ex_vel.z as f32),
                rotation: Vector2::new(ex_entity.pitch.load(), ex_entity.yaw.load()),
                y_head_rotation: ex_entity.head_yaw.load(),
                carried_item: NetworkItemStackDescriptor::default(),
                player_game_type: existing_player.gamemode.load().into(),
                entity_data: ex_entity.bedrock_metadata(),
                synced_properties: PropertySyncData::default(),
                abilities_data: pumpkin_protocol::bedrock::client::SerializedAbilitiesData {
                    target_player_raw_id: existing_player.entity_id() as i64,
                    player_permissions:
                        pumpkin_protocol::bedrock::client::PlayerPermissionLevel::Visitor,
                    command_permissions:
                        pumpkin_protocol::bedrock::client::CommandPermissionLevel::Any,
                    layers: vec![
                        pumpkin_protocol::bedrock::client::SerializedAbilitiesDataSerializedLayer {
                            serialized_layer: 0,
                            abilities_set: 0,
                            ability_value: 0,
                            fly_speed: 0.05,
                            vertical_fly_speed: 0.05,
                            walk_speed: 0.1,
                        },
                    ],
                },
                actor_links: Vec::new(),
                device_id: String::new(),
                build_platform: BuildPlatform::Unknown,
            };

            client.send_packet(&ex_add_player).await;

            let ex_held_item = existing_player.inventory().held_item();

            let ex_be_mob_equipment = pumpkin_protocol::bedrock::client::CMobEquipment {
                target_runtime_id: (existing_player.entity_id() as u64).into(),
                item: (&ex_held_item).into(),
                slot: 0,
                selected_slot: 0,
                container_id: 0,
            };

            client.send_packet(&ex_be_mob_equipment).await;
        }

        player.has_played_before.store(true, Ordering::Relaxed);

        // 3. Trigger Join Event and Broadcast Join Message
        let msg_comp = TextComponent::translate_cross(
            translation::java::MULTIPLAYER_PLAYER_JOINED,
            translation::bedrock::MULTIPLAYER_PLAYER_JOINED,
            [TextComponent::text(player.gameprofile.name.clone())],
        )
        .color_named(NamedColor::Yellow);

        let mut event = PlayerJoinEvent::new(player.clone(), msg_comp);
        server.plugin_manager.fire(server, &mut event).await;

        if !event.cancelled {
            self.broadcast_system_message(&event.join_message, false);
            info!("{}", event.join_message.to_pretty_console());
        }
    }

    #[expect(clippy::too_many_lines)]
    pub async fn spawn_java_player(
        &self,
        base_config: &BasicConfiguration,
        player: &Arc<Player>,
        server: &Arc<Server>,
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

        let Some(client) = player.client.java() else {
            return;
        };
        // Send the login packet for our new player
        client
            .send_packet(&CLogin::new(
                entity_id,
                base_config.hardcore,
                &dimensions,
                server
                    .advanced_config
                    .networking
                    .java
                    .max_players
                    .try_into()
                    .unwrap_or(u16::MAX.into()),
                server
                    .advanced_config
                    .networking
                    .java
                    .view_distance
                    .get()
                    .into(), //  TODO: view distance
                server
                    .advanced_config
                    .networking
                    .java
                    .simulation_distance
                    .get()
                    .into(), // TODO: sim view dinstance
                false,
                true,
                false,
                PlayerSpawnData::new(
                    self.dimension.clone(),
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
                ),
                server.advanced_config.networking.java.online_mode,
                // This should stay true even when reports are disabled.
                // It prevents the annoying popup when joining the server.
                true,
            ))
            .await;

        // Send the current ticking state to the new player so they are in sync.
        server.tick_rate_manager.update_joining_player(player).await;

        // Permissions, i.e. the commands a player may use.
        player.send_permission_lvl_update();

        // Difficulty of the world
        player.send_difficulty_update();
        {
            let command_dispatcher = server.command_dispatcher.load();

            client_suggestions::send_c_commands_packet(player, server, &command_dispatcher);
        };
        if client.version.load() < JavaMinecraftVersion::V_1_20_2
            && client.version.load() >= JavaMinecraftVersion::V_1_13
        {
            let version = client.version.load();
            if let Ok(Ok(packet_data)) = tokio::task::spawn_blocking(move || {
                let mut tags = Vec::new();
                for &key in pumpkin_data::tag::RegistryKey::NETWORK_KEYS {
                    if pumpkin_data::tag::get_registry_key_tags(version, key)
                        .is_some_and(|map| !map.is_empty())
                    {
                        tags.push(key);
                    }
                }
                let packet = pumpkin_protocol::java::client::play::CUpdateTagsPlay::new(&tags);
                JavaClient::serialize_packet_for_version(&packet, version)
            })
            .await
            {
                client.send_packet_now(packet_data).await;
            }
        }

        let (position, yaw, pitch) = if player.has_played_before.load(Ordering::Relaxed) {
            let position = player.position();
            let yaw = player.get_entity().yaw.load(); //info.spawn_angle;
            let pitch = player.get_entity().pitch.load();

            (position, yaw, pitch)
        } else {
            let info = &self.level_info.load();
            let spawn_position = Vector2::new(info.spawn_x, info.spawn_z);
            let chunk_pos = Vector2::new(info.spawn_x >> 4, info.spawn_z >> 4);
            self.level.get_or_fetch_chunk(chunk_pos, |_| ()).await;
            let top = self.get_top_block(spawn_position);
            let pos_y = if top > self.dimension.min_y {
                top + 1
            } else {
                info.spawn_y
            };

            let position = Vector3::new(
                f64::from(info.spawn_x) + 0.5,
                f64::from(pos_y),
                f64::from(info.spawn_z) + 0.5,
            );
            (position, info.spawn_yaw, info.spawn_pitch)
        };

        // Load chunks around the real spawn position before teleporting the client there.
        player.living_entity.entity.set_pos(position);
        player.living_entity.entity.set_rotation(yaw, pitch);
        player.living_entity.entity.last_pos.store(position);
        chunker::update_position(player).await;

        let center_chunk = player.living_entity.entity.chunk_pos.load();
        let chunk = self
            .level
            .get_or_fetch_chunk(center_chunk, std::clone::Clone::clone)
            .await;
        if let Some(server) = self.server.upgrade() {
            let mut event =
                crate::plugin::world::chunk_send::ChunkSend::new(player.world(), chunk.clone());
            server.plugin_manager.fire(&server, &mut event).await;
            if event.cancelled {
                return;
            }
        }
        client.send_chunks(&[chunk]).await;

        let velocity = player.living_entity.entity.velocity.load();

        debug!("Sending player teleport to {}", player.gameprofile.name);
        player.request_teleport(position, yaw, pitch).await;

        let gameprofile = &player.gameprofile;
        let bedrock_player_list = CPlayerList {
            action: CPlayerList::ACTION_ADD,
            entries: vec![PlayerListEntry {
                uuid: gameprofile.id,
                entity_unique_id: VarLong(entity_id as i64),
                username: gameprofile.name.clone(),
                xuid: String::new(),
                platform_chat_id: String::new(),
                build_platform: BuildPlatform::Unknown,
                skin: (**player.bedrock_skin.load()).clone(),
                is_teacher: false,
                is_host: false,
                is_sub_client: false,
                player_color: [0, 0, 0, 0],
            }],
        };

        let player_actions = [
            PlayerAction::AddPlayer {
                name: &gameprofile.name,
                properties: &gameprofile.properties.load(),
            },
            PlayerAction::UpdateGameMode(VarInt(gamemode as i32)),
            PlayerAction::UpdateListed(true),
            PlayerAction::UpdateLatency(VarInt(0)),
            PlayerAction::UpdateListOrder(VarInt(0)),
            PlayerAction::UpdateHat(true),
        ];
        let java_player = [pumpkin_protocol::java::client::play::Player {
            uuid: gameprofile.id,
            actions: &player_actions,
        }];
        let player_info_update = CPlayerInfoUpdate::new(
            (PlayerInfoFlags::ADD_PLAYER
                | PlayerInfoFlags::UPDATE_GAME_MODE
                | PlayerInfoFlags::UPDATE_LISTED
                | PlayerInfoFlags::UPDATE_LATENCY
                | PlayerInfoFlags::UPDATE_LIST_PRIORITY
                | PlayerInfoFlags::UPDATE_HAT)
                .bits(),
            &java_player,
        );

        self.broadcast_editioned(&player_info_update, &bedrock_player_list);

        // If the player has a custom tab_list_name, send an update for it
        if let Some(tab_list_name) = player.get_tab_list_name() {
            let actions = [PlayerAction::UpdateDisplayName(Some(&tab_list_name))];
            let java_player = [pumpkin_protocol::java::client::play::Player {
                uuid: gameprofile.id,
                actions: &actions,
            }];
            self.broadcast_packet_all(&CPlayerInfoUpdate::new(
                PlayerInfoFlags::UPDATE_DISPLAY_NAME.bits(),
                &java_player,
            ));
        }

        // Here, we send all the infos of players who already joined.
        let mut players_tab_list_names = Vec::new();
        {
            let players = self.players.load();
            let mut data_to_process = Vec::new();
            for p in players
                .iter()
                .filter(|p| p.gameprofile.id != player.gameprofile.id)
            {
                let props_guard = p.gameprofile.properties.load();
                data_to_process.push((props_guard, p));
            }

            let mut current_player_data = Vec::new();
            for (properties, player) in &data_to_process {
                let chat_session = player
                    .chat_session
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let tab_list_name = player.get_tab_list_name();

                let mut player_actions = vec![PlayerAction::AddPlayer {
                    name: &player.gameprofile.name,
                    properties,
                }];

                if base_config.allow_chat_reports {
                    let initialized = chat_session.session_id != uuid::Uuid::nil()
                        && !chat_session.public_key.is_empty()
                        && !chat_session.signature.is_empty();
                    player_actions.push(PlayerAction::InitializeChat(initialized.then(|| {
                        InitChat {
                            session_id: chat_session.session_id,
                            expires_at: chat_session.expires_at,
                            public_key: chat_session.public_key.clone(),
                            signature: chat_session.signature.clone(),
                        }
                    })));
                }

                player_actions.extend([
                    PlayerAction::UpdateGameMode(VarInt(player.gamemode.load() as i32)),
                    PlayerAction::UpdateListed(player.tab_list_listed.load(Ordering::Relaxed)),
                    PlayerAction::UpdateLatency(VarInt(
                        player.tab_list_latency.load(Ordering::Relaxed),
                    )),
                    PlayerAction::UpdateListOrder(VarInt(
                        player.tab_list_order.load(Ordering::Relaxed),
                    )),
                    PlayerAction::UpdateHat(true),
                ]);
                drop(chat_session);

                current_player_data.push((&player.gameprofile.id, player_actions));

                // Collect tab_list_names for sending later
                if tab_list_name.is_some() {
                    players_tab_list_names.push((player.gameprofile.id, tab_list_name));
                }
            }

            let mut action_flags = PlayerInfoFlags::ADD_PLAYER
                | PlayerInfoFlags::UPDATE_LISTED
                | PlayerInfoFlags::UPDATE_LATENCY
                | PlayerInfoFlags::UPDATE_LIST_PRIORITY
                | PlayerInfoFlags::UPDATE_GAME_MODE
                | PlayerInfoFlags::UPDATE_HAT;
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
                .enqueue_client_packet(&CPlayerInfoUpdate::new(action_flags.bits(), &entries))
                .await;

            // Send tab_list_names for existing players with custom names
            for (player_id, tab_list_name) in &players_tab_list_names {
                if let Some(name) = tab_list_name {
                    let actions = [PlayerAction::UpdateDisplayName(Some(name))];
                    let java_player = [pumpkin_protocol::java::client::play::Player {
                        uuid: *player_id,
                        actions: &actions,
                    }];
                    client
                        .enqueue_client_packet(&CPlayerInfoUpdate::new(
                            PlayerInfoFlags::UPDATE_DISPLAY_NAME.bits(),
                            &java_player,
                        ))
                        .await;
                }
            }
        };

        let gameprofile = &player.gameprofile;

        let bedrock_add_player = CAddPlayer {
            uuid: gameprofile.id,
            player_name: gameprofile.name.clone(),
            target_runtime_id: VarULong(entity_id as u64),
            platform_chat_id: String::new(),
            position: Vector3::new(position.x as f32, position.y as f32, position.z as f32),
            velocity: Vector3::new(velocity.x as f32, velocity.y as f32, velocity.z as f32),
            rotation: Vector2::new(pitch, yaw),
            y_head_rotation: yaw,
            carried_item: NetworkItemStackDescriptor::default(),
            player_game_type: player.gamemode.load().into(),
            entity_data: player.get_entity().bedrock_metadata(),
            synced_properties: PropertySyncData::default(),
            abilities_data: pumpkin_protocol::bedrock::client::SerializedAbilitiesData {
                target_player_raw_id: entity_id as i64,
                player_permissions:
                    pumpkin_protocol::bedrock::client::PlayerPermissionLevel::Visitor,
                command_permissions: pumpkin_protocol::bedrock::client::CommandPermissionLevel::Any,
                layers: vec![
                    pumpkin_protocol::bedrock::client::SerializedAbilitiesDataSerializedLayer {
                        serialized_layer: 0,
                        abilities_set: 0,
                        ability_value: 0,
                        fly_speed: 0.05,
                        vertical_fly_speed: 0.05,
                        walk_speed: 0.1,
                    },
                ],
            },
            actor_links: Vec::new(),
            device_id: String::new(),
            build_platform: BuildPlatform::Unknown,
        };

        // Spawn the player for every client.
        let spawn_entity = CSpawnEntity::new(
            entity_id.into(),
            gameprofile.id,
            i32::from(EntityType::PLAYER.id).into(),
            position,
            pitch,
            yaw,
            yaw,
            0.into(),
            velocity,
        );

        self.broadcast_packet_except_editioned(
            &[player.gameprofile.id],
            &spawn_entity,
            &bedrock_add_player,
        );

        // Broadcast metadata to Java players so they can correctly interact with the new player
        let skin_parts = player.config.load().skin_parts;

        self.broadcast_skin_parts(
            &[gameprofile.id],
            entity_id,
            skin_parts,
            &CSetActorData {
                target_runtime_id: VarULong(entity_id as u64),
                actor_data: player.get_entity().bedrock_metadata(),
                synced_properties: PropertySyncData {
                    int_entries_list: HashMap::new(),
                    float_entries_list: HashMap::new(),
                },
                tick: VarULong(0),
            },
        );

        // Spawn players for our client.
        let id = player.gameprofile.id;
        for existing_player in self
            .players
            .load()
            .iter()
            .filter(|c| c.gameprofile.id != id)
        {
            let entity = &existing_player.get_entity();
            let pos = entity.pos.load();
            let gameprofile = &existing_player.gameprofile;
            let bedrock_add_player = CAddPlayer {
                uuid: gameprofile.id,
                player_name: gameprofile.name.clone(),
                target_runtime_id: VarULong(existing_player.entity_id() as u64),
                platform_chat_id: String::new(),
                position: Vector3::new(pos.x as f32, pos.y as f32, pos.z as f32),
                velocity: Vector3::new(
                    entity.velocity.load().x as f32,
                    entity.velocity.load().y as f32,
                    entity.velocity.load().z as f32,
                ),
                rotation: Vector2::new(entity.pitch.load(), entity.yaw.load()),
                y_head_rotation: entity.head_yaw.load(),
                carried_item: NetworkItemStackDescriptor::default(),
                player_game_type: existing_player.gamemode.load().into(),
                entity_data: entity.bedrock_metadata(),
                synced_properties: PropertySyncData::default(),
                abilities_data: pumpkin_protocol::bedrock::client::SerializedAbilitiesData {
                    target_player_raw_id: existing_player.entity_id() as i64,
                    player_permissions:
                        pumpkin_protocol::bedrock::client::PlayerPermissionLevel::Visitor,
                    command_permissions:
                        pumpkin_protocol::bedrock::client::CommandPermissionLevel::Any,
                    layers: vec![
                        pumpkin_protocol::bedrock::client::SerializedAbilitiesDataSerializedLayer {
                            serialized_layer: 0,
                            abilities_set: 0,
                            ability_value: 0,
                            fly_speed: 0.05,
                            vertical_fly_speed: 0.05,
                            walk_speed: 0.1,
                        },
                    ],
                },
                actor_links: Vec::new(),
                device_id: String::new(),
                build_platform: BuildPlatform::Unknown,
            };

            let bedrock_player_list = CPlayerList {
                action: CPlayerList::ACTION_ADD,
                entries: vec![PlayerListEntry {
                    uuid: gameprofile.id,
                    entity_unique_id: VarLong(existing_player.entity_id() as i64),
                    username: gameprofile.name.clone(),
                    xuid: String::new(),
                    platform_chat_id: String::new(),
                    build_platform: BuildPlatform::Unknown,
                    skin: (**existing_player.bedrock_skin.load()).clone(),
                    is_teacher: false,
                    is_host: false,
                    is_sub_client: false,
                    player_color: [0, 0, 0, 0],
                }],
            };

            let actions = [
                PlayerAction::AddPlayer {
                    name: &gameprofile.name,
                    properties: &gameprofile.properties.load(),
                },
                PlayerAction::UpdateGameMode(VarInt(existing_player.gamemode.load() as i32)),
                PlayerAction::UpdateListed(existing_player.tab_list_listed.load(Ordering::Relaxed)),
                PlayerAction::UpdateLatency(VarInt(
                    existing_player.tab_list_latency.load(Ordering::Relaxed),
                )),
                PlayerAction::UpdateListOrder(VarInt(
                    existing_player.tab_list_order.load(Ordering::Relaxed),
                )),
                PlayerAction::UpdateHat(true),
            ];
            let java_player = [pumpkin_protocol::java::client::play::Player {
                uuid: gameprofile.id,
                actions: &actions,
            }];
            player
                .client
                .enqueue_packet_editioned(
                    &CPlayerInfoUpdate::new(
                        (PlayerInfoFlags::ADD_PLAYER
                            | PlayerInfoFlags::UPDATE_LISTED
                            | PlayerInfoFlags::UPDATE_GAME_MODE
                            | PlayerInfoFlags::UPDATE_LATENCY
                            | PlayerInfoFlags::UPDATE_LIST_PRIORITY
                            | PlayerInfoFlags::UPDATE_HAT)
                            .bits(),
                        &java_player,
                    ),
                    &bedrock_player_list,
                )
                .await;

            player
                .client
                .enqueue_packet_editioned(
                    &CSpawnEntity::new(
                        existing_player.entity_id().into(),
                        gameprofile.id,
                        i32::from(EntityType::PLAYER.id).into(),
                        pos,
                        entity.pitch.load(),
                        entity.yaw.load(),
                        entity.head_yaw.load(),
                        0.into(),
                        entity.velocity.load(),
                    ),
                    &bedrock_add_player,
                )
                .await;

            if client.version.load() >= CURRENT_MC_VERSION {
                let config = existing_player.config.load();
                let mut buf = Vec::new();
                {
                    let meta = Metadata::new(
                        pumpkin_data::tracked_data::player::PLAYER_MODE_CUSTOMISATION,
                        config.skin_parts,
                    );
                    let _ = meta.write(&mut buf, &client.version.load());
                };
                {
                    let meta = Metadata::new(
                        pumpkin_data::tracked_data::player::PLAYER_MODE_CUSTOMIZATION_ID,
                        config.skin_parts,
                    );
                    let _ = meta.write(&mut buf, &client.version.load());
                };
                drop(config);
                // END
                buf.put_u8(255);
                client
                    .enqueue_client_packet(&CSetEntityMetadata::new(
                        existing_player.get_entity().entity_id.into(),
                        buf.into(),
                    ))
                    .await;
            }

            {
                let held_item = existing_player.inventory.held_item();
                let equipment_list = {
                    let mut equipment_list =
                        vec![(EquipmentSlot::MAIN_HAND.discriminant(), held_item.clone())];

                    let equipment_guard = existing_player
                        .inventory
                        .entity_equipment
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    for (slot, item_stack) in &equipment_guard.equipment {
                        equipment_list.push((slot.discriminant(), item_stack.clone()));
                    }
                    equipment_list
                };

                let equipment: Vec<(i8, ItemStackSerializer)> = equipment_list
                    .iter()
                    .map(|(slot, stack)| (*slot, ItemStackSerializer::from(stack.clone())))
                    .collect();

                let je_packet = CSetEquipment::new(existing_player.entity_id().into(), equipment);

                let be_mob_equipment = pumpkin_protocol::bedrock::client::CMobEquipment {
                    target_runtime_id: (existing_player.entity_id() as u64).into(),
                    item: (&held_item).into(),
                    slot: 0,
                    selected_slot: 0,
                    container_id: 0,
                };

                player
                    .client
                    .enqueue_packet_editioned(&je_packet, &be_mob_equipment)
                    .await;
            }
        }
        player.send_client_information();

        player.send_abilities_update();

        // Sync selected slot
        player.enqueue_set_held_item_packet(&CSetSelectedSlot::new(
            player.get_inventory().get_selected_slot() as i8,
        ));

        if client.version.load() >= JavaMinecraftVersion::V_1_20_2 {
            // Start waiting for level chunks. Sets the "Loading Terrain" screen (Added in 1.20.2)
            debug!("Sending waiting chunks to {}", player.gameprofile.name);
            client
                .send_packet(&CGameEvent::new(GameEvent::StartWaitingChunks, 0.0))
                .await;
        }

        self.worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .init_client(client);

        // Sends initial time
        player.send_time(self);

        // Sends initial scoreboard state
        player.send_scoreboard();

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
            .send_packet(&CPlayerSpawnPosition::new(
                spawn_block_pos,
                yaw,
                pitch,
                self.dimension.minecraft_name.to_owned(),
            ))
            .await;

        // Send initial weather state
        let (is_raining, rain_level, thunder_level) = {
            let weather = self
                .weather
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (
                weather.raining,
                weather.rain_level.clamp(0.0, 1.0),
                weather.thunder_level.clamp(0.0, 1.0),
            )
        };
        if is_raining {
            client
                .enqueue_client_packet(&CGameEvent::new(GameEvent::BeginRaining, 0.0))
                .await;

            client
                .enqueue_client_packet(&CGameEvent::new(GameEvent::RainLevelChange, rain_level))
                .await;
            client
                .enqueue_client_packet(&CGameEvent::new(
                    GameEvent::ThunderLevelChange,
                    thunder_level,
                ))
                .await;
        }

        let player_bossbars = server
            .bossbars
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_player_bars(&player.gameprofile.id)
            .map(|bars| bars.into_iter().cloned().collect::<Vec<_>>());
        if let Some(bossbars) = player_bossbars {
            for bossbar in &bossbars {
                player.send_bossbar(bossbar);
            }
        }

        player.has_played_before.store(true, Ordering::Relaxed);
        player.on_screen_handler_opened(&player.player_screen_handler);

        player.send_active_effects();
        player.breath_manager.send_air_supply(player);
        self.send_player_equipment(player);

        if let crate::net::ClientPlatform::Java(java_client) = player.client.as_ref()
            && server.advanced_config.recipe.send_recipes
            && java_client.version.load() >= JavaMinecraftVersion::V_1_21_2
        {
            let settings_packet = CRecipeBookSettings::default_closed();
            if let Ok(data) = java_client.serialize_packet(&settings_packet) {
                java_client.send_packet_now(data).await;
            }
            let dynamic_recipes = server.recipe_manager.get_dynamic_recipes();
            let add_packet = CRecipeBookAdd::new(true, &dynamic_recipes);
            if let Ok(data) = java_client.serialize_packet(&add_packet) {
                java_client.send_packet_now(data).await;
            }
        }
        let msg_comp = TextComponent::translate_cross(
            translation::java::MULTIPLAYER_PLAYER_JOINED,
            translation::bedrock::MULTIPLAYER_PLAYER_JOINED,
            [TextComponent::text(player.gameprofile.name.clone())],
        )
        .color_named(NamedColor::Yellow);
        let mut event = PlayerJoinEvent::new(player.clone(), msg_comp);

        server.plugin_manager.fire(server, &mut event).await;

        if !event.cancelled {
            self.broadcast_system_message(&event.join_message, false);
            // TODO: Switch to structured logging, e.g. info!(player = %name, "connected")
            info!("{}", event.join_message.to_pretty_console());
        }
    }

    fn send_player_equipment(&self, from: &Player) {
        let held_item = from.inventory.held_item();
        let mut equipment_list = vec![(EquipmentSlot::MAIN_HAND.discriminant(), held_item.clone())];

        let equipment_guard = from
            .inventory
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (slot, item_stack) in &equipment_guard.equipment {
            equipment_list.push((slot.discriminant(), item_stack.clone()));
        }
        drop(equipment_guard);

        let equipment: Vec<(i8, ItemStackSerializer)> = equipment_list
            .iter()
            .map(|(slot, stack)| (*slot, ItemStackSerializer::from(stack.clone())))
            .collect();
        let je_packet = CSetEquipment::new(from.entity_id().into(), equipment);

        let be_mob_equipment = pumpkin_protocol::bedrock::client::CMobEquipment {
            target_runtime_id: (from.entity_id() as u64).into(),
            item: (&held_item).into(),
            slot: 0,
            selected_slot: 0,
            container_id: 0,
        };

        let chunk_pos = from.get_entity().chunk_pos.load();
        self.broadcast_to_chunk_except_editioned(
            chunk_pos,
            &[from.get_entity().entity_uuid],
            &je_packet,
            &be_mob_equipment,
        );
    }

    pub async fn send_world_info(
        &self,
        player: &Arc<Player>,
        position: Vector3<f64>,
        yaw: f32,
        pitch: f32,
    ) {
        if let ClientPlatform::Java(client) = player.client.as_ref() {
            self.worldborder
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .init_client(client);
        }

        // TODO: World spawn (compass stuff)

        if let ClientPlatform::Java(client) = player.client.as_ref()
            && client.version.load() >= JavaMinecraftVersion::V_1_20_2
        {
            player
                .send_client_packet(&CGameEvent::new(GameEvent::StartWaitingChunks, 0.0))
                .await;
        }

        let entity = &player.get_entity();

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
        );

        player.send_client_information();

        chunker::update_position(player).await;
        // Update commands

        player.set_health(20.0);
    }

    pub fn explode(
        self: &Arc<Self>,
        position: Vector3<f64>,
        power: f32,
        interaction: ExplosionInteraction,
    ) {
        self.explode_with_calculator(position, power, interaction, None);
    }

    pub fn explode_with_calculator(
        self: &Arc<Self>,
        position: Vector3<f64>,
        power: f32,
        interaction: ExplosionInteraction,
        damage_calculator: Option<Arc<dyn ExplosionDamageCalculator>>,
    ) {
        let block_interaction = self.get_block_interaction(interaction);
        let mut explosion = Explosion::new(power, position, block_interaction);
        if let Some(calc) = damage_calculator {
            explosion = explosion.with_damage_calculator(calc);
        }
        self.run_explosion(&explosion, position, power);
    }

    pub fn explode_tnt_minecart(self: &Arc<Self>, position: Vector3<f64>, power: f32) {
        let block_interaction = self.get_block_interaction(ExplosionInteraction::Tnt);
        let explosion = Explosion::new(power, position, block_interaction).preserving_rails();
        self.run_explosion(&explosion, position, power);
    }

    #[must_use]
    pub fn get_block_interaction(&self, interaction: ExplosionInteraction) -> BlockInteraction {
        let game_rules = &self.level_info.load().game_rules;
        match interaction {
            ExplosionInteraction::None => BlockInteraction::Keep,
            ExplosionInteraction::Block => {
                Self::get_destroy_type(game_rules.block_explosion_drop_decay)
            }
            ExplosionInteraction::Mob => {
                if game_rules.mob_griefing {
                    Self::get_destroy_type(game_rules.mob_explosion_drop_decay)
                } else {
                    BlockInteraction::Keep
                }
            }
            ExplosionInteraction::Tnt => {
                Self::get_destroy_type(game_rules.tnt_explosion_drop_decay)
            }
            ExplosionInteraction::Trigger => BlockInteraction::TriggerBlock,
        }
    }

    #[must_use]
    pub const fn get_destroy_type(drop_decay: bool) -> BlockInteraction {
        if drop_decay {
            BlockInteraction::DestroyWithDecay
        } else {
            BlockInteraction::Destroy
        }
    }

    fn run_explosion(self: &Arc<Self>, explosion: &Explosion, position: Vector3<f64>, power: f32) {
        let mut event = crate::plugin::api::events::entity::entity_explode::EntityExplodeEvent::new(
            0, position, power,
        );
        if let Some(server) = self.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            return;
        }

        let block_count = explosion.explode(self);
        let particle = if power < 2.0 {
            Particle::Explosion
        } else {
            Particle::ExplosionEmitter
        };
        for player in self.players.load().iter() {
            let mut sound_id = Sound::EntityGenericExplode as u16;
            if let ClientPlatform::Java(java_client) = player.client.as_ref() {
                sound_id = remap_sound_id_for_version(sound_id, java_client.version.load());
            }
            let sound = IdOr::<SoundEvent>::Id(sound_id);
            if player.position().squared_distance_to_vec(&position) > 4096.0 {
                continue;
            }
            player.try_send_client_packet(&CExplosion::new(
                position,
                power,
                block_count as i32,
                None,
                VarInt(particle as i32),
                sound,
            ));
        }
    }

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

        let server = self.server.upgrade();
        let default_world = server.as_ref().map_or_else(
            || self.clone(),
            |s| s.get_world_from_dimension(&Dimension::OVERWORLD),
        );

        // Copy spawn info from default world level_info to avoid holding lock across await
        let (spawn_x, spawn_y, spawn_z, spawn_yaw, spawn_pitch, keep_inventory) = {
            let info = default_world.level_info.load();
            (
                info.spawn_x,
                info.spawn_y,
                info.spawn_z,
                info.spawn_yaw,
                info.spawn_pitch,
                info.game_rules.keep_inventory,
            )
        };

        // Get respawn position and dimension
        let (position, yaw, pitch, respawn_dimension) = if let Some(respawn) =
            player.calculate_respawn_point().await
        {
            (
                respawn.position,
                respawn.yaw,
                respawn.pitch,
                respawn.dimension,
            )
        } else {
            // No valid respawn point - send notification if player had one set
            if player
                .respawn_point
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
            {
                player
                    .send_client_packet(&CGameEvent::new(GameEvent::NoRespawnBlockAvailable, 0.0))
                    .await;
                let mut guard = player
                    .respawn_point
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(point) = guard.as_ref()
                    && !point.force
                {
                    *guard = None;
                }
            }

            // FIXME: This spawn position calculation is incorrect. Should use vanilla's
            // proper spawn position calculation (see #1381). The y-level calculation
            // needs to account for spawn radius and find a safe spawn position.
            let chunk_pos = Vector2::new(spawn_x >> 4, spawn_z >> 4);
            default_world
                .level
                .get_or_fetch_chunk(chunk_pos, |_| ())
                .await;
            let top = default_world.get_top_block(Vector2::new(spawn_x, spawn_z));
            let pos_y = if top > default_world.dimension.min_y {
                top + 1
            } else {
                spawn_y
            };

            (
                Vector3::new(
                    f64::from(spawn_x) + 0.5,
                    f64::from(pos_y),
                    f64::from(spawn_z) + 0.5,
                ),
                spawn_yaw,
                spawn_pitch,
                default_world.dimension.clone(),
            )
        };

        let mut spawn_loc_event = crate::plugin::api::events::player::player_spawn_location::PlayerSpawnLocationEvent::new(
            player.clone(),
            position,
        );
        if let Some(ref s) = server {
            s.plugin_manager.fire(s, &mut spawn_loc_event).await;
        }
        let position = spawn_loc_event.spawn_pos;

        // Candidate destination world for a cross-dimension respawn.
        let candidate_world = if respawn_dimension == self.dimension {
            None
        } else {
            server.as_ref().map_or_else(
                || {
                    warn!("Could not get server for cross-dimension respawn");
                    None
                },
                |s| {
                    let worlds = s.worlds.load();
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
            if let Some(ref s) = server {
                let mut event = PlayerChangeWorldEvent {
                    player: player.clone(),
                    previous_world: self.clone(),
                    new_world: new_world.clone(),
                    position,
                    yaw,
                    pitch,
                    cancelled: false,
                };
                s.plugin_manager.fire(s, &mut event).await;

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
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
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
        let (target_world, position, yaw, pitch) = resolved_world.as_ref().map_or_else(
            || (self.clone(), position, yaw, pitch),
            |new_world| (new_world.clone(), position, yaw, pitch),
        );

        // Notify plugins that the player has respawned (non-cancellable).
        if let Some(server) = self.server.upgrade() {
            server
                .plugin_manager
                .fire(
                    &server,
                    &mut PlayerRespawnEvent::new(
                        player.clone(),
                        self.clone(),
                        target_world.clone(),
                        position,
                        yaw,
                        pitch,
                        alive,
                    ),
                )
                .await;
        }

        // Send respawn packet with target dimension (using send_packet_now to ensure proper order)
        player
            .send_client_packet(&CRespawn::new(
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
            .send_packet_now_editioned(
                &CPlayerSpawnPosition::new(
                    spawn_block_pos,
                    yaw,
                    pitch,
                    target_world.dimension.minecraft_name.to_string(),
                ),
                &pumpkin_protocol::bedrock::client::CSetSpawnPosition {
                    spawn_position_type:
                        pumpkin_protocol::bedrock::client::SpawnPositionType::WorldRespawn,
                    block_position: spawn_block_pos,
                    dimension_type: bedrock_dimension.into(),
                    spawn_block_pos,
                },
            )
            .await;

        player.living_entity.reset_state();

        player.send_permission_lvl_update();

        player.hunger_manager.restart();

        if !keep_inventory {
            player.set_experience(0, 0.0, 0);
            player.inventory.clear();
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
            java_client.send_chunks(&[chunk]).await;
        }

        // Send teleport packet after at least the center chunk was delivered
        player.request_teleport(position, yaw, pitch).await;
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

                let Some((chunk_weak, first_load)) = recv_result else {
                    break;
                };

                let Some(chunk) = chunk_weak.upgrade() else {
                    continue;
                };

                let position = Vector2::new(chunk.x, chunk.z);

                if !level.is_chunk_watched(&position) {
                    // No longer watched: don't make its entities live. Leave the
                    // serialized data untouched so the normal unload path persists
                    // it as-is (nothing went live, so there is nothing to save).
                    trace!(
                        "Received entity chunk {:?}, but it is no longer watched; leaving it for the unload path",
                        &position
                    );
                    continue 'main;
                }

                if first_load {
                    // First watcher: consume the serialized entities and make them
                    // live. The live entity list becomes the single source of
                    // truth, so the chunk's NBT is taken (cleared) to avoid keeping
                    // a duplicate copy that would be re-appended on the next unload
                    // and doubled on every reload.
                    let entity_nbts = std::mem::take(
                        &mut *chunk
                            .data
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner),
                    );
                    let mut entities_to_add: Vec<Arc<dyn EntityBase>> =
                        Vec::with_capacity(entity_nbts.len());
                    for entity_nbt in &entity_nbts {
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

                        // Keep the persisted UUID so the entity keeps its identity
                        // across reloads (matching vanilla); only fall back to a
                        // fresh one if it is missing/corrupt.
                        let uuid = entity_nbt.get_uuid("UUID").unwrap_or_else(Uuid::new_v4);
                        // Pos is zero since it will be read from nbt.
                        let entity =
                            from_type(entity_type, Vector3::new(0.0, 0.0, 0.0), &world, uuid);
                        entity.read_nbt_non_mut(entity_nbt);
                        entity.init_data_tracker();

                        let base_entity = entity.get_entity();
                        // Clear velocity so the client does not replay the drop
                        // animation; residual velocity from the original drop is
                        // stale data.
                        base_entity.velocity.store(Vector3::default());

                        player.client.enqueue_spawn_packet(&entity).await;
                        player.try_restore_vehicle(&entity);
                        entities_to_add.push(entity);
                    }

                    if !entities_to_add.is_empty() {
                        world.entities.rcu(|current_entities| {
                            let mut new_entities = (**current_entities).clone();
                            new_entities.extend(entities_to_add.iter().cloned());
                            new_entities
                        });
                    }
                } else {
                    // The chunk's entities are already live (another watcher loaded
                    // them). Just send this player the spawn packets for the live
                    // entities currently in this chunk.
                    for entity in world.entities.load().iter() {
                        let base_entity = entity.get_entity();
                        if base_entity.chunk_pos.load() == position {
                            player.client.enqueue_spawn_packet(entity).await;
                            player.try_restore_vehicle(entity);
                        }
                    }
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

    /// Retrieves an entity by their unique UUID.
    ///
    /// This function searches the world's entities for one with the specified UUID.
    /// If found, it returns an `Arc<dyn EntityBase>` reference to that entity. Otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `id`: The UUID of the entity to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<Arc<dyn EntityBase>>` containing the player if found, or `None` if not.
    pub fn get_entity_by_uuid(&self, id: uuid::Uuid) -> Option<Arc<dyn EntityBase>> {
        self.entities
            .load()
            .iter()
            .find(|p| p.get_entity().entity_uuid == id)
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
                a.get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&pos)
                    .total_cmp(&b.get_entity().pos.load().squared_distance_to_vec(&pos))
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
                    .total_cmp(&b.1.get_entity().pos.load().squared_distance_to_vec(&pos))
            })
            .map(|p| p.1.clone())
    }

    /// Adds entities to the provided [`Vec`] that satisfy a particular condition and are
    /// present in the provided [`BoundingBox`].
    ///
    /// # Arguments
    ///
    /// * `list`: The `Vec` to add to.
    /// * `max_list_capacity`: The maximum capacity of `list` for adding entities. If this limit is reached, no more
    ///   entities will be added to the list. If `list` already reaches this limit, nothing happens.
    /// * `bounding_box`: The bounding box to filter any added entities.
    /// * `predicate`: A predicate function, which has to be `true` for an entity to be added to the list.
    pub fn extend_entities_in_box_where(
        &self,
        list: &mut Vec<Arc<dyn EntityBase>>,
        max_list_capacity: usize,
        bounding_box: BoundingBox,
        predicate: impl Fn(&dyn EntityBase) -> bool,
    ) {
        self.extend_entities_where(list, max_list_capacity, |e| {
            bounding_box.intersects(&e.get_entity().bounding_box.load()) && predicate(e)
        });
    }

    /// Adds entities to the provided [`Vec`] that satisfy a particular condition.
    ///
    /// # Arguments
    ///
    /// * `list`: The `Vec` to add to.
    /// * `max_list_capacity`: The maximum capacity of `list` for adding entities. If this limit is reached, no more
    ///   entities will be added to the list. If `list` already reaches this limit, nothing happens.
    /// * `predicate`: A predicate function, which has to be `true` for an entity to be added to the list.
    pub fn extend_entities_where(
        &self,
        list: &mut Vec<Arc<dyn EntityBase>>,
        max_list_capacity: usize,
        predicate: impl Fn(&dyn EntityBase) -> bool,
    ) {
        if list.len() >= max_list_capacity {
            return;
        }
        // Loop the players.
        for player in self.players.load().iter() {
            if !predicate(player.as_ref()) {
                continue;
            }
            // We add the player to the list.
            list.push(player.clone());
            // Check if the list is too big.
            if list.len() > max_list_capacity {
                return;
            }
        }
        // Same with entities.
        for entity in self.entities.load().iter() {
            if !predicate(entity.as_ref()) {
                continue;
            }
            list.push(entity.clone());
            if list.len() > max_list_capacity {
                return;
            }
            // TODO: Implement ender dragon handling
        }
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
            let pos = new_list
                .iter()
                .position(|p| p.gameprofile.id == player.gameprofile.id);
            if let Some(pos) = pos {
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
                    build_platform: BuildPlatform::Unknown,
                    skin: Skin::steve(),
                    is_teacher: false,
                    is_host: false,
                    is_sub_client: false,
                    player_color: [0, 0, 0, 0],
                }],
            };

            self.broadcast_editioned(&CRemovePlayerInfo::new(&[uuid]), &bedrock_remove_player);

            self.broadcast_editioned(
                &CRemoveEntities::new(&[entity_id.into()]),
                &CRemoveActor::new(VarLong(entity_id as i64)),
            );

            if fire_event {
                let msg_comp = TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_PLAYER_LEFT,
                    translation::bedrock::MULTIPLAYER_PLAYER_LEFT,
                    [TextComponent::text(player.gameprofile.name.clone())],
                )
                .color_named(NamedColor::Yellow);
                let mut event = PlayerLeaveEvent::new(player.clone(), msg_comp);

                if let Some(server) = self.server.upgrade() {
                    server.plugin_manager.fire(&server, &mut event).await;

                    if !event.cancelled {
                        for player in self.players.load().iter() {
                            player.send_system_message(&event.leave_message);
                        }
                        info!("{}", event.leave_message.to_pretty_console());
                    }
                }
            }
        }
        removed_player
    }

    #[expect(clippy::needless_pass_by_value)]
    pub fn spawn_entity_non_save(&self, entity: Arc<dyn EntityBase>) {
        let _base_entity = entity.get_entity();
        self.broadcast_entity_spawn(&entity);
        self.spawn_state.load().add_entity(self, entity.as_ref());

        self.entities.rcu(|current_entities| {
            let mut new_entities = (**current_entities).clone();
            new_entities.push(entity.clone());
            new_entities
        });
    }

    pub fn spawn_entity(self: &Arc<Self>, entity: Arc<dyn EntityBase>) {
        let mut event = crate::plugin::api::events::entity::entity_spawn::EntitySpawnEvent::new(
            entity.get_entity().entity_id,
            entity.get_entity().entity_type.id.to_string(),
            entity.get_entity().pos.load(),
            self.clone(),
        );
        if let Some(server) = self.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            return;
        }

        self.broadcast_entity_spawn(&entity);
        entity.init_data_tracker();
        self.add_entity_silent(entity);
    }

    pub fn broadcast_entity_spawn(&self, entity: &Arc<dyn EntityBase>) {
        let base_entity = entity.get_entity();
        let chunk_pos = base_entity.chunk_pos.load();

        let players = self.players.load();
        for player in players.iter() {
            let center = player.get_entity().chunk_pos.load();
            let view_distance = get_view_distance(player).get() as i32;

            if is_within_view_distance(chunk_pos, center, view_distance) {
                player.client.try_enqueue_spawn_packet(entity);
            }
        }
    }

    #[expect(clippy::needless_pass_by_value)]
    pub fn add_entity_silent(&self, entity: Arc<dyn EntityBase>) {
        let base_entity = entity.get_entity();

        // Guard against duplicate entities with the same UUID.
        // This can happen when chunk entity data is loaded while the entity
        // already exists in the world (e.g. another player is still tracking it).
        let already_exists = self
            .entities
            .load()
            .iter()
            .any(|e| e.get_entity().entity_uuid == base_entity.entity_uuid);
        if already_exists {
            return;
        }

        // The entity stays live-only: it is written to its chunk's saved data on
        // unload (see `save_entity`), never at spawn, so it can't be both live and
        // serialized at once (which would double it on the next reload).
        self.spawn_state.load().add_entity(self, entity.as_ref());

        self.entities.rcu(|current_entities| {
            let mut new_entities = (**current_entities).clone();
            new_entities.push(entity.clone());
            new_entities
        });
    }

    pub fn remove_entity(&self, entity: &dyn EntityBase) {
        let base_entity = entity.get_entity();
        if base_entity
            .removal_reason
            .swap(Some(RemovalReason::Discarded))
            .is_some()
        {
            return;
        }
        base_entity.removed.store(true, Ordering::Release);

        self.spawn_state.load().remove_entity(self, entity);
        self.entities.rcu(|current_entities| {
            let mut new_entities = (**current_entities).clone();
            new_entities.retain(|e| e.get_entity().entity_uuid != base_entity.entity_uuid);
            new_entities
        });

        let chunk_pos = base_entity.chunk_pos.load();
        self.broadcast_to_chunk_editioned(
            chunk_pos,
            &CRemoveEntities::new(&[base_entity.entity_id.into()]),
            &CRemoveActor::new(VarLong(base_entity.entity_id as i64)),
        );
    }

    pub async fn remove_entities_in_chunks(
        &self,
        chunks: impl IntoIterator<Item = impl std::borrow::Borrow<Vector2<i32>>>,
    ) {
        let chunks_set: FxHashSet<_> = chunks.into_iter().map(|c| *c.borrow()).collect();
        if chunks_set.is_empty() {
            return;
        }
        let mut entities_to_remove = Vec::new();

        self.entities.rcu(|current_entities| {
            let mut new_entities = (**current_entities).clone();
            new_entities.retain(|entity| {
                let base_entity = entity.get_entity();
                let pos = base_entity.chunk_pos.load();
                if chunks_set.contains(&pos) {
                    entities_to_remove.push(entity.clone());
                    false
                } else {
                    true
                }
            });
            new_entities
        });

        for entity in entities_to_remove {
            self.save_entity(&entity).await;
            self.spawn_state.load().remove_entity(self, entity.as_ref());
        }

        for chunk_pos in &chunks_set {
            self.save_block_entities(*chunk_pos);
            self.block_entities.remove(chunk_pos);
        }
    }

    pub(crate) fn set_block_breaking(
        &self,
        from: &Entity,
        location: BlockPos,
        progress: BlockBreakingProgress,
    ) {
        let chunk_pos = location.chunk_position(); // pumpkin's BlockPos already has this method
        let (stage, bedrock_event) = match progress {
            BlockBreakingProgress::Start { stage, speed } => (
                stage,
                Some((
                    LevelEvent::BlockStartBreak,
                    bedrock_block_breaking_rate(speed),
                )),
            ),
            BlockBreakingProgress::Update { stage, speed } => (
                stage,
                speed.map(|speed| {
                    (
                        LevelEvent::BlockUpdateBreak,
                        bedrock_block_breaking_rate(speed),
                    )
                }),
            ),
            BlockBreakingProgress::Stop => (-1, Some((LevelEvent::BlockStopBreak, 0))),
        };
        let je_packet = CSetBlockDestroyStage::new(from.entity_id.into(), location, stage as i8);

        if let Some((event_id, data)) = bedrock_event {
            let be_packet = CLevelEvent {
                event_id: VarInt(event_id as i32),
                position: Vector3::new(
                    location.0.x as f32,
                    location.0.y as f32,
                    location.0.z as f32,
                ),
                data: VarInt(data),
            };

            if let Some(player) = self.get_player_by_uuid(from.entity_uuid)
                && let ClientPlatform::Bedrock(client) = player.client.as_ref()
                && let Ok(packet_data) = client.serialize_packet(&be_packet)
            {
                client.try_enqueue_packet(packet_data);
            }

            self.broadcast_to_chunk_except_editioned(
                chunk_pos,
                &[from.entity_uuid],
                &je_packet,
                &be_packet,
            );
        } else {
            self.broadcast_to_chunk_except(chunk_pos, &[from.entity_uuid], &je_packet);
        }
    }

    pub fn set_block_state(
        self: &Arc<Self>,
        position: &BlockPos,
        block_state_id: BlockStateId,
        _flags: BlockFlags,
    ) -> BlockStateId {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let replaced_block_state_id = self
            .level
            .read_chunk_sync(&chunk_coordinate, |chunk| {
                let replaced_block_state_id = chunk.set_block_absolute_y(
                    relative.x as usize,
                    relative.y,
                    relative.z as usize,
                    block_state_id,
                );
                // Mark chunk dirty if it isn't already
                if replaced_block_state_id != block_state_id && !chunk.is_dirty() {
                    chunk.mark_dirty(true);
                }
                replaced_block_state_id
            })
            .unwrap_or(Block::AIR.default_state.id);

        if replaced_block_state_id == block_state_id {
            return block_state_id;
        }

        self.unsent_block_changes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(*position, block_state_id);

        let old_block = Block::from_state_id(replaced_block_state_id);
        let new_block = Block::from_state_id(block_state_id);

        self.villager_poi
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .update_block(*position, new_block);

        let is_new_block = old_block != new_block;

        if is_new_block {
            let mut poi = self
                .portal_poi
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if villager_poi::profession_for_block(old_block).is_some() {
                poi.remove(position);
            }
            if let Some(poi_type) = villager_poi::poi_type_for_block(new_block) {
                poi.add_with_free_tickets(*position, poi_type, 1);
            }
        }

        if is_new_block
            && old_block.default_state.block_entity_type != u16::MAX
            && self.get_block_entity(position).is_some()
        {
            self.remove_block_entity(position);
        }

        self.level
            .light_engine
            .update_lighting_at(&self.level, *position);

        replaced_block_state_id
    }

    pub fn break_block(
        self: &Arc<Self>,
        position: &BlockPos,
        _cause: Option<Arc<Player>>,
        flags: BlockFlags,
    ) -> Option<BlockStateId> {
        let (broken_block, broken_block_state) = self.get_block_and_state_id(position);
        if is_air(broken_block_state) {
            return None;
        }
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
            Block::WATER.default_state.id
        } else {
            Block::AIR.default_state.id
        };

        Some(self.set_block_state(position, new_state_id, flags))
    }

    pub fn get_max_local_raw_brightness(&self, pos: &BlockPos) -> u8 {
        let sky_light = self.get_sky_light_level(pos);
        let block_light = self.get_block_light_level(pos).unwrap_or(0);
        sky_light.max(block_light) // TODO: getSkyDarken
    }

    pub fn get_block_light_level(&self, position: &BlockPos) -> Option<u8> {
        self.level
            .light_engine
            .get_block_light_level(&self.level, position)
    }

    pub fn get_sky_light_level(&self, position: &BlockPos) -> u8 {
        self.level
            .light_engine
            .get_sky_light_level(&self.level, position)
    }

    #[must_use]
    pub fn can_see_sky(&self, position: &BlockPos) -> bool {
        position.0.y >= self.dimension.min_y
            && position.0.y < self.dimension.min_y + self.dimension.height
            && self.get_sky_light_level(position) >= MAX_LIGHT_LEVEL
    }

    pub fn set_block_light_level(&self, position: &BlockPos, light_level: u8) {
        let _ = self
            .level
            .light_engine
            .set_block_light_level(&self.level, position, light_level);
    }

    pub fn set_sky_light_level(&self, position: &BlockPos, light_level: u8) {
        let _ = self
            .level
            .light_engine
            .set_sky_light_level(&self.level, position, light_level);
    }

    pub fn get_biome(&self, position: &BlockPos) -> &'static Biome {
        let chunk_pos = position.chunk_position();
        if let Some(chunk) = self.level.loaded_chunks.get(&chunk_pos) {
            let id = chunk
                .section
                .get_rough_biome_absolute_y(
                    (position.0.x & 15) as usize,
                    position.0.y,
                    (position.0.z & 15) as usize,
                )
                .unwrap_or(0);
            Biome::from_id(id).unwrap_or(&Biome::PLAINS)
        } else {
            &Biome::PLAINS
        }
    }

    pub fn schedule_block_tick(
        &self,
        block: &Block,
        block_pos: BlockPos,
        delay: u8,
        priority: TickPriority,
    ) {
        self.level
            .schedule_block_tick(block, block_pos, delay, priority);
    }

    pub fn schedule_fluid_tick(
        &self,
        fluid: &Fluid,
        block_pos: BlockPos,
        delay: u8,
        priority: TickPriority,
    ) {
        self.level
            .schedule_fluid_tick(fluid, block_pos, delay, priority);
    }

    pub fn is_block_tick_scheduled(&self, block_pos: &BlockPos, block: &Block) -> bool {
        self.level.is_block_tick_scheduled(block_pos, block)
    }

    pub fn is_fluid_tick_scheduled(&self, block_pos: &BlockPos, fluid: &Fluid) -> bool {
        self.level.is_fluid_tick_scheduled(block_pos, fluid)
    }

    /// Close container screens for all players who have a container open at the given block position.
    pub fn close_container_screens_at(&self, position: &BlockPos) {
        let players = self.players.load();
        for player in players.iter() {
            if player.open_container_pos.load() == Some(*position) {
                player.close_handled_screen();
            }
        }
    }

    pub fn drop_stack(self: &Arc<Self>, pos: &BlockPos, stack: ItemStack) {
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
        let mut item_event = crate::plugin::api::events::entity::item_spawn::ItemSpawnEvent::new(
            entity.entity_id,
            spawn_pos,
            stack.item.registry_key.to_string(),
        );
        if let Some(server) = self.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut item_event);
        }
        if item_event.cancelled {
            return;
        }

        let item_entity = Arc::new(ItemEntity::new(entity, stack));
        self.spawn_entity(item_entity);
    }

    pub fn strike_lightning(self: &Arc<Self>, pos: Vector3<f64>, effect_only: bool) {
        use pumpkin_data::entity::EntityType;
        use uuid::Uuid;
        let server_ref = self.server.upgrade();
        if let Some(server_ref) = server_ref {
            let mut event =
                crate::plugin::api::events::world::lightning_strike::LightningStrikeEvent::new(
                    pos,
                    effect_only,
                );
            server_ref
                .plugin_manager
                .fire_blocking(&server_ref, &mut event);
            if event.cancelled {
                return;
            }
        }

        let lightning = crate::entity::r#type::from_type(
            &EntityType::LIGHTNING_BOLT,
            pos,
            self,
            Uuid::new_v4(),
        );

        if let Some(bolt) = lightning
            .cast_any()
            .downcast_ref::<crate::entity::lightning::LightningBoltEntity>()
        {
            bolt.set_visual_only(effect_only);
        }

        self.spawn_entity(lightning);
    }

    /* ItemScatterer.java */
    pub fn scatter_inventory(
        self: &Arc<Self>,
        position: &BlockPos,
        inventory: &Arc<dyn Inventory>,
    ) {
        for i in 0..inventory.size() {
            self.scatter_stack(
                f64::from(position.0.x),
                f64::from(position.0.y),
                f64::from(position.0.z),
                inventory.remove_stack(i),
            );
        }
    }
    pub fn scatter_stack(self: &Arc<Self>, x: f64, y: f64, z: f64, mut stack: ItemStack) {
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
            let entity = Arc::new(ItemEntity::new_with_velocity(entity, item, velocity, 10));
            self.spawn_entity(entity);
        }
    }
    /* End ItemScatterer.java */

    pub fn sync_world_event(&self, world_event: WorldEvent, position: BlockPos, data: i32) {
        let chunk_pos = position.chunk_position();
        self.broadcast_to_chunk(
            chunk_pos,
            &CWorldEvent::new(world_event as i32, position, data, false),
        );
    }

    pub fn set_block_destroy_stage(&self, entity_id: i32, location: BlockPos, stage: i8) {
        let chunk_pos = location.chunk_position();
        let packet = CSetBlockDestroyStage::new(entity_id.into(), location, stage);
        self.broadcast_to_chunk(chunk_pos, &packet);
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
    pub fn get_block(&self, position: &BlockPos) -> &'static Block {
        self.get_block_state_id_if_loaded(position)
            .map_or(&Block::AIR, Block::from_state_id)
    }

    #[must_use]
    pub fn get_block_state_id_if_loaded(&self, position: &BlockPos) -> Option<BlockStateId> {
        if !self.is_in_build_limit(*position) {
            return None;
        }

        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        self.level.read_chunk_sync(&chunk_coordinate, |chunk| {
            chunk
                .section
                .get_block_absolute_y(relative.x as usize, relative.y, relative.z as usize)
        })?
    }

    #[must_use]
    pub fn get_block_state_if_loaded(&self, position: &BlockPos) -> Option<&'static BlockState> {
        self.get_block_state_id_if_loaded(position)
            .map(BlockState::from_id)
    }

    #[must_use]
    pub fn is_loaded(&self, position: &BlockPos) -> bool {
        self.get_block_state_id_if_loaded(position).is_some()
    }

    pub fn get_fluid(&self, position: &BlockPos) -> &'static pumpkin_data::fluid::Fluid {
        let id = self.get_block_state_id(position);
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

    pub fn get_block_and_fluid(
        &self,
        position: &BlockPos,
    ) -> (
        &'static pumpkin_data::Block,
        &'static pumpkin_data::fluid::Fluid,
    ) {
        let id = self.get_block_state_id(position);
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

    pub fn get_fluid_and_fluid_state(
        &self,
        position: &BlockPos,
    ) -> (&'static Fluid, &'static FluidState) {
        let id = self.get_block_state_id(position);

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

    pub fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
        self.get_block_state_id_if_loaded(position)
            .unwrap_or(Block::AIR.default_state.id)
    }

    /// Gets the `BlockState` from the block registry. Returns Air if the block state was not found.
    pub fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
        let id = self.get_block_state_id(position);
        BlockState::from_id(id)
    }

    /// Gets the Block + Block state from the Block Registry, Returns Air if the Block state has not been found
    pub fn get_block_and_state(
        &self,
        position: &BlockPos,
    ) -> (&'static Block, &'static BlockState) {
        let id = self.get_block_state_id(position);
        BlockState::from_id_with_block(id)
    }

    /// Gets the Block + state id from the Block Registry, Returns Air if the Block state has not been found
    pub fn get_block_and_state_id(&self, position: &BlockPos) -> (&'static Block, BlockStateId) {
        let id = self.get_block_state_id(position);
        (Block::from_state_id(id), id)
    }

    /// Updates neighboring blocks of a block
    pub fn update_neighbors(
        self: &Arc<Self>,
        block_pos: &BlockPos,
        except: Option<BlockDirection>,
    ) {
        let source_block = self.get_block(block_pos);
        for direction in BlockDirection::update_order() {
            if except.is_some_and(|d| d == direction) {
                continue;
            }

            let neighbor_pos = block_pos.offset(direction.to_offset());
            let (neighbor_block, neighbor_fluid) = self.get_block_and_fluid(&neighbor_pos);

            let mut event =
                crate::plugin::api::events::block::block_physics::BlockPhysicsEvent::new(
                    neighbor_pos,
                    *block_pos,
                );
            if let Some(server) = self.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            if event.cancelled {
                continue;
            }

            if let Some(neighbor_pumpkin_block) =
                self.block_registry.get_pumpkin_block(neighbor_block.id)
            {
                neighbor_pumpkin_block.on_neighbor_update(OnNeighborUpdateArgs {
                    world: self,
                    block: neighbor_block,
                    position: &neighbor_pos,
                    source_block,
                    notify: false,
                });
            }

            if let Some(neighbor_pumpkin_fluid) =
                self.block_registry.get_pumpkin_fluid(neighbor_fluid.id)
            {
                neighbor_pumpkin_fluid.on_neighbor_update(
                    self,
                    neighbor_fluid,
                    &neighbor_pos,
                    false,
                );
            }
        }
    }

    pub fn update_neighbor(self: &Arc<Self>, neighbor_block_pos: &BlockPos, source_block: &Block) {
        let neighbor_block = self.get_block(neighbor_block_pos);

        let mut event = crate::plugin::api::events::block::block_physics::BlockPhysicsEvent::new(
            *neighbor_block_pos,
            *neighbor_block_pos,
        );
        if let Some(server) = self.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            return;
        }

        if let Some(neighbor_pumpkin_block) =
            self.block_registry.get_pumpkin_block(neighbor_block.id)
        {
            neighbor_pumpkin_block.on_neighbor_update(OnNeighborUpdateArgs {
                world: self,
                block: neighbor_block,
                position: neighbor_block_pos,
                source_block,
                notify: false,
            });
        }
    }

    pub fn update_from_neighbor_shapes(
        self: &Arc<Self>,
        state_id: BlockStateId,
        pos: &BlockPos,
    ) -> BlockStateId {
        let mut current_state_id = state_id;
        let block = Block::from_state_id(state_id);
        for direction in BlockDirection::all() {
            let neighbor_pos = pos.offset(direction.to_offset());
            let neighbor_state_id = self.get_block_state_id(&neighbor_pos);
            current_state_id = self.block_registry.get_state_for_neighbor_update(
                self,
                block,
                current_state_id,
                pos,
                direction,
                &neighbor_pos,
                neighbor_state_id,
            );
        }
        current_state_id
    }

    pub fn replace_with_state_for_neighbor_update(
        self: &Arc<Self>,
        block_pos: &BlockPos,
        direction: BlockDirection,
        flags: BlockFlags,
    ) {
        let (block, block_state_id) = self.get_block_and_state_id(block_pos);

        if flags.contains(BlockFlags::SKIP_REDSTONE_WIRE_STATE_REPLACEMENT)
            && *block == Block::REDSTONE_WIRE
        {
            return;
        }

        let neighbor_pos = block_pos.offset(direction.to_offset());
        let neighbor_state_id = self.get_block_state_id(&neighbor_pos);

        let new_state_id = self.block_registry.get_state_for_neighbor_update(
            self,
            block,
            block_state_id,
            block_pos,
            direction,
            &neighbor_pos,
            neighbor_state_id,
        );

        if new_state_id != block_state_id {
            if is_air(new_state_id) {
                self.break_block(block_pos, None, flags | BlockFlags::NOTIFY_ALL);
            } else {
                self.set_block_state(block_pos, new_state_id, flags);
            }
        }
    }

    /// Returns whether monsters can be spawned in the world
    pub fn should_spawn_monsters(&self) -> bool {
        let level_data = self.level_info.load();
        level_data.game_rules.spawn_mobs
            && level_data.game_rules.spawn_monsters
            && level_data.difficulty != Difficulty::Peaceful
    }

    pub fn get_block_entity(&self, block_pos: &BlockPos) -> Option<Arc<dyn BlockEntity>> {
        let chunk_pos = block_pos.chunk_position();
        if let Some(entity) = self
            .block_entities
            .get(&chunk_pos)
            .and_then(|m| m.get(block_pos).cloned())
        {
            return Some(entity);
        }

        let nbt = self
            .level
            .read_chunk_sync(&chunk_pos, |chunk| {
                chunk
                    .pending_block_entities
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .get(block_pos)
                    .cloned()
            })
            .flatten()?;
        if let Some(custom_data) = nbt
            .get_compound("PumpkinCustomData")
            .or_else(|| nbt.get_compound("BukkitValues"))
        {
            self.custom_block_entity_data
                .insert(*block_pos, custom_data.clone());
        }
        let entity = block_entity_from_nbt(&nbt)?;
        self.block_entities
            .entry(chunk_pos)
            .or_default()
            .insert(*block_pos, entity.clone());
        Some(entity)
    }

    fn bedrock_block_entity_data(
        &self,
        state_id: BlockStateId,
        position: BlockPos,
    ) -> Option<NbtCompound> {
        self.get_block_entity(&position)?
            .bedrock_block_actor_data(state_id)
    }

    /// Builds Bedrock block actor tags that are not represented by Java block states alone.
    pub fn bedrock_chunk_block_actors(&self, chunk: &ChunkData) -> Vec<NbtCompound> {
        let chunk_pos = Vector2::new(chunk.x, chunk.z);
        let live_entities: FxHashMap<_, _> = self
            .block_entities
            .get(&chunk_pos)
            .map(|entities| {
                entities
                    .iter()
                    .map(|(position, entity)| (*position, entity.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let pending = chunk
            .pending_block_entities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        live_entities
            .iter()
            .filter_map(|(position, entity)| {
                let relative = position.chunk_relative_position();
                chunk
                    .section
                    .get_block_absolute_y(relative.x as usize, relative.y, relative.z as usize)
                    .and_then(|state_id| {
                        bedrock_chest_block_actor(state_id, *position)
                            .or_else(|| entity.bedrock_block_actor_data(state_id))
                    })
            })
            .chain(
                pending
                    .iter()
                    .filter(|(position, _)| !live_entities.contains_key(position))
                    .filter_map(|(position, nbt)| {
                        let relative = position.chunk_relative_position();
                        let state_id = chunk.section.get_block_absolute_y(
                            relative.x as usize,
                            relative.y,
                            relative.z as usize,
                        )?;
                        bedrock_chest_block_actor(state_id, *position).or_else(|| {
                            block_entity_from_nbt(nbt)?.bedrock_block_actor_data(state_id)
                        })
                    }),
            )
            .collect()
    }

    pub fn add_block_entity(&self, block_entity: Arc<dyn BlockEntity>) {
        let block_pos = block_entity.get_position();
        let chunk_pos = block_pos.chunk_position();
        let block_entity_nbt = block_entity.chunk_data_nbt();
        let entity_id = block_entity.resource_location().to_string();

        if let Some(nbt) = &block_entity_nbt {
            let bytes = pumpkin_nbt::Nbt::from(nbt.clone()).write_unnamed();
            self.broadcast_to_chunk(
                chunk_pos,
                &CBlockEntityData::new(
                    block_entity.get_position(),
                    VarInt(block_entity.get_id() as i32),
                    bytes.as_ref().into(),
                ),
            );
        }

        self.block_entities
            .entry(chunk_pos)
            .or_default()
            .insert(block_pos, block_entity);

        if let Some(nbt) = block_entity_nbt {
            let mut full_nbt = nbt;
            full_nbt.put_string("id", entity_id);
            full_nbt.put_int("x", block_pos.0.x);
            full_nbt.put_int("y", block_pos.0.y);
            full_nbt.put_int("z", block_pos.0.z);
            self.add_block_entity_nbt(block_pos, &full_nbt);
        }

        self.level.read_chunk_sync(&chunk_pos, |chunk| {
            chunk.mark_dirty(true);
        });
    }

    pub(crate) fn add_block_entity_nbt(&self, block_pos: BlockPos, nbt: &NbtCompound) {
        self.level
            .read_chunk_sync(&block_pos.chunk_position(), |chunk| {
                chunk
                    .pending_block_entities
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .insert(block_pos, nbt.clone());
                chunk.mark_dirty(true);
            });
    }

    pub fn remove_block_entity(&self, block_pos: &BlockPos) {
        let chunk_pos = block_pos.chunk_position();
        let removed =
            self.block_entities
                .get_mut(&chunk_pos)
                .is_some_and(|mut chunk_block_entities| {
                    chunk_block_entities.remove(block_pos).is_some()
                });
        if removed {
            self.custom_block_entity_data.remove(block_pos);
            // Drop the chunk's map once its last block entity is gone.
            self.block_entities
                .remove_if(&chunk_pos, |_, entities| entities.is_empty());
            self.level.read_chunk_sync(&chunk_pos, |chunk| {
                chunk.mark_dirty(true);
            });
        }
    }

    fn migrate_pending_block_entities(&self, chunk_pos: Vector2<i32>) {
        let positions: Vec<BlockPos> = self
            .level
            .read_chunk_sync(&chunk_pos, |chunk| {
                chunk
                    .pending_block_entities
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .keys()
                    .copied()
                    .collect()
            })
            .unwrap_or_default();
        for pos in positions {
            let already_loaded = self
                .block_entities
                .get(&chunk_pos)
                .is_some_and(|m| m.contains_key(&pos));
            if !already_loaded && let Some(entity) = self.get_block_entity(&pos) {
                self.update_block_entity(&entity);
            }
        }
    }

    pub fn update_block_entity(&self, block_entity: &Arc<dyn BlockEntity>) {
        let block_pos = block_entity.get_position();
        let chunk_pos = block_pos.chunk_position();
        let block_entity_nbt = block_entity.chunk_data_nbt();

        if let Some(nbt) = &block_entity_nbt {
            let bytes = pumpkin_nbt::Nbt::from(nbt.clone()).write_unnamed();
            self.broadcast_to_chunk(
                chunk_pos,
                &CBlockEntityData::new(
                    block_entity.get_position(),
                    VarInt(block_entity.get_id() as i32),
                    bytes.as_ref().into(),
                ),
            );
            let mut full_nbt = nbt.clone();
            full_nbt.put_string("id", block_entity.resource_location().to_string());
            let pos = block_entity.get_position();
            full_nbt.put_int("x", pos.0.x);
            full_nbt.put_int("y", pos.0.y);
            full_nbt.put_int("z", pos.0.z);
            self.add_block_entity_nbt(block_pos, &full_nbt);
        }
        self.level.read_chunk_sync(&chunk_pos, |chunk| {
            chunk.mark_dirty(true);
        });
    }

    #[must_use]
    pub fn intersects_aabb_with_hit(
        from: Vector3<f64>,
        to: Vector3<f64>,
        min: Vector3<f64>,
        max: Vector3<f64>,
    ) -> Option<(f64, BlockDirection, Vector3<f64>)> {
        let dir = to.sub(&from);
        let mut tmin: f64 = 0.0;
        let mut tmax: f64 = 1.0;

        let mut hit_axis = None;
        let mut hit_is_min = false;

        macro_rules! check_axis {
            ($axis:ident, $dir_axis:ident, $min_axis:ident, $max_axis:ident) => {{
                if dir.$dir_axis.abs() < 1e-8 {
                    if from.$dir_axis < min.$min_axis || from.$dir_axis > max.$max_axis {
                        return None;
                    }
                } else {
                    let inv_d = 1.0 / dir.$dir_axis;
                    let t_near = (min.$min_axis - from.$dir_axis) * inv_d;
                    let t_far = (max.$max_axis - from.$dir_axis) * inv_d;

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

        check_axis!(x, x, x, x);
        check_axis!(y, y, y, y);
        check_axis!(z, z, z, z);

        if tmax < 0.0 || tmin > 1.0 {
            return None;
        }

        let direction = match (hit_axis, hit_is_min) {
            (Some("x"), true) => BlockDirection::West,
            (Some("x"), false) => BlockDirection::East,
            (Some("y"), true) => BlockDirection::Down,
            (Some("y"), false) => BlockDirection::Up,
            (Some("z"), true) => BlockDirection::North,
            (Some("z"), false) => BlockDirection::South,
            _ => {
                if dir.y < 0.0 {
                    BlockDirection::Up
                } else if dir.y > 0.0 {
                    BlockDirection::Down
                } else {
                    BlockDirection::North
                }
            }
        };

        let t_hit = tmin.max(0.0);
        let hit_pos = from + dir * t_hit;
        Some((t_hit, direction, hit_pos))
    }

    pub fn ray_outline_check_detailed(
        &self,
        block_pos: &BlockPos,
        from: Vector3<f64>,
        to: Vector3<f64>,
    ) -> Option<(BlockDirection, Vector3<f64>)> {
        let state = self.get_block_state(block_pos);

        if state.outline_shapes.is_empty() {
            let block_min = block_pos.0.to_f64();
            let block_max = block_min.add_raw(1.0, 1.0, 1.0);
            return Self::intersects_aabb_with_hit(from, to, block_min, block_max)
                .map(|(_, dir, hit_pos)| (dir, hit_pos));
        }

        let bounding_boxes = state.get_block_outline_shapes_at(block_pos);
        let mut closest_hit: Option<(f64, BlockDirection, Vector3<f64>)> = None;

        for shape in bounding_boxes {
            let world_min = shape.min.add(&block_pos.0.to_f64());
            let world_max = shape.max.add(&block_pos.0.to_f64());

            if let Some((t, dir, hit_pos)) =
                Self::intersects_aabb_with_hit(from, to, world_min, world_max)
                && closest_hit
                    .as_ref()
                    .is_none_or(|(closest_t, _, _)| t < *closest_t)
            {
                closest_hit = Some((t, dir, hit_pos));
            }
        }

        closest_hit.map(|(_, dir, hit_pos)| (dir, hit_pos))
    }

    fn ray_outline_check(
        &self,
        block_pos: &BlockPos,
        from: Vector3<f64>,
        to: Vector3<f64>,
    ) -> (bool, Option<BlockDirection>) {
        if let Some((dir, _)) = self.ray_outline_check_detailed(block_pos, from, to) {
            (true, Some(dir))
        } else {
            let state = self.get_block_state(block_pos);
            if state.outline_shapes.is_empty() {
                (true, None)
            } else {
                (false, None)
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn ray_trace_block(
        &self,
        start_pos: Vector3<f64>,
        end_pos: Vector3<f64>,
        include_fluids: bool,
    ) -> Option<(BlockPos, BlockDirection, Vector3<f64>)> {
        if start_pos == end_pos {
            return None;
        }

        let adjust = -1.0e-7f64;
        let to = end_pos.lerp(&start_pos, adjust);
        let from = start_pos.lerp(&end_pos, adjust);

        let mut block = BlockPos::floored(from.x, from.y, from.z);

        let state = self.get_block_state(&block);
        let valid_start = if include_fluids {
            !state.is_air()
        } else {
            !state.is_air() && !state.is_liquid()
        };
        if valid_start
            && let Some((dir, hit_pos)) = self.ray_outline_check_detailed(&block, from, to)
        {
            return Some((block, dir, hit_pos));
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

            let state = self.get_block_state(&block);
            let hit = if include_fluids {
                !state.is_air()
            } else {
                !state.is_air() && !state.is_liquid()
            };

            if hit {
                if let Some((dir, hit_pos)) = self.ray_outline_check_detailed(&block, from, to) {
                    return Some((block, dir, hit_pos));
                }
                let block_min = block.0.to_f64();
                let block_max = block_min.add_raw(1.0, 1.0, 1.0);
                if let Some((_, dir, hit_pos)) =
                    Self::intersects_aabb_with_hit(from, to, block_min, block_max)
                {
                    return Some((block, dir, hit_pos));
                }
                return Some((block, block_direction, to));
            }
        }

        None
    }

    pub fn ray_trace_entities(
        &self,
        start: Vector3<f64>,
        end: Vector3<f64>,
    ) -> Vec<(Arc<dyn EntityBase>, Vector3<f64>, f64)> {
        if start == end {
            return Vec::new();
        }

        let min_x = start.x.min(end.x) - 1.0;
        let max_x = start.x.max(end.x) + 1.0;
        let min_y = start.y.min(end.y) - 1.0;
        let max_y = start.y.max(end.y) + 1.0;
        let min_z = start.z.min(end.z) - 1.0;
        let max_z = start.z.max(end.z) + 1.0;
        let ray_box = BoundingBox::new(
            Vector3::new(min_x, min_y, min_z),
            Vector3::new(max_x, max_y, max_z),
        );

        let mut hits = Vec::new();

        for entity in self.entities.load().iter() {
            let bb = entity.get_entity().bounding_box.load();
            if bb.intersects(&ray_box)
                && let Some((t, _, hit_pos)) =
                    Self::intersects_aabb_with_hit(start, end, bb.min, bb.max)
            {
                let distance = (hit_pos - start).length();
                hits.push((entity.clone(), hit_pos, distance, t));
            }
        }

        for player in self.players.load().iter() {
            let bb = player.get_entity().bounding_box.load();
            if bb.intersects(&ray_box)
                && let Some((t, _, hit_pos)) =
                    Self::intersects_aabb_with_hit(start, end, bb.min, bb.max)
            {
                let distance = (hit_pos - start).length();
                hits.push((player.clone() as Arc<dyn EntityBase>, hit_pos, distance, t));
            }
        }

        hits.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
        hits.into_iter()
            .map(|(ent, hit_pos, dist, _)| (ent, hit_pos, dist))
            .collect()
    }

    pub fn ray_trace_entity(
        &self,
        start: Vector3<f64>,
        end: Vector3<f64>,
    ) -> Option<(Arc<dyn EntityBase>, Vector3<f64>, f64)> {
        self.ray_trace_entities(start, end).into_iter().next()
    }

    pub fn raycast(
        self: &Arc<Self>,
        start_pos: Vector3<f64>,
        end_pos: Vector3<f64>,
        hit_check: impl Fn(&BlockPos, &Arc<Self>) -> bool,
    ) -> Option<(BlockPos, BlockDirection)> {
        if start_pos == end_pos {
            return None;
        }

        let adjust = -1.0e-7f64;
        let to = end_pos.lerp(&start_pos, adjust);
        let from = start_pos.lerp(&end_pos, adjust);

        let mut block = BlockPos::floored(from.x, from.y, from.z);

        let (collision, direction) = self.ray_outline_check(&block, from, to);
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

            if hit_check(&block, self) {
                let (collision, direction) = self.ray_outline_check(&block, from, to);
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

    /// Broadcasts a packet to all players who currently have the target chunk loaded.
    /// This uses highly optimized Chebyshev distance math (Chunk Grid) instead of floating point distance checks.
    pub fn broadcast_to_chunk<P: ClientPacket>(&self, chunk_pos: Vector2<i32>, packet: &P) {
        let players = self.players.load();

        let recipients = players.iter().filter(|p| {
            let center = p.get_entity().chunk_pos.load();
            let view_distance = get_view_distance(p).get() as i32;

            // Chebyshev distance (Minecraft's chunk loading shape)
            is_within_view_distance(chunk_pos, center, view_distance)
        });

        let recipients_by_version = Self::collect_java_recipients_by_version(recipients);
        Self::broadcast_java_grouped(packet, recipients_by_version);
    }

    pub fn broadcast_to_chunk_bedrock<P: BClientPacket>(
        &self,
        chunk_pos: Vector2<i32>,
        packet: &P,
    ) {
        let players = self.players.load();
        let recipients = players.iter().filter_map(|player| {
            let center = player.get_entity().chunk_pos.load();
            let view_distance = get_view_distance(player).get() as i32;
            if is_within_view_distance(chunk_pos, center, view_distance)
                && let ClientPlatform::Bedrock(client) = player.client.as_ref()
            {
                return Some(client);
            }
            None
        });
        Self::broadcast_bedrock_grouped(packet, recipients);
    }

    pub fn broadcast_to_chunk_editioned<J: ClientPacket, B: BClientPacket>(
        &self,
        chunk_pos: Vector2<i32>,
        je_packet: &J,
        be_packet: &B,
    ) {
        let players = self.players.load();
        let mut java_recipients = Vec::new();
        let mut bedrock_recipients = Vec::new();

        let recipients = players.iter().filter(|p| {
            let center = p.get_entity().chunk_pos.load();
            let view_distance = get_view_distance(p).get() as i32;
            is_within_view_distance(chunk_pos, center, view_distance)
        });

        for p in recipients {
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => bedrock_recipients.push(be_client),
            }
        }

        let recipients_by_version =
            Self::collect_java_recipients_by_version(java_recipients.into_iter());
        Self::broadcast_java_grouped(je_packet, recipients_by_version);
        Self::broadcast_bedrock_grouped(be_packet, bedrock_recipients.into_iter());
    }

    /// Broadcasts a packet to chunk watchers, excluding specific players.
    pub fn broadcast_to_chunk_except<P: ClientPacket>(
        &self,
        chunk_pos: Vector2<i32>,
        except: &[uuid::Uuid],
        packet: &P,
    ) {
        let players = self.players.load();

        let recipients = players.iter().filter(|p| {
            if except.contains(&p.get_entity().entity_uuid) {
                return false;
            }
            let center = p.get_entity().chunk_pos.load();
            let view_distance = get_view_distance(p).get() as i32;

            is_within_view_distance(chunk_pos, center, view_distance)
        });

        let recipients_by_version = Self::collect_java_recipients_by_version(recipients);
        Self::broadcast_java_grouped(packet, recipients_by_version);
    }

    pub fn broadcast_to_chunk_except_editioned<J: ClientPacket, B: BClientPacket>(
        &self,
        chunk_pos: Vector2<i32>,
        except: &[uuid::Uuid],
        je_packet: &J,
        be_packet: &B,
    ) {
        let players = self.players.load();
        let recipients = players.iter().filter(|p| {
            if except.contains(&p.get_entity().entity_uuid) {
                return false;
            }
            let center = p.get_entity().chunk_pos.load();
            let view_distance = get_view_distance(p).get() as i32;

            is_within_view_distance(chunk_pos, center, view_distance)
        });

        let mut java_recipients = Vec::new();
        let mut bedrock_recipients = Vec::new();

        for p in recipients {
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => bedrock_recipients.push(be_client),
            }
        }

        let je_recipients_by_version =
            Self::collect_java_recipients_by_version(java_recipients.into_iter());
        Self::broadcast_java_grouped(je_packet, je_recipients_by_version);
        Self::broadcast_bedrock_grouped(be_packet, bedrock_recipients.into_iter());
    }

    pub fn emit_game_event(&self, event_key: impl Into<String>, position: Vector3<f64>) {
        let mut event = crate::plugin::api::events::world::generic_game::GenericGameEvent::new(
            event_key.into(),
            position,
        );
        if let Some(server) = self.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }

    pub async fn unload(self: &Arc<Self>) {
        let mut event =
            crate::plugin::api::events::world::world_load::WorldUnloadEvent::new(self.clone());
        if let Some(server) = self.server.upgrade() {
            server.plugin_manager.fire(&server, &mut event).await;
        }
    }

    pub async fn save(&self) {
        for entity in self.entities.load().iter() {
            self.save_entity(entity).await;
        }

        let chunks: Vec<Vector2<i32>> = self
            .block_entities
            .iter()
            .map(|chunk_block_entities| *chunk_block_entities.key())
            .collect();
        for chunk_pos in chunks {
            self.save_block_entities(chunk_pos);
        }

        if let Ok(mut portal_poi) = self.portal_poi.try_lock() {
            let _ = portal_poi.save_all();
        }

        {
            let custom_data = self
                .custom_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !custom_data.is_empty() {
                let custom_data_path = self
                    .level
                    .level_folder
                    .root_folder
                    .join("pumpkin_custom_data.nbt");
                let nbt = pumpkin_nbt::Nbt::from(custom_data.clone());
                let _ = std::fs::write(custom_data_path, nbt.write());
            }
        }

        self.level
            .should_save
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.level.level_channel.notify();

        let mut save_event = crate::plugin::api::events::world::world_save::WorldSaveEvent::new(
            format!("{:?}", self.dimension),
        );
        if let Some(server) = self.server.upgrade() {
            server.plugin_manager.fire(&server, &mut save_event).await;
        }
    }

    pub fn set_custom_data(&self, namespace: &str, key: &str, value: pumpkin_nbt::tag::NbtTag) {
        let mut custom_data = self
            .custom_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let mut namespace_data = custom_data
            .child_tags
            .remove(namespace)
            .and_then(|tag| match tag {
                pumpkin_nbt::tag::NbtTag::Compound(compound) => Some(compound),
                _ => None,
            })
            .unwrap_or_default();

        namespace_data.child_tags.insert(key.into(), value);
        custom_data.child_tags.insert(
            namespace.into(),
            pumpkin_nbt::tag::NbtTag::Compound(namespace_data),
        );
    }

    pub fn get_custom_data(&self, namespace: &str, key: &str) -> Option<pumpkin_nbt::tag::NbtTag> {
        let custom_data = self
            .custom_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        custom_data
            .get(namespace)?
            .extract_compound()?
            .get(key)
            .cloned()
    }

    pub fn remove_custom_data(&self, namespace: &str, key: &str) {
        let mut custom_data = self
            .custom_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let Some(pumpkin_nbt::tag::NbtTag::Compound(mut namespace_data)) =
            custom_data.child_tags.remove(namespace)
        else {
            return;
        };

        namespace_data.child_tags.remove(key);
        if !namespace_data.is_empty() {
            custom_data.child_tags.insert(
                namespace.into(),
                pumpkin_nbt::tag::NbtTag::Compound(namespace_data),
            );
        }
    }

    pub fn has_custom_data(&self, namespace: &str, key: &str) -> bool {
        self.get_custom_data(namespace, key).is_some()
    }

    pub fn set_block_entity_custom_data(
        &self,
        pos: &BlockPos,
        namespace: &str,
        key: &str,
        value: pumpkin_nbt::tag::NbtTag,
    ) {
        let mut entry = self.custom_block_entity_data.entry(*pos).or_default();
        let mut namespace_data = entry
            .child_tags
            .remove(namespace)
            .and_then(|tag| match tag {
                pumpkin_nbt::tag::NbtTag::Compound(compound) => Some(compound),
                _ => None,
            })
            .unwrap_or_default();

        namespace_data.child_tags.insert(key.into(), value);
        entry.child_tags.insert(
            namespace.into(),
            pumpkin_nbt::tag::NbtTag::Compound(namespace_data),
        );
    }

    pub fn get_block_entity_custom_data(
        &self,
        pos: &BlockPos,
        namespace: &str,
        key: &str,
    ) -> Option<pumpkin_nbt::tag::NbtTag> {
        self.custom_block_entity_data
            .get(pos)?
            .get(namespace)?
            .extract_compound()?
            .get(key)
            .cloned()
    }

    pub fn remove_block_entity_custom_data(&self, pos: &BlockPos, namespace: &str, key: &str) {
        if let Some(mut entry) = self.custom_block_entity_data.get_mut(pos) {
            let Some(pumpkin_nbt::tag::NbtTag::Compound(mut namespace_data)) =
                entry.child_tags.remove(namespace)
            else {
                return;
            };

            namespace_data.child_tags.remove(key);
            if !namespace_data.is_empty() {
                entry.child_tags.insert(
                    namespace.into(),
                    pumpkin_nbt::tag::NbtTag::Compound(namespace_data),
                );
            }
        }
    }

    pub fn has_block_entity_custom_data(&self, pos: &BlockPos, namespace: &str, key: &str) -> bool {
        self.get_block_entity_custom_data(pos, namespace, key)
            .is_some()
    }

    pub fn populate_chunk(&self, chunk_pos: Vector2<i32>) {
        let mut populate_event =
            crate::plugin::api::events::world::chunk_populate::ChunkPopulateEvent::new(chunk_pos);
        if let Some(server) = self.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut populate_event);
        }
    }

    pub fn unload_chunk(&self, chunk_pos: Vector2<i32>) {
        let mut unload_event =
            crate::plugin::api::events::world::chunk_unload::ChunkUnloadEvent::new(chunk_pos);
        if let Some(server) = self.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut unload_event);
        }
    }

    pub fn load_entities(&self, chunk_pos: Vector2<i32>, entity_count: usize) {
        let mut load_event =
            crate::plugin::api::events::world::entities_load::EntitiesLoadEvent::new(
                chunk_pos,
                entity_count,
            );
        if let Some(server) = self.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut load_event);
        }
    }

    pub fn unload_entities(&self, chunk_pos: Vector2<i32>, entity_count: usize) {
        let mut unload_event =
            crate::plugin::api::events::world::entities_unload::EntitiesUnloadEvent::new(
                chunk_pos,
                entity_count,
            );
        if let Some(server) = self.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut unload_event);
        }
    }

    pub fn generate_loot(&self, loot_table: String) {
        let mut loot_event =
            crate::plugin::api::events::world::loot_generate::LootGenerateEvent::new(loot_table);
        if let Some(server) = self.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut loot_event);
        }
    }

    pub fn skip_time(&self, skip_amount: i64) {
        let mut time_event =
            crate::plugin::api::events::world::time_skip::TimeSkipEvent::new(skip_amount);
        if let Some(server) = self.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut time_event);
        }
    }

    pub fn trigger_raid(&self, pos: BlockPos) {
        let mut raid_event =
            crate::plugin::api::events::raid::raid_trigger::RaidTriggerEvent::new(pos);
        if let Some(server) = self.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut raid_event);
        }
    }

    pub fn spawn_raid_wave(&self, wave: u32, pos: BlockPos) {
        let mut wave_event =
            crate::plugin::api::events::raid::raid_spawn_wave::RaidSpawnWaveEvent::new(wave, pos);
        if let Some(server) = self.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut wave_event);
        }
    }

    pub fn finish_raid(&self, victory: bool) {
        let mut raid_event =
            crate::plugin::api::events::raid::raid_finish::RaidFinishEvent::new(victory);
        if let Some(server) = self.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut raid_event);
        }
    }

    pub fn stop_raid(&self, reason: String) {
        let mut raid_event =
            crate::plugin::api::events::raid::raid_stop::RaidStopEvent::new(reason);
        if let Some(server) = self.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut raid_event);
        }
    }

    pub fn async_structure_generate(
        &self,
        world_name: String,
        structure_name: String,
        pos: BlockPos,
    ) {
        let mut event = crate::plugin::api::events::world::async_structure_generate::AsyncStructureGenerateEvent::new(
            world_name,
            structure_name,
            pos,
        );
        if let Some(server) = self.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }

    pub fn async_structure_spawn(&self, world_name: String, structure_name: String, pos: BlockPos) {
        let mut event =
            crate::plugin::api::events::world::async_structure_spawn::AsyncStructureSpawnEvent::new(
                world_name,
                structure_name,
                pos,
            );
        if let Some(server) = self.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }
}

impl BlockAccessor for World {
    fn get_block(&self, position: &BlockPos) -> &'static Block {
        self.get_block_state_id_if_loaded(position)
            .map_or(&Block::AIR, Block::from_state_id)
    }
    fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
        self.get_block_state_id_if_loaded(position)
            .map_or(Block::AIR.default_state, BlockState::from_id)
    }

    fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
        self.get_block_state_id_if_loaded(position)
            .unwrap_or(Block::AIR.default_state.id)
    }

    fn get_block_and_state(&self, position: &BlockPos) -> (&'static Block, &'static BlockState) {
        let id = self
            .get_block_state_id_if_loaded(position)
            .unwrap_or(Block::AIR.default_state.id);
        BlockState::from_id_with_block(id)
    }
}

fn bedrock_block_breaking_rate(speed: f32) -> i32 {
    (speed.clamp(0.0, 1.0) * f32::from(u16::MAX)) as i32
}

pub struct WorldPortal(pub Arc<World>);

// Pure Beauty :cap:
impl WorldPortalExt for WorldPortal {
    fn can_place_at(
        &self,
        block: &pumpkin_data::Block,
        state: &BlockState,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
    ) -> bool {
        self.0.block_registry.can_place_at(
            None,
            None,
            block_accessor,
            None,
            block,
            state,
            block_pos,
            None,
            None,
        )
    }

    fn mirror(&self, block: &Block, state_id: BlockStateId, mirror: Mirror) -> &'static BlockState {
        self.0.block_registry.mirror(block, state_id, mirror)
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        self.0.block_registry.rotate(block, state_id, rotation)
    }

    fn spawn_mobs_for_chunk_generation(
        &self,
        cache: &mut dyn GenerationCache,
        biome: &'static Biome,
        chunk_x: i32,
        chunk_z: i32,
    ) {
        natural_spawner::spawn_mobs_for_chunk_generation(&self.0, cache, biome, chunk_x, chunk_z);
    }

    fn spawn_structure_entities(&self, entities: Vec<NbtCompound>) {
        for nbt in entities {
            let Some(id) = nbt.get_string("id") else {
                continue;
            };
            let Some(entity_type) =
                EntityType::from_name(id.strip_prefix("minecraft:").unwrap_or(id))
            else {
                warn!("Unknown structure entity type: {id}");
                continue;
            };
            let entity = from_type(
                entity_type,
                Vector3::new(0.0, 0.0, 0.0),
                &self.0,
                Uuid::new_v4(),
            );
            entity.get_entity().read_nbt_non_mut(&nbt);
            entity.read_nbt_non_mut(&nbt);
            self.0.spawn_entity(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::{
        Block,
        block_properties::{BlockProperties, ChestLikeProperties, ChestType, HorizontalFacing},
    };
    use pumpkin_util::math::position::BlockPos;

    use super::{bedrock_block_breaking_rate, bedrock_chest_block_actor};

    #[test]
    fn bedrock_block_breaking_rate_uses_progress_per_tick() {
        assert_eq!(bedrock_block_breaking_rate(0.0), 0);
        assert_eq!(bedrock_block_breaking_rate(1.0 / 30.0), 2_184);
        assert_eq!(bedrock_block_breaking_rate(1.0), 65_535);
    }

    #[test]
    fn bedrock_double_chest_block_actor_identifies_pair_and_lead() {
        let position = BlockPos::new(5, 64, 7);
        let properties = ChestLikeProperties {
            facing: HorizontalFacing::North,
            r#type: ChestType::Right,
            waterlogged: false,
        };
        let actor =
            bedrock_chest_block_actor(properties.to_state_id(&Block::CHEST), position).unwrap();

        assert_eq!(actor.get_int("pairx"), Some(4));
        assert_eq!(actor.get_int("pairz"), Some(7));
        assert_eq!(actor.get_bool("pairlead"), Some(true));
    }

    #[test]
    fn game_rules_registry() {
        use pumpkin_data::game_rules::{GameRule, GameRuleRegistry, GameRuleValue};

        let mut registry = GameRuleRegistry::default();
        match registry.get(&GameRule::KeepInventory) {
            GameRuleValue::Bool(v) => assert!(!v),
            GameRuleValue::Int(_) => panic!("expected bool"),
        }

        match registry.get_mut(&GameRule::KeepInventory) {
            GameRuleValue::Bool(v) => *v = true,
            GameRuleValue::Int(_) => panic!("expected bool"),
        }

        match registry.get(&GameRule::KeepInventory) {
            GameRuleValue::Bool(v) => assert!(v),
            GameRuleValue::Int(_) => panic!("expected bool"),
        }

        match registry.get(&GameRule::RandomTickSpeed) {
            GameRuleValue::Int(v) => assert_eq!(*v, 3),
            GameRuleValue::Bool(_) => panic!("expected int"),
        }

        match registry.get_mut(&GameRule::RandomTickSpeed) {
            GameRuleValue::Int(v) => *v = 20,
            GameRuleValue::Bool(_) => panic!("expected int"),
        }

        match registry.get(&GameRule::RandomTickSpeed) {
            GameRuleValue::Int(v) => assert_eq!(*v, 20),
            GameRuleValue::Bool(_) => panic!("expected int"),
        }
    }
}
