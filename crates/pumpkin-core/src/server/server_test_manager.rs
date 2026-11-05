use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex as StdMutex};

use async_trait::async_trait;
use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_gametest::{
    BlockBasedTest, GameTestError, GameTestManager, GameTestReporter, GameTestResult,
    GameTestRotation, GameTestRunner, GameTestSession, GameTestStructureTemplate, GameTestWorld,
    TestType,
};
pub use pumpkin_gametest::{GameTestBatchReport, GameTestRetryOptions};
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use pumpkin_util::text::TextComponent;
use pumpkin_world::{chunk::ChunkHeightmapType, world::BlockFlags};
use tracing::{info, warn};

use crate::{
    block::entities::{
        BlockEntity, block_entity_from_nbt, test_block::TestBlockBlockEntity,
        test_instance_block::TestInstanceBlockBlockEntity,
    },
    server::Server,
    world::World,
};

#[derive(Default)]
pub(crate) struct GameTestServerState {
    queue: StdMutex<Vec<GameTestQueueEntry>>,
    stop: AtomicBool,
    pending: AtomicUsize,
}

#[must_use]
pub fn has_pending_game_tests(server: &Server) -> bool {
    server.game_test_state.stop.load(Ordering::Acquire)
        || server.game_test_state.pending.load(Ordering::Acquire) > 0
}

pub struct GameTestQueueEntry {
    test_id: String,
    world: Arc<World>,
    test_x: i32,
    test_y: Option<i32>,
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
            test_y: None,
            test_z,
            rotation_steps,
            retry_options,
            report,
        }
    }

    #[must_use]
    pub const fn with_y(mut self, test_y: i32) -> Self {
        self.test_y = Some(test_y);
        self
    }
}

struct GameTestWorldReporter {
    world: Arc<World>,
    fallback: Arc<dyn GameTestReporter>,
}

impl GameTestReporter for GameTestWorldReporter {
    fn send_message(&self, message: TextComponent) {
        let players = self.world.players.load_full();
        if players.is_empty() {
            self.fallback.send_message(message);
        } else {
            for player in players.iter() {
                player.send_system_message(&message);
            }
        }
    }
}

pub fn enqueue_game_test(request: GameTestQueueEntry) {
    let Some(server) = request.world.server.upgrade() else {
        request.report.fail_to_start(&GameTestError::World(
            "World is not attached to a running server".to_string(),
        ));
        return;
    };
    let mut queue = server
        .game_test_state
        .queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    queue.push(request);
    server
        .game_test_state
        .pending
        .store(queue.len(), Ordering::Release);
}

pub fn stop_game_tests(server: &Server) {
    // Keep the queue mutex held while publishing the stop request. drain_game_test_queue
    // takes the same mutex before consuming the stop flag, so a stop+new-run command
    // cannot race between the runner-clear and queue-drain phases.
    let mut queue = server
        .game_test_state
        .queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    queue.clear();
    server.game_test_state.pending.store(0, Ordering::Release);
    server.game_test_state.stop.store(true, Ordering::Release);
}

