use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use pumpkin_data::item::Item;
use pumpkin_data::villager::{
    VillagerTrade, VillagerTradeModifier, VillagerTradeSet, VillagerType,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DynamicTradeModifier {
    None,
    EnchantRandomly,
    EnchantWithLevels { min: i32, max: i32 },
    ExplorationMap { destination: String },
    RandomDyes,
    RandomPotion,
    SuspiciousStew,
    Potion(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicVillagerTradeItem {
    pub item: &'static Item,
    pub count: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicVillagerTrade {
    pub wants: DynamicVillagerTradeItem,
    pub wants_b: Option<DynamicVillagerTradeItem>,
    pub gives: DynamicVillagerTradeItem,
    pub max_uses: i32,
    pub xp: i32,
    pub price_multiplier: f32,
    pub modifier: DynamicTradeModifier,
    pub allowed_types: Vec<VillagerType>,
}

impl From<&VillagerTrade> for DynamicVillagerTrade {
    fn from(t: &VillagerTrade) -> Self {
        Self {
            wants: DynamicVillagerTradeItem {
                item: t.wants.item,
                count: t.wants.count,
            },
            wants_b: t.wants_b.map(|b| DynamicVillagerTradeItem {
                item: b.item,
                count: b.count,
            }),
            gives: DynamicVillagerTradeItem {
                item: t.gives.item,
                count: t.gives.count,
            },
            max_uses: t.max_uses,
            xp: t.xp,
            price_multiplier: t.price_multiplier,
            modifier: match t.modifier {
                VillagerTradeModifier::None => DynamicTradeModifier::None,
                VillagerTradeModifier::EnchantRandomly => DynamicTradeModifier::EnchantRandomly,
                VillagerTradeModifier::EnchantWithLevels { min, max } => {
                    DynamicTradeModifier::EnchantWithLevels { min, max }
                }
                VillagerTradeModifier::ExplorationMap { destination } => {
                    DynamicTradeModifier::ExplorationMap {
                        destination: destination.to_string(),
                    }
                }
                VillagerTradeModifier::RandomDyes => DynamicTradeModifier::RandomDyes,
                VillagerTradeModifier::RandomPotion => DynamicTradeModifier::RandomPotion,
                VillagerTradeModifier::SuspiciousStew => DynamicTradeModifier::SuspiciousStew,
                VillagerTradeModifier::Potion(potion) => {
                    DynamicTradeModifier::Potion(potion.to_string())
                }
            },
            allowed_types: t.allowed_types.to_vec(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicVillagerTradeSet {
    pub trades: Vec<DynamicVillagerTrade>,
    pub amount: i32,
}

impl From<VillagerTradeSet> for DynamicVillagerTradeSet {
    fn from(set: VillagerTradeSet) -> Self {
        Self {
            trades: set.trades.iter().map(DynamicVillagerTrade::from).collect(),
            amount: set.amount,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TradeItemDefinition {
    pub id: String,
    #[serde(default)]
    pub count: Option<f32>,
}

fn deserialize_one_or_many_values<'de, D>(deserializer: D) -> Result<Vec<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = Value::deserialize(deserializer)?;
    match val {
        Value::Array(vec) => Ok(vec),
        Value::Null => Ok(Vec::new()),
        other => Ok(vec![other]),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VillagerTradeDefinition {
    pub wants: TradeItemDefinition,
    #[serde(alias = "wants_b", default)]
    pub additional_wants: Option<TradeItemDefinition>,
    pub gives: TradeItemDefinition,
    #[serde(default)]
    pub max_uses: Option<f32>,
    #[serde(default)]
    pub xp: Option<f32>,
    #[serde(alias = "price_multiplier", default)]
    pub reputation_discount: Option<f32>,
    #[serde(
        default,
        alias = "given_item_modifiers",
        rename = "given_item_modifier",
        deserialize_with = "deserialize_one_or_many_values"
    )]
    pub given_item_modifiers: Vec<Value>,
    #[serde(default)]
    pub merchant_predicate: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TradesReference {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TradeSetDefinition {
    #[serde(default = "default_amount")]
    pub amount: f32,
    pub trades: TradesReference,
    #[serde(default)]
    pub random_sequence: Option<String>,
}

const fn default_amount() -> f32 {
    1.0
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TradeTagDefinition {
    pub values: Vec<Value>,
}

#[must_use]
pub fn parse_trade_modifier(modifiers: &[Value]) -> DynamicTradeModifier {
    for modifier in modifiers {
        let Some(func) = modifier.get("type").and_then(Value::as_str) else {
            continue;
        };
        match func {
            "minecraft:enchant_randomly" => return DynamicTradeModifier::EnchantRandomly,
            "minecraft:enchant_with_levels" => {
                let (min, max) = modifier.get("levels").map_or((5, 19), |levels| {
                    let min = levels.get("min").and_then(Value::as_f64).unwrap_or(5.0) as i32;
                    let max = levels.get("max").and_then(Value::as_f64).unwrap_or(19.0) as i32;
                    (min, max)
                });
                return DynamicTradeModifier::EnchantWithLevels { min, max };
            }
            "minecraft:exploration_map" => {
                if let Some(dest) = modifier.get("destination").and_then(Value::as_str) {
                    return DynamicTradeModifier::ExplorationMap {
                        destination: dest.to_string(),
                    };
                }
            }
            "minecraft:set_random_dyes" => return DynamicTradeModifier::RandomDyes,
            "minecraft:set_random_potion" => return DynamicTradeModifier::RandomPotion,
            "minecraft:set_stew_effect" => return DynamicTradeModifier::SuspiciousStew,
            "minecraft:set_potion" => {
                if let Some(potion) = modifier.get("id").and_then(Value::as_str) {
                    return DynamicTradeModifier::Potion(potion.to_string());
                }
            }
            _ => {}
        }
    }
    DynamicTradeModifier::None
}

#[must_use]
pub fn parse_trade_allowed_types(predicate: Option<&Value>) -> Vec<VillagerType> {
    let Some(predicate) = predicate else {
        return Vec::new();
    };
    let variants = predicate
        .pointer("/predicate/minecraft:predicates/minecraft:villager~1variant")
        .or_else(|| {
            predicate.pointer("/predicate/minecraft:predicates/minecraft:villager/variant")
        });
    let Some(variants) = variants else {
        return Vec::new();
    };

    let variant_strings: Vec<&str> = match variants {
        Value::Array(arr) => arr.iter().filter_map(Value::as_str).collect(),
        Value::String(s) => vec![s.as_str()],
        _ => Vec::new(),
    };

    variant_strings
        .into_iter()
        .filter_map(|s| {
            let name = s.strip_prefix("minecraft:").unwrap_or(s);
            VillagerType::from_name(name)
        })
        .collect()
}

#[derive(Default, Clone, Debug)]
pub struct TradeRegistry {
    pub trades: HashMap<String, VillagerTradeDefinition>,
    pub trade_sets: HashMap<String, TradeSetDefinition>,
    pub trade_tags: HashMap<String, Vec<String>>,
}

impl TradeRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.trades.is_empty() && self.trade_sets.is_empty()
    }

    pub fn resolve_trade_tag(&self, tag: &str, visited: &mut HashSet<String>) -> Vec<String> {
        if !visited.insert(tag.to_string()) {
            return Vec::new();
        }

        let full_tag = if tag.starts_with("minecraft:") {
            tag.to_string()
        } else {
            format!("minecraft:{tag}")
        };
        let stripped_tag = tag.strip_prefix("minecraft:").unwrap_or(tag);

        let values = self
            .trade_tags
            .get(&full_tag)
            .or_else(|| self.trade_tags.get(stripped_tag));

        let Some(values) = values else {
            return Vec::new();
        };

        let mut result = Vec::new();
        for v in values {
            if let Some(sub) = v.strip_prefix('#') {
                for child in self.resolve_trade_tag(sub, visited) {
                    if !result.contains(&child) {
                        result.push(child);
                    }
                }
            } else {
                let full_v = if v.starts_with("minecraft:") {
                    v.clone()
                } else {
                    format!("minecraft:{v}")
                };
                if !result.contains(&full_v) {
                    result.push(full_v);
                }
            }
        }
        result
    }

    #[must_use]
    pub fn resolve_trade_definition(&self, trade_id: &str) -> Option<DynamicVillagerTrade> {
        let full_id = if trade_id.starts_with("minecraft:") {
            trade_id.to_string()
        } else {
            format!("minecraft:{trade_id}")
        };
        let stripped_id = trade_id.strip_prefix("minecraft:").unwrap_or(trade_id);

        let def = self
            .trades
            .get(&full_id)
            .or_else(|| self.trades.get(stripped_id))?;

        let wants_key = def
            .wants
            .id
            .strip_prefix("minecraft:")
            .unwrap_or(&def.wants.id);
        let wants_item = Item::from_registry_key(wants_key)?;
        let wants_count = def.wants.count.unwrap_or(1.0) as i32;

        let gives_key = def
            .gives
            .id
            .strip_prefix("minecraft:")
            .unwrap_or(&def.gives.id);
        let gives_item = Item::from_registry_key(gives_key)?;
        let gives_count = def.gives.count.unwrap_or(1.0) as i32;

        let wants_b = if let Some(b) = &def.additional_wants {
            let key = b.id.strip_prefix("minecraft:").unwrap_or(&b.id);
            let item = Item::from_registry_key(key)?;
            let count = b.count.unwrap_or(1.0) as i32;
            Some(DynamicVillagerTradeItem { item, count })
        } else {
            None
        };

        let modifier = parse_trade_modifier(&def.given_item_modifiers);

        let allowed_types = parse_trade_allowed_types(def.merchant_predicate.as_ref());

        Some(DynamicVillagerTrade {
            wants: DynamicVillagerTradeItem {
                item: wants_item,
                count: wants_count,
            },
            wants_b,
            gives: DynamicVillagerTradeItem {
                item: gives_item,
                count: gives_count,
            },
            max_uses: def.max_uses.unwrap_or(16.0) as i32,
            xp: def.xp.unwrap_or(2.0) as i32,
            price_multiplier: def.reputation_discount.unwrap_or(0.05),
            modifier,
            allowed_types,
        })
    }

    #[must_use]
    pub fn resolve_trade_set(&self, set_key: &str) -> Option<DynamicVillagerTradeSet> {
        let full_key = if set_key.starts_with("minecraft:") {
            set_key.to_string()
        } else {
            format!("minecraft:{set_key}")
        };
        let stripped_key = set_key.strip_prefix("minecraft:").unwrap_or(set_key);

        let set_def = self
            .trade_sets
            .get(&full_key)
            .or_else(|| self.trade_sets.get(stripped_key))?;

        let mut trade_ids = Vec::new();
        match &set_def.trades {
            TradesReference::Single(s) => {
                if let Some(tag) = s.strip_prefix('#') {
                    trade_ids = self.resolve_trade_tag(tag, &mut HashSet::new());
                } else {
                    trade_ids.push(s.clone());
                }
            }
            TradesReference::Multiple(list) => {
                for item in list {
                    if let Some(tag) = item.strip_prefix('#') {
                        trade_ids.extend(self.resolve_trade_tag(tag, &mut HashSet::new()));
                    } else {
                        trade_ids.push(item.clone());
                    }
                }
            }
        }

        let mut trades = Vec::new();
        for id in trade_ids {
            if let Some(trade) = self.resolve_trade_definition(&id) {
                trades.push(trade);
            }
        }

        Some(DynamicVillagerTradeSet {
            trades,
            amount: set_def.amount.round() as i32,
        })
    }

    #[must_use]
    pub fn get_villager_trade_set(
        &self,
        profession: &str,
        level: i32,
    ) -> Option<DynamicVillagerTradeSet> {
        let candidates = [
            format!("{profession}/level_{level}"),
            format!("minecraft:{profession}/level_{level}"),
            format!("trade_set/{profession}/level_{level}"),
            format!("minecraft:trade_set/{profession}/level_{level}"),
        ];

        for key in &candidates {
            if let Some(trade_set) = self.resolve_trade_set(key) {
                return Some(trade_set);
            }
        }

        None
    }

    #[must_use]
    pub fn get_wandering_trader_trade_set(&self, tier: &str) -> Option<DynamicVillagerTradeSet> {
        let candidates = [
            format!("wandering_trader/{tier}"),
            format!("minecraft:wandering_trader/{tier}"),
            format!("trade_set/wandering_trader/{tier}"),
            format!("minecraft:trade_set/wandering_trader/{tier}"),
        ];

        for key in &candidates {
            if let Some(trade_set) = self.resolve_trade_set(key) {
                return Some(trade_set);
            }
        }

        None
    }
}

pub fn load_trades_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    dir: &Path,
    registry: &mut HashMap<String, VillagerTradeDefinition, S>,
) -> usize {
    let before = registry.len();
    load_trades_recursive(namespace, dir, dir, registry);
    registry.len() - before
}

fn load_trades_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    registry: &mut HashMap<String, VillagerTradeDefinition, S>,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_trades_recursive(namespace, base_dir, &path, registry);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            && let Ok(rel_path) = path.strip_prefix(base_dir)
        {
            let mut stem_path = rel_path.to_string_lossy().to_string();
            if let Some(stem) = stem_path.strip_suffix(".json") {
                stem_path = stem.to_string();
            }
            let stem_path = stem_path.replace('\\', "/");
            let full_id = format!("{namespace}:{stem_path}");

            if let Ok(content) = fs::read_to_string(&path) {
                match serde_json::from_str::<VillagerTradeDefinition>(&content) {
                    Ok(def) => {
                        registry.insert(full_id, def.clone());
                        registry.insert(stem_path, def);
                    }
                    Err(err) => {
                        warn!("Failed to parse villager trade {full_id}: {err}");
                    }
                }
            }
        }
    }
}

pub fn load_trade_sets_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    dir: &Path,
    registry: &mut HashMap<String, TradeSetDefinition, S>,
) -> usize {
    let before = registry.len();
    load_trade_sets_recursive(namespace, dir, dir, registry);
    registry.len() - before
}

fn load_trade_sets_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    registry: &mut HashMap<String, TradeSetDefinition, S>,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_trade_sets_recursive(namespace, base_dir, &path, registry);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            && let Ok(rel_path) = path.strip_prefix(base_dir)
        {
            let mut stem_path = rel_path.to_string_lossy().to_string();
            if let Some(stem) = stem_path.strip_suffix(".json") {
                stem_path = stem.to_string();
            }
            let stem_path = stem_path.replace('\\', "/");
            let full_id = format!("{namespace}:{stem_path}");

            if let Ok(content) = fs::read_to_string(&path) {
                match serde_json::from_str::<TradeSetDefinition>(&content) {
                    Ok(def) => {
                        registry.insert(full_id, def.clone());
                        registry.insert(stem_path, def);
                    }
                    Err(err) => {
                        warn!("Failed to parse trade set {full_id}: {err}");
                    }
                }
            }
        }
    }
}

pub fn load_trade_tags_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    dir: &Path,
    registry: &mut HashMap<String, Vec<String>, S>,
) -> usize {
    let before = registry.len();
    load_trade_tags_recursive(namespace, dir, dir, registry);
    registry.len() - before
}

fn load_trade_tags_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    registry: &mut HashMap<String, Vec<String>, S>,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_trade_tags_recursive(namespace, base_dir, &path, registry);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            && let Ok(rel_path) = path.strip_prefix(base_dir)
        {
            let mut stem_path = rel_path.to_string_lossy().to_string();
            if let Some(stem) = stem_path.strip_suffix(".json") {
                stem_path = stem.to_string();
            }
            let stem_path = stem_path.replace('\\', "/");
            let full_id = format!("{namespace}:{stem_path}");

            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(tag_file) = serde_json::from_str::<TradeTagDefinition>(&content)
            {
                let values = tag_file
                    .values
                    .into_iter()
                    .filter_map(|v| match v {
                        Value::String(s) => Some(s),
                        Value::Object(obj) => {
                            obj.get("id").and_then(Value::as_str).map(String::from)
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                registry.insert(full_id, values.clone());
                registry.insert(stem_path, values);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_trade_json() {
        let json = r#"{
            "gives": {
                "id": "minecraft:iron_leggings"
            },
            "max_uses": 12,
            "reputation_discount": 0.2,
            "wants": {
                "count": 7,
                "id": "minecraft:emerald"
            }
        }"#;

        let def: VillagerTradeDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.gives.id, "minecraft:iron_leggings");
        assert_eq!(def.wants.id, "minecraft:emerald");
        assert_eq!(def.wants.count, Some(7.0));
        assert_eq!(def.max_uses, Some(12.0));
        assert_eq!(def.reputation_discount, Some(0.2));
    }

    #[test]
    fn parse_trade_set_json() {
        let json = r##"{
            "amount": 2,
            "random_sequence": "minecraft:trade_set/armorer/level_1",
            "trades": "#minecraft:armorer/level_1"
        }"##;

        let def: TradeSetDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.amount, 2.0);
        match def.trades {
            TradesReference::Single(tag) => assert_eq!(tag, "#minecraft:armorer/level_1"),
            TradesReference::Multiple(_) => panic!("expected single trades reference"),
        }
    }

    #[test]
    fn resolve_trade_registry_chain() {
        let mut reg = TradeRegistry::new();

        reg.trade_sets.insert(
            "minecraft:armorer/level_1".to_string(),
            TradeSetDefinition {
                amount: 2.0,
                trades: TradesReference::Single("#minecraft:armorer/level_1".to_string()),
                random_sequence: None,
            },
        );

        reg.trade_tags.insert(
            "minecraft:armorer/level_1".to_string(),
            vec!["minecraft:armorer/1/emerald_iron_leggings".to_string()],
        );

        reg.trades.insert(
            "minecraft:armorer/1/emerald_iron_leggings".to_string(),
            VillagerTradeDefinition {
                wants: TradeItemDefinition {
                    id: "minecraft:emerald".to_string(),
                    count: Some(7.0),
                },
                additional_wants: None,
                gives: TradeItemDefinition {
                    id: "minecraft:iron_leggings".to_string(),
                    count: Some(1.0),
                },
                max_uses: Some(12.0),
                xp: Some(2.0),
                reputation_discount: Some(0.2),
                given_item_modifiers: Vec::new(),
                merchant_predicate: None,
            },
        );

        let resolved = reg.get_villager_trade_set("armorer", 1).unwrap();
        assert_eq!(resolved.amount, 2);
        assert_eq!(resolved.trades.len(), 1);
        assert_eq!(resolved.trades[0].wants.item.id, Item::EMERALD.id);
        assert_eq!(resolved.trades[0].wants.count, 7);
        assert_eq!(resolved.trades[0].gives.item.id, Item::IRON_LEGGINGS.id);
        assert_eq!(resolved.trades[0].max_uses, 12);
    }

    #[test]
    fn load_from_directory_structure() {
        let temp_dir =
            std::env::temp_dir().join(format!("pumpkin_trade_test_{}", rand::random::<u64>()));
        let trade_set_dir = temp_dir.join("trade_set").join("cleric");
        let tag_dir = temp_dir.join("tags").join("villager_trade").join("cleric");
        let trade_dir = temp_dir.join("villager_trade").join("cleric").join("1");

        fs::create_dir_all(&trade_set_dir).unwrap();
        fs::create_dir_all(&tag_dir).unwrap();
        fs::create_dir_all(&trade_dir).unwrap();

        fs::write(
            trade_set_dir.join("level_1.json"),
            r##"{
                "amount": 2,
                "trades": "#custom:cleric/level_1"
            }"##,
        )
        .unwrap();

        fs::write(
            tag_dir.join("level_1.json"),
            r#"{
                "values": [
                    "custom:cleric/1/rotten_flesh_emerald"
                ]
            }"#,
        )
        .unwrap();

        fs::write(
            trade_dir.join("rotten_flesh_emerald.json"),
            r#"{
                "wants": {
                    "id": "minecraft:rotten_flesh",
                    "count": 32
                },
                "gives": {
                    "id": "minecraft:emerald",
                    "count": 1
                },
                "max_uses": 16,
                "xp": 2,
                "reputation_discount": 0.05
            }"#,
        )
        .unwrap();

        let mut reg = TradeRegistry::new();
        let loaded_sets =
            load_trade_sets_from_dir("custom", &temp_dir.join("trade_set"), &mut reg.trade_sets);
        let loaded_tags = load_trade_tags_from_dir(
            "custom",
            &temp_dir.join("tags").join("villager_trade"),
            &mut reg.trade_tags,
        );
        let loaded_trades =
            load_trades_from_dir("custom", &temp_dir.join("villager_trade"), &mut reg.trades);

        assert_eq!(loaded_sets, 2); // full and stem
        assert_eq!(loaded_tags, 2);
        assert_eq!(loaded_trades, 2);

        let resolved = reg.resolve_trade_set("custom:cleric/level_1").unwrap();
        assert_eq!(resolved.amount, 2);
        assert_eq!(resolved.trades.len(), 1);
        assert_eq!(resolved.trades[0].wants.item.id, Item::ROTTEN_FLESH.id);
        assert_eq!(resolved.trades[0].wants.count, 32);
        assert_eq!(resolved.trades[0].gives.item.id, Item::EMERALD.id);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn trade_modifiers_and_predicate() {
        let json = r#"{
            "wants": { "id": "minecraft:emerald", "count": 1 },
            "gives": { "id": "minecraft:iron_sword", "count": 1 },
            "given_item_modifier": [
                {
                    "type": "minecraft:enchant_with_levels",
                    "levels": { "min": 10, "max": 25 }
                }
            ],
            "merchant_predicate": {
                "type": "minecraft:entity_properties",
                "entity": "this",
                "predicate": {
                    "minecraft:predicates": {
                        "minecraft:villager/variant": [
                            "minecraft:desert",
                            "minecraft:taiga"
                        ]
                    }
                }
            }
        }"#;

        let def: VillagerTradeDefinition = serde_json::from_str(json).unwrap();
        let mut reg = TradeRegistry::new();
        reg.trades.insert("test:trade".to_string(), def);

        let trade = reg.resolve_trade_definition("test:trade").unwrap();
        assert_eq!(
            trade.modifier,
            DynamicTradeModifier::EnchantWithLevels { min: 10, max: 25 }
        );
        assert_eq!(
            trade.allowed_types,
            vec![VillagerType::Desert, VillagerType::Taiga]
        );
    }
}
