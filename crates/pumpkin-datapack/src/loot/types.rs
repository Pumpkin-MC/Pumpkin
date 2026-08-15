use std::collections::HashMap;

use crate::Identifier;

/// A simple item stack produced by the datapack loot evaluator,
/// carrying optional component data from functions like `set_components`.
#[derive(Debug, Clone)]
pub struct DpItemStack {
    pub item_id: String,
    pub count: u8,
    /// Raw component data from loot functions, keyed by component name
    /// (e.g. `"minecraft:profile"`).
    pub components: HashMap<String, serde_json::Value>,
}

impl DpItemStack {
    #[must_use]
    pub fn new(item_id: String, count: u8) -> Self {
        Self {
            item_id,
            count,
            components: HashMap::new(),
        }
    }
}

/// A fully parsed loot table from a datapack.
#[derive(Debug, Clone)]
pub struct LootTable {
    pub id: Identifier,
    pub loot_type: String,
    pub pools: Vec<LootPool>,
    pub random_sequence: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LootPool {
    pub rolls: LootNumberProvider,
    pub bonus_rolls: LootNumberProvider,
    pub conditions: Vec<LootCondition>,
    pub entries: Vec<LootEntry>,
    pub functions: Vec<LootFunction>,
}

#[derive(Debug, Clone)]
pub struct LootEntry {
    pub weight: i32,
    pub quality: i32,
    pub content: LootEntryType,
    pub conditions: Vec<LootCondition>,
    pub functions: Vec<LootFunction>,
}

#[derive(Debug, Clone)]
pub enum LootEntryType {
    Empty,
    Item(String),
    LootTable(String),
    Tag { name: String, expand: bool },
    Alternatives(Vec<LootEntry>),
    Sequence(Vec<LootEntry>),
    Group(Vec<LootEntry>),
}

#[derive(Debug, Clone)]
pub enum LootNumberProvider {
    Constant(f32),
    Uniform { min: f32, max: f32 },
    Binomial { n: i32, p: f32 },
}

/// All supported loot conditions with owned data.
#[derive(Debug, Clone)]
pub enum LootCondition {
    SurvivesExplosion,
    KilledByPlayer,
    RandomChance(f32),
    RandomChanceWithLooting {
        enchantment: String,
        unenchanted_chance: f32,
        enchanted_chance: LootNumberProvider,
    },
    MatchTool {
        items: Option<Vec<String>>,
        /// If true, requires silk touch (when no items specified)
        require_silk_touch: bool,
    },
    EntityProperties {
        entity: String,
        predicate: serde_json::Value,
    },
    DamageSourceProperties(serde_json::Value),
    Inverted(Box<Self>),
    AllOf(Vec<Self>),
    AnyOf(Vec<Self>),
    LocationCheck(serde_json::Value),
    WeatherCheck {
        raining: Option<bool>,
        thundering: Option<bool>,
    },
    TableBonus {
        enchantment: String,
        chances: Vec<f32>,
    },
    EntityScores {
        entity: String,
    },
    TimeCheck {
        value: Option<LootNumberProvider>,
        period: Option<i64>,
    },
    ValueCheck {
        value: LootNumberProvider,
        range: (Option<f32>, Option<f32>),
    },
    Reference(String),
    EnchantmentActiveCheck(bool),
    BlockStateProperty(serde_json::Value),
}

#[derive(Debug, Clone)]
pub struct LootFunction {
    pub content: LootFunctionType,
    pub conditions: Vec<LootCondition>,
}

#[derive(Debug, Clone)]
pub enum LootFunctionType {
    SetCount {
        count: LootNumberProvider,
        add: bool,
    },
    SetDamage {
        damage: LootNumberProvider,
        add: bool,
    },
    SetComponents(serde_json::Value),
    CopyComponents {
        source: String,
    },
    FurnaceSmelt,
    EnchantedCountIncrease {
        enchantment: String,
        count: LootNumberProvider,
        limit: Option<f32>,
    },
    ApplyBonus {
        enchantment: String,
        formula: String,
        parameters: Option<serde_json::Value>,
    },
    LimitCount {
        min: Option<f32>,
        max: Option<f32>,
    },
    ExplosionDecay,
    SetPotion {
        id: String,
    },
    SetOminousBottleAmplifier,
    CopyState(serde_json::Value),
    EnchantRandomly(serde_json::Value),
    EnchantWithLevels(serde_json::Value),
    SetStewEffect(serde_json::Value),
    SetInstrument(serde_json::Value),
    ExplorationMap(serde_json::Value),
    SetName(serde_json::Value),
    SetEnchantments(serde_json::Value),
    CopyCustomData(serde_json::Value),
    SetCustomData(serde_json::Value),
    Filtered(serde_json::Value),
}
