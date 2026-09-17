use super::{
    BlockEntity,
    components::{BlockEntityComponents, ComponentFields},
};
use pumpkin_data::data_component::DataComponent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

pub struct BannerBlockEntity {
    pub position: BlockPos,
    pub components: BlockEntityComponents,
}

impl BlockEntity for BannerBlockEntity {
    /// Returns the registry identifier used for persistence and chunk updates.
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    /// Returns the block position belonging to this entity.
    fn get_position(&self) -> BlockPos {
        self.position
    }

    /// Loads implicit fields and retained item additions without flattening text.
    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self {
        Self {
            position,
            components: BlockEntityComponents::from_nbt(nbt, Self::COMPONENT_FIELDS),
        }
    }

    /// Saves implicit fields and retained additions in their vanilla NBT locations.
    fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.components.write_nbt(nbt);
    }

    /// Returns the owned component state used by placement and loot collection.
    fn component_state(&self) -> Option<&BlockEntityComponents> {
        Some(&self.components)
    }

    /// Encodes the custom block-entity data used by chunk updates.
    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.components.write_nbt(&mut nbt);
        Some(nbt)
    }

    /// Exposes the concrete entity for existing specialized block behavior.
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BannerBlockEntity {
    pub const ID: &'static str = "minecraft:banner";
    const COMPONENT_FIELDS: ComponentFields = &[
        (DataComponent::CustomName, "CustomName"),
        (DataComponent::BannerPatterns, "patterns"),
    ];

    /// Creates an entity with empty component state at the supplied position.
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            components: BlockEntityComponents::new(Self::COMPONENT_FIELDS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Saving a named banner preserves its structured text and retained item metadata.
    #[test]
    fn preserves_structured_name_and_components() {
        let mut input = NbtCompound::new();
        let mut name = NbtCompound::new();
        name.put_string("text", "Pattern archive".to_owned());
        name.put_bool("bold", true);
        input.put_compound("CustomName", name);
        let mut components = NbtCompound::new();
        components.put_int("minecraft:repair_cost", 7);
        input.put_compound("components", components);
        let banner = BannerBlockEntity::from_nbt(&input, BlockPos::new(0, 64, 0));
        let mut saved = NbtCompound::new();
        banner.write_nbt(&mut saved);
        assert_eq!(saved.get("CustomName"), input.get("CustomName"));
        assert_eq!(saved.get("components"), input.get("components"));
    }
}
