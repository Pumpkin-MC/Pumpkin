use pumpkin_data::item::Item;

mod categories;
#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// Item Rarity
pub enum Rarity {
    Common,
    UnCommon,
    Rare,
    Epic,
}

#[derive(Clone, Copy, Debug)]
pub struct ItemStack {
    pub item_count: u8,
    pub item: Item,
}

impl PartialEq for ItemStack {
    fn eq(&self, other: &Self) -> bool {
        self.item.id == other.item.id
    }
}

impl ItemStack {
    pub fn new(item_count: u8, item: Item) -> Self {
        Self { item_count, item }
    }
}
