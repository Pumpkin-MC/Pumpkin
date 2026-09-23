use std::sync::Arc;

use crossbeam::atomic::AtomicCell;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use uuid::Uuid;

use crate::world::World;

use super::BlockEntity;

pub struct CreakingHeartBlockEntity {
    pub position: BlockPos,
    pub creaking_uuid: AtomicCell<Option<Uuid>>,
}

impl BlockEntity for CreakingHeartBlockEntity {
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
        let creaking_uuid = nbt
            .get_uuid("creaking")
            .or_else(|| {
                nbt.get_string("creaking")
                    .and_then(|uuid_str| Uuid::parse_str(uuid_str).ok())
            })
            .or_else(|| {
                nbt.get_string("creaking_uuid")
                    .and_then(|uuid_str| Uuid::parse_str(uuid_str).ok())
            });
        Self {
            position,
            creaking_uuid: AtomicCell::new(creaking_uuid),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(uuid) = self.creaking_uuid.load() {
            nbt.put_uuid("creaking", uuid);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Some(uuid) = self.creaking_uuid.load() {
            nbt.put_uuid("creaking", uuid);
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CreakingHeartBlockEntity {
    pub const ID: &'static str = "minecraft:creaking_heart";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            creaking_uuid: AtomicCell::new(None),
        }
    }

    pub fn is_protector(&self, creaking_uuid: Uuid) -> bool {
        self.creaking_uuid
            .load()
            .is_none_or(|uuid| uuid == creaking_uuid)
    }

    pub fn set_creaking_uuid(&self, uuid: Option<Uuid>) {
        self.creaking_uuid.store(uuid);
    }

    pub fn creaking_hurt(&self, world: &Arc<World>) {
        world.play_sound(
            Sound::BlockCreakingHeartHurt,
            SoundCategory::Blocks,
            &self.position.to_f64(),
        );
    }
}
