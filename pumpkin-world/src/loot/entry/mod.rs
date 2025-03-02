use alternative::AlternativeEntry;
use item::ItemEntry;
use serde::Deserialize;

use crate::item::ItemStack;

mod alternative;
mod item;

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum LootPoolEntryTypes {
    #[serde(rename = "minecraft:empty")]
    Empty,
    #[serde(rename = "minecraft:item")]
    Item(ItemEntry),
    #[serde(rename = "minecraft:loot_table")]
    LootTable,
    #[serde(rename = "minecraft:dynamic")]
    Dynamic,
    #[serde(rename = "minecraft:tag")]
    Tag,
    #[serde(rename = "minecraft:alternatives")]
    Alternatives(AlternativeEntry),
    #[serde(rename = "minecraft:sequence")]
    Sequence,
    #[serde(rename = "minecraft:group")]
    Group,
}

impl LootPoolEntryTypes {
    pub fn get_items(&self) -> Vec<ItemStack> {
        match self {
            LootPoolEntryTypes::Empty => todo!(),
            LootPoolEntryTypes::Item(item_entry) => item_entry.get_items(),
            LootPoolEntryTypes::LootTable => todo!(),
            LootPoolEntryTypes::Dynamic => todo!(),
            LootPoolEntryTypes::Tag => todo!(),
            LootPoolEntryTypes::Alternatives(alternative) => alternative.get_items(),
            LootPoolEntryTypes::Sequence => todo!(),
            LootPoolEntryTypes::Group => todo!(),
        }
    }
}
