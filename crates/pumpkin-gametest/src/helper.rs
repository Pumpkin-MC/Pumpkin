use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, LazyLock, Mutex, RwLock};

use async_trait::async_trait;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::world::BlockFlags;

use crate::error::{GameTestError, GameTestResult};
use crate::model::{GameTestDefinition, GameTestRotation, TestType};
use crate::structure::TestStructureInstance;
use crate::world::GameTestWorld;

pub type BoxedGameTestFuture = Pin<Box<dyn Future<Output = GameTestResult<()>> + Send>>;
pub type GameTestCallback = Box<dyn FnMut(GameTestHelper) -> BoxedGameTestFuture + Send + Sync>;

fn box_callback<F, Fut>(mut callback: F) -> GameTestCallback
where
    F: FnMut(GameTestHelper) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = GameTestResult<()>> + Send + 'static,
{
    Box::new(move |helper| Box::pin(callback(helper)))
}

#[async_trait]
pub trait GameTestFunction: Send + Sync {
    async fn run(&self, helper: GameTestHelper) -> GameTestResult<()>;
}

#[async_trait]
impl<F, Fut> GameTestFunction for F
where
    F: Fn(GameTestHelper) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = GameTestResult<()>> + Send + 'static,
{
    async fn run(&self, helper: GameTestHelper) -> GameTestResult<()> {
        (self)(helper).await
    }
}

type TestFunctionMap = RwLock<HashMap<String, Arc<dyn GameTestFunction>>>;
type TestDefinitionMap = RwLock<HashMap<String, GameTestDefinition>>;

static TEST_FUNCTIONS: LazyLock<TestFunctionMap> = LazyLock::new(|| {
    let mut map: HashMap<String, Arc<dyn GameTestFunction>> = HashMap::new();
    let always_pass: Arc<dyn GameTestFunction> = Arc::new(|helper: GameTestHelper| async move {
        helper.succeed();
        Ok(())
    });
    map.insert("minecraft:always_pass".to_string(), always_pass);
    RwLock::new(map)
});

static REGISTERED_TEST_DEFINITIONS: LazyLock<TestDefinitionMap> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn register_test_function(
    id: impl Into<String>,
    function: impl GameTestFunction + 'static,
) -> Arc<dyn GameTestFunction> {
    let id = normalize_resource_id(id.into());
    let arc: Arc<dyn GameTestFunction> = Arc::new(function);
    TEST_FUNCTIONS
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(id, arc.clone());
    arc
}

pub fn register_function_test(
    id: impl Into<String>,
    structure: impl Into<String>,
    max_ticks: i32,
    setup_ticks: i32,
    function: impl GameTestFunction + 'static,
) {
    register_flaky_function_test(id, structure, max_ticks, setup_ticks, 1, 1, function);
}

pub fn register_flaky_function_test(
    id: impl Into<String>,
    structure: impl Into<String>,
    max_ticks: i32,
    setup_ticks: i32,
    max_attempts: i32,
    required_successes: i32,
    function: impl GameTestFunction + 'static,
) {
    let id = normalize_resource_id(id.into());
    let structure = normalize_resource_id(structure.into());
    register_test_function(id.clone(), function);
    let definition = GameTestDefinition {
        instance_type: TestType::Function,
        environment: serde_json::Value::String("minecraft:default".to_string()),
        structure,
        function: Some(id.clone()),
        max_ticks,
        setup_ticks,
        required: true,
        rotation: GameTestRotation::None,
        manual_only: false,
        max_attempts,
        required_successes,
        sky_access: false,
        padding: 0,
    };
    REGISTERED_TEST_DEFINITIONS
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(id, definition);
}

#[must_use]
pub fn get_test_function(id: &str) -> Option<Arc<dyn GameTestFunction>> {
    let normalized = normalize_resource_id(id.to_owned());
    TEST_FUNCTIONS
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&normalized)
        .cloned()
}

