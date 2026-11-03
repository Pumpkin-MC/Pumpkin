use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::math::position::BlockPos;

/// Fluent builder for constructing Bedrock block-actor NBT compounds.
pub struct BedrockBlockActorBuilder {
    nbt: NbtCompound,
}

impl BedrockBlockActorBuilder {
    /// Creates a new builder initialized with standard coordinates and `isMovable: true`.
    #[must_use]
    pub fn new(id: &'static str, position: BlockPos) -> Self {
        let mut nbt = NbtCompound::new();
        nbt.put_string("id", id.to_string());
        nbt.put_int("x", position.0.x);
        nbt.put_int("y", position.0.y);
        nbt.put_int("z", position.0.z);
        nbt.put_bool("isMovable", true);
        Self { nbt }
    }

    /// Sets whether this block entity can be moved by pistons.
    #[must_use]
    pub fn movable(mut self, is_movable: bool) -> Self {
        self.nbt.put_bool("isMovable", is_movable);
        self
    }

    /// Optionally sets the custom name tag of this block entity.
    #[must_use]
    pub fn custom_name(mut self, name: Option<&str>) -> Self {
        if let Some(name) = name
            && !name.is_empty()
        {
            self.nbt.put_string("CustomName", name.to_string());
        }
        self
    }

    #[must_use]
    pub fn put_int(mut self, key: &'static str, val: i32) -> Self {
        self.nbt.put_int(key, val);
        self
    }

    #[must_use]
    pub fn put_short(mut self, key: &'static str, val: i16) -> Self {
        self.nbt.put_short(key, val);
        self
    }

    #[must_use]
    pub fn put_byte(mut self, key: &'static str, val: i8) -> Self {
        self.nbt.put_byte(key, val);
        self
    }

    #[must_use]
    pub fn put_long(mut self, key: &'static str, val: i64) -> Self {
        self.nbt.put_long(key, val);
        self
    }

    #[must_use]
    pub fn put_bool(mut self, key: &'static str, val: bool) -> Self {
        self.nbt.put_bool(key, val);
        self
    }

    #[must_use]
    pub fn put_float(mut self, key: &'static str, val: f32) -> Self {
        self.nbt.put_float(key, val);
        self
    }

    #[must_use]
    pub fn put_string(mut self, key: &'static str, val: impl Into<String>) -> Self {
        self.nbt.put_string(key, val.into());
        self
    }

    #[must_use]
    pub fn put_compound(mut self, key: &'static str, val: NbtCompound) -> Self {
        self.nbt.put_compound(key, val);
        self
    }

    #[must_use]
    pub fn put_list(mut self, key: &'static str, val: Vec<NbtTag>) -> Self {
        self.nbt.put_list(key, val);
        self
    }

    /// Returns the finalized Bedrock NBT compound.
    #[must_use]
    pub fn finish(self) -> NbtCompound {
        self.nbt
    }

    /// Alias for [`Self::finish`].
    #[must_use]
    pub fn build(self) -> NbtCompound {
        self.finish()
    }
}

/// Normalizes an item or entity identifier to include the `minecraft:` namespace if not present.
#[must_use]
pub(crate) fn bedrock_item_name(registry_key: &str) -> String {
    if registry_key.starts_with("minecraft:") {
        registry_key.to_string()
    } else {
        format!("minecraft:{registry_key}")
    }
}

/// Serializes a slice of [`ItemStack`]s into Bedrock's standard `Items` list tag format.
#[must_use]
pub(crate) fn build_bedrock_slot_items(items: &[ItemStack]) -> Vec<NbtTag> {
    let mut item_tags = Vec::with_capacity(items.len());
    for (slot_idx, stack) in items.iter().enumerate() {
        let mut comp = NbtCompound::new();
        if stack.is_empty() {
            comp.put_string("Name", String::new());
            comp.put_byte("Count", 0);
        } else {
            comp.put_string("Name", bedrock_item_name(stack.item.registry_key));
            comp.put_byte("Count", stack.item_count as i8);
        }
        comp.put_short("Damage", 0);
        comp.put_byte("Slot", slot_idx as i8);
        item_tags.push(NbtTag::Compound(comp));
    }
    item_tags
}

/// Builds the Bedrock block-actor NBT for standard container blocks (Dispenser, Dropper, Hopper, Barrel).
#[must_use]
pub fn build_container_actor(
    id: &'static str,
    position: BlockPos,
    custom_name: Option<&str>,
) -> NbtCompound {
    BedrockBlockActorBuilder::new(id, position)
        .custom_name(custom_name)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_actor_builder() {
        let pos = BlockPos::new(1, 64, -2);
        let nbt = build_container_actor("Barrel", pos, Some("My Barrel"));
        assert_eq!(nbt.get_string("id"), Some("Barrel"));
        assert_eq!(nbt.get_int("x"), Some(1));
        assert_eq!(nbt.get_int("y"), Some(64));
        assert_eq!(nbt.get_int("z"), Some(-2));
        assert_eq!(nbt.get_bool("isMovable"), Some(true));
        assert_eq!(nbt.get_string("CustomName"), Some("My Barrel"));
    }
}
