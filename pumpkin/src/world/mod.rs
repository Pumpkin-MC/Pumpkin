use crate::block::entities::BlockEntity;
use dashmap::DashMap;
use indexmap::IndexSet;
use pumpkin_data::chunk::Biome;
use pumpkin_world::generation::proto_chunk::GenerationCache;
use std::sync::atomic::AtomicU8;
use std::sync::{Arc, Weak};
use tracing::error;

pub mod chunker;
pub mod entity_lookup;
pub mod explosion;
pub mod loot;
pub mod map;
pub mod portal;
pub mod time;
pub mod vibrations;

use crate::{
    block::{BlockEvent, registry::BlockRegistry},
    entity::player::Player,
    error::PumpkinError,
    server::Server,
};
use arc_swap::ArcSwap;
use border::Worldborder;
use pumpkin_data::BlockState;
use pumpkin_data::block_rotation::{Mirror, Rotation};
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::level::Level;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::{GetBlockError, WorldPortalExt};
pub use pumpkin_world::{world::BlockFlags, world_info::LevelData};
use scoreboard::Scoreboard;
use time::LevelTime;
use tokio::sync::Mutex;

pub mod border;
pub mod bossbar;
pub mod custom_bossbar;
pub mod dragon_fight;
pub mod end_podium;
pub mod natural_spawner;
pub mod phantom_spawner;
pub mod scoreboard;
pub mod weather;

mod block_updates;
mod blocks;
mod broadcast;
mod chunks;
mod collision;
mod entities;
mod player_bedrock;
mod player_java;
mod players;
mod tick;

use crate::world::natural_spawner::SpawnState;
use uuid::Uuid;
use weather::Weather;

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
    /// Live non-player entities — vanilla `EntityLookup` (id + uuid maps, O(1)
    /// add/remove). Does not include players.
    pub entities: entity_lookup::EntityLookup,
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
    /// Vanilla's `ObjectLinkedOpenHashSet<BlockEventData>`: preserve insertion
    /// order while coalescing duplicate events in the same tick.
    synced_block_event_queue: Mutex<IndexSet<BlockEvent>>,
    /// Vibrations traveling toward sculk sensors (1 block per tick).
    pub pending_vibrations: std::sync::Mutex<Vec<crate::world::vibrations::PendingVibration>>,
    /// Set once a sculk sensor block entity registers; lets `emit_vibration`
    /// skip the 9-chunk scan entirely on the vast majority of worlds.
    pub has_sculk_sensors: std::sync::atomic::AtomicBool,
    /// Serializes block-event processing and its client packet enqueue order.
    synced_block_event_flush_lock: Mutex<()>,
    /// Dirty block positions waiting to be broadcast to clients.
    ///
    /// State changes may race while chunk, player, and entity ticks run in
    /// parallel. Keep only positions here and read the authoritative state when
    /// flushing, otherwise an older writer can leave a stale client snapshot.
    unsent_block_changes: Mutex<FxHashSet<BlockPos>>,
    /// Block entities that need an authoritative state/data update pair.
    unsent_block_entity_updates: std::sync::Mutex<FxHashSet<BlockPos>>,
    /// Serializes broadcasts and direct corrections to preserve block packet order.
    block_update_flush_lock: Mutex<()>,
    /// POI storage for fast portal lookups
    pub portal_poi: Mutex<portal::PortalPoiStorage>,
    /// End Dragon fight manager (only present in `THE_END` dimension).
    pub dragon_fight: Option<Mutex<dragon_fight::DragonFight>>,
    pub spawn_state: ArcSwap<SpawnState>,
    pub active_chunks: ArcSwap<FxHashSet<Vector2<i32>>>,
    pub forced_chunks: std::sync::Mutex<FxHashSet<Vector2<i32>>>,
    /// Block entities indexed by chunk, so ticking only visits the currently
    /// active chunks instead of scanning every loaded block entity each tick.
    pub block_entities: DashMap<Vector2<i32>, FxHashMap<BlockPos, Arc<dyn BlockEntity>>>,
    /// Cached ambient sky darken (0–11). Updated each environment tick so
    /// monster spawn light checks can run without locking time/weather.
    pub sky_darken: AtomicU8,
    /// Vanilla `Level.neighborUpdater` — `CollectingNeighborUpdater` queue.
    pub neighbor_updater: crate::block::blocks::redstone::neighbor_updater::WorldNeighborUpdater,
    /// Vanilla `PhantomSpawner` (insomnia / TIME_SINCE_REST custom spawner).
    pub phantom_spawner: phantom_spawner::PhantomSpawner,
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
        let portal_poi = portal::PortalPoiStorage::new(level.level_folder.poi_folder.clone());
        let dragon_fight = (dimension.minecraft_name == Dimension::THE_END.minecraft_name)
            .then(|| Mutex::new(dragon_fight::DragonFight::new()));
        Self {
            uuid: Uuid::new_v4(),
            level,
            level_info,
            players: ArcSwap::new(Arc::new(Vec::new())),
            entities: entity_lookup::EntityLookup::new(),
            scoreboard: Mutex::new(Scoreboard::default()),
            worldborder: Mutex::new(Worldborder::new(0.0, 0.0, 5.999_996_8E7, 0, 5, 300)),
            level_time: Mutex::new(LevelTime::new()),
            dimension,
            weather: Mutex::new(Weather::new()),
            block_registry,
            sea_level: generation_settings.sea_level,
            min_y: i32::from(generation_settings.shape.min_y),
            synced_block_event_queue: Mutex::new(IndexSet::new()),
            pending_vibrations: std::sync::Mutex::new(Vec::new()),
            has_sculk_sensors: std::sync::atomic::AtomicBool::new(false),
            synced_block_event_flush_lock: Mutex::new(()),
            unsent_block_changes: Mutex::new(FxHashSet::default()),
            unsent_block_entity_updates: std::sync::Mutex::new(FxHashSet::default()),
            block_update_flush_lock: Mutex::new(()),
            portal_poi: Mutex::new(portal_poi),
            dragon_fight,
            spawn_state: ArcSwap::new(Arc::new(SpawnState::empty())),
            active_chunks: ArcSwap::new(Arc::new(FxHashSet::default())),
            forced_chunks: std::sync::Mutex::new(FxHashSet::default()),
            server,
            block_entities: DashMap::new(),
            sky_darken: AtomicU8::new(0),
            neighbor_updater:
                crate::block::blocks::redstone::neighbor_updater::WorldNeighborUpdater::new(),
            phantom_spawner: phantom_spawner::PhantomSpawner::default(),
        }
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

    pub async fn get_world_age(&self) -> i64 {
        self.level_time.lock().await.world_age
    }

    pub async fn get_time_of_day(&self) -> i64 {
        self.level_time.lock().await.time_of_day
    }

    pub async fn set_time_of_day(&self, time: i64) {
        let mut level_time = self.level_time.lock().await;
        level_time.set_time(time);
        level_time.send_time(self).await;
    }

    pub async fn is_raining(&self) -> bool {
        self.weather.lock().await.raining
    }

    pub async fn set_raining(&self, raining: bool) {
        let mut weather = self.weather.lock().await;
        if weather.raining != raining {
            let thunder = weather.thundering;
            weather.set_weather_parameters(self, 0, 0, raining, thunder);
        }
    }

    pub async fn is_thundering(&self) -> bool {
        self.weather.lock().await.thundering
    }

    pub async fn set_thundering(&self, thundering: bool) {
        let mut weather = self.weather.lock().await;
        if weather.thundering != thundering {
            let raining = weather.raining;
            weather.set_weather_parameters(self, 0, 0, raining, thundering);
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
}
