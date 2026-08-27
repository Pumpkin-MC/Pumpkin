use std::sync::{
    RwLock,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::math::position::BlockPos;

use super::BlockEntity;

const STRUCTURE_OFFSET: [i32; 3] = [0, 1, 1];

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestInstanceData {
    pub test: Option<String>,
    pub size: [i32; 3],
    pub rotation: TestInstanceRotation,
    pub ignore_entities: bool,
    pub status: TestInstanceStatus,
    pub error_message: Option<String>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestInstanceErrorMarker {
    pub position: BlockPos,
    pub text: String,
}

pub struct TestInstanceBlockBlockEntity {
    pub position: BlockPos,
    data: RwLock<TestInstanceData>,
    error_markers: RwLock<Vec<TestInstanceErrorMarker>>,
    dirty: AtomicBool,
}

impl TestInstanceBlockBlockEntity {
    pub const ID: &'static str = "minecraft:test_instance_block";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            data: RwLock::new(TestInstanceData::default()),
            error_markers: RwLock::new(Vec::new()),
            dirty: AtomicBool::new(false),
        }
    }

    pub fn data(&self) -> TestInstanceData {
        self.data
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn error_markers(&self) -> Vec<TestInstanceErrorMarker> {
        self.error_markers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn set_running(&self) {
        let mut data = self
            .data
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        data.status = TestInstanceStatus::Running;
        data.error_message = None;
        self.dirty.store(true, Ordering::Release);
    }

    pub fn set_success(&self) {
        let mut data = self
            .data
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        data.status = TestInstanceStatus::Finished;
        data.error_message = None;
        self.dirty.store(true, Ordering::Release);
    }

    pub fn set_error_message(&self, message: String) {
        let mut data = self
            .data
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        data.status = TestInstanceStatus::Finished;
        data.error_message = Some(message);
        self.dirty.store(true, Ordering::Release);
    }

    pub fn mark_error(&self, position: BlockPos, text: String) {
        self.error_markers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(TestInstanceErrorMarker { position, text });
        self.dirty.store(true, Ordering::Release);
    }

    pub fn clear_error_markers(&self) {
        let mut markers = self
            .error_markers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !markers.is_empty() {
            markers.clear();
            self.dirty.store(true, Ordering::Release);
        }
    }

    /// Vanilla's client derives the beacon beam from status, error state, and the
    /// test definition's `required` flag. These ARGB values match 26.2.
    pub fn beam_argb(&self, required: bool) -> Option<u32> {
        let data = self
            .data
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match data.status {
            TestInstanceStatus::Cleared => None,
            TestInstanceStatus::Running => Some(0xFF80_8080),
            TestInstanceStatus::Finished if data.error_message.is_none() => Some(0xFF00_FF00),
            TestInstanceStatus::Finished if required => Some(0xFFFF_0000),
            TestInstanceStatus::Finished => Some(0xFFFF_8000),
        }
    }

    /// Returns the controller-relative box used by the vanilla client renderer.
    /// `effective_rotation` is the test definition rotation combined with the
    /// controller's extra rotation.
    #[must_use]
    pub const fn renderable_box(
        &self,
        padding: i32,
        effective_rotation: TestInstanceRotation,
        size: [i32; 3],
    ) -> ([i32; 3], [i32; 3]) {
        (
            [
                STRUCTURE_OFFSET[0] + padding,
                STRUCTURE_OFFSET[1] + padding,
                STRUCTURE_OFFSET[2] + padding,
            ],
            effective_rotation.transform_size(size),
        )
    }
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
        let data = nbt.get_compound("data").map(parse_data).unwrap_or_default();
        let error_markers = nbt
            .get_list("errors")
            .map(parse_error_markers)
            .unwrap_or_default();

        Self {
            position,
            data: RwLock::new(data),
            error_markers: RwLock::new(error_markers),
            dirty: AtomicBool::new(false),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        let data = self
            .data
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        nbt.put_compound("data", encode_data(&data));

        let markers = self
            .error_markers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if !markers.is_empty() {
            nbt.put(
                "errors",
                NbtTag::List(markers.iter().map(encode_error_marker).collect()),
            );
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_nbt(&mut nbt);
        Some(nbt)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Acquire)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Release);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn parse_data(nbt: &NbtCompound) -> TestInstanceData {
    TestInstanceData {
        test: nbt.get_string("test").map(ToString::to_string),
        size: read_vec3(nbt, "size").unwrap_or([0, 0, 0]),
        rotation: nbt
            .get_string("rotation")
            .and_then(TestInstanceRotation::from_serialized_name)
            .unwrap_or_default(),
        ignore_entities: nbt.get_bool("ignore_entities").unwrap_or(false),
        status: nbt
            .get_string("status")
            .and_then(TestInstanceStatus::from_serialized_name)
            .unwrap_or_default(),
        error_message: nbt.get_string("error_message").map(ToString::to_string),
    }
}

fn encode_data(data: &TestInstanceData) -> NbtCompound {
    let mut nbt = NbtCompound::new();
    if let Some(test) = &data.test {
        nbt.put_string("test", test.clone());
    }
    nbt.put("size", NbtTag::IntArray(data.size.to_vec()));
    nbt.put_string("rotation", data.rotation.serialized_name().to_string());
    nbt.put_bool("ignore_entities", data.ignore_entities);
    nbt.put_string("status", data.status.serialized_name().to_string());
    if let Some(error_message) = &data.error_message {
        // ComponentSerialization.CODEC accepts a string as a literal component.
        nbt.put_string("error_message", error_message.clone());
    }
    nbt
}

fn parse_error_markers(tags: &[NbtTag]) -> Vec<TestInstanceErrorMarker> {
    tags.iter()
        .filter_map(NbtTag::extract_compound)
        .filter_map(|marker| {
            Some(TestInstanceErrorMarker {
                position: {
                    let [x, y, z] = read_vec3(marker, "pos")?;
                    BlockPos::new(x, y, z)
                },
                text: marker.get_string("text")?.to_string(),
            })
        })
        .collect()
}

fn encode_error_marker(marker: &TestInstanceErrorMarker) -> NbtTag {
    let mut nbt = NbtCompound::new();
    nbt.put(
        "pos",
        NbtTag::IntArray(vec![
            marker.position.0.x,
            marker.position.0.y,
            marker.position.0.z,
        ]),
    );
    // ComponentSerialization.CODEC accepts a string as a literal component.
    nbt.put_string("text", marker.text.clone());
    NbtTag::Compound(nbt)
}

fn read_vec3(nbt: &NbtCompound, name: &str) -> Option<[i32; 3]> {
    if let Some(values) = nbt.get_int_array(name) {
        let [x, y, z] = values else {
            return None;
        };
        return Some([*x, *y, *z]);
    }

    let values = nbt.get_list(name)?;
    let [x, y, z] = values else {
        return None;
    };
    Some([x.extract_int()?, y.extract_int()?, z.extract_int()?])
}
