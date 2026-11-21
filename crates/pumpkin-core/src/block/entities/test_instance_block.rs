use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_data::translation::java as translation;
use pumpkin_data::{Block, Mirror, Rotation};
use pumpkin_gametest::{
    GameTestBatchReport, GameTestDefinition, GameTestReporter, GameTestRetryOptions,
    GameTestStructureTemplate, TestType,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use pumpkin_world::world::BlockFlags;

use crate::block::entities::structure_block::{
    StructurePlacementSettings, load_or_cache_structure_template, parse_structure_identifier,
    place_structure_template_in_world, save_structure_from_world,
};
use crate::entity::player::Player;
use crate::server::Server;
use crate::server::server_test_manager::{GameTestQueueEntry, enqueue_game_test, stop_game_tests};
use crate::world::World;

use super::BlockEntity;

const STRUCTURE_OFFSET: [i32; 3] = [0, 1, 1];
pub const MAX_TEST_STRUCTURE_SIZE: i32 = 128;
const INVALID_TEST_TRANSLATION_KEY: &str = "test_instance_block.invalid_test";
const DEFAULT_TEST_DIMENSION: &str = "minecraft:overworld";

struct PlayerTestReporter {
    player: Arc<Player>,
}

impl GameTestReporter for PlayerTestReporter {
    fn send_message(&self, message: TextComponent) {
        self.player.send_system_message(&message);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TestInstanceRotation {
    #[default]
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl TestInstanceRotation {
    #[must_use]
    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Clockwise90 => "clockwise_90",
            Self::Clockwise180 => "180",
            Self::Counterclockwise90 => "counterclockwise_90",
        }
    }

    #[must_use]
    pub const fn from_serialized_name(name: &str) -> Option<Self> {
        match name.as_bytes() {
            b"none" => Some(Self::None),
            b"clockwise_90" => Some(Self::Clockwise90),
            b"180" => Some(Self::Clockwise180),
            b"counterclockwise_90" => Some(Self::Counterclockwise90),
            _ => None,
        }
    }

    #[must_use]
    pub const fn from_block_rotation(rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => Self::None,
            Rotation::Clockwise90 => Self::Clockwise90,
            Rotation::Rotate180 => Self::Clockwise180,
            Rotation::CounterClockwise90 => Self::Counterclockwise90,
        }
    }

    #[must_use]
    pub const fn as_block_rotation(self) -> Rotation {
        match self {
            Self::None => Rotation::None,
            Self::Clockwise90 => Rotation::Clockwise90,
            Self::Clockwise180 => Rotation::Rotate180,
            Self::Counterclockwise90 => Rotation::CounterClockwise90,
        }
    }

    #[must_use]
    pub const fn rotation_steps(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Clockwise90 => 1,
            Self::Clockwise180 => 2,
            Self::Counterclockwise90 => 3,
        }
    }

    #[must_use]
    pub const fn transform_size(self, size: [i32; 3]) -> [i32; 3] {
        match self {
            Self::None | Self::Clockwise180 => size,
            Self::Clockwise90 | Self::Counterclockwise90 => [size[2], size[1], size[0]],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TestInstanceStatus {
    #[default]
    Cleared,
    Running,
    Finished,
}

impl TestInstanceStatus {
    #[must_use]
    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::Cleared => "cleared",
            Self::Running => "running",
            Self::Finished => "finished",
        }
    }

    #[must_use]
    pub const fn from_serialized_name(name: &str) -> Option<Self> {
        match name.as_bytes() {
            b"cleared" => Some(Self::Cleared),
            b"running" => Some(Self::Running),
            b"finished" => Some(Self::Finished),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestInstanceData {
    pub test: Option<String>,
    pub size: [i32; 3],
    pub rotation: TestInstanceRotation,
    pub ignore_entities: bool,
    pub status: TestInstanceStatus,
    pub error_message: Option<TextComponent>,
}

impl Default for TestInstanceData {
    fn default() -> Self {
        Self {
            test: None,
            size: [0, 0, 0],
            rotation: TestInstanceRotation::None,
            ignore_entities: false,
            status: TestInstanceStatus::Cleared,
            error_message: None,
        }
    }
}

impl TestInstanceData {
    #[must_use]
    pub fn to_nbt(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        if let Some(ref test) = self.test {
            nbt.put_string("test", test.clone());
        }
        nbt.put(
            "size",
            NbtTag::IntArray(vec![
                self.size[0].clamp(0, MAX_TEST_STRUCTURE_SIZE),
                self.size[1].clamp(0, MAX_TEST_STRUCTURE_SIZE),
                self.size[2].clamp(0, MAX_TEST_STRUCTURE_SIZE),
            ]),
        );
        nbt.put_string("rotation", self.rotation.serialized_name().to_string());
        nbt.put_bool("ignore_entities", self.ignore_entities);
        nbt.put_string("status", self.status.serialized_name().to_string());
        if let Some(ref error_message) = self.error_message {
            nbt.put(
                "error_message",
                error_message.to_nbt_tag_for_version(&CURRENT_MC_VERSION),
            );
        }
        nbt
    }

    #[must_use]
    pub fn from_nbt(nbt: &NbtCompound) -> Self {
        let test = nbt
            .get_string("test")
            .and_then(|raw| parse_structure_identifier(raw).map(|(_, _, id)| id));
        let size = read_clamped_size(nbt);
        let rotation = nbt
            .get_string("rotation")
            .and_then(TestInstanceRotation::from_serialized_name)
            .unwrap_or(TestInstanceRotation::None);
        let ignore_entities = nbt.get_bool("ignore_entities").unwrap_or(false);
        let status = nbt
            .get_string("status")
            .and_then(TestInstanceStatus::from_serialized_name)
            .unwrap_or(TestInstanceStatus::Cleared);
        let error_message = nbt.get("error_message").map(TextComponent::from_nbt);

        Self {
            test,
            size,
            rotation,
            ignore_entities,
            status,
            error_message,
        }
    }
}

fn read_clamped_size(nbt: &NbtCompound) -> [i32; 3] {
    match nbt.get("size") {
        Some(NbtTag::IntArray(arr)) if arr.len() == 3 => [
            arr[0].clamp(0, MAX_TEST_STRUCTURE_SIZE),
            arr[1].clamp(0, MAX_TEST_STRUCTURE_SIZE),
            arr[2].clamp(0, MAX_TEST_STRUCTURE_SIZE),
        ],
        Some(NbtTag::List(list)) if list.len() == 3 => {
            let x = list[0].extract_int().unwrap_or(0);
            let y = list[1].extract_int().unwrap_or(0);
            let z = list[2].extract_int().unwrap_or(0);
            [
                x.clamp(0, MAX_TEST_STRUCTURE_SIZE),
                y.clamp(0, MAX_TEST_STRUCTURE_SIZE),
                z.clamp(0, MAX_TEST_STRUCTURE_SIZE),
            ]
        }
        _ => [0, 0, 0],
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestInstanceErrorMarker {
    pub position: BlockPos,
    pub text: String,
}

#[must_use]
pub fn lookup_test_instance(server: &Server, raw_test_id: &str) -> Option<GameTestDefinition> {
    let (_, _, canonical_id) = parse_structure_identifier(raw_test_id)?;
    crate::server::builtin_game_tests::ensure_builtin_game_tests_registered();
    server
        .datapack_manager
        .get_test_instance(&canonical_id)
        .or_else(|| {
            pumpkin_gametest::registered_test_definitions()
                .into_iter()
                .find_map(|(id, def)| (id == canonical_id).then_some(def))
        })
}

#[must_use]
pub fn resolve_test_structure_size(
    world: &World,
    server: &Server,
    raw_test_id: &str,
) -> Option<[i32; 3]> {
    let test_instance = lookup_test_instance(server, raw_test_id)?;
    if let Some(template) = load_or_cache_structure_template(world, &test_instance.structure) {
        return Some([template.size.x, template.size.y, template.size.z]);
    }
    GameTestStructureTemplate::parse_empty_id(&test_instance.structure).map(|t| t.size())
}

fn description_row(key: &'static str, value: TextComponent) -> TextComponent {
    TextComponent::translate(key, vec![value.color_named(NamedColor::Blue)])
        .add_child(TextComponent::text("\n"))
}

#[must_use]
pub fn describe_test_instance(test_instance: &GameTestDefinition) -> TextComponent {
    let mut description = match test_instance.instance_type {
        TestType::BlockBased => description_row(
            translation::TEST_INSTANCE_DESCRIPTION_TYPE,
            TextComponent::translate(translation::TEST_INSTANCE_TYPE_BLOCK_BASED, Vec::new()),
        ),
        TestType::Function => {
            let row = description_row(
                translation::TEST_INSTANCE_DESCRIPTION_TYPE,
                TextComponent::translate(translation::TEST_INSTANCE_TYPE_FUNCTION, Vec::new()),
            );
            let function_name = test_instance
                .function
                .as_deref()
                .unwrap_or("[unregistered]");
            row.add_child(description_row(
                translation::TEST_INSTANCE_DESCRIPTION_FUNCTION,
                TextComponent::text(function_name.to_string()),
            ))
        }
    };

    let environment_name = test_instance
        .environment
        .as_str()
        .unwrap_or("[unregistered]");

    description = description
        .add_child(description_row(
            translation::TEST_INSTANCE_DESCRIPTION_STRUCTURE,
            TextComponent::text(test_instance.structure.clone()),
        ))
        .add_child(description_row(
            translation::TEST_INSTANCE_DESCRIPTION_BATCH,
            TextComponent::text(environment_name.to_string()),
        ))
        .add_child(description_row(
            translation::TEST_INSTANCE_DESCRIPTION_DIMENSION,
            TextComponent::text(DEFAULT_TEST_DIMENSION),
        ));

    description
}

#[must_use]
pub const fn compute_start_corner(
    structure_pos: BlockPos,
    size: [i32; 3],
    rotation: Rotation,
) -> BlockPos {
    match rotation {
        Rotation::None => structure_pos,
        Rotation::Clockwise90 => BlockPos::new(
            structure_pos.0.x + size[2] - 1,
            structure_pos.0.y,
            structure_pos.0.z,
        ),
        Rotation::Rotate180 => BlockPos::new(
            structure_pos.0.x + size[0] - 1,
            structure_pos.0.y,
            structure_pos.0.z + size[2] - 1,
        ),
        Rotation::CounterClockwise90 => BlockPos::new(
            structure_pos.0.x,
            structure_pos.0.y,
            structure_pos.0.z + size[0] - 1,
        ),
    }
}

pub struct TestInstanceBlockBlockEntity {
    pub position: BlockPos,
    pub data: Mutex<Option<NbtCompound>>,
    pub errors: Mutex<Option<Vec<NbtTag>>>,
    pub dirty: AtomicBool,
}

impl BlockEntity for TestInstanceBlockBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let data = nbt.get_compound("data").cloned();
        let errors = nbt.get_list("errors").map(<[_]>::to_vec);
        Self {
            position,
            data: Mutex::new(data),
            errors: Mutex::new(errors),
            dirty: AtomicBool::new(false),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(data) = self.data.lock()
            && let Some(d) = data.as_ref()
        {
            nbt.put_compound("data", d.clone());
        }
        if let Ok(errors) = self.errors.lock()
            && let Some(errs) = errors.as_ref()
            && !errs.is_empty()
        {
            nbt.put_list("errors", errs.clone());
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(data) = self.data.try_lock()
            && let Some(ref d) = *data
        {
            nbt.put_compound("data", d.clone());
        }
        if let Ok(errors) = self.errors.try_lock()
            && let Some(ref errs) = *errors
            && !errs.is_empty()
        {
            nbt.put_list("errors", errs.clone());
        }
        Some(nbt)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TestInstanceBlockBlockEntity {
    pub const ID: &'static str = "minecraft:test_instance_block";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            data: Mutex::new(None),
            errors: Mutex::new(None),
            dirty: AtomicBool::new(false),
        }
    }

    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Release);
    }

    #[must_use]
    pub fn read_data(&self) -> TestInstanceData {
        let guard = self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard
            .as_ref()
            .map_or_else(TestInstanceData::default, TestInstanceData::from_nbt)
    }

    pub fn set_data(&self, data: &TestInstanceData) {
        *self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(data.to_nbt());
        self.mark_dirty();
    }

    pub fn set_cleared(&self) {
        let mut data = self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let data = data.get_or_insert_with(NbtCompound::new);
        data.put_string(
            "status",
            TestInstanceStatus::Cleared.serialized_name().to_string(),
        );
        data.child_tags.remove("error_message");
        self.mark_dirty();
    }

    pub fn set_running(&self) {
        let mut data = self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let data = data.get_or_insert_with(NbtCompound::new);
        data.put_string(
            "status",
            TestInstanceStatus::Running.serialized_name().to_string(),
        );
        data.child_tags.remove("error_message");
        self.mark_dirty();
    }

    pub fn set_success(&self) {
        let mut data = self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let data = data.get_or_insert_with(NbtCompound::new);
        data.put_string(
            "status",
            TestInstanceStatus::Finished.serialized_name().to_string(),
        );
        data.child_tags.remove("error_message");
        self.mark_dirty();
    }

    pub fn set_error_message(&self, message: String) {
        let mut data = self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let data = data.get_or_insert_with(NbtCompound::new);
        data.put_string(
            "status",
            TestInstanceStatus::Finished.serialized_name().to_string(),
        );
        data.put_string("error_message", message);
        self.mark_dirty();
    }

    pub fn mark_error(&self, position: BlockPos, text: String) {
        let mut marker = NbtCompound::new();
        marker.put(
            "pos",
            NbtTag::IntArray(vec![position.0.x, position.0.y, position.0.z]),
        );
        marker.put_string("text", text);

        let mut errors = self
            .errors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        errors
            .get_or_insert_with(Vec::new)
            .push(NbtTag::Compound(marker));
        self.mark_dirty();
    }

    pub fn clear_error_markers(&self) {
        let mut errors = self
            .errors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if errors.is_some() {
            *errors = None;
            self.mark_dirty();
        }
    }

    #[must_use]
    pub fn test_name_component(&self) -> TextComponent {
        self.read_data().test.map_or_else(
            || TextComponent::translate(INVALID_TEST_TRANSLATION_KEY, Vec::new()),
            TextComponent::text,
        )
    }

    #[must_use]
    pub const fn structure_pos(&self, padding: i32) -> BlockPos {
        BlockPos::new(
            self.position.0.x + padding + STRUCTURE_OFFSET[0],
            self.position.0.y + padding + STRUCTURE_OFFSET[1],
            self.position.0.z + padding + STRUCTURE_OFFSET[2],
        )
    }

    fn effective_rotation_and_meta(&self, world: &World) -> (Rotation, i32, bool) {
        let data = self.read_data();
        let test_def = data.test.as_deref().and_then(|test_id| {
            world
                .server
                .upgrade()
                .and_then(|server| lookup_test_instance(&server, test_id))
        });

        let base_rotation = test_def
            .as_ref()
            .map_or(Rotation::None, |def| def.rotation.as_block_rotation());
        let padding = test_def.as_ref().map_or(0, |def| def.padding);
        let sky_access = test_def.as_ref().is_some_and(|def| def.sky_access);
        let combined = base_rotation.then(data.rotation.as_block_rotation());
        (combined, padding, sky_access)
    }

    pub fn remove_barriers(&self, world: &Arc<World>) {
        let data = self.read_data();
        if data.size.iter().any(|&s| s <= 0) {
            return;
        }
        let (combined_rotation, padding, sky_access) = self.effective_rotation_and_meta(world);
        let transformed = combined_rotation.transform_size(Vector3::new(
            data.size[0],
            data.size[1],
            data.size[2],
        ));
        let structure_pos = self.structure_pos(padding);
        let min = BlockPos::new(
            structure_pos.0.x - 1,
            structure_pos.0.y - 1,
            structure_pos.0.z - 1,
        );
        let max = BlockPos::new(
            structure_pos.0.x + transformed.x,
            structure_pos.0.y + transformed.y,
            structure_pos.0.z + transformed.z,
        );
        let include_ceiling = !sky_access;

        for x in min.0.x..=max.0.x {
            for y in min.0.y..=max.0.y {
                for z in min.0.z..=max.0.z {
                    let is_wall_or_floor = x == min.0.x
                        || x == max.0.x
                        || z == min.0.z
                        || z == max.0.z
                        || y == min.0.y;
                    let is_ceiling = y == max.0.y;
                    if !(is_wall_or_floor || (is_ceiling && include_ceiling)) {
                        continue;
                    }
                    let pos = BlockPos::new(x, y, z);
                    if world.is_in_build_limit(pos) && world.get_block(&pos) == &Block::BARRIER {
                        world.set_block_state(
                            &pos,
                            Block::AIR.default_state.id,
                            BlockFlags::NOTIFY_ALL,
                        );
                    }
                }
            }
        }
    }

    pub fn place_structure(&self, world: &Arc<World>) -> bool {
        let Some(server) = world.server.upgrade() else {
            return false;
        };
        let data = self.read_data();
        let Some(ref test_id) = data.test else {
            return false;
        };
        let Some(test_instance) = lookup_test_instance(&server, test_id) else {
            return false;
        };

        let loaded_template = load_or_cache_structure_template(world, &test_instance.structure);
        let empty_template = if loaded_template.is_none() {
            GameTestStructureTemplate::parse_empty_id(&test_instance.structure)
        } else {
            None
        };
        if loaded_template.is_none() && empty_template.is_none() {
            return false;
        }

        let source_size = if let Some(ref tpl) = loaded_template {
            [tpl.size.x, tpl.size.y, tpl.size.z]
        } else if let Some(ref empty) = empty_template {
            empty.size()
        } else {
            return false;
        };

        let combined_rotation = test_instance
            .rotation
            .as_block_rotation()
            .then(data.rotation.as_block_rotation());
        let padding = test_instance.padding;
        let structure_pos = self.structure_pos(padding);
        let transformed_size = combined_rotation.transform_size(Vector3::new(
            source_size[0],
            source_size[1],
            source_size[2],
        ));

        let test_min = BlockPos::new(
            structure_pos.0.x - padding,
            structure_pos.0.y - padding,
            structure_pos.0.z - padding,
        );
        let test_max = BlockPos::new(
            structure_pos.0.x + transformed_size.x + padding,
            structure_pos.0.y + transformed_size.y + padding,
            structure_pos.0.z + transformed_size.z + padding,
        );

        clear_non_player_entities_in_bounds(world, test_min, test_max);
        clear_space_for_test_structure(world, self.position, test_min, test_max);

        if let Some(ref template) = loaded_template {
            let start_corner = compute_start_corner(structure_pos, source_size, combined_rotation);
            place_structure_template_in_world(
                world,
                template,
                start_corner,
                &StructurePlacementSettings {
                    mirror: Mirror::None,
                    rotation: combined_rotation,
                    integrity: 1.0,
                    seed: 0,
                    strict: true,
                },
            );
        }

        true
    }

    pub fn reset_test(&self, world: &Arc<World>) -> Option<TextComponent> {
        self.remove_barriers(world);
        self.clear_error_markers();
        let placed = self.place_structure(world);
        let feedback = placed.then(|| {
            TextComponent::translate(
                translation::TEST_INSTANCE_BLOCK_RESET_SUCCESS,
                vec![self.test_name_component()],
            )
            .color_named(NamedColor::Green)
        });
        self.set_cleared();
        feedback
    }

    pub fn save_test(&self, world: &Arc<World>) -> Result<String, TextComponent> {
        let data = self.read_data();
        let target_structure_id = data.test.as_deref().and_then(|test_id| {
            let from_def = world
                .server
                .upgrade()
                .and_then(|server| lookup_test_instance(&server, test_id))
                .map(|def| def.structure);
            from_def.or_else(|| Some(test_id.to_string()))
        });

        let Some(structure_id) = target_structure_id else {
            return Err(self
                .positional_error_message(translation::TEST_INSTANCE_BLOCK_ERROR_UNABLE_TO_SAVE));
        };

        let (_, padding, _) = self.effective_rotation_and_meta(world);
        let structure_pos = self.structure_pos(padding);
        let size = Vector3::new(data.size[0], data.size[1], data.size[2]);
        if !save_structure_from_world(
            world,
            &structure_id,
            structure_pos,
            size,
            "",
            true,
            &[&Block::AIR],
        ) {
            return Err(self
                .positional_error_message(translation::TEST_INSTANCE_BLOCK_ERROR_UNABLE_TO_SAVE));
        }

        Ok(structure_id)
    }

    pub fn run_test(&self, world: &Arc<World>, player: &Arc<Player>) -> TextComponent {
        let Some(server) = world.server.upgrade() else {
            return self.positional_error_message(translation::TEST_INSTANCE_BLOCK_ERROR_NO_TEST);
        };
        let data = self.read_data();
        let Some(ref test_id) = data.test else {
            return self.positional_error_message(translation::TEST_INSTANCE_BLOCK_ERROR_NO_TEST);
        };
        if lookup_test_instance(&server, test_id).is_none() {
            return self.positional_error_message(translation::TEST_INSTANCE_BLOCK_ERROR_NO_TEST);
        }
        if !self.place_structure(world) {
            return self.positional_error_message(
                translation::TEST_INSTANCE_BLOCK_ERROR_NO_TEST_STRUCTURE,
            );
        }

        self.clear_error_markers();
        stop_game_tests();

        let report = Arc::new(GameTestBatchReport::new(
            Arc::new(PlayerTestReporter {
                player: player.clone(),
            }),
            1,
        ));

        enqueue_game_test(
            GameTestQueueEntry::new(
                test_id.clone(),
                world.clone(),
                self.position.0.x,
                self.position.0.z,
                data.rotation.rotation_steps(),
                GameTestRetryOptions::no_retries(),
                report,
            )
            .with_y(self.position.0.y),
        );

        TextComponent::translate(
            translation::TEST_INSTANCE_BLOCK_STARTING,
            vec![TextComponent::text(test_id.clone())],
        )
    }

    fn positional_error_message(&self, key: &'static str) -> TextComponent {
        TextComponent::translate(
            key,
            vec![
                TextComponent::text(self.position.0.x.to_string()),
                TextComponent::text(self.position.0.y.to_string()),
                TextComponent::text(self.position.0.z.to_string()),
            ],
        )
        .color_named(NamedColor::Red)
    }
}

fn clear_non_player_entities_in_bounds(world: &World, min: BlockPos, max: BlockPos) {
    let min_x = f64::from(min.0.x);
    let min_y = f64::from(min.0.y);
    let min_z = f64::from(min.0.z);
    let max_x = f64::from(max.0.x);
    let max_y = f64::from(max.0.y);
    let max_z = f64::from(max.0.z);

    let entities = world.entities.load_full();
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
        world.remove_entity(entity.as_ref());
    }
}

fn clear_space_for_test_structure(
    world: &Arc<World>,
    controller_pos: BlockPos,
    min: BlockPos,
    max: BlockPos,
) {
    for x in min.0.x..max.0.x {
        for y in min.0.y..max.0.y {
            for z in min.0.z..max.0.z {
                let pos = BlockPos::new(x, y, z);
                if pos == controller_pos || !world.is_in_build_limit(pos) {
                    continue;
                }
                if world.get_block(&pos) != &Block::AIR {
                    world.remove_block_entity(&pos);
                    world.set_block_state(
                        &pos,
                        Block::AIR.default_state.id,
                        BlockFlags::NOTIFY_LISTENERS | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_corner_shifts_rotated_structure_into_positive_quadrant() {
        let origin = BlockPos::new(10, 65, 21);
        let size = [5, 3, 8];

        assert_eq!(
            compute_start_corner(origin, size, Rotation::None),
            BlockPos::new(10, 65, 21)
        );
        assert_eq!(
            compute_start_corner(origin, size, Rotation::Clockwise90),
            BlockPos::new(17, 65, 21)
        );
        assert_eq!(
            compute_start_corner(origin, size, Rotation::Rotate180),
            BlockPos::new(14, 65, 28)
        );
        assert_eq!(
            compute_start_corner(origin, size, Rotation::CounterClockwise90),
            BlockPos::new(10, 65, 25)
        );
    }

    #[test]
    fn instance_data_nbt_roundtrip_canonicalizes_id_and_preserves_component() {
        let original = TestInstanceData {
            test: Some("minecraft:always_pass".to_string()),
            size: [4, 5, 6],
            rotation: TestInstanceRotation::Clockwise90,
            ignore_entities: true,
            status: TestInstanceStatus::Finished,
            error_message: Some(TextComponent::text("Expected lever powered")),
        };
        let nbt = original.to_nbt();
        let parsed = TestInstanceData::from_nbt(&nbt);
        assert_eq!(parsed, original);
    }
}
