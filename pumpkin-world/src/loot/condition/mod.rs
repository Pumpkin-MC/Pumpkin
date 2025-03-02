use serde::Deserialize;

mod survives_explosion;

#[derive(Deserialize, Clone)]
#[serde(tag = "condition")]
pub enum LootCondition {
    #[serde(rename = "minecraft:inverted")]
    Inverted,
    #[serde(rename = "minecraft:any_of")]
    AnyOf,
    #[serde(rename = "minecraft:all_of")]
    AllOf,
    #[serde(rename = "minecraft:random_chance")]
    RandomChance,
    #[serde(rename = "minecraft:random_chance_with_enchanted_bonus")]
    RandomChanceWithEnchantedBonus,
    #[serde(rename = "minecraft:entity_properties")]
    EntityProperties,
    #[serde(rename = "minecraft:killed_by_player")]
    KilledByPlayer,
    #[serde(rename = "minecraft:entity_scores")]
    EntityScores,
    #[serde(rename = "minecraft:block_state_property")]
    BlockStateProperty,
    #[serde(rename = "minecraft:match_tool")]
    MatchTool,
    #[serde(rename = "minecraft:table_bonus")]
    TableBonus,
    #[serde(rename = "minecraft:survives_explosion")]
    SurvivesExplosion,
    #[serde(rename = "minecraft:damage_source_properties")]
    DamageSourceProperties,
    #[serde(rename = "minecraft:location_check")]
    LocationCheck,
    #[serde(rename = "minecraft:weather_check")]
    WeatherCheck,
    #[serde(rename = "minecraft:reference")]
    Reference,
    #[serde(rename = "minecraft:time_check")]
    TimeCheck,
    #[serde(rename = "minecraft:value_check")]
    ValueCheck,
    #[serde(rename = "minecraft:enchantment_active_check")]
    EnchantmentActiveCheck,
}

#[expect(clippy::match_like_matches_macro)]
impl LootCondition {
    // TODO: This is trash, Make this right
    pub fn test(&self) -> bool {
        match self {
            LootCondition::SurvivesExplosion => true,
            _ => false,
        }
    }
}
