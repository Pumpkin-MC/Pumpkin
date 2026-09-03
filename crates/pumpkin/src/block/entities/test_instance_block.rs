use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

use super::BlockEntity;

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
    pub data: Mutex<Option<NbtCompound>>,
    pub errors: Mutex<Option<Vec<NbtTag>>>,
}

impl BlockEntity for TestInstanceBlockBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let data = nbt.get_compound("data").cloned();
        let errors = nbt.get_list("errors").map(<[_]>::to_vec);
        Self {
            position,
            data: Mutex::new(data),
            errors: Mutex::new(errors),
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
        {
            nbt.put_list("errors", errs.clone());
        }
        Some(nbt)
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
        }
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
    }

    pub fn clear_error_markers(&self) {
        let mut errors = self
            .errors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *errors = None;
    }
}
