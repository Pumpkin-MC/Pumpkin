use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex as StdMutex};

use async_trait::async_trait;
use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_gametest::{
    BlockBasedTest, GameTestError, GameTestManager, GameTestReporter, GameTestResult,
    GameTestRotation, GameTestRunner, GameTestSession, GameTestStructureTemplate, GameTestWorld,
};
pub use pumpkin_gametest::{GameTestBatchReport, GameTestRetryOptions};
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use pumpkin_util::text::TextComponent;
use pumpkin_world::{chunk::ChunkHeightmapType, world::BlockFlags};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{
    block::entities::{
        BlockEntity, block_entity_from_nbt, test_block::TestBlockBlockEntity,
        test_instance_block::TestInstanceBlockBlockEntity,
    },
    server::Server,
    world::World,
};

static GAME_TEST_QUEUE: LazyLock<Mutex<Vec<GameTestQueueEntry>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
static STOP_GAME_TESTS: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy)]
struct ForcedGameTestChunk {
    users: usize,
    was_forced: bool,
}

/// `GameTest` structures must keep their chunks loaded and ticking just like vanilla's
/// `TestInstanceBlockEntity::forceLoadChunks`. Keep a small reference count so
/// overlapping tests share the same force-load lease and pre-existing `/forceload`
/// chunks are never released by the `GameTest` runtime.
/// Key uniquely identifying a force-loaded chunk requested by a `GameTest`.
type ForcedGameTestChunkKey = (uuid::Uuid, i32, i32);

/// Reference-counted map of chunks force-loaded by active `GameTest`s.
type ForcedGameTestChunkMap = StdMutex<HashMap<ForcedGameTestChunkKey, ForcedGameTestChunk>>;

/// Global registry of chunks force-loaded by the `GameTest` runtime.
static FORCED_GAME_TEST_CHUNKS: LazyLock<ForcedGameTestChunkMap> =
    LazyLock::new(|| StdMutex::new(HashMap::new()));

pub struct GameTestQueueEntry {
    test_id: String,
    world: Arc<World>,
    test_x: i32,
    test_z: i32,
    rotation_steps: i32,
    retry_options: GameTestRetryOptions,
    report: Arc<GameTestBatchReport>,
}

impl GameTestQueueEntry {
    #[must_use]
    pub fn new(
        test_id: impl Into<String>,
        world: Arc<World>,
        test_x: i32,
        test_z: i32,
        rotation_steps: i32,
        retry_options: GameTestRetryOptions,
        report: Arc<GameTestBatchReport>,
    ) -> Self {
        Self {
            test_id: test_id.into(),
            world,
            test_x,
            test_z,
            rotation_steps,
            retry_options,
            report,
        }
    }
}

struct GameTestWorldReporter {
    world: Arc<World>,
}

impl GameTestReporter for GameTestWorldReporter {
    fn send_message(&self, message: TextComponent) {
        broadcast_world(&self.world, &message);
    }
}

pub async fn enqueue_game_test(request: GameTestQueueEntry) {
    GAME_TEST_QUEUE.lock().await.push(request);
}

pub async fn stop_game_tests() {
    // Keep the queue mutex held while publishing the stop request. drain_game_test_queue
    // takes the same mutex before consuming STOP_GAME_TESTS, so a stop+new-run command
    // cannot race between the runner-clear and queue-drain phases.
    let mut queue = GAME_TEST_QUEUE.lock().await;
    queue.clear();
    STOP_GAME_TESTS.store(true, Ordering::Release);
}

pub(super) async fn drain_game_test_queue(server: &Arc<Server>, runner: &mut GameTestRunner) {
    // Hold the same queue mutex used by stop_game_tests while consuming the stop
    // flag and draining requests. This closes the async race where a new /test run
    // could otherwise be drained before the old runner was cleared.
    let queued = {
        let mut queue = GAME_TEST_QUEUE.lock().await;
        if STOP_GAME_TESTS.swap(false, Ordering::AcqRel) {
            runner.clear();
        }
        std::mem::take(&mut *queue)
    };

    for request in queued {
        let test_id = request.test_id.clone();
        let report = request.report.clone();
        match prepare_test_run(server, request).await {
            Ok(run) => {
                info!(target: "pumpkin::gametest", test = %test_id, "Starting queued GameTest");
                runner.enqueue(run);
            }
            Err(error) => {
                warn!(
                    target: "pumpkin::gametest",
                    test = %test_id,
                    error = %error,
                    "Unable to start queued GameTest"
                );
                report.fail_to_start(&error);
            }
        }
    }
}

