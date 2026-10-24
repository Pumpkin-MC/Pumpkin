use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

pub struct SkullBlockEntity {
    pub position: BlockPos,
    pub note_block_sound: Mutex<Option<String>>,
    pub profile: Mutex<Option<NbtCompound>>,
    pub custom_name: Mutex<Option<String>>,
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
        let custom_name = nbt
            .get_string("custom_name")
            .or_else(|| nbt.get_string("CustomName"))
            .map(ToString::to_string);
        Self {
            position,
            note_block_sound: Mutex::new(note_block_sound),
            profile: Mutex::new(profile),
            custom_name: Mutex::new(custom_name),
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
        if let Ok(name) = self.custom_name.lock()
            && let Some(name) = name.as_ref()
        {
            nbt.put_string("custom_name", name.clone());
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
        if let Ok(name) = self.custom_name.try_lock()
            && let Some(ref name) = *name
        {
            nbt.put_string("custom_name", name.clone());
        }
        Some(nbt)
    }

    fn collect_item_components(&self, stack: &mut pumpkin_data::item_stack::ItemStack) {
        if let Ok(sound) = self.note_block_sound.lock()
            && let Some(sound) = sound.as_ref()
        {
            stack.set_data_component(pumpkin_data::data_component_impl::NoteBlockSoundImpl {
                sound: sound.clone(),
            });
        }
        if let Ok(prof) = self.profile.lock()
            && let Some(prof) = prof.as_ref()
            && let Some(profile) = pumpkin_data::data_component_impl::ProfileImpl::read_data(
                &pumpkin_nbt::tag::NbtTag::Compound(prof.clone()),
            )
        {
            stack.set_data_component(profile);
        }
        if let Ok(name) = self.custom_name.lock()
            && let Some(name) = name.as_ref()
        {
            stack.set_data_component(pumpkin_data::data_component_impl::CustomNameImpl {
                name: pumpkin_util::text::TextComponent::text(name.clone()),
            });
        }
    }

    fn apply_item_components(&self, stack: &pumpkin_data::item_stack::ItemStack) {
        if let Some(sound) =
            stack.get_data_component::<pumpkin_data::data_component_impl::NoteBlockSoundImpl>()
        {
            *self
                .note_block_sound
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(sound.sound.clone());
        }
        if let Some(profile) =
            stack.get_data_component::<pumpkin_data::data_component_impl::ProfileImpl>()
        {
            use pumpkin_data::data_component_impl::DataComponentImpl;
            if let pumpkin_nbt::tag::NbtTag::Compound(prof) = profile.write_data() {
                *self
                    .profile
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(prof);
            }
        }
        if let Some(custom_name) =
            stack.get_data_component::<pumpkin_data::data_component_impl::CustomNameImpl>()
        {
            *self
                .custom_name
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                Some(custom_name.name.clone().get_text());
        }
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
            custom_name: Mutex::new(None),
        }
    }
}
