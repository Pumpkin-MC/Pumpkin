use crate::VarInt;
use crate::codec::data_component::{deserialize, serialize};
use crate::ser::{WritingError, serializer};
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::item::Item;
use pumpkin_data::item_id_remap::{remap_item_id_for_version, remap_item_id_from_version};
use pumpkin_util::version::MinecraftVersion;
use pumpkin_world::item::ItemStack;
use serde::ser::SerializeStruct;
use serde::{
    Deserialize, Serialize, Serializer,
    de::{self, SeqAccess},
};
use std::borrow::Cow;

pub struct ItemStackSerializer<'a>(pub Cow<'a, ItemStack>);

fn item_component_counts(stack: &ItemStack) -> (u8, u8) {
    let mut to_add = 0u8;
    let mut to_remove = 0u8;

    for (_id, data) in &stack.patch {
        if data.is_none() {
            to_remove += 1;
        } else {
            to_add += 1;
        }
    }

    (to_add, to_remove)
}

fn serialize_item_stack_with_id<S: Serializer>(
    stack: &ItemStack,
    item_id: u16,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if stack.is_empty() {
        VarInt(0).serialize(serializer)
    } else {
        let (to_add, to_remove) = item_component_counts(stack);
        let mut seq = serializer.serialize_struct("", 0)?;
        seq.serialize_field::<VarInt>("", &VarInt::from(stack.item_count))?;
        seq.serialize_field::<VarInt>("", &VarInt::from(item_id))?;
        seq.serialize_field::<VarInt>("", &VarInt::from(to_add))?;
        seq.serialize_field::<VarInt>("", &VarInt::from(to_remove))?;

        for (id, data) in &stack.patch {
            if let Some(data) = data {
                seq.serialize_field::<VarInt>("", &VarInt::from(id.to_id()))?;
                serialize(*id, data.as_ref(), &mut seq)?;
            }
        }

        for (id, data) in &stack.patch {
            if data.is_none() {
                seq.serialize_field::<VarInt>("", &VarInt::from(id.to_id()))?;
            }
        }

        seq.end()
    }
}

impl<'de> Deserialize<'de> for ItemStackSerializer<'static> {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = ItemStackSerializer<'static>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Slot encoded in a byte sequence")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let item_count = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("Failed to decode VarInt"))?;

                let slot = if item_count.0 == 0 {
                    ItemStackSerializer(Cow::Borrowed(ItemStack::EMPTY))
                } else {
                    let item_id = seq
                        .next_element::<VarInt>()?
                        .ok_or(de::Error::custom("No item id VarInt!"))?;

                    let num_components_to_add = seq
                        .next_element::<VarInt>()?
                        .ok_or(de::Error::custom("No component add length VarInt!"))?
                        .0 as usize;
                    let num_components_to_remove = seq
                        .next_element::<VarInt>()?
                        .ok_or(de::Error::custom("No component remove length VarInt!"))?
                        .0 as usize;

                    let mut patch =
                        Vec::with_capacity(num_components_to_add + num_components_to_remove);
                    for _ in 0..num_components_to_add {
                        let id = seq
                            .next_element::<VarInt>()?
                            .ok_or(de::Error::custom("No component id VarInt!"))?
                            .0;
                        let id = u8::try_from(id)
                            .map_err(|_| de::Error::custom("Unknown component id VarInt!"))?;
                        let id = DataComponent::try_from_id(id)
                            .ok_or(de::Error::custom("Unknown component id VarInt!"))?;
                        let _byte_len = seq
                            .next_element::<VarInt>()?
                            .ok_or(de::Error::custom("No data len VarInt!"))?;
                        patch.push((id, Some(deserialize(id, &mut seq)?)));
                    }
                    for _ in 0..num_components_to_remove {
                        let id = seq
                            .next_element::<VarInt>()?
                            .ok_or(de::Error::custom("No component id VarInt!"))?
                            .0;
                        let id = u8::try_from(id)
                            .map_err(|_| de::Error::custom("Unknown component id VarInt!"))?;
                        let id = DataComponent::try_from_id(id)
                            .ok_or(de::Error::custom("Unknown component id VarInt!"))?;
                        patch.push((id, None));
                    }

                    let item_id: u16 = item_id
                        .0
                        .try_into()
                        .map_err(|_| de::Error::custom("Invalid item id!"))?;

                    ItemStackSerializer(Cow::Owned(ItemStack::new_with_component(
                        item_count.0 as u8,
                        Item::from_id(item_id).unwrap_or(&Item::AIR),
                        patch,
                    )))
                };

                Ok(slot)
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

impl Serialize for ItemStackSerializer<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_item_stack_with_id(self.0.as_ref(), self.0.item.id, serializer)
    }
}

