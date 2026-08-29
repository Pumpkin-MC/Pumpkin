use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

pub struct SkullBlockEntity {
    pub position: BlockPos,
    pub note_block_sound: Mutex<Option<String>>,
    pub profile: Mutex<Option<NbtCompound>>,
}

impl BlockEntity for SkullBlockEntity {
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
        let note_block_sound = nbt.get_string("note_block_sound").map(ToString::to_string);
        let profile = nbt.get_compound("profile").cloned();
        Self {
            position,
            note_block_sound: Mutex::new(note_block_sound),
            profile: Mutex::new(profile),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(sound) = self.note_block_sound.lock()
            && let Some(sound) = sound.as_ref()
        {
            nbt.put_string("note_block_sound", sound.clone());
        }
        if let Ok(prof) = self.profile.lock()
            && let Some(prof) = prof.as_ref()
        {
            nbt.put_compound("profile", prof.clone());
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(sound) = self.note_block_sound.try_lock()
            && let Some(ref sound) = *sound
        {
            nbt.put_string("note_block_sound", sound.clone());
        }
        if let Ok(profile) = self.profile.try_lock()
            && let Some(ref prof) = *profile
        {
            nbt.put_compound("profile", prof.clone());
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SkullBlockEntity {
    pub const ID: &'static str = "minecraft:skull";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            note_block_sound: Mutex::new(None),
            profile: Mutex::new(None),
        }
    }
}
