use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

use super::BlockEntity;

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
}
