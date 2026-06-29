use crate::VarInt;
use crate::codec::data_component::{deserialize, serialize};
use crate::ser::{NetworkReadExt, ReadingError, WritingError, deserializer, serializer};
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{CustomNameImpl, DataComponentImpl};
use pumpkin_data::item::Item;
use pumpkin_data::item_id_remap::{remap_item_id_for_version, remap_item_id_from_version};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;
use serde::ser::SerializeStruct;
use serde::{
    Deserialize, Serialize, Serializer,
    de::{self, DeserializeSeed, SeqAccess},
};
use std::borrow::Cow;
use std::io::{Cursor, Read};

#[derive(Clone)]
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

fn serialize_any_item_stack_with_id<S: Serializer>(
    stack: &ItemStack,
    item_id: u16,
    is_template: bool,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if stack.is_empty() {
        VarInt(0).serialize(serializer)
    } else {
        let (to_add, to_remove) = item_component_counts(stack);
        let mut seq = serializer.serialize_struct("", 0)?;
        if is_template {
            seq.serialize_field("", &VarInt::from(item_id))?;
            seq.serialize_field("", &VarInt::from(stack.item_count))?;
        } else {
            seq.serialize_field("", &VarInt::from(stack.item_count))?;
            seq.serialize_field("", &VarInt::from(item_id))?;
        }
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
fn serialize_item_stack_with_id<S: Serializer>(
    stack: &ItemStack,
    item_id: u16,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serialize_any_item_stack_with_id(stack, item_id, false, serializer)
}

struct ComponentAccess<R: Read> {
    deserializer: deserializer::Deserializer<R>,
}

impl<'de, R: Read> SeqAccess<'de> for ComponentAccess<R> {
    type Error = ReadingError;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        seed.deserialize(&mut self.deserializer).map(Some)
    }
}

fn read_component_id(read: &mut impl Read) -> Result<DataComponent, ReadingError> {
    let id_val = read.get_var_int()?.0;
    let id_u8 = id_val
        .try_into()
        .map_err(|_| ReadingError::Message(format!("Invalid component ID: {id_val}")))?;
    DataComponent::try_from_id(id_u8)
        .ok_or_else(|| ReadingError::Message(format!("Unknown component ID: {id_val}")))
}

fn decode_custom_name(component_data: &[u8]) -> Result<Box<dyn DataComponentImpl>, ReadingError> {
    let mut cursor = Cursor::new(component_data);
    let mut nbt_reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
    let tag = NbtTag::deserialize(&mut nbt_reader)
        .map_err(|err| ReadingError::Message(format!("Failed to decode CustomName NBT: {err}")))?;
    let name = match tag {
        NbtTag::String(name) => TextComponent::text(name.to_string()),
        NbtTag::Compound(compound) => compound
            .get_string("text")
            .map_or_else(TextComponent::empty, |name| {
                TextComponent::text(name.to_string())
            }),
        _ => TextComponent::empty(),
    };
    Ok(CustomNameImpl { name }.to_dyn())
}

fn read_length_prefixed_component(
    read: &mut impl Read,
) -> Result<(DataComponent, Box<dyn DataComponentImpl>), ReadingError> {
    let id = read_component_id(read)?;
    let byte_len = read.get_var_int()?.0;
    let byte_len = byte_len
        .try_into()
        .map_err(|_| ReadingError::Message("Negative component data length".into()))?;
    let component_data = read.read_boxed_slice(byte_len)?;

    let component_impl = if id == DataComponent::CustomName {
        decode_custom_name(component_data.as_ref())?
    } else {
        let cursor = Cursor::new(component_data);
        let mut access = ComponentAccess {
            deserializer: deserializer::Deserializer::new(cursor),
        };
        deserialize(id, &mut access)?
    };

    Ok((id, component_impl))
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
                const MAX_COMPONENTS: i32 = 256;

                let item_count = seq
                    .next_element::<VarInt>()?
                    .ok_or_else(|| de::Error::custom("Failed to decode VarInt"))?;

                if item_count.0 == 0 {
                    return Ok(ItemStackSerializer(Cow::Borrowed(ItemStack::EMPTY)));
                }

                let item_id = seq
                    .next_element::<VarInt>()?
                    .ok_or_else(|| de::Error::custom("No item id VarInt!"))?;

                let num_to_add = seq.next_element::<VarInt>()?.map_or(0, |v| v.0);
                let num_to_remove = seq.next_element::<VarInt>()?.map_or(0, |v| v.0);

                if num_to_add < 0 || num_to_remove < 0 {
                    return Err(de::Error::custom("Negative component count"));
                }

                let total_components = num_to_add
                    .checked_add(num_to_remove)
                    .ok_or_else(|| de::Error::custom("Component count overflow"))?;

                if total_components > MAX_COMPONENTS {
                    return Err(de::Error::custom("Too many components in ItemStack patch"));
                }

                let mut patch = Vec::with_capacity((num_to_add + num_to_remove) as usize);

                for _ in 0..num_to_add {
                    let id_val = seq
                        .next_element::<VarInt>()?
                        .ok_or_else(|| de::Error::custom("Missing component ID"))?
                        .0;
                    let id = DataComponent::try_from_id(id_val as u8).ok_or_else(|| {
                        de::Error::custom(format!("Unknown component ID: {id_val}"))
                    })?;

                    // Minecraft protocol sends a byte length for the component data here
                    let _byte_len = seq
                        .next_element::<VarInt>()?
                        .ok_or_else(|| de::Error::custom("No data len VarInt!"))?;

                    let component_impl = deserialize(id, &mut seq)?;

                    patch.push((id, Some(component_impl)));
                }

                for _ in 0..num_to_remove {
                    let id_val = seq
                        .next_element::<VarInt>()?
                        .ok_or_else(|| de::Error::custom("Missing remove component ID"))?
                        .0;
                    let id = DataComponent::try_from_id(id_val as u8)
                        .ok_or_else(|| de::Error::custom("Unknown component ID"))?;
                    patch.push((id, None));
                }

                let item_id_u16: u16 = item_id
                    .0
                    .try_into()
                    .map_err(|_| de::Error::custom("Invalid item id!"))?;

                Ok(ItemStackSerializer(Cow::Owned(
                    ItemStack::new_with_component(
                        item_count.0 as u8,
                        Item::from_id(item_id_u16).unwrap_or(&Item::AIR),
                        patch,
                    ),
                )))
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
    pub fn read_length_prefixed_optional(
        mut read: impl Read,
    ) -> Result<ItemStackSerializer<'static>, ReadingError> {
        const MAX_COMPONENTS: i32 = 256;

        let item_count = read.get_var_int()?;
        if item_count.0 == 0 {
            return Ok(ItemStackSerializer(Cow::Borrowed(ItemStack::EMPTY)));
        }
        let item_count_u8 = item_count
            .0
            .try_into()
            .map_err(|_| ReadingError::Message("Invalid item count!".into()))?;

        let item_id = read.get_var_int()?;
        let num_to_add = read.get_var_int()?.0;
        let num_to_remove = read.get_var_int()?.0;

        if num_to_add < 0 || num_to_remove < 0 {
            return Err(ReadingError::Message("Negative component count".into()));
        }

        let total_components = num_to_add
            .checked_add(num_to_remove)
            .ok_or_else(|| ReadingError::Message("Component count overflow".into()))?;

        if total_components > MAX_COMPONENTS {
            return Err(ReadingError::Message(
                "Too many components in ItemStack patch".into(),
            ));
        }

        let mut patch = Vec::with_capacity(total_components as usize);

        for _ in 0..num_to_add {
            let (id, component_impl) = read_length_prefixed_component(&mut read)?;
            patch.push((id, Some(component_impl)));
        }

        for _ in 0..num_to_remove {
            patch.push((read_component_id(&mut read)?, None));
        }

        let item_id_u16 = item_id
            .0
            .try_into()
            .map_err(|_| ReadingError::Message("Invalid item id!".into()))?;

        Ok(ItemStackSerializer(Cow::Owned(
            ItemStack::new_with_component(
                item_count_u8,
                Item::from_id(item_id_u16).unwrap_or(&Item::AIR),
                patch,
            ),
        )))
    }

    pub fn write_with_version(
        &self,
        write: impl std::io::Write,
        version: &JavaMinecraftVersion,
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
    pub fn to_stack_for_version(self, version: &JavaMinecraftVersion) -> ItemStack {
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
                const MAX_COMPONENTS: i32 = 256;

                let mut added = Vec::new();
                let mut removed = Vec::new();

                let added_length = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("No added length VarInt!"))?;
                if added_length.0 < 0 || added_length.0 > MAX_COMPONENTS {
                    return Err(de::Error::custom("added_length out of bounds"));
                }
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
                if removed_length.0 < 0 || removed_length.0 > MAX_COMPONENTS {
                    return Err(de::Error::custom("removed_length out of bounds"));
                }
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

pub struct ItemStackTemplate<'a>(pub Cow<'a, ItemStack>);

impl From<ItemStack> for ItemStackTemplate<'_> {
    fn from(item: ItemStack) -> Self {
        ItemStackTemplate(Cow::Owned(item))
    }
}

impl Serialize for ItemStackTemplate<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_any_item_stack_with_id(self.0.as_ref(), self.0.item.id, true, serializer)
    }
}
