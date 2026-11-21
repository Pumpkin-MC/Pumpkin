use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashSet;
use std::sync::Mutex;
use uuid::Uuid;

pub struct VaultBlockEntity {
    pub position: BlockPos,
    pub config: Mutex<Option<NbtCompound>>,
    pub server_data: Mutex<Option<NbtCompound>>,
    pub shared_data: Mutex<Option<NbtCompound>>,
    pub rewarded_players: Mutex<FxHashSet<Uuid>>,
}

impl BlockEntity for VaultBlockEntity {
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
            config: Mutex::new(nbt.get_compound("config").cloned()),
            server_data: Mutex::new(nbt.get_compound("server_data").cloned()),
            shared_data: Mutex::new(nbt.get_compound("shared_data").cloned()),
            rewarded_players: Mutex::new(FxHashSet::default()),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(cfg) = self.config.lock()
            && let Some(cfg) = cfg.as_ref()
        {
            nbt.put_compound("config", cfg.clone());
        }
        if let Ok(data) = self.server_data.lock()
            && let Some(data) = data.as_ref()
        {
            nbt.put_compound("server_data", data.clone());
        }
        if let Ok(shared) = self.shared_data.lock()
            && let Some(shared) = shared.as_ref()
        {
            nbt.put_compound("shared_data", shared.clone());
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(shared) = self.shared_data.try_lock()
            && let Some(ref shared) = *shared
        {
            nbt.put_compound("shared_data", shared.clone());
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl VaultBlockEntity {
    pub const ID: &'static str = "minecraft:vault";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            config: Mutex::new(None),
            server_data: Mutex::new(None),
            shared_data: Mutex::new(None),
            rewarded_players: Mutex::new(FxHashSet::default()),
        }
    }

    pub fn has_rewarded(&self, player_id: &Uuid) -> bool {
        self.rewarded_players
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(player_id)
    }

    pub fn mark_rewarded(&self, player_id: Uuid) {
        self.rewarded_players
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(player_id);
    }
}
