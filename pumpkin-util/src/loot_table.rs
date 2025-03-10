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
pub struct LootPoolEntry {
    pub content: LootPoolEntryTypes,
    pub conditions: Option<&'static [LootCondition]>,
}

#[derive(Clone, Debug)]
pub enum LootTableType {
    /// Nothing will be dropped
    Empty,
    /// A Block will be dropped
    Block,
    /// A Item will be dropped
    Chest,
}
