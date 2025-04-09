use crate::VarInt;
use pumpkin_data::item::Item;
use pumpkin_world::item::ItemStack;
use serde::{
    Deserialize, Serialize, Serializer,
    de::{self, SeqAccess},
};

#[derive(Debug, Clone)]
pub struct ItemStackSerializer(pub ItemStack);

impl<'de> Deserialize<'de> for ItemStackSerializer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = ItemStackSerializer;

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
                    ItemStackSerializer(ItemStack::EMPTY)
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

                    ItemStackSerializer(ItemStack::new(
                        item_count.0 as u8,
                        Item::from_id(item_id).unwrap_or(Item::AIR),
                    ))
                };

                Ok(slot)
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

impl Serialize for ItemStackSerializer {
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

impl ItemStackSerializer {
    pub fn to_stack(self) -> ItemStack {
        self.0
    }
}

impl From<ItemStack> for ItemStackSerializer {
    fn from(item: ItemStack) -> Self {
        ItemStackSerializer(item)
    }
}

impl From<Option<ItemStack>> for ItemStackSerializer {
    fn from(item: Option<ItemStack>) -> Self {
        match item {
            Some(item) => ItemStackSerializer::from(item),
            None => ItemStackSerializer::from(ItemStack::EMPTY),
        }
    }
}
