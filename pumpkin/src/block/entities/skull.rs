use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::pin::Pin;
use tokio::sync::Mutex;

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

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if let Some(sound) = self.note_block_sound.lock().await.as_ref() {
                nbt.put_string("note_block_sound", sound.clone());
            }
            if let Some(prof) = self.profile.lock().await.as_ref() {
                nbt.put_compound("profile", prof.clone());
            }
        })
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
            note_block_sound: Mutex::const_new(None),
            profile: Mutex::const_new(None),
        }
    }
}
