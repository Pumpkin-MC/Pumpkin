use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use pumpkin_util::loot_table::LootNumberProviderTypes;
use quote::{ToTokens, quote};
use serde::Deserialize;
use syn::LitStr;

/// These are required to be defined twice because serde can't deseralize into static context for obvious reasons.
#[derive(Deserialize, Clone, Debug)]
pub struct LootTableStruct {
    r#type: LootTableTypeStruct,
    random_sequence: Option<String>,
    pools: Option<Vec<LootPoolStruct>>,
}

impl ToTokens for LootTableStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let loot_table_type = self.r#type.to_token_stream();
        let random_sequence = match &self.random_sequence {
            Some(seq) => quote! { Some(#seq) },
            None => quote! { None },
        };
        let pools = match &self.pools {
            Some(pools) => {
                let pool_tokens: Vec<_> = pools.iter().map(|pool| pool.to_token_stream()).collect();
                quote! { Some(&[#(#pool_tokens),*]) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            LootTable {
                r#type: #loot_table_type,
                random_sequence: #random_sequence,
                pools: #pools,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootPoolStruct {
    entries: Vec<LootPoolEntryStruct>,
    rolls: LootNumberProviderTypes, // TODO
    bonus_rolls: f32,
}

impl ToTokens for LootPoolStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let entries_tokens: Vec<_> = self
            .entries
            .iter()
            .map(|entry| entry.to_token_stream())
            .collect();
        let rolls = &self.rolls;
        let bonus_rolls = &self.bonus_rolls;

        tokens.extend(quote! {
            LootPool {
                entries: &[#(#entries_tokens),*],
                rolls: #rolls,
                bonus_rolls: #bonus_rolls,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ItemEntryStruct {
    name: String,
}

impl ToTokens for ItemEntryStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = LitStr::new(&self.name, Span::call_site());

        tokens.extend(quote! {
            ItemEntry {
                name: #name,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct AlternativeEntryStruct {
    children: Vec<LootPoolEntryStruct>,
}

impl ToTokens for AlternativeEntryStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let children = self.children.iter().map(|entry| entry.to_token_stream());

        tokens.extend(quote! {
            AlternativeEntry {
                children: &[#(#children),*],
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum LootPoolEntryTypesStruct {
    #[serde(rename = "minecraft:empty")]
    Empty,
    #[serde(rename = "minecraft:item")]
    Item(ItemEntryStruct),
    #[serde(rename = "minecraft:loot_table")]
    LootTable,
    #[serde(rename = "minecraft:dynamic")]
    Dynamic,
    #[serde(rename = "minecraft:tag")]
    Tag,
    #[serde(rename = "minecraft:alternatives")]
    Alternatives(AlternativeEntryStruct),
    #[serde(rename = "minecraft:sequence")]
    Sequence,
    #[serde(rename = "minecraft:group")]
    Group,
}

impl ToTokens for LootPoolEntryTypesStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            LootPoolEntryTypesStruct::Empty => {
                tokens.extend(quote! { LootPoolEntryTypes::Empty });
            }
            LootPoolEntryTypesStruct::Item(item) => {
                tokens.extend(quote! { LootPoolEntryTypes::Item(#item) });
            }
            LootPoolEntryTypesStruct::LootTable => {
                tokens.extend(quote! { LootPoolEntryTypes::LootTable });
            }
            LootPoolEntryTypesStruct::Dynamic => {
                tokens.extend(quote! { LootPoolEntryTypes::Dynamic });
            }
            LootPoolEntryTypesStruct::Tag => {
                tokens.extend(quote! { LootPoolEntryTypes::Tag });
            }
            LootPoolEntryTypesStruct::Alternatives(alt) => {
                tokens.extend(quote! { LootPoolEntryTypes::Alternatives(#alt) });
            }
            LootPoolEntryTypesStruct::Sequence => {
                tokens.extend(quote! { LootPoolEntryTypes::Sequence });
            }
            LootPoolEntryTypesStruct::Group => {
                tokens.extend(quote! { LootPoolEntryTypes::Group });
            }
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "condition")]
pub enum LootConditionStruct {
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
    BlockStateProperty {
        block: String,
        properties: HashMap<String, String>,
    },
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

impl ToTokens for LootConditionStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            LootConditionStruct::Inverted => quote! { LootCondition::Inverted },
            LootConditionStruct::AnyOf => quote! { LootCondition::AnyOf },
            LootConditionStruct::AllOf => quote! { LootCondition::AllOf },
            LootConditionStruct::RandomChance => quote! { LootCondition::RandomChance },
            LootConditionStruct::RandomChanceWithEnchantedBonus => {
                quote! { LootCondition::RandomChanceWithEnchantedBonus }
            }
            LootConditionStruct::EntityProperties => quote! { LootCondition::EntityProperties },
            LootConditionStruct::KilledByPlayer => quote! { LootCondition::KilledByPlayer },
            LootConditionStruct::EntityScores => quote! { LootCondition::EntityScores },
            LootConditionStruct::BlockStateProperty { block, properties } => {
                let properties: Vec<_> = properties
                    .iter()
                    .map(|(k, v)| quote! { (#k, #v) })
                    .collect();
                quote! { LootCondition::BlockStateProperty { block: #block, properties: &[#(#properties),*] } }
            }
            LootConditionStruct::MatchTool => quote! { LootCondition::MatchTool },
            LootConditionStruct::TableBonus => quote! { LootCondition::TableBonus },
            LootConditionStruct::SurvivesExplosion => quote! { LootCondition::SurvivesExplosion },
            LootConditionStruct::DamageSourceProperties => {
                quote! { LootCondition::DamageSourceProperties }
            }
            LootConditionStruct::LocationCheck => quote! { LootCondition::LocationCheck },
            LootConditionStruct::WeatherCheck => quote! { LootCondition::WeatherCheck },
            LootConditionStruct::Reference => quote! { LootCondition::Reference },
            LootConditionStruct::TimeCheck => quote! { LootCondition::TimeCheck },
            LootConditionStruct::ValueCheck => quote! { LootCondition::ValueCheck },
            LootConditionStruct::EnchantmentActiveCheck => {
                quote! { LootCondition::EnchantmentActiveCheck }
            }
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootFunctionStruct {
    #[serde(flatten)]
    content: LootFunctionTypesStruct,
    conditions: Option<Vec<LootConditionStruct>>,
}

impl ToTokens for LootFunctionStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let functions_tokens = &self.content.to_token_stream();

        let conditions_tokens = match &self.conditions {
            Some(conds) => {
                let cond_tokens: Vec<_> = conds.iter().map(|c| c.to_token_stream()).collect();
                quote! { Some(&[#(#cond_tokens),*]) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            LootFunction {
                content: #functions_tokens,
                conditions: #conditions_tokens,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "function")]
pub enum LootFunctionTypesStruct {
    #[serde(rename = "minecraft:set_count")]
    SetCount {
        count: LootFunctionNumberProviderStruct,
        add: Option<bool>,
    },
    #[serde(rename = "minecraft:enchanted_count_increase")]
    EnchantedCountIncrease,
    #[serde(rename = "minecraft:furnace_smelt")]
    FurnaceSmelt,
    #[serde(rename = "minecraft:set_potion")]
    SetPotion,
    #[serde(rename = "minecraft:set_ominous_bottle_amplifier")]
    SetOminousBottleAmplifier,
    #[serde(rename = "minecraft:limit_count")]
    LimitCount { limit: LootFunctionLimitCountStruct },
    #[serde(rename = "minecraft:apply_bonus")]
    ApplyBonus {
        enchantment: String,
        formula: String,
        parameters: Option<LootFunctionBonusParameterStruct>,
    },
    #[serde(rename = "minecraft:copy_components")]
    CopyComponents {
        source: String,
        include: Vec<String>,
    },
    #[serde(rename = "minecraft:copy_state")]
    CopyState {
        block: String,
        properties: Vec<String>,
    },
    #[serde(rename = "minecraft:explosion_decay")]
    ExplosionDecay,
}

impl ToTokens for LootFunctionTypesStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            LootFunctionTypesStruct::SetCount { count, add } => {
                let count = count.to_token_stream();
                let add = add.unwrap_or(false);
                quote! { LootFunctionTypes::SetCount { count: #count, add: #add } }
            }
            LootFunctionTypesStruct::SetOminousBottleAmplifier => {
                quote! { LootFunctionTypes::SetOminousBottleAmplifier }
            }
            LootFunctionTypesStruct::FurnaceSmelt => {
                quote! { LootFunctionTypes::FurnaceSmelt }
            }
            LootFunctionTypesStruct::SetPotion => {
                quote! { LootFunctionTypes::SetPotion }
            }
            LootFunctionTypesStruct::EnchantedCountIncrease => {
                quote! { LootFunctionTypes::EnchantedCountIncrease }
            }
            LootFunctionTypesStruct::LimitCount { limit } => {
                let min = match limit.min {
                    Some(min) => quote! { Some(#min) },
                    None => quote! { None },
                };
                let max = match limit.max {
                    Some(max) => quote! { Some(#max) },
                    None => quote! { None },
                };
                quote! { LootFunctionTypes::LimitCount { min: #min, max: #max } }
            }
            LootFunctionTypesStruct::ApplyBonus {
                enchantment,
                formula,
                parameters,
            } => {
                let parameters = match parameters {
                    Some(params) => {
                        let params = params.to_token_stream();
                        quote! { Some(#params) }
                    }
                    None => quote! { None },
                };

                quote! {
                    LootFunctionTypes::ApplyBonus {
                        enchantment: #enchantment,
                        formula: #formula,
                        parameters: #parameters,
                    }
                }
            }
            LootFunctionTypesStruct::CopyComponents { source, include } => {
                quote! {
                    LootFunctionTypes::CopyComponents {
                        source: #source,
                        include: &[#(#include),*],
                    }
                }
            }
            LootFunctionTypesStruct::CopyState { block, properties } => {
                quote! {
                    LootFunctionTypes::CopyState {
                        block: #block,
                        properties: &[#(#properties),*],
                    }
                }
            }
            LootFunctionTypesStruct::ExplosionDecay => {
                quote! { LootFunctionTypes::ExplosionDecay }
            }
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum LootFunctionNumberProviderStruct {
    #[serde(rename = "minecraft:uniform")]
    Uniform { min: f32, max: f32 },
    #[serde(rename = "minecraft:binomial")]
    Binomial { n: f32, p: f32 },
    #[serde(rename = "minecraft:constant", untagged)]
    Constant(f32),
}

impl ToTokens for LootFunctionNumberProviderStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Self::Constant(value) => {
                quote! { LootFunctionNumberProvider::Constant { value: #value } }
            }
            Self::Uniform { min, max } => {
                quote! { LootFunctionNumberProvider::Uniform { min: #min, max: #max } }
            }
            Self::Binomial { n, p } => {
                quote! { LootFunctionNumberProvider::Binomial { n: #n, p: #p } }
            }
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootFunctionLimitCountStruct {
    min: Option<f32>,
    max: Option<f32>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum LootFunctionBonusParameterStruct {
    Multiplier {
        #[serde(rename = "bonusMultiplier")]
        bonus_multiplier: i32,
    },
    Probability {
        extra: i32,
        probability: f32,
    },
}

impl ToTokens for LootFunctionBonusParameterStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Self::Multiplier { bonus_multiplier } => {
                quote! { LootFunctionBonusParameter::Multiplier { bonus_multiplier: #bonus_multiplier } }
            }
            Self::Probability { extra, probability } => {
                quote! { LootFunctionBonusParameter::Probability { extra: #extra, probability: #probability } }
            }
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootPoolEntryStruct {
    #[serde(flatten)]
    content: LootPoolEntryTypesStruct,
    conditions: Option<Vec<LootConditionStruct>>,
    functions: Option<Vec<LootFunctionStruct>>,
}

impl ToTokens for LootPoolEntryStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let content = &self.content;
        let conditions_tokens = match &self.conditions {
            Some(conds) => {
                let cond_tokens: Vec<_> = conds.iter().map(|c| c.to_token_stream()).collect();
                quote! { Some(&[#(#cond_tokens),*]) }
            }
            None => quote! { None },
        };
        let functions_tokens = match &self.functions {
            Some(fns) => {
                let cond_tokens: Vec<_> = fns.iter().map(|c| c.to_token_stream()).collect();
                quote! { Some(&[#(#cond_tokens),*]) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            LootPoolEntry {
                content: #content,
                conditions: #conditions_tokens,
                functions: #functions_tokens,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename = "snake_case")]
pub enum LootTableTypeStruct {
    #[serde(rename = "minecraft:empty")]
    /// Nothing will be dropped.
    Empty,
    #[serde(rename = "minecraft:entity")]
    /// The Entity loot will be dropped.
    Entity,
    #[serde(rename = "minecraft:block")]
    /// A block will be dropped.
    Block,
    #[serde(rename = "minecraft:chest")]
    /// An item will be dropped.
    Chest,
}

impl ToTokens for LootTableTypeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            LootTableTypeStruct::Empty => quote! { LootTableType::Empty },
            LootTableTypeStruct::Entity => quote! { LootTableType::Entity },
            LootTableTypeStruct::Block => quote! { LootTableType::Block },
            LootTableTypeStruct::Chest => quote! { LootTableType::Chest },
        };

        tokens.extend(name);
    }
}