async fn prepare_test_run(
    server: &Arc<Server>,
    request: GameTestQueueEntry,
) -> GameTestResult<GameTestManager> {
    let test_instance = server
        .datapack_manager
        .get_test_instance(&request.test_id)
        .ok_or_else(|| {
            GameTestError::World(format!("Unknown test instance '{}'", request.test_id))
        })?;

    let structure = server
        .datapack_manager
        .load_structure(&test_instance.structure)
        .await
        .map_err(GameTestError::World)?;
    let template = GameTestStructureTemplate::from_nbt(&structure)?;
    let test = BlockBasedTest::new(request.test_id, test_instance);
    let report_sink: Arc<dyn GameTestReporter> = Arc::new(GameTestWorldReporter {
        world: request.world.clone(),
    });
    let adapter_world: Arc<dyn GameTestWorld> = Arc::new(ServerGameTestWorld {
        world: request.world,
        forced_chunks: StdMutex::new(HashSet::new()),
    });
    let extra_rotation = GameTestRotation::from_steps(request.rotation_steps);
    let run = GameTestSession::new_with_extra_rotation(
        test,
        adapter_world,
        Arc::new(template),
        request.test_x,
        request.test_z,
        extra_rotation,
    );

    Ok(GameTestManager::new(
        run,
        request.retry_options,
        request.report.clone(),
        report_sink,
    ))
}

fn broadcast_world(world: &World, message: &TextComponent) {
    let players = world.players.load_full();
    for player in players.iter() {
        player.send_system_message(message);
    }
}

struct ServerGameTestWorld {
    world: Arc<World>,
    /// Chunks for which this logical test owns one reference in
    /// `FORCED_GAME_TEST_CHUNKS`. The same adapter is retained across copy-reset
    /// retries, so the chunks stay active for the complete retry lifecycle.
    forced_chunks: StdMutex<HashSet<(i32, i32)>>,
}

impl ServerGameTestWorld {
    fn test_block_entity(&self, position: &BlockPos) -> GameTestResult<Arc<TestBlockBlockEntity>> {
        let entity = self.world.get_block_entity(position).ok_or_else(|| {
            GameTestError::World(format!("Missing test block entity at {position}"))
        })?;

        Arc::downcast::<TestBlockBlockEntity>(entity).map_err(|_| {
            GameTestError::World(format!("Block entity at {position} is not a test block"))
        })
    }

    fn test_instance_block_entity(
        &self,
        position: &BlockPos,
    ) -> GameTestResult<Arc<TestInstanceBlockBlockEntity>> {
        let entity = self.world.get_block_entity(position).ok_or_else(|| {
            GameTestError::World(format!("Missing test instance block entity at {position}"))
        })?;

        Arc::downcast::<TestInstanceBlockBlockEntity>(entity).map_err(|_| {
            GameTestError::World(format!(
                "Block entity at {position} is not a test instance block"
            ))
        })
    }

    fn sync_block_entity<T: BlockEntity + 'static>(&self, entity: Arc<T>) {
        let entity: Arc<dyn BlockEntity> = entity;
        self.world.update_block_entity(&entity);
    }

    async fn ensure_chunk_loaded(&self, position: &BlockPos) {
        let chunk = position.chunk_position();
        let chunk_key = (chunk.x, chunk.y);
        let needs_lease = self
            .forced_chunks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(chunk_key);

        if needs_lease {
            acquire_forced_game_test_chunk(&self.world, chunk);
        }

        // World::set_block_state is intentionally synchronous and only mutates an
        // already-loaded chunk. Vanilla force-loads the complete GameTest structure
        // before placing it; make the async GameTest adapter provide that guarantee.
        if !self.world.level.is_chunk_loaded(&chunk) {
            self.world.level.get_or_fetch_chunk(chunk, |_| ()).await;
        }
    }

    fn area_loaded_and_ticking(&self, min: &BlockPos, max: &BlockPos) -> bool {
        if max.0.x <= min.0.x || max.0.z <= min.0.z {
            return true;
        }

        let min_chunk_x = min.0.x >> 4;
        let max_chunk_x = (max.0.x - 1) >> 4;
        let min_chunk_z = min.0.z >> 4;
        let max_chunk_z = (max.0.z - 1) >> 4;
        let active_chunks = self
            .world
            .active_chunks
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        for chunk_x in min_chunk_x..=max_chunk_x {
            for chunk_z in min_chunk_z..=max_chunk_z {
                let chunk = Vector2::new(chunk_x, chunk_z);
                if !self.world.level.is_chunk_loaded(&chunk) || !active_chunks.contains(&chunk) {
                    return false;
                }
            }
        }
        true
    }
}