impl ItemStackSerializer<'_> {
    pub fn write_with_version(
        &self,
        write: impl std::io::Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let remapped_item_id = remap_item_id_for_version(self.0.item.id, *version);
        let mut network_serializer = serializer::Serializer::new(write);
        serialize_item_stack_with_id(self.0.as_ref(), remapped_item_id, &mut network_serializer)
    }

    #[must_use]
    pub fn to_stack(self) -> ItemStack {
        self.0.into_owned()
    }

    #[must_use]
    pub fn to_stack_for_version(self, version: &MinecraftVersion) -> ItemStack {
        let mut stack = self.0.into_owned();
        if stack.is_empty() {
            return stack;
        }

        let remapped_item_id = remap_item_id_from_version(stack.item.id, *version);
        stack.item = Item::from_id(remapped_item_id).unwrap_or(&Item::AIR);
        stack
    }
}

impl From<ItemStack> for ItemStackSerializer<'_> {
    fn from(item: ItemStack) -> Self {
        ItemStackSerializer(Cow::Owned(item))
    }
}

impl From<Option<ItemStack>> for ItemStackSerializer<'_> {
    fn from(item: Option<ItemStack>) -> Self {
        item.map_or_else(
            || ItemStackSerializer(Cow::Borrowed(ItemStack::EMPTY)),
            ItemStackSerializer::from,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ItemComponentHash {
    pub added: Vec<(VarInt, i32)>,
    pub removed: Vec<VarInt>,
}

#[derive(Debug, Clone)]
pub struct ItemStackHash {
    item_id: VarInt,
    count: VarInt,
    components: ItemComponentHash,
}

impl OptionalItemStackHash {
    #[must_use]
    pub fn hash_equals(&self, other: &ItemStack) -> bool {
        if let Some(hash) = &self.0 {
            if hash.item_id != other.item.id.into() || hash.count != other.item_count.into() {
                return false;
            }
            let calc = || {
                let mut to_add = 0u8;
                let mut to_remove = 0u8;
                for (_id, data) in &other.patch {
                    if data.is_none() {
                        to_remove += 1;
                    } else {
                        to_add += 1;
                    }
                }
                (to_add, to_remove)
            };
            let (to_add, to_remove) = calc();
            if to_add as usize != hash.components.added.len()
                || to_remove as usize != hash.components.removed.len()
            {
                return false;
            }
            for (other_id, data) in &other.patch {
                if let Some(data) = data {
                    let checksum = data.get_hash();
                    for (id, hash) in &hash.components.added {
                        if id == &VarInt::from(other_id.to_id()) {
                            if hash == &checksum {
                                break;
                            }
                            return false;
                        }
                    }
                } else if !hash
                    .components
                    .removed
                    .contains(&VarInt::from(other_id.to_id()))
                {
                    return false;
                }
            }
            true
        } else {
            other.is_empty()
        }
    }
}

#[derive(Debug, Clone)]
pub struct OptionalItemStackHash(pub Option<ItemStackHash>);

impl<'de> Deserialize<'de> for OptionalItemStackHash {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = OptionalItemStackHash;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Slot encoded in a byte sequence")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let is_some = seq
                    .next_element::<bool>()?
                    .ok_or(de::Error::custom("No is some bool!"))?;
                if is_some {
                    let item_id = seq
                        .next_element::<VarInt>()?
                        .ok_or(de::Error::custom("No item id VarInt!"))?;
                    let count = seq
                        .next_element::<VarInt>()?
                        .ok_or(de::Error::custom("No item count VarInt!"))?;

                    let hashed_components = seq
                        .next_element::<ItemComponentHash>()?
                        .ok_or(de::Error::custom("No item component hash!"))?;

                    let item_stack_hash = ItemStackHash {
                        item_id,
                        count,
                        components: hashed_components,
                    };
                    Ok(OptionalItemStackHash(Some(item_stack_hash)))
                } else {
                    Ok(OptionalItemStackHash(None))
                }
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

impl<'de> Deserialize<'de> for ItemComponentHash {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = ItemComponentHash;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Slot encoded in a byte sequence")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut added = Vec::new();
                let mut removed = Vec::new();

                let added_length = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("No added length VarInt!"))?;
                for _ in 0..added_length.0 {
                    let component_id = seq
                        .next_element::<VarInt>()?
                        .ok_or(de::Error::custom("No component id VarInt!"))?;
                    let component_value = seq
                        .next_element::<i32>()?
                        .ok_or(de::Error::custom("No component value i32!"))?;
                    added.push((component_id, component_value));
                }

                let removed_length = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("No removed length VarInt!"))?;
                for _ in 0..removed_length.0 {
                    let component_id = seq
                        .next_element::<VarInt>()?
                        .ok_or(de::Error::custom("No component id VarInt!"))?;
                    removed.push(component_id);
                }

                Ok(ItemComponentHash { added, removed })
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}
