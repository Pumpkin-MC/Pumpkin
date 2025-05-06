use std::borrow::Cow;

use crate::VarInt;
use pumpkin_data::item::Item;
use pumpkin_world::item::ItemStack;
use serde::{
    Deserialize, Serialize, Serializer,
    de::{self, SeqAccess},
};

#[derive(Debug, Clone)]
pub struct ItemStackSerializer<'a>(pub Cow<'a, ItemStack>);

impl<'de> Deserialize<'de> for ItemStackSerializer<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = ItemStackSerializer<'static>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Slot encoded in a byte sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let item_count = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("Failed to decode VarInt"))?;

                let slot = if item_count.0 == 0 {
                    ItemStackSerializer(Cow::Borrowed(&ItemStack::EMPTY))
                } else {
                    let item_id = seq
                        .next_element::<VarInt>()?
                        .ok_or(de::Error::custom("No item id VarInt!"))?;

                    let num_components_to_add = seq
                        .next_element::<VarInt>()?
                        .ok_or(de::Error::custom("No component add length VarInt!"))?;
                    let num_components_to_remove = seq
                        .next_element::<VarInt>()?
                        .ok_or(de::Error::custom("No component remove length VarInt!"))?;

                    if num_components_to_add.0 != 0 || num_components_to_remove.0 != 0 {
                        return Err(de::Error::custom(
                            "Slot components are currently unsupported",
                        ));
                    }

                    let item_id: u16 = item_id
                        .0
                        .try_into()
                        .map_err(|_| de::Error::custom("Invalid item id!"))?;

                    ItemStackSerializer(Cow::Owned(ItemStack::new(
                        item_count.0 as u8,
                        Item::from_id(item_id).unwrap_or(&Item::AIR),
                    )))
                };

                Ok(slot)
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

impl Serialize for ItemStackSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.0.is_empty() {
            VarInt(0).serialize(serializer)
        } else {
            // TODO: Components

            #[derive(Serialize)]
            struct NetworkRepr {
                item_count: VarInt,
                item_id: VarInt,
                components_to_add: VarInt,
                components_to_remove: VarInt,
            }

            NetworkRepr {
                item_count: self.0.item_count.into(),
                item_id: self.0.item.id.into(),
                components_to_add: 0.into(),
                components_to_remove: 0.into(),
            }
            .serialize(serializer)
        }
    }
}

impl ItemStackSerializer<'_> {
    pub fn to_stack(self) -> ItemStack {
        self.0.into_owned()
    }
}

impl From<ItemStack> for ItemStackSerializer<'_> {
    fn from(item: ItemStack) -> Self {
        ItemStackSerializer(Cow::Owned(item))
    }
}

impl From<Option<ItemStack>> for ItemStackSerializer<'_> {
    fn from(item: Option<ItemStack>) -> Self {
        match item {
            Some(item) => ItemStackSerializer::from(item),
            None => ItemStackSerializer(Cow::Borrowed(&ItemStack::EMPTY)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemComponentHash {
    pub added: Vec<(VarInt, VarInt)>,
    pub removed: Vec<VarInt>,
}

#[derive(Debug, Clone)]
pub struct ItemStackHash {
    item_id: VarInt,
    count: VarInt,
    #[allow(dead_code)]
    components: ItemComponentHash,
}

impl OptionalItemStackHash {
    pub fn hash_equals(&self, other: &ItemStack) -> bool {
        if let Some(hash) = &self.0 {
            // TODO: Components
            hash.item_id == other.item.id.into() && hash.count == other.item_count.into()
        } else {
            other.is_empty()
        }
    }
}

#[derive(Debug, Clone)]
pub struct OptionalItemStackHash(pub Option<ItemStackHash>);

impl<'de> Deserialize<'de> for OptionalItemStackHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = OptionalItemStackHash;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Slot encoded in a byte sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = ItemComponentHash;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Slot encoded in a byte sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
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
                        .next_element::<VarInt>()?
                        .ok_or(de::Error::custom("No component value VarInt!"))?;
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
