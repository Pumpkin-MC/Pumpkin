pub mod parser;
pub use serde_json::Value as LootPredicateValue;

/// Conditions required for an entry or pool to be eligible for loot generation.
#[derive(Clone, Copy, Debug, Default)]
pub enum LootCondition {
    #[default]
    None,
    Never,
    SilkTouch,
    NoSilkTouch,
    Shears,
    SilkTouchOrShears,
    NoSilkTouchOrShears,
    SurvivesExplosion,
    KilledByPlayer,
    RandomChance {
        chance: f32,
    },
    RandomChanceWithEnchantedBonus {
        enchantment: &'static str,
        unenchanted_chance: f32,
        enchanted_chance_base: f32,
        enchanted_chance_per_level_above_first: f32,
    },
    TableBonus {
        enchantment: &'static str,
        chances: &'static [f32],
    },
    AllOf(&'static [Self]),
    AnyOf(&'static [Self]),
    Inverted(&'static Self),
    EntityOnFire,
    DirectAttackerWeaponEnchantment(&'static str),
    EntityProperties {
        target: LootEntityTarget,
        predicate: &'static std::sync::LazyLock<serde_json::Value>,
    },
    MatchBlock(&'static std::sync::LazyLock<serde_json::Value>),
    LocationCheck {
        offset: [i32; 3],
        predicate: &'static std::sync::LazyLock<serde_json::Value>,
    },
    MatchTool(&'static std::sync::LazyLock<serde_json::Value>),
    WeatherCheck {
        raining: Option<bool>,
        thundering: Option<bool>,
    },
    DamageSourceTag {
        tag: &'static str,
        expected: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LootEntityTarget {
    This,
    Attacker,
    DirectAttacker,
}

/// Bonus count formulas when tools have fortune or looting enchantments.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LootBonusFormula {
    OreDrops,
    UniformBonusCount(i32),
    BinomialWithBonusCount { extra: i32, probability: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LootNumberProvider {
    Constant(f64),
    Uniform { min: f64, max: f64 },
    Binomial { n: i32, probability: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LootFunctionKind {
    SetCount {
        count: LootNumberProvider,
        add: bool,
    },
    EnchantedCountIncrease {
        count: LootNumberProvider,
        limit: i32,
    },
    LimitCount {
        min: Option<LootNumberProvider>,
        max: Option<LootNumberProvider>,
    },
    ExplosionDecay,
    ApplyBonus(LootBonusFormula),
    FurnaceSmelt,
}

#[derive(Clone, Copy, Debug)]
pub struct LootFunction {
    pub kind: LootFunctionKind,
    pub enchantment: Option<&'static str>,
    pub condition: LootCondition,
}

#[derive(Clone, Copy, Debug)]
pub enum LootEntryKind {
    Item(&'static str),
    Dynamic(&'static str),
    Empty,
    Tag { name: &'static str, expand: bool },
    Alternatives(&'static [LootEntry]),
    Group(&'static [LootEntry]),
    Sequence(&'static [LootEntry]),
    Table(&'static str),
    InlineTable(&'static LootTable),
}

/// A weighted leaf or an ordered composite entry in a loot pool.
#[derive(Clone, Copy, Debug)]
pub struct LootEntry {
    pub kind: LootEntryKind,
    pub weight: i32,
    pub quality: i32,
    pub condition: LootCondition,
    pub functions: &'static [LootFunction],
}

#[derive(Clone, Copy, Debug)]
pub struct LootPool {
    pub entries: &'static [LootEntry],
    pub rolls: LootNumberProvider,
    pub bonus_rolls: LootNumberProvider,
    pub condition: LootCondition,
    pub functions: &'static [LootFunction],
}

#[derive(Clone, Copy, Debug)]
pub struct LootTable {
    pub pools: &'static [LootPool],
    pub functions: &'static [LootFunction],
}

pub type ChestLootEntry = LootEntry;
pub type ChestLootPool = LootPool;
pub type ChestLootTable = LootTable;

#[derive(Clone, Debug, PartialEq, Default)]
pub enum DynamicLootCondition {
    #[default]
    None,
    Never,
    SilkTouch,
    NoSilkTouch,
    Shears,
    SilkTouchOrShears,
    NoSilkTouchOrShears,
    SurvivesExplosion,
    KilledByPlayer,
    RandomChance {
        chance: f32,
    },
    RandomChanceWithEnchantedBonus {
        enchantment: String,
        unenchanted_chance: f32,
        enchanted_chance_base: f32,
        enchanted_chance_per_level_above_first: f32,
    },
    TableBonus {
        enchantment: String,
        chances: Box<[f32]>,
    },
    AllOf(Vec<Self>),
    AnyOf(Vec<Self>),
    Inverted(Box<Self>),
    EntityOnFire,
    DirectAttackerWeaponEnchantment(String),
    EntityProperties {
        target: LootEntityTarget,
        predicate: serde_json::Value,
    },
    MatchBlock(serde_json::Value),
    LocationCheck {
        offset: [i32; 3],
        predicate: serde_json::Value,
    },
    MatchTool(serde_json::Value),
    WeatherCheck {
        raining: Option<bool>,
        thundering: Option<bool>,
    },
    DamageSourceTag {
        tag: String,
        expected: bool,
    },
}

impl From<LootCondition> for DynamicLootCondition {
    fn from(condition: LootCondition) -> Self {
        match condition {
            LootCondition::None => Self::None,
            LootCondition::Never => Self::Never,
            LootCondition::SilkTouch => Self::SilkTouch,
            LootCondition::NoSilkTouch => Self::NoSilkTouch,
            LootCondition::Shears => Self::Shears,
            LootCondition::SilkTouchOrShears => Self::SilkTouchOrShears,
            LootCondition::NoSilkTouchOrShears => Self::NoSilkTouchOrShears,
            LootCondition::SurvivesExplosion => Self::SurvivesExplosion,
            LootCondition::KilledByPlayer => Self::KilledByPlayer,
            LootCondition::RandomChance { chance } => Self::RandomChance { chance },
            LootCondition::RandomChanceWithEnchantedBonus {
                enchantment,
                unenchanted_chance,
                enchanted_chance_base,
                enchanted_chance_per_level_above_first,
            } => Self::RandomChanceWithEnchantedBonus {
                enchantment: enchantment.into(),
                unenchanted_chance,
                enchanted_chance_base,
                enchanted_chance_per_level_above_first,
            },
            LootCondition::TableBonus {
                enchantment,
                chances,
            } => Self::TableBonus {
                enchantment: enchantment.into(),
                chances: chances.into(),
            },
            LootCondition::AllOf(conditions) => {
                Self::AllOf(conditions.iter().copied().map(Self::from).collect())
            }
            LootCondition::AnyOf(conditions) => {
                Self::AnyOf(conditions.iter().copied().map(Self::from).collect())
            }
            LootCondition::Inverted(condition) => Self::Inverted(Box::new(Self::from(*condition))),
            LootCondition::EntityOnFire => Self::EntityOnFire,
            LootCondition::DirectAttackerWeaponEnchantment(tag) => {
                Self::DirectAttackerWeaponEnchantment(tag.into())
            }
            LootCondition::EntityProperties { target, predicate } => Self::EntityProperties {
                target,
                predicate: (**predicate).clone(),
            },
            LootCondition::LocationCheck { offset, predicate } => Self::LocationCheck {
                offset,
                predicate: (**predicate).clone(),
            },
            LootCondition::MatchBlock(predicate) => Self::MatchBlock((**predicate).clone()),
            LootCondition::MatchTool(predicate) => Self::MatchTool((**predicate).clone()),
            LootCondition::WeatherCheck {
                raining,
                thundering,
            } => Self::WeatherCheck {
                raining,
                thundering,
            },
            LootCondition::DamageSourceTag { tag, expected } => Self::DamageSourceTag {
                tag: tag.into(),
                expected,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct DynamicLootFunction {
    pub kind: LootFunctionKind,
    pub enchantment: Option<String>,
    pub condition: DynamicLootCondition,
}

impl From<LootFunction> for DynamicLootFunction {
    fn from(function: LootFunction) -> Self {
        Self {
            kind: function.kind,
            enchantment: function.enchantment.map(str::to_owned),
            condition: function.condition.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DynamicLootEntryKind {
    Item(String),
    Dynamic(String),
    Empty,
    Tag { name: String, expand: bool },
    Alternatives(Vec<DynamicLootEntry>),
    Group(Vec<DynamicLootEntry>),
    Sequence(Vec<DynamicLootEntry>),
    Table(String),
    InlineTable(Box<DynamicLootTable>),
}

#[derive(Clone, Debug)]
pub struct DynamicLootEntry {
    pub kind: DynamicLootEntryKind,
    pub weight: i32,
    pub quality: i32,
    pub condition: DynamicLootCondition,
    pub functions: Vec<DynamicLootFunction>,
}

impl From<LootEntry> for DynamicLootEntry {
    fn from(entry: LootEntry) -> Self {
        let children = |entries: &[LootEntry]| entries.iter().copied().map(Self::from).collect();
        Self {
            kind: match entry.kind {
                LootEntryKind::Item(item) => DynamicLootEntryKind::Item(item.into()),
                LootEntryKind::Dynamic(name) => DynamicLootEntryKind::Dynamic(name.into()),
                LootEntryKind::Empty => DynamicLootEntryKind::Empty,
                LootEntryKind::Tag { name, expand } => DynamicLootEntryKind::Tag {
                    name: name.into(),
                    expand,
                },
                LootEntryKind::Alternatives(entries) => {
                    DynamicLootEntryKind::Alternatives(children(entries))
                }
                LootEntryKind::Group(entries) => DynamicLootEntryKind::Group(children(entries)),
                LootEntryKind::Sequence(entries) => {
                    DynamicLootEntryKind::Sequence(children(entries))
                }
                LootEntryKind::Table(key) => DynamicLootEntryKind::Table(key.into()),
                LootEntryKind::InlineTable(table) => {
                    DynamicLootEntryKind::InlineTable(Box::new(table.into()))
                }
            },
            weight: entry.weight,
            quality: entry.quality,
            condition: entry.condition.into(),
            functions: entry
                .functions
                .iter()
                .copied()
                .map(DynamicLootFunction::from)
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DynamicLootPool {
    pub entries: Vec<DynamicLootEntry>,
    pub rolls: LootNumberProvider,
    pub bonus_rolls: LootNumberProvider,
    pub condition: DynamicLootCondition,
    pub functions: Vec<DynamicLootFunction>,
}

impl From<LootPool> for DynamicLootPool {
    fn from(pool: LootPool) -> Self {
        Self {
            entries: pool
                .entries
                .iter()
                .copied()
                .map(DynamicLootEntry::from)
                .collect(),
            rolls: pool.rolls,
            bonus_rolls: pool.bonus_rolls,
            condition: pool.condition.into(),
            functions: pool
                .functions
                .iter()
                .copied()
                .map(DynamicLootFunction::from)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DynamicLootTable {
    pub pools: Vec<DynamicLootPool>,
    pub functions: Vec<DynamicLootFunction>,
}

impl From<&LootTable> for DynamicLootTable {
    fn from(table: &LootTable) -> Self {
        Self {
            pools: table
                .pools
                .iter()
                .copied()
                .map(DynamicLootPool::from)
                .collect(),
            functions: table
                .functions
                .iter()
                .copied()
                .map(DynamicLootFunction::from)
                .collect(),
        }
    }
}