impl Drop for ServerGameTestWorld {
    fn drop(&mut self) {
        let chunks: Vec<_> = self
            .forced_chunks
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain()
            .collect();
        for (chunk_x, chunk_z) in chunks {
            release_forced_game_test_chunk(&self.world, Vector2::new(chunk_x, chunk_z));
        }
    }
}

fn acquire_forced_game_test_chunk(world: &World, chunk: Vector2<i32>) {
    let key = (world.uuid, chunk.x, chunk.y);
    let mut leases = FORCED_GAME_TEST_CHUNKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(lease) = leases.get_mut(&key) {
        lease.users = lease.users.saturating_add(1);
        return;
    }

    let was_forced = {
        let mut forced_chunks = world
            .forced_chunks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let was_forced = forced_chunks.contains(&chunk);
        forced_chunks.insert(chunk);
        was_forced
    };
    leases.insert(
        key,
        ForcedGameTestChunk {
            users: 1,
            was_forced,
        },
    );
}

fn release_forced_game_test_chunk(world: &World, chunk: Vector2<i32>) {
    let key = (world.uuid, chunk.x, chunk.y);
    let mut leases = FORCED_GAME_TEST_CHUNKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let release_world_chunk = match leases.get_mut(&key) {
        Some(lease) if lease.users > 1 => {
            lease.users -= 1;
            return;
        }
        Some(lease) => !lease.was_forced,
        None => return,
    };
    leases.remove(&key);
    drop(leases);

    if release_world_chunk {
        world
            .forced_chunks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&chunk);
    }
}

#[async_trait]
impl GameTestWorld for ServerGameTestWorld {
    async fn block_state_id(&self, position: &BlockPos) -> BlockStateId {
        self.world.get_block_state_id_async(position).await
    }

    async fn set_block_state(
        &self,
        position: &BlockPos,
        block_state_id: BlockStateId,
        flags: BlockFlags,
    ) -> GameTestResult<()> {
        self.ensure_chunk_loaded(position).await;
        self.world.set_block_state(position, block_state_id, flags);
        Ok(())
    }

    async fn rotate_block_state(
        &self,
        block_state_id: BlockStateId,
        rotation: GameTestRotation,
    ) -> GameTestResult<BlockStateId> {
        let (block, _) = BlockState::from_id_with_block(block_state_id);
        Ok(self
            .world
            .block_registry
            .rotate(block, block_state_id, rotation.as_block_rotation())
            .id)
    }

    async fn set_block_entity_nbt(
        &self,
        position: &BlockPos,
        nbt: &NbtCompound,
    ) -> GameTestResult<()> {
        // This normally follows set_block_state for the same position, but keeping
        // the guarantee here makes block-entity placement correct independently too.
        self.ensure_chunk_loaded(position).await;

        let mut nbt = nbt.clone();
        nbt.put_int("x", position.0.x);
        nbt.put_int("y", position.0.y);
        nbt.put_int("z", position.0.z);

        let entity = block_entity_from_nbt(&nbt).ok_or_else(|| {
            let id = nbt.get_string("id").unwrap_or("<missing id>");
            GameTestError::World(format!(
                "Unable to create block entity '{id}' at {position}"
            ))
        })?;

        self.world.remove_block_entity(position);
        self.world.add_block_entity(entity);
        Ok(())
    }

