#[derive(Clone, Debug)]
pub struct LootTable {
    pub r#type: LootTableType,
    pub random_sequence: Option<&'static str>,
    pub pools: Option<&'static [LootPool]>,
}

#[derive(Clone, Debug)]
pub struct LootPool {
    pub entries: &'static [LootPoolEntry],
    pub rolls: f32,
    pub bonus_rolls: f32,
}

#[derive(Clone, Debug)]
pub struct ItemEntry {
    pub name: &'static str,
}

#[derive(Clone, Debug)]
pub struct AlternativeEntry {
    pub children: &'static [LootPoolEntry],
}

#[derive(Clone, Debug)]
pub enum LootPoolEntryTypes {
    Empty,
    Item(ItemEntry),
    LootTable,
    Dynamic,
    Tag,
    Alternatives(AlternativeEntry),
    Sequence,
    Group,
}

#[derive(Clone, Debug)]
pub enum LootCondition {
    Inverted,
    AnyOf,
    AllOf,
    RandomChance,
    RandomChanceWithEnchantedBonus,
    EntityProperties,
    KilledByPlayer,
    EntityScores,
    BlockStateProperty {
        block: &'static str,
        properties: &'static [(&'static str, &'static str)],
    },
    MatchTool,
    TableBonus,
    SurvivesExplosion,
    DamageSourceProperties,
    LocationCheck,
    WeatherCheck,
    Reference,
    TimeCheck,
    ValueCheck,
    EnchantmentActiveCheck,
}

#[derive(Clone, Debug)]
pub struct LootFunction {
    pub content: LootFunctionTypes,
    pub conditions: Option<&'static [LootCondition]>,
}

#[derive(Clone, Debug)]
pub enum LootFunctionTypes {
    SetCount {
        count: LootFunctionNumberProvider,
        add: bool,
    },
    LimitCount {
        min: Option<f32>,
        max: Option<f32>,
    },
    ApplyBonus {
        enchantment: &'static str,
        formula: &'static str,
        parameters: Option<LootFunctionBonusParameter>,
    },
    CopyComponents {
        source: &'static str,
        include: &'static [&'static str],
    },
    CopyState {
        block: &'static str,
        properties: &'static [&'static str],
    },
    ExplosionDecay,
}

#[derive(Clone, Debug)]
pub enum LootFunctionNumberProvider {
    Constant { value: f32 },
    Uniform { min: f32, max: f32 },
    Binomial { n: f32, p: f32 },
}

#[derive(Clone, Debug)]
pub enum LootFunctionBonusParameter {
    Multiplier { bonus_multiplier: i32 },
    Probability { extra: i32, probability: f32 },
}

#[derive(Clone, Debug)]
pub struct LootPoolEntry {
    pub content: LootPoolEntryTypes,
    pub conditions: Option<&'static [LootCondition]>,
    pub functions: Option<&'static [LootFunction]>,
}

#[derive(Clone, Debug)]
pub enum LootTableType {
    /// Nothing will be dropped
    Empty,
    /// A block will be dropped
    Block,
    /// An item will be dropped
    Chest,
}
