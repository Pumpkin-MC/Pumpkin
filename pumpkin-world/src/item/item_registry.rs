use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

const ITEMS_JSON: &str = include_str!("../../../assets/items.json");

pub static ITEMS: LazyLock<HashMap<String, Item>> = LazyLock::new(|| {
    serde_json::from_str(ITEMS_JSON).expect("Could not parse items.json registry.")
});

pub fn get_item(name: &str) -> Option<&Item> {
    ITEMS.get(&name.replace("minecraft:", ""))
}

#[derive(Deserialize, Clone, Debug)]
pub struct Item {
    pub id: u16,
    pub components: ItemComponents,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ItemComponents {
    #[serde(rename = "minecraft:max_stack_size")]
    pub max_stack_size: u8,
}