    async fn clear_non_player_entities(
        &self,
        min: &BlockPos,
        max: &BlockPos,
    ) -> GameTestResult<()> {
        let min_x = f64::from(min.0.x);
        let min_y = f64::from(min.0.y);
        let min_z = f64::from(min.0.z);
        let max_x = f64::from(max.0.x);
        let max_y = f64::from(max.0.y);
        let max_z = f64::from(max.0.z);

        // World::entities intentionally excludes players, matching vanilla's
        // `removeEntities` filter while avoiding any player removal path entirely.
        let entities = self.world.entities.load_full();
        let to_remove: Vec<_> = entities
            .iter()
            .filter(|entity| {
                let bounds = entity.get_entity().bounding_box.load();
                bounds.max.x > min_x
                    && bounds.min.x < max_x
                    && bounds.max.y > min_y
                    && bounds.min.y < max_y
                    && bounds.max.z > min_z
                    && bounds.min.z < max_z
            })
            .cloned()
            .collect();
        drop(entities);

        for entity in to_remove {
            self.world.remove_entity(entity.as_ref());
        }
        Ok(())
    }

    async fn clear_scheduled_block_ticks(
        &self,
        min: &BlockPos,
        max: &BlockPos,
    ) -> GameTestResult<()> {
        if max.0.x <= min.0.x || max.0.y <= min.0.y || max.0.z <= min.0.z {
            return Ok(());
        }

        let min_chunk_x = min.0.x >> 4;
        let max_chunk_x = (max.0.x - 1) >> 4;
        let min_chunk_z = min.0.z >> 4;
        let max_chunk_z = (max.0.z - 1) >> 4;

        for chunk_x in min_chunk_x..=max_chunk_x {
            for chunk_z in min_chunk_z..=max_chunk_z {
                let chunk_pos = Vector2::new(chunk_x, chunk_z);
                if let Some(chunk) = self.world.level.loaded_chunks.get(&chunk_pos) {
                    chunk.block_ticks.clear_area(min, max);
                    if !chunk.block_ticks.has_ticks() && !chunk.fluid_ticks.has_ticks() {
                        self.world
                            .level
                            .chunks_with_scheduled_ticks
                            .remove(&chunk_pos);
                    }
                }
            }
        }
        Ok(())
    }

    async fn clear_block_events(&self, min: &BlockPos, max: &BlockPos) -> GameTestResult<()> {
        self.world.clear_synced_block_events_in_box(min, max);
        Ok(())
    }

    async fn test_area_loaded_and_ticking(&self, min: &BlockPos, max: &BlockPos) -> bool {
        self.area_loaded_and_ticking(min, max)
    }

    async fn set_test_instance_running(&self, position: &BlockPos) -> GameTestResult<()> {
        let entity = self.test_instance_block_entity(position)?;
        entity.clear_error_markers();
        entity.set_running();
        self.sync_block_entity(entity);
        Ok(())
    }

    async fn set_test_instance_success(&self, position: &BlockPos) -> GameTestResult<()> {
        let entity = self.test_instance_block_entity(position)?;
        entity.clear_error_markers();
        entity.set_success();
        self.sync_block_entity(entity);
        Ok(())
    }

    async fn set_test_instance_failure(
        &self,
        position: &BlockPos,
        message: &str,
        marker: Option<(BlockPos, String)>,
    ) -> GameTestResult<()> {
        let entity = self.test_instance_block_entity(position)?;
        entity.clear_error_markers();
        if let Some((marker_position, marker_text)) = marker {
            entity.mark_error(marker_position, marker_text);
        }
        entity.set_error_message(message.to_string());
        self.sync_block_entity(entity);
        Ok(())
    }

    async fn trigger_test_block(&self, position: &BlockPos) -> GameTestResult<()> {
        self.test_block_entity(position)?.trigger(&self.world);
        Ok(())
    }

    async fn reset_test_block(&self, position: &BlockPos) -> GameTestResult<()> {
        self.test_block_entity(position)?.reset(&self.world);
        Ok(())
    }

    async fn test_block_triggered(&self, position: &BlockPos) -> GameTestResult<bool> {
        Ok(self.test_block_entity(position)?.has_triggered())
    }

    async fn test_block_message(&self, position: &BlockPos) -> GameTestResult<String> {
        Ok(self.test_block_entity(position)?.message())
    }

    async fn surface_height(&self, x: i32, z: i32) -> i32 {
        self.world
            .get_heightmap_height_async(ChunkHeightmapType::WorldSurface, x, z)
            .await
    }
}