#[must_use]
pub fn registered_test_definitions() -> Vec<(String, GameTestDefinition)> {
    REGISTERED_TEST_DEFINITIONS
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn normalize_resource_id(id: String) -> String {
    if id.contains(':') {
        id
    } else {
        format!("minecraft:{id}")
    }
}

pub(crate) struct ScheduledCallback {
    pub(crate) tick: u32,
    pub(crate) callback: GameTestCallback,
}

pub(crate) enum SequenceEventKind {
    WaitUntil(GameTestCallback),
    Execute(GameTestCallback),
    ExecuteAfter {
        delta: u32,
        callback: GameTestCallback,
    },
    ExecuteFor {
        delta: u32,
        callback: GameTestCallback,
    },
    Succeed,
    Fail(String),
}

pub(crate) struct SequenceEvent {
    pub(crate) minimum_delay: Option<u32>,
    pub(crate) expected_delay: Option<u32>,
    pub(crate) kind: SequenceEventKind,
}

pub(crate) struct SequenceState {
    pub(crate) last_tick: u32,
    pub(crate) events: VecDeque<SequenceEvent>,
}

#[derive(Default)]
pub(crate) struct GameTestControllerState {
    pub(crate) succeeded: bool,
    pub(crate) error: Option<GameTestError>,
    pub(crate) final_check_added: bool,
    pub(crate) scheduled: Vec<ScheduledCallback>,
    pub(crate) sequences: Vec<SequenceState>,
}

impl GameTestControllerState {
    pub(crate) fn fail(&mut self, error: GameTestError) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    pub(crate) const fn succeed(&mut self) {
        if self.error.is_none() {
            self.succeeded = true;
        }
    }
}

pub struct GameTestSequence {
    controller: Arc<Mutex<GameTestControllerState>>,
    sequence_index: usize,
}

impl GameTestSequence {
    fn push_event(&self, event: SequenceEvent) {
        let mut state = self
            .controller
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(sequence) = state.sequences.get_mut(self.sequence_index) {
            sequence.events.push_back(event);
        }
    }

    #[must_use]
    pub fn then_wait_until<F, Fut>(self, assertion: F) -> Self
    where
        F: FnMut(GameTestHelper) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GameTestResult<()>> + Send + 'static,
    {
        self.push_event(SequenceEvent {
            minimum_delay: None,
            expected_delay: None,
            kind: SequenceEventKind::WaitUntil(box_callback(assertion)),
        });
        self
    }

    #[must_use]
    pub fn then_wait_until_after<F, Fut>(self, expected_delay: u32, assertion: F) -> Self
    where
        F: FnMut(GameTestHelper) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GameTestResult<()>> + Send + 'static,
    {
        self.push_event(SequenceEvent {
            minimum_delay: None,
            expected_delay: Some(expected_delay),
            kind: SequenceEventKind::WaitUntil(box_callback(assertion)),
        });
        self
    }

    #[must_use]
    pub fn then_wait_at_least<F, Fut>(self, minimum_delay: u32, assertion: F) -> Self
    where
        F: FnMut(GameTestHelper) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GameTestResult<()>> + Send + 'static,
    {
        self.push_event(SequenceEvent {
            minimum_delay: Some(minimum_delay),
            expected_delay: None,
            kind: SequenceEventKind::WaitUntil(box_callback(assertion)),
        });
        self
    }

    #[must_use]
    pub fn then_idle(self, delta: u32) -> Self {
        self.then_execute_after(delta, |_| async { Ok(()) })
    }

    #[must_use]
    pub fn then_execute<F, Fut>(self, action: F) -> Self
    where
        F: FnMut(GameTestHelper) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GameTestResult<()>> + Send + 'static,
    {
        self.push_event(SequenceEvent {
            minimum_delay: None,
            expected_delay: None,
            kind: SequenceEventKind::Execute(box_callback(action)),
        });
        self
    }

    #[must_use]
    pub fn then_execute_after<F, Fut>(self, delta: u32, action: F) -> Self
    where
        F: FnMut(GameTestHelper) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GameTestResult<()>> + Send + 'static,
    {
        self.push_event(SequenceEvent {
            minimum_delay: None,
            expected_delay: None,
            kind: SequenceEventKind::ExecuteAfter {
                delta,
                callback: box_callback(action),
            },
        });
        self
    }

    #[must_use]
    pub fn then_execute_for<F, Fut>(self, delta: u32, check: F) -> Self
    where
        F: FnMut(GameTestHelper) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GameTestResult<()>> + Send + 'static,
    {
        self.push_event(SequenceEvent {
            minimum_delay: None,
            expected_delay: None,
            kind: SequenceEventKind::ExecuteFor {
                delta,
                callback: box_callback(check),
            },
        });
        self
    }

    pub fn then_succeed(self) {
        self.push_event(SequenceEvent {
            minimum_delay: None,
            expected_delay: None,
            kind: SequenceEventKind::Succeed,
        });
    }

    pub fn then_fail(self, message: impl Into<String>) {
        self.push_event(SequenceEvent {
            minimum_delay: None,
            expected_delay: None,
            kind: SequenceEventKind::Fail(message.into()),
        });
    }
}

#[derive(Clone)]
pub struct GameTestHelper {
    world: Arc<dyn GameTestWorld>,
    placement: TestStructureInstance,
    tick: u32,
    max_ticks: u32,
    controller: Arc<Mutex<GameTestControllerState>>,
}

impl GameTestHelper {
    #[must_use]
    pub fn new(world: Arc<dyn GameTestWorld>, placement: TestStructureInstance, tick: u32) -> Self {
        Self {
            world,
            placement,
            tick,
            max_ticks: 100,
            controller: Arc::new(Mutex::new(GameTestControllerState::default())),
        }
    }

    #[must_use]
    pub(crate) const fn with_controller(
        world: Arc<dyn GameTestWorld>,
        placement: TestStructureInstance,
        tick: u32,
        max_ticks: u32,
        controller: Arc<Mutex<GameTestControllerState>>,
    ) -> Self {
        Self {
            world,
            placement,
            tick,
            max_ticks,
            controller,
        }
    }

    #[must_use]
    pub(crate) fn with_tick(&self, tick: u32) -> Self {
        let mut clone = self.clone();
        clone.tick = tick;
        clone
    }

    #[must_use]
    pub const fn tick(&self) -> u32 {
        self.tick
    }

    #[must_use]
    pub const fn max_ticks(&self) -> u32 {
        self.max_ticks
    }

    #[must_use]
    pub const fn placement(&self) -> &TestStructureInstance {
        &self.placement
    }

    #[must_use]
    pub const fn world(&self) -> &Arc<dyn GameTestWorld> {
        &self.world
    }

    #[must_use]
    pub const fn absolute_pos(&self, relative: &BlockPos) -> BlockPos {
        self.placement.transform(relative)
    }

    #[must_use]
    pub fn absolute_vec(&self, relative: Vector3<f64>) -> Vector3<f64> {
        self.placement.transform_vec(relative)
    }

    #[must_use]
    pub fn absolute_bounds(&self) -> (Vector3<f64>, Vector3<f64>) {
        let origin = self.placement.origin();
        let size = self.placement.size();
        (
            Vector3::new(
                f64::from(origin.0.x),
                f64::from(origin.0.y),
                f64::from(origin.0.z),
            ),
            Vector3::new(
                f64::from(origin.0.x + size[0]),
                f64::from(origin.0.y + size[1]),
                f64::from(origin.0.z + size[2]),
            ),
        )
    }

    pub async fn block_state_id(&self, relative: &BlockPos) -> BlockStateId {
        let position = self.absolute_pos(relative);
        self.world.block_state_id(&position).await
    }

    pub async fn set_block(
        &self,
        relative: &BlockPos,
        block_state_id: BlockStateId,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        self.world
            .set_block_state(&position, block_state_id, BlockFlags::NOTIFY_ALL)
            .await
    }

    pub async fn set_block_default(
        &self,
        relative: &BlockPos,
        block: &Block,
    ) -> GameTestResult<()> {
        self.set_block(relative, block.default_state.id).await
    }

    pub async fn set_block_with_nbt(
        &self,
        relative: &BlockPos,
        block_state_id: BlockStateId,
        nbt: &NbtCompound,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        self.world
            .set_block_state(&position, block_state_id, BlockFlags::NOTIFY_ALL)
            .await?;
        self.world.set_block_entity_nbt(&position, nbt).await
    }

    pub async fn set_air(&self, relative: &BlockPos) -> GameTestResult<()> {
        self.set_block(relative, Block::AIR.default_state.id).await
    }

    pub async fn block_entity_nbt(
        &self,
        relative: &BlockPos,
    ) -> GameTestResult<Option<NbtCompound>> {
        let position = self.absolute_pos(relative);
        self.world.block_entity_nbt(&position).await
    }

    pub async fn pulse_redstone(&self, relative: &BlockPos, duration: u32) -> GameTestResult<()> {
        self.set_block(relative, Block::REDSTONE_BLOCK.default_state.id)
            .await?;
        let pos = *relative;
        self.run_after_delay(
            duration,
            move |helper| async move { helper.set_air(&pos).await },
        );
        Ok(())
    }

    pub async fn press_button(&self, relative: &BlockPos) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        self.world.press_button(&position).await
    }

    pub async fn pull_lever(&self, relative: &BlockPos) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        self.world.pull_lever(&position).await
    }

    pub async fn use_block(&self, relative: &BlockPos) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        self.world.use_block(&position).await
    }

    pub async fn random_tick(&self, relative: &BlockPos) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        self.world.random_tick(&position).await
    }

    pub async fn spawn_entity(&self, entity_name: &str, relative: &BlockPos) -> GameTestResult<()> {
        let center = Vector3::new(
            f64::from(relative.0.x) + 0.5,
            f64::from(relative.0.y),
            f64::from(relative.0.z) + 0.5,
        );
        self.spawn_entity_at(entity_name, center).await
    }

    pub async fn spawn_entity_at(
        &self,
        entity_name: &str,
        relative: Vector3<f64>,
    ) -> GameTestResult<()> {
        let position = self.absolute_vec(relative);
        self.world.spawn_entity(entity_name, position).await
    }

    pub async fn spawn_item(
        &self,
        item_name: &str,
        count: u8,
        relative: &BlockPos,
    ) -> GameTestResult<()> {
        let center = Vector3::new(
            f64::from(relative.0.x) + 0.5,
            f64::from(relative.0.y) + 0.25,
            f64::from(relative.0.z) + 0.5,
        );
        self.spawn_item_at(item_name, count, center).await
    }

    pub async fn spawn_item_at(
        &self,
        item_name: &str,
        count: u8,
        relative: Vector3<f64>,
    ) -> GameTestResult<()> {
        let position = self.absolute_vec(relative);
        self.world.spawn_item(item_name, count, position).await
    }

    pub async fn set_container_item(
        &self,
        relative: &BlockPos,
        slot: usize,
        item_name: &str,
        count: u8,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        self.world
            .set_container_item(&position, slot, item_name, count)
            .await
    }

    pub async fn execute_command(&self, relative: &BlockPos, command: &str) -> GameTestResult<u32> {
        let pos = self.absolute_vec(Vector3::new(
            f64::from(relative.0.x) + 0.5,
            f64::from(relative.0.y),
            f64::from(relative.0.z) + 0.5,
        ));
        self.world.execute_command(pos, command).await
    }

    pub fn assert_true(&self, condition: bool, message: impl Into<String>) -> GameTestResult<()> {
        if condition {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: None,
            message: message.into(),
        })
    }

    pub fn assert_false(&self, condition: bool, message: impl Into<String>) -> GameTestResult<()> {
        self.assert_true(!condition, message)
    }

    pub async fn assert_block_state(
        &self,
        relative: &BlockPos,
        expected: BlockStateId,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        let actual = self.world.block_state_id(&position).await;

        if actual == expected {
            return Ok(());
        }

        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: format!("expected block state {expected:?}, found {actual:?}"),
        })
    }

    pub async fn assert_block_present(
        &self,
        relative: &BlockPos,
        expected: &Block,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        let actual_id = self.world.block_state_id(&position).await;
        let actual_block = Block::from_state_id(actual_id);

        if actual_block.id == expected.id {
            return Ok(());
        }

        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: format!(
                "expected block '{}', found '{}'",
                expected.name, actual_block.name
            ),
        })
    }

    pub async fn assert_block_not_present(
        &self,
        relative: &BlockPos,
        unexpected: &Block,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        let actual_id = self.world.block_state_id(&position).await;
        let actual_block = Block::from_state_id(actual_id);

        if actual_block.id != unexpected.id {
            return Ok(());
        }

        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: format!("unexpected block '{}' at {relative}", unexpected.name),
        })
    }

    pub async fn assert_entity_present(&self, entity_name: &str) -> GameTestResult<()> {
        let (min, max) = self.absolute_bounds();
        let count = self
            .world
            .entity_count_in_box(entity_name, min, max)
            .await?;
        if count > 0 {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: None,
            message: format!("expected entity '{entity_name}' in structure bounds"),
        })
    }

    pub async fn assert_entity_present_at(
        &self,
        entity_name: &str,
        relative: &BlockPos,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        let min = Vector3::new(
            f64::from(position.0.x),
            f64::from(position.0.y),
            f64::from(position.0.z),
        );
        let max = Vector3::new(min.x + 1.0, min.y + 1.0, min.z + 1.0);
        let count = self
            .world
            .entity_count_in_box(entity_name, min, max)
            .await?;
        if count > 0 {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: format!("expected entity '{entity_name}' at {relative}"),
        })
    }

    pub async fn assert_entity_not_present(&self, entity_name: &str) -> GameTestResult<()> {
        let (min, max) = self.absolute_bounds();
        let count = self
            .world
            .entity_count_in_box(entity_name, min, max)
            .await?;
        if count == 0 {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: None,
            message: format!(
                "unexpected entity '{entity_name}' in structure bounds (found {count})"
            ),
        })
    }

    pub async fn assert_entity_not_present_at(
        &self,
        entity_name: &str,
        relative: &BlockPos,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        let min = Vector3::new(
            f64::from(position.0.x),
            f64::from(position.0.y),
            f64::from(position.0.z),
        );
        let max = Vector3::new(min.x + 1.0, min.y + 1.0, min.z + 1.0);
        let count = self
            .world
            .entity_count_in_box(entity_name, min, max)
            .await?;
        if count == 0 {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: format!("unexpected entity '{entity_name}' at {relative}"),
        })
    }

    pub async fn assert_entities_present(
        &self,
        entity_name: &str,
        expected: usize,
    ) -> GameTestResult<()> {
        let (min, max) = self.absolute_bounds();
        let actual = self
            .world
            .entity_count_in_box(entity_name, min, max)
            .await?;
        if actual == expected {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: None,
            message: format!(
                "expected {expected} '{entity_name}' entities in structure bounds, found {actual}"
            ),
        })
    }

    pub async fn assert_entity_present_in_box(
        &self,
        entity_name: &str,
        relative_min: BlockPos,
        relative_max: BlockPos,
    ) -> GameTestResult<()> {
        let abs_a = self.absolute_pos(&relative_min);
        let abs_b = self.absolute_pos(&relative_max);
        let min = Vector3::new(
            f64::from(abs_a.0.x.min(abs_b.0.x)),
            f64::from(abs_a.0.y.min(abs_b.0.y)),
            f64::from(abs_a.0.z.min(abs_b.0.z)),
        );
        let max = Vector3::new(
            f64::from(abs_a.0.x.max(abs_b.0.x)),
            f64::from(abs_a.0.y.max(abs_b.0.y)),
            f64::from(abs_a.0.z.max(abs_b.0.z)),
        );
        let count = self
            .world
            .entity_count_in_box(entity_name, min, max)
            .await?;
        if count > 0 {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(abs_a),
            message: format!(
                "expected entity '{entity_name}' in box {relative_min}..{relative_max}"
            ),
        })
    }

    pub async fn assert_item_entity_present(
        &self,
        item_name: &str,
        relative: &BlockPos,
        radius: f64,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        let min = Vector3::new(
            f64::from(position.0.x) - radius,
            f64::from(position.0.y) - radius,
            f64::from(position.0.z) - radius,
        );
        let max = Vector3::new(
            f64::from(position.0.x + 1) + radius,
            f64::from(position.0.y + 1) + radius,
            f64::from(position.0.z + 1) + radius,
        );
        let count = self.world.item_count_in_box(item_name, min, max).await?;
        if count > 0 {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: format!("expected item entity '{item_name}' near {relative}"),
        })
    }

    pub async fn assert_item_entity_not_present(
        &self,
        item_name: &str,
        relative: &BlockPos,
        radius: f64,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        let min = Vector3::new(
            f64::from(position.0.x) - radius,
            f64::from(position.0.y) - radius,
            f64::from(position.0.z) - radius,
        );
        let max = Vector3::new(
            f64::from(position.0.x + 1) + radius,
            f64::from(position.0.y + 1) + radius,
            f64::from(position.0.z + 1) + radius,
        );
        let count = self.world.item_count_in_box(item_name, min, max).await?;
        if count == 0 {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: format!("unexpected item entity '{item_name}' near {relative}"),
        })
    }

    pub async fn assert_item_entity_count(
        &self,
        item_name: &str,
        relative: &BlockPos,
        radius: f64,
        expected: u32,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        let min = Vector3::new(
            f64::from(position.0.x) - radius,
            f64::from(position.0.y) - radius,
            f64::from(position.0.z) - radius,
        );
        let max = Vector3::new(
            f64::from(position.0.x + 1) + radius,
            f64::from(position.0.y + 1) + radius,
            f64::from(position.0.z + 1) + radius,
        );
        let actual = self.world.item_count_in_box(item_name, min, max).await?;
        if actual == expected {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: format!(
                "expected {expected} of item '{item_name}' near {relative}, found {actual}"
            ),
        })
    }

    pub async fn assert_container_empty(&self, relative: &BlockPos) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        if self.world.is_container_empty(&position).await? {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: format!("expected empty container at {relative}"),
        })
    }

    pub async fn assert_container_contains(
        &self,
        relative: &BlockPos,
        item_name: &str,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        let count = self
            .world
            .container_item_count(&position, item_name)
            .await?;
        if count > 0 {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: format!("expected container at {relative} to contain '{item_name}'"),
        })
    }

    pub async fn assert_container_contains_count(
        &self,
        relative: &BlockPos,
        item_name: &str,
        expected: u32,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        let actual = self
            .world
            .container_item_count(&position, item_name)
            .await?;
        if actual == expected {
            return Ok(());
        }
        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: format!(
                "expected container at {relative} to contain {expected} '{item_name}', found {actual}"
            ),
        })
    }

    pub fn succeed(&self) {
        self.controller
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .succeed();
    }

    pub fn fail(&self, message: impl Into<String>) -> GameTestError {
        let error = GameTestError::Assertion {
            tick: self.tick,
            position: None,
            message: message.into(),
        };
        self.controller
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .fail(error.clone());
        error
    }

    pub fn fail_at(&self, relative: &BlockPos, message: impl Into<String>) -> GameTestError {
        let position = self.absolute_pos(relative);
        let error = GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: message.into(),
        };
        self.controller
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .fail(error.clone());
        error
    }

    pub fn run_at_tick_time<F, Fut>(&self, tick: u32, callback: F)
    where
        F: FnMut(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GameTestResult<()>> + Send + 'static,
    {
        self.controller
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .scheduled
            .push(ScheduledCallback {
                tick,
                callback: box_callback(callback),
            });
    }

    pub fn run_after_delay<F, Fut>(&self, delay: u32, callback: F)
    where
        F: FnMut(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GameTestResult<()>> + Send + 'static,
    {
        self.run_at_tick_time(self.tick.saturating_add(delay), callback);
    }

    pub fn on_each_tick<F, Fut>(&self, callback: F)
    where
        F: Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GameTestResult<()>> + Send + 'static,
    {
        let shared = Arc::new(callback);
        for tick in self.tick..self.max_ticks {
            let shared = shared.clone();
            self.run_at_tick_time(tick, move |helper| {
                let shared = shared.clone();
                async move { (shared)(helper).await }
            });
        }
    }

    #[must_use]
    pub fn start_sequence(&self) -> GameTestSequence {
        let mut state = self
            .controller
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let sequence_index = state.sequences.len();
        state.sequences.push(SequenceState {
            last_tick: self.tick,
            events: VecDeque::new(),
        });
        drop(state);
        GameTestSequence {
            controller: self.controller.clone(),
            sequence_index,
        }
    }

    fn ensure_single_final_check(&self) -> GameTestResult<()> {
        let mut state = self
            .controller
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.final_check_added {
            return Err(GameTestError::Assertion {
                tick: self.tick,
                position: None,
                message: "This test already has a final clause".to_string(),
            });
        }
        state.final_check_added = true;
        Ok(())
    }

    pub fn succeed_when<F, Fut>(&self, asserter: F) -> GameTestResult<()>
    where
        F: FnMut(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GameTestResult<()>> + Send + 'static,
    {
        self.ensure_single_final_check()?;
        self.start_sequence()
            .then_wait_until(asserter)
            .then_succeed();
        Ok(())
    }

    pub fn succeed_if<F, Fut>(&self, asserter: F) -> GameTestResult<()>
    where
        F: FnMut(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GameTestResult<()>> + Send + 'static,
    {
        self.ensure_single_final_check()?;
        self.start_sequence()
            .then_wait_until_after(0, asserter)
            .then_succeed();
        Ok(())
    }

    pub fn succeed_on_tick_when<F, Fut>(&self, tick: u32, asserter: F) -> GameTestResult<()>
    where
        F: FnMut(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GameTestResult<()>> + Send + 'static,
    {
        self.ensure_single_final_check()?;
        self.start_sequence()
            .then_wait_until_after(tick, asserter)
            .then_succeed();
        Ok(())
    }
}
