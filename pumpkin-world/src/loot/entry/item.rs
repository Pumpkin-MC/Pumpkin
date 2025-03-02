use pumpkin_data::item::Item;
use serde::Deserialize;

use crate::item::ItemStack;

#[derive(Deserialize, Clone)]
pub struct ItemEntry {
    name: String,
}

impl ItemEntry {
    pub fn get_items(&self) -> Vec<ItemStack> {
        let item = Item::from_registry_key(&self.name.replace("minecraft:", "")).unwrap();
        vec![ItemStack::new(1, item)]
    }
}