pub async fn drain_game_test_queue(server: &Arc<Server>, runner: &mut GameTestRunner) {
    // Hold the same queue mutex used by stop_game_tests while consuming the stop
    // flag and draining requests. This closes the async race where a new /test run
    // could otherwise be drained before the old runner was cleared.
    let queued = {
        let mut queue = server
            .game_test_state
            .queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if server.game_test_state.stop.swap(false, Ordering::AcqRel) {
            runner.clear();
        }
        server.game_test_state.pending.store(0, Ordering::Release);
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

pub async fn prepare_test_run(
    server: &Arc<Server>,
    request: GameTestQueueEntry,
) -> GameTestResult<GameTestManager> {
    let report_sink: Arc<dyn GameTestReporter> = Arc::new(GameTestWorldReporter {
        world: request.world.clone(),
        fallback: request.report.reporter().clone(),
    });
    prepare_test_run_with_reporter(server, request, report_sink).await
}

pub async fn prepare_test_run_with_reporter(
    server: &Arc<Server>,
    request: GameTestQueueEntry,
    report_sink: Arc<dyn GameTestReporter>,
) -> GameTestResult<GameTestManager> {
    crate::server::builtin_game_tests::ensure_builtin_game_tests_registered();
    let test_instance = server
        .datapack_manager
        .get_test_instance(&request.test_id)
        .or_else(|| {
            pumpkin_gametest::registered_test_definitions()
                .into_iter()
                .find_map(|(id, def)| (id == request.test_id).then_some(def))
        })
        .ok_or_else(|| {
            GameTestError::World(format!("Unknown test instance '{}'", request.test_id))
        })?;

    let function = match test_instance.instance_type {
        TestType::BlockBased => None,
        TestType::Function => {
            let function_id = test_instance.function.as_deref().ok_or_else(|| {
                GameTestError::World(format!(
                    "Function test instance '{}' is missing a 'function' field",
                    request.test_id
                ))
            })?;
            let func = pumpkin_gametest::get_test_function(function_id).ok_or_else(|| {
                GameTestError::World(format!(
                    "Unknown GameTest function '{function_id}' for test '{}'",
                    request.test_id
                ))
            })?;
            Some(func)
        }
    };

    let template = match server
        .datapack_manager
        .load_structure(&test_instance.structure)
        .await
    {
        Ok(structure) => GameTestStructureTemplate::from_nbt(&structure)?,
        Err(error) => GameTestStructureTemplate::parse_empty_id(&test_instance.structure)
            .ok_or(GameTestError::World(error))?,
    };
    let mut test = BlockBasedTest::new(request.test_id, test_instance);
    if let Some(function) = function {
        test = test.with_function(function);
    }
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
    )
    .with_y(request.test_y);

    Ok(GameTestManager::new(
        run,
        request.retry_options,
        request.report.clone(),
        report_sink,
    ))
}

struct ServerGameTestWorld {
    world: Arc<World>,
    /// Chunks leased by this test across copy-reset retries.
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
                if !self.world.level.is_chunk_loaded(&chunk)
                    || !active_chunks.contains(&chunk)
                    || !self
                        .world
                        .level
                        .read_entity_chunk_sync(&chunk, |entities| {
                            entities.live.load(Ordering::Acquire)
                        })
                        .unwrap_or(false)
                {
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

fn acquire_forced_game_test_chunk(world: &Arc<World>, chunk: Vector2<i32>) {
    let mut leases = world
        .game_test_forced_chunks
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(users) = leases.get_mut(&chunk) {
        *users = users.saturating_add(1);
        return;
    }
    let mut chunk_loading = world
        .level
        .chunk_loading
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    chunk_loading.add_force_ticket(chunk);
    chunk_loading.send_change();
    drop(chunk_loading);
    leases.insert(chunk, 1);
    drop(leases);
    world.update_active_chunks();
}

fn release_forced_game_test_chunk(world: &Arc<World>, chunk: Vector2<i32>) {
    let mut leases = world
        .game_test_forced_chunks
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match leases.get_mut(&chunk) {
        Some(users) if *users > 1 => {
            *users -= 1;
            return;
        }
        Some(_) => {}
        None => return,
    }
    leases.remove(&chunk);

    let mut chunk_loading = world
        .level
        .chunk_loading
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    chunk_loading.remove_force_ticket(chunk);
    chunk_loading.send_change();
    drop(chunk_loading);

    drop(leases);
    world.update_active_chunks();
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
        if max.0.x > min.0.x && max.0.z > min.0.z {
            for chunk_x in (min.0.x >> 4)..=((max.0.x - 1) >> 4) {
                for chunk_z in (min.0.z >> 4)..=((max.0.z - 1) >> 4) {
                    let pos = Vector2::new(chunk_x, chunk_z);
                    self.ensure_chunk_loaded(&BlockPos::new(chunk_x * 16, 0, chunk_z * 16))
                        .await;
                    let chunk = self.world.level.get_entity_chunk(pos).await;
                    self.world.activate_entity_chunk(&chunk);
                }
            }
        }

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
        if max.0.x > min.0.x && max.0.z > min.0.z {
            let min_chunk_x = min.0.x >> 4;
            let max_chunk_x = (max.0.x - 1) >> 4;
            let min_chunk_z = min.0.z >> 4;
            let max_chunk_z = (max.0.z - 1) >> 4;
            for chunk_x in min_chunk_x..=max_chunk_x {
                for chunk_z in min_chunk_z..=max_chunk_z {
                    self.ensure_chunk_loaded(&BlockPos::new(chunk_x * 16, 0, chunk_z * 16))
                        .await;
                }
            }
        }
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
        self.ensure_chunk_loaded(&BlockPos::new(x, 0, z)).await;
        self.world
            .get_heightmap_height_async(ChunkHeightmapType::WorldSurface, x, z)
            .await
    }

    async fn block_entity_nbt(&self, position: &BlockPos) -> GameTestResult<Option<NbtCompound>> {
        self.ensure_chunk_loaded(position).await;
        let Some(entity) = self.world.get_block_entity(position) else {
            return Ok(None);
        };
        let mut nbt = NbtCompound::new();
        entity.write_internal(&mut nbt);
        Ok(Some(nbt))
    }

    async fn spawn_entity(
        &self,
        entity_name: &str,
        position: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> GameTestResult<()> {
        let normalized = entity_name
            .strip_prefix("minecraft:")
            .unwrap_or(entity_name);
        let entity_type = pumpkin_data::entity::EntityType::from_name(normalized)
            .ok_or_else(|| GameTestError::World(format!("Unknown entity type '{entity_name}'")))?;
        self.ensure_chunk_loaded(&BlockPos::floored_v(position))
            .await;
        let entity = crate::entity::r#type::from_type(
            entity_type,
            position,
            &self.world,
            uuid::Uuid::new_v4(),
        );
        if let Some(mob) = entity.get_mob() {
            mob.get_mob_entity()
                .persistence_required
                .store(true, Ordering::Relaxed);
        }
        self.world.spawn_entity(entity);
        Ok(())
    }

    async fn spawn_item(
        &self,
        item_name: &str,
        count: u8,
        position: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> GameTestResult<()> {
        let normalized = item_name.strip_prefix("minecraft:").unwrap_or(item_name);
        let item = pumpkin_data::item::Item::from_registry_key(normalized)
            .ok_or_else(|| GameTestError::World(format!("Unknown item '{item_name}'")))?;
        self.ensure_chunk_loaded(&BlockPos::floored_v(position))
            .await;
        let base_entity = crate::entity::Entity::new(
            self.world.clone(),
            position,
            &pumpkin_data::entity::EntityType::ITEM,
        );
        let item_entity = Arc::new(crate::entity::item::ItemEntity::new_with_velocity(
            base_entity,
            pumpkin_data::item_stack::ItemStack::new(count, item),
            pumpkin_util::math::vector3::Vector3::default(),
            0,
        ));
        self.world.spawn_entity(item_entity);
        Ok(())
    }

    async fn entity_count_in_box(
        &self,
        entity_name: &str,
        min: pumpkin_util::math::vector3::Vector3<f64>,
        max: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> GameTestResult<usize> {
        let normalized = entity_name
            .strip_prefix("minecraft:")
            .unwrap_or(entity_name);
        let box_bounds = pumpkin_util::math::boundingbox::BoundingBox::new(min, max);
        let entities = self.world.entities.load_full();
        let count = entities
            .iter()
            .filter(|entity| {
                let base = entity.get_entity();
                !base.is_removed()
                    && base.entity_type.resource_name == normalized
                    && base.bounding_box.load().intersects(&box_bounds)
            })
            .count();
        Ok(count)
    }

    async fn item_count_in_box(
        &self,
        item_name: &str,
        min: pumpkin_util::math::vector3::Vector3<f64>,
        max: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> GameTestResult<u32> {
        let normalized = item_name.strip_prefix("minecraft:").unwrap_or(item_name);
        let target_item = pumpkin_data::item::Item::from_registry_key(normalized)
            .ok_or_else(|| GameTestError::World(format!("Unknown item '{item_name}'")))?;
        let box_bounds = pumpkin_util::math::boundingbox::BoundingBox::new(min, max);
        let entities = self.world.entities.load_full();
        let mut total = 0u32;
        for entity in entities.iter() {
            let base = entity.get_entity();
            if base.is_removed() || !base.bounding_box.load().intersects(&box_bounds) {
                continue;
            }
            if let Some(item_entity) = entity.get_item_entity() {
                let stack = item_entity
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if !stack.is_empty() && stack.get_item().id == target_item.id {
                    total = total.saturating_add(u32::from(stack.item_count));
                }
            }
        }
        Ok(total)
    }

    async fn container_item_count(
        &self,
        position: &BlockPos,
        item_name: &str,
    ) -> GameTestResult<u32> {
        self.ensure_chunk_loaded(position).await;
        let normalized = item_name.strip_prefix("minecraft:").unwrap_or(item_name);
        let target_item = pumpkin_data::item::Item::from_registry_key(normalized)
            .ok_or_else(|| GameTestError::World(format!("Unknown item '{item_name}'")))?;
        let entity =
            self.world
                .get_block_entity(position)
                .ok_or_else(|| GameTestError::Assertion {
                    message: format!("Expected container block entity at {position}"),
                    position: Some(*position),
                    tick: 0,
                })?;
        let inventory = entity
            .get_inventory()
            .ok_or_else(|| GameTestError::Assertion {
                message: format!("Block entity at {position} is not a container"),
                position: Some(*position),
                tick: 0,
            })?;
        let mut total = 0u32;
        for slot in 0..inventory.size() {
            let stack = inventory.get_stack(slot);
            if !stack.is_empty() && stack.get_item().id == target_item.id {
                total = total.saturating_add(u32::from(stack.item_count));
            }
        }
        Ok(total)
    }

    async fn is_container_empty(&self, position: &BlockPos) -> GameTestResult<bool> {
        self.ensure_chunk_loaded(position).await;
        let entity =
            self.world
                .get_block_entity(position)
                .ok_or_else(|| GameTestError::Assertion {
                    message: format!("Expected container block entity at {position}"),
                    position: Some(*position),
                    tick: 0,
                })?;
        let inventory = entity
            .get_inventory()
            .ok_or_else(|| GameTestError::Assertion {
                message: format!("Block entity at {position} is not a container"),
                position: Some(*position),
                tick: 0,
            })?;
        Ok(inventory.is_empty())
    }

    async fn set_container_item(
        &self,
        position: &BlockPos,
        slot: usize,
        item_name: &str,
        count: u8,
    ) -> GameTestResult<()> {
        self.ensure_chunk_loaded(position).await;
        let normalized = item_name.strip_prefix("minecraft:").unwrap_or(item_name);
        let item = pumpkin_data::item::Item::from_registry_key(normalized)
            .ok_or_else(|| GameTestError::World(format!("Unknown item '{item_name}'")))?;
        let entity =
            self.world
                .get_block_entity(position)
                .ok_or_else(|| GameTestError::Assertion {
                    message: format!("Expected container block entity at {position}"),
                    position: Some(*position),
                    tick: 0,
                })?;
        let inventory = entity
            .get_inventory()
            .ok_or_else(|| GameTestError::Assertion {
                message: format!("Block entity at {position} is not a container"),
                position: Some(*position),
                tick: 0,
            })?;
        if slot >= inventory.size() {
            return Err(GameTestError::Assertion {
                message: format!(
                    "Slot {slot} is out of bounds for container at {position} (size {})",
                    inventory.size()
                ),
                position: Some(*position),
                tick: 0,
            });
        }
        inventory.set_stack(slot, pumpkin_data::item_stack::ItemStack::new(count, item));
        inventory.mark_dirty();
        self.world.update_neighbors(position, None);
        Ok(())
    }

    async fn press_button(&self, position: &BlockPos) -> GameTestResult<()> {
        use pumpkin_data::block_properties::BlockProperties;
        self.ensure_chunk_loaded(position).await;
        let block = self.world.get_block(position);
        if !pumpkin_data::block_properties::LeverLikeProperties::handles_block_id(block.id)
            || block == &pumpkin_data::Block::LEVER
        {
            return Err(GameTestError::Assertion {
                message: format!("Expected a button at {position}, found {}", block.name),
                position: Some(*position),
                tick: 0,
            });
        }
        crate::block::blocks::redstone::buttons::ButtonBlock::press(&self.world, position);
        Ok(())
    }

    async fn pull_lever(&self, position: &BlockPos) -> GameTestResult<()> {
        self.ensure_chunk_loaded(position).await;
        let block = self.world.get_block(position);
        if block != &pumpkin_data::Block::LEVER {
            return Err(GameTestError::Assertion {
                message: format!("Expected a lever at {position}, found {}", block.name),
                position: Some(*position),
                tick: 0,
            });
        }
        crate::block::blocks::redstone::lever::LeverBlock::pull(&self.world, position);
        Ok(())
    }

    async fn random_tick(&self, position: &BlockPos) -> GameTestResult<()> {
        self.ensure_chunk_loaded(position).await;
        let (block, state_id) = self.world.get_block_and_state_id(position);
        if let Some(pumpkin_block) = self.world.block_registry.get_pumpkin_block(block.id) {
            pumpkin_block.random_tick(crate::block::RandomTickArgs {
                world: &self.world,
                block,
                position,
            });
        }
        if let Some(fluid) = pumpkin_data::fluid::Fluid::from_state_id(state_id)
            && let Some(pumpkin_fluid) = self.world.block_registry.get_pumpkin_fluid(fluid.id)
        {
            pumpkin_fluid.random_tick(fluid, &self.world, position);
        }
        Ok(())
    }

    async fn execute_command(
        &self,
        position: pumpkin_util::math::vector3::Vector3<f64>,
        command: &str,
    ) -> GameTestResult<u32> {
        let Some(server) = self.world.server.upgrade() else {
            return Err(GameTestError::World(
                "World is not attached to a running server".to_string(),
            ));
        };
        let command = command.strip_prefix('/').unwrap_or(command);
        let source = crate::command::CommandSource::new(
            crate::command::CommandSender::Console,
            self.world.clone(),
            None,
            position,
            pumpkin_util::math::vector2::Vector2::new(0.0, 0.0),
            "GameTest".to_string(),
            TextComponent::text("GameTest"),
            server.clone(),
        )
        .with_silent();
        let dispatcher = server.command_dispatcher.load();
        let count =
            dispatcher
                .execute_input(command, &source)
                .map_err(|err| GameTestError::Assertion {
                    message: err.message.get_text(),
                    position: Some(BlockPos::floored_v(position)),
                    tick: 0,
                })?;
        Ok(count.max(0).cast_unsigned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world as create_world;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_world::chunk::ChunkData;
    use std::time::Duration;

    #[tokio::test]
    async fn forced_chunk_waits_for_entities_then_becomes_ready()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = create_world(&folder);
        let chunk = Vector2::new(0, 0);
        world
            .level
            .loaded_chunks
            .insert(chunk, ChunkData::empty_sync(0, 0));
        world.level.spawn_entity_generation(chunk);

        let adapter = ServerGameTestWorld {
            world: world.clone(),
            forced_chunks: StdMutex::new(HashSet::new()),
        };
        let min = BlockPos::new(0, 64, 0);
        let max = BlockPos::new(16, 65, 16);
        adapter.ensure_chunk_loaded(&min).await;
        assert!(!adapter.area_loaded_and_ticking(&min, &max));

        tokio::time::timeout(Duration::from_secs(2), async {
            while !adapter.area_loaded_and_ticking(&min, &max) {
                tokio::task::yield_now().await;
            }
        })
        .await?;
        assert!(world.level.clean_memory().is_empty());

        drop(adapter);
        assert_eq!(world.level.clean_memory(), vec![chunk]);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn clearing_test_entities_removes_saved_entities()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let pos = Vector2::new(0, 0);
        let min = BlockPos::new(0, 64, 0);
        let max = BlockPos::new(16, 80, 16);

        let world = create_world(&folder);
        world
            .level
            .loaded_chunks
            .insert(pos, ChunkData::empty_sync(0, 0));
        let adapter = ServerGameTestWorld {
            world: world.clone(),
            forced_chunks: StdMutex::new(HashSet::new()),
        };
        adapter
            .spawn_entity("minecraft:pig", Vector3::new(8.5, 65.0, 8.5))
            .await?;
        assert_eq!(world.entities.load().len(), 1);
        drop(adapter);
        world.shutdown().await;
        drop(world);

        let world = create_world(&folder);
        world
            .level
            .loaded_chunks
            .insert(pos, ChunkData::empty_sync(0, 0));
        let chunk = world.level.get_entity_chunk(pos).await;
        assert_eq!(
            chunk
                .data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len(),
            1
        );
        assert!(world.entities.load().is_empty());

        let adapter = ServerGameTestWorld {
            world: world.clone(),
            forced_chunks: StdMutex::new(HashSet::new()),
        };
        adapter.clear_non_player_entities(&min, &max).await?;
        assert!(world.entities.load().is_empty());
        drop(adapter);
        world.shutdown().await;
        drop(world);

        let world = create_world(&folder);
        let chunk = world.level.get_entity_chunk(pos).await;
        assert!(
            chunk
                .data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
        );
        world.level.shutdown().await;
        Ok(())
    }
}
