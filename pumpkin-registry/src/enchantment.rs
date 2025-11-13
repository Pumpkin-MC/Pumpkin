use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enchantment {
    description: Description,
    #[serde(skip_serializing_if = "Option::is_none")]
    exclusive_set: Option<String>,
    supported_items: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_items: Option<String>,
    weight: i32,
    max_level: i32,
    min_cost: Cost,
    max_cost: Cost,
    anvil_cost: i32,
    slots: Vec<String>,
    #[serde(skip_serializing)]
    effects: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description {
    translate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    base: i32,
    per_level_above_first: i32,
}
