use crate::VarInt;
use pumpkin_data::item::Item;
use pumpkin_world::item::ItemStack;
use serde::{
    Deserialize, Serialize, Serializer,
    de::{self, SeqAccess},
    ser,
};

#[derive(Debug, Clone)]
pub enum Slot {
    NoItem,
    Item {
        // This also handles items on the ground which can have >64 items
        item_count: u32,
        item_id: u16,
        // TODO: Implement item components
    },
}

impl<'de> Deserialize<'de> for Slot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Slot;

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
                    Slot::NoItem
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

                    Slot::Item {
                        // i32 can always be u32
                        item_count: item_count.0 as u32,
                        item_id,
                    }
                };

                Ok(slot)
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

impl Serialize for Slot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::NoItem => VarInt(0).serialize(serializer),
            Self::Item {
                item_count,
                item_id,
            } => {
                // TODO: Components

                #[derive(Serialize)]
                struct NetworkRepr {
                    item_count: VarInt,
                    item_id: VarInt,
                    components_to_add: VarInt,
                    components_to_remove: VarInt,
                }

                let item_count: i32 = (*item_count)
                    .try_into()
                    .map_err(|_| ser::Error::custom("Item count overflows an i32!"))?;

                NetworkRepr {
                    item_count: item_count.into(),
                    item_id: (*item_id).into(),
                    components_to_add: 0.into(),
                    components_to_remove: 0.into(),
                }
                .serialize(serializer)
            }
        }
    }
}

impl Slot {
    pub fn new(item_id: u16, count: u32) -> Self {
        Self::Item {
            item_count: count,
            item_id,
        }
    }

    pub fn to_stack(self) -> Result<Option<ItemStack>, &'static str> {
        match self {
            Self::NoItem => Ok(None),
            Self::Item {
                item_count,
                item_id,
            } => {
                let item = Item::from_id(item_id).ok_or("Item id invalid")?;
                if item_count > item.components.max_stack_size as u32 {
                    Err("Stack item count greater than allowed")
                } else {
                    let stack = ItemStack {
                        item,
                        // This is checked above
                        item_count: item_count as u8,
                    };
                    Ok(Some(stack))
                }
            }
        }
    }

    pub const fn empty() -> Self {
        Self::NoItem
    }
}

impl From<&ItemStack> for Slot {
    fn from(item: &ItemStack) -> Self {
        Slot::new(item.item.id, item.item_count as u32)
    }
}

impl From<Option<&ItemStack>> for Slot {
    fn from(item: Option<&ItemStack>) -> Self {
        item.map(Slot::from).unwrap_or(Slot::empty())
    }
}

// impl From<&Option<ItemStack>> for Slot {
//     fn from(item: &Option<ItemStack>) -> Self {
//         item.map(|stack| Self::from(&stack))
//             .unwrap_or(Slot::empty())
//     }
// }
