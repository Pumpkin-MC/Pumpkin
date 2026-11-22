use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;
use std::sync::Mutex;

use super::BlockEntity;

const COMPONENTS: &str = "components";
const CUSTOM_NAME_COMPONENT: &str = "minecraft:custom_name";

pub struct CopperGolemStatueBlockEntity {
    pub position: BlockPos,
    pub custom_name: Mutex<Option<TextComponent>>,
}

impl BlockEntity for CopperGolemStatueBlockEntity {
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
        // Earlier Pumpkin saves kept the name as plain text under `custom_name`.
        let custom_name = nbt
            .get_compound(COMPONENTS)
            .and_then(|components| components.get(CUSTOM_NAME_COMPONENT))
            .map(TextComponent::from_nbt)
            .or_else(|| {
                nbt.get_string("custom_name")
                    .or_else(|| nbt.get_string("CustomName"))
                    .map(|name| TextComponent::text(name.to_string()))
            });
        Self {
            position,
            custom_name: Mutex::new(custom_name),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(components) = self.components_nbt() {
            nbt.put_compound(COMPONENTS, components);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_nbt(&mut nbt);
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CopperGolemStatueBlockEntity {
    pub const ID: &'static str = "minecraft:copper_golem_statue";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            custom_name: Mutex::new(None),
        }
    }

    pub fn set_custom_name(&self, name: Option<TextComponent>) {
        *self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = name;
    }

    fn components_nbt(&self) -> Option<NbtCompound> {
        let name = self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()?;
        let mut components = NbtCompound::new();
        components.put(
            CUSTOM_NAME_COMPONENT,
            name.to_nbt_tag_for_version(&JavaMinecraftVersion::V_26_3),
        );
        Some(components)
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::text::color::NamedColor;

    use super::*;

    #[test]
    fn custom_name_keeps_its_formatting_through_nbt() {
        let statue = CopperGolemStatueBlockEntity::new(BlockPos::ZERO);
        let name = TextComponent::text("Rusty").color_named(NamedColor::Gold);
        statue.set_custom_name(Some(name.clone()));
        let mut nbt = NbtCompound::new();
        statue.write_nbt(&mut nbt);

        let read = CopperGolemStatueBlockEntity::from_nbt(&nbt, BlockPos::ZERO);
        assert_eq!(read.custom_name.into_inner().ok().flatten(), Some(name));
    }

    #[test]
    fn plain_custom_name_from_an_older_save_still_loads() {
        let mut nbt = NbtCompound::new();
        nbt.put_string("custom_name", "Rusty".to_string());

        let read = CopperGolemStatueBlockEntity::from_nbt(&nbt, BlockPos::ZERO);
        assert_eq!(
            read.custom_name.into_inner().ok().flatten(),
            Some(TextComponent::text("Rusty"))
        );
    }
}
