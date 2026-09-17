use super::{
    BlockEntity,
    components::{BlockEntityComponents, ComponentFields},
};
use pumpkin_data::data_component::DataComponent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

pub struct EnchantingTableBlockEntity {
    pub position: BlockPos,
    pub components: BlockEntityComponents,
}

impl BlockEntity for EnchantingTableBlockEntity {
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

    /// Sends the table's name without retained item additions.
    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.components.write_custom_nbt(&mut nbt);
        Some(nbt)
    }

    /// Exposes the concrete entity for existing specialized block behavior.
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl EnchantingTableBlockEntity {
    pub const ID: &'static str = "minecraft:enchanting_table";
    const COMPONENT_FIELDS: ComponentFields = &[(DataComponent::CustomName, "CustomName")];

    /// Creates an entity with empty component state at the supplied position.
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            components: BlockEntityComponents::new(Self::COMPONENT_FIELDS),
        }
    }
}
