use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

pub struct TrialSpawnerBlockEntity {
    pub position: BlockPos,
    pub normal_config: Mutex<Option<NbtCompound>>,
    pub ominous_config: Mutex<Option<NbtCompound>>,
    pub spawner_data: Mutex<Option<NbtCompound>>,
}

impl BlockEntity for TrialSpawnerBlockEntity {
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
        Self {
            position,
            normal_config: Mutex::new(nbt.get_compound("normal_config").cloned()),
            ominous_config: Mutex::new(nbt.get_compound("ominous_config").cloned()),
            spawner_data: Mutex::new(nbt.get_compound("spawner_data").cloned()),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(cfg) = self.normal_config.lock()
            && let Some(cfg) = cfg.as_ref()
        {
            nbt.put_compound("normal_config", cfg.clone());
        }
        if let Ok(cfg) = self.ominous_config.lock()
            && let Some(cfg) = cfg.as_ref()
        {
            nbt.put_compound("ominous_config", cfg.clone());
        }
        if let Ok(data) = self.spawner_data.lock()
            && let Some(data) = data.as_ref()
        {
            nbt.put_compound("spawner_data", data.clone());
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(cfg) = self.normal_config.try_lock()
            && let Some(ref cfg) = *cfg
        {
            nbt.put_compound("normal_config", cfg.clone());
        }
        if let Ok(cfg) = self.ominous_config.try_lock()
            && let Some(ref cfg) = *cfg
        {
            nbt.put_compound("ominous_config", cfg.clone());
        }
        if let Ok(data) = self.spawner_data.try_lock()
            && let Some(ref data) = *data
        {
            nbt.put_compound("spawner_data", data.clone());
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TrialSpawnerBlockEntity {
    pub const ID: &'static str = "minecraft:trial_spawner";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            normal_config: Mutex::new(None),
            ominous_config: Mutex::new(None),
            spawner_data: Mutex::new(None),
        }
    }
}
