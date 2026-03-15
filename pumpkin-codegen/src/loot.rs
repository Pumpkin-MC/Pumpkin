use std::collections::BTreeMap;

use proc_macro2::{Span, TokenStream};
use pumpkin_util::loot_table::LootNumberProviderTypes;
use quote::{ToTokens, quote};
use serde::Deserialize;
use syn::LitStr;

/// Deserialized loot table as stored in the asset files.
///
/// These are required to be defined twice because serde can't deserialize into static context for obvious reasons.
#[derive(Deserialize)]
pub struct LootTableStruct {
    /// Category of this loot table (block, entity, chest, etc.).
    r#type: LootTableTypeStruct,
    /// Namespaced random-sequence key used for deterministic rolls, if any.
    random_sequence: Option<String>,
    /// Roll pools contained in this table, if any.
    pools: Option<Vec<LootPoolStruct>>,
}

impl ToTokens for LootTableStruct {
    /// Emits a `LootTable { … }` struct literal token stream for code generation.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let loot_table_type = self.r#type.to_token_stream();
        let random_sequence = if let Some(seq) = &self.random_sequence {
            quote! { Some(#seq) }
        } else {
            quote! { None }
        };
        let pools = if let Some(pools) = &self.pools {
            let pool_tokens: Vec<_> = pools.iter().map(ToTokens::to_token_stream).collect();
            quote! { Some(&[#(#pool_tokens),*]) }
        } else {
            quote! { None }
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

/// Deserialized loot pool describing a group of entries rolled together.
#[derive(Deserialize, Clone, Debug)]
pub struct LootPoolStruct {
    /// Entries that can be selected during a roll of this pool.
    entries: Vec<LootPoolEntryStruct>,
    /// Number of times the pool is rolled.
    rolls: LootNumberProviderTypes, // TODO
    /// Extra rolls granted by luck-related enchantments.
    bonus_rolls: f32,
    /// Conditions that must all pass for this pool to be rolled, if any.
    conditions: Option<Vec<LootConditionStruct>>,
    /// Functions applied to the selected entries, if any.
    functions: Option<Vec<LootFunctionStruct>>,
}

impl ToTokens for LootPoolStruct {
    /// Emits a `LootPool { … }` struct literal token stream for code generation.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let entries_tokens: Vec<_> = self
            .entries
            .iter()
            .map(ToTokens::to_token_stream)
            .collect();
        let rolls = &self.rolls;
        let bonus_rolls = &self.bonus_rolls;
        let conditions_tokens = if let Some(conds) = &self.conditions {
            let cond_tokens: Vec<_> = conds.iter().map(ToTokens::to_token_stream).collect();
            quote! { Some(&[#(#cond_tokens),*]) }
        } else {
            quote! { None }
        };
        let functions_tokens = if let Some(fns) = &self.functions {
            let cond_tokens: Vec<_> = fns.iter().map(ToTokens::to_token_stream).collect();
            quote! { Some(&[#(#cond_tokens),*]) }
        } else {
            quote! { None }
        };

        tokens.extend(quote! {
            LootPool {
                entries: &[#(#entries_tokens),*],
                rolls: #rolls,
                bonus_rolls: #bonus_rolls,
                conditions: #conditions_tokens,
                functions: #functions_tokens,
            }
        });
    }
}

/// Deserialized single-item loot entry holding the item's registry key.
#[derive(Deserialize, Clone, Debug)]
pub struct ItemEntryStruct {
    /// Namespaced item key (e.g., `"minecraft:diamond"`).
    name: String,
}

impl ToTokens for ItemEntryStruct {
    /// Emits an `ItemEntry { … }` struct literal token stream for code generation.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = LitStr::new(&self.name, Span::call_site());

        tokens.extend(quote! {
            ItemEntry {
                name: #name,
            }
        });
    }
}

/// Deserialized alternatives loot entry that tries each child in order until one succeeds.
#[derive(Deserialize, Clone, Debug)]
pub struct AlternativeEntryStruct {
    /// Child entries evaluated sequentially until the first successful one.
    children: Vec<LootPoolEntryStruct>,
}

impl ToTokens for AlternativeEntryStruct {
    /// Emits an `AlternativeEntry { … }` struct literal token stream for code generation.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let children = self.children.iter().map(ToTokens::to_token_stream);

        tokens.extend(quote! {
            AlternativeEntry {
                children: &[#(#children),*],
            }
        });
    }
}

/// Deserialized variant of a loot pool entry, tagged by the `"type"` field.
#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum LootPoolEntryTypesStruct {
    /// Yields nothing.
    #[serde(rename = "minecraft:empty")]
    Empty,
    /// Yields a specific item.
    #[serde(rename = "minecraft:item")]
    Item(ItemEntryStruct),
    /// References another loot table.
    #[serde(rename = "minecraft:loot_table")]
    LootTable,
    /// Yields dynamically determined drops (e.g., shulker box contents).
    #[serde(rename = "minecraft:dynamic")]
    Dynamic,
    /// Yields all items in a tag.
    #[serde(rename = "minecraft:tag")]
    Tag,
    /// Tries each child entry until one succeeds.
    #[serde(rename = "minecraft:alternatives")]
    Alternatives(AlternativeEntryStruct),
    /// Evaluates all children in order, stopping on the first failure.
    #[serde(rename = "minecraft:sequence")]
    Sequence,
    /// Evaluates all children regardless of individual success.
    #[serde(rename = "minecraft:group")]
    Group,
}

impl ToTokens for LootPoolEntryTypesStruct {
    /// Emits the matching `LootPoolEntryTypes::*` token stream for code generation.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Empty => {
                tokens.extend(quote! { LootPoolEntryTypes::Empty });
            }
            Self::Item(item) => {
                tokens.extend(quote! { LootPoolEntryTypes::Item(#item) });
            }
            Self::LootTable => {
                tokens.extend(quote! { LootPoolEntryTypes::LootTable });
            }
            Self::Dynamic => {
                tokens.extend(quote! { LootPoolEntryTypes::Dynamic });
            }
            Self::Tag => {
                tokens.extend(quote! { LootPoolEntryTypes::Tag });
            }
            Self::Alternatives(alt) => {
                tokens.extend(quote! { LootPoolEntryTypes::Alternatives(#alt) });
            }
            Self::Sequence => {
                tokens.extend(quote! { LootPoolEntryTypes::Sequence });
            }
            Self::Group => {
                tokens.extend(quote! { LootPoolEntryTypes::Group });
            }
        }
    }
}

/// Deserialized loot condition, tagged by the `"condition"` field.
#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "condition")]
pub enum LootConditionStruct {
    /// Passes if the wrapped condition fails.
    #[serde(rename = "minecraft:inverted")]
    Inverted,
    /// Passes if any of the child conditions pass.
    #[serde(rename = "minecraft:any_of")]
    AnyOf,
    /// Passes if all child conditions pass.
    #[serde(rename = "minecraft:all_of")]
    AllOf,
    /// Passes with the given probability.
    #[serde(rename = "minecraft:random_chance")]
    RandomChance,
    /// Passes with probability scaled by an enchantment level.
    #[serde(rename = "minecraft:random_chance_with_enchanted_bonus")]
    RandomChanceWithEnchantedBonus,
    /// Passes based on entity NBT predicates.
    #[serde(rename = "minecraft:entity_properties")]
    EntityProperties,
    /// Passes if the block was killed by a player.
    #[serde(rename = "minecraft:killed_by_player")]
    KilledByPlayer,
    /// Passes based on entity scoreboard values.
    #[serde(rename = "minecraft:entity_scores")]
    EntityScores,
    /// Passes if the source block has the specified block-state properties.
    #[serde(rename = "minecraft:block_state_property")]
    BlockStateProperty {
        /// Namespaced block key to match.
        block: String,
        /// Required block-state property key-value pairs.
        properties: BTreeMap<String, String>,
    },
    /// Passes if the tool matches an item predicate.
    #[serde(rename = "minecraft:match_tool")]
    MatchTool,
    /// Passes with probability based on an enchantment's level.
    #[serde(rename = "minecraft:table_bonus")]
    TableBonus,
    /// Passes if the item survives an explosion.
    #[serde(rename = "minecraft:survives_explosion")]
    SurvivesExplosion,
    /// Passes based on the damage source's properties.
    #[serde(rename = "minecraft:damage_source_properties")]
    DamageSourceProperties,
    /// Passes based on the block's location.
    #[serde(rename = "minecraft:location_check")]
    LocationCheck,
    /// Passes based on current weather conditions.
    #[serde(rename = "minecraft:weather_check")]
    WeatherCheck,
    /// References an external predicate by ID.
    #[serde(rename = "minecraft:reference")]
    Reference,
    /// Passes based on the current in-game time.
    #[serde(rename = "minecraft:time_check")]
    TimeCheck,
    /// Passes based on a numeric value range check.
    #[serde(rename = "minecraft:value_check")]
    ValueCheck,
    /// Passes if an enchantment is currently active.
    #[serde(rename = "minecraft:enchantment_active_check")]
    EnchantmentActiveCheck,
}

impl ToTokens for LootConditionStruct {
    /// Emits the matching `LootCondition::*` token stream for code generation.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Self::Inverted => quote! { LootCondition::Inverted },
            Self::AnyOf => quote! { LootCondition::AnyOf },
            Self::AllOf => quote! { LootCondition::AllOf },
            Self::RandomChance => quote! { LootCondition::RandomChance },
            Self::RandomChanceWithEnchantedBonus => {
                quote! { LootCondition::RandomChanceWithEnchantedBonus }
            }
            Self::EntityProperties => quote! { LootCondition::EntityProperties },
            Self::KilledByPlayer => quote! { LootCondition::KilledByPlayer },
            Self::EntityScores => quote! { LootCondition::EntityScores },
            Self::BlockStateProperty { block, properties } => {
                let properties: Vec<_> = properties
                    .iter()
                    .map(|(k, v)| quote! { (#k, #v) })
                    .collect();
                quote! { LootCondition::BlockStateProperty { block: #block, properties: &[#(#properties),*] } }
            }
            Self::MatchTool => quote! { LootCondition::MatchTool },
            Self::TableBonus => quote! { LootCondition::TableBonus },
            Self::SurvivesExplosion => quote! { LootCondition::SurvivesExplosion },
            Self::DamageSourceProperties => {
                quote! { LootCondition::DamageSourceProperties }
            }
            Self::LocationCheck => quote! { LootCondition::LocationCheck },
            Self::WeatherCheck => quote! { LootCondition::WeatherCheck },
            Self::Reference => quote! { LootCondition::Reference },
            Self::TimeCheck => quote! { LootCondition::TimeCheck },
            Self::ValueCheck => quote! { LootCondition::ValueCheck },
            Self::EnchantmentActiveCheck => {
                quote! { LootCondition::EnchantmentActiveCheck }
            }
        };

        tokens.extend(name);
    }
}

/// Deserialized loot function wrapper combining a function type with optional conditions.
#[derive(Deserialize, Clone, Debug)]
pub struct LootFunctionStruct {
    /// The concrete function to apply (e.g., set count, apply bonus).
    #[serde(flatten)]
    content: LootFunctionTypesStruct,
    /// Conditions that must all pass for this function to be applied, if any.
    conditions: Option<Vec<LootConditionStruct>>,
}

impl ToTokens for LootFunctionStruct {
    /// Emits a `LootFunction { … }` struct literal token stream for code generation.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let functions_tokens = &self.content.to_token_stream();

        let conditions_tokens = if let Some(conds) = &self.conditions {
            let cond_tokens: Vec<_> = conds.iter().map(ToTokens::to_token_stream).collect();
            quote! { Some(&[#(#cond_tokens),*]) }
        } else {
            quote! { None }
        };

        tokens.extend(quote! {
            LootFunction {
                content: #functions_tokens,
                conditions: #conditions_tokens,
            }
        });
    }
}

/// Deserialized loot function variant, tagged by the `"function"` field.
#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "function")]
pub enum LootFunctionTypesStruct {
    /// Sets the stack count using a number provider.
    #[serde(rename = "minecraft:set_count")]
    SetCount {
        /// Number provider determining the new stack size.
        count: LootFunctionNumberProviderStruct,
        /// If `true`, adds to the existing count instead of replacing it.
        add: Option<bool>,
    },
    /// Increases count based on the level of a relevant enchantment.
    #[serde(rename = "minecraft:enchanted_count_increase")]
    EnchantedCountIncrease,
    /// Smelts the item as if processed in a furnace.
    #[serde(rename = "minecraft:furnace_smelt")]
    FurnaceSmelt,
    /// Sets the potion type on the item.
    #[serde(rename = "minecraft:set_potion")]
    SetPotion,
    /// Sets the amplifier on an ominous bottle item.
    #[serde(rename = "minecraft:set_ominous_bottle_amplifier")]
    SetOminousBottleAmplifier,
    /// Clamps the stack count to a min/max range.
    #[serde(rename = "minecraft:limit_count")]
    LimitCount {
        /// The min/max bounds to clamp the count to.
        limit: LootFunctionLimitCountStruct,
    },
    /// Applies an enchantment bonus using a named formula.
    #[serde(rename = "minecraft:apply_bonus")]
    ApplyBonus {
        /// Namespaced enchantment ID whose level feeds the formula.
        enchantment: String,
        /// Name of the bonus formula to apply.
        formula: String,
        /// Optional numeric parameters for the chosen formula.
        parameters: Option<LootFunctionBonusParameterStruct>,
    },
    /// Copies data components from the block entity source to the dropped item.
    #[serde(rename = "minecraft:copy_components")]
    CopyComponents {
        /// The source to copy components from (e.g., `"block_entity"`).
        source: String,
        /// List of component keys to copy.
        include: Vec<String>,
    },
    /// Copies specified block-state properties onto the item's `block_state` component.
    #[serde(rename = "minecraft:copy_state")]
    CopyState {
        /// Namespaced block whose state properties are copied.
        block: String,
        /// Names of the block-state properties to copy.
        properties: Vec<String>,
    },
    /// Randomly removes items from the stack to simulate explosion damage.
    #[serde(rename = "minecraft:explosion_decay")]
    ExplosionDecay,
}

impl ToTokens for LootFunctionTypesStruct {
    /// Emits the matching `LootFunctionTypes::*` token stream for code generation.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Self::SetCount { count, add } => {
                let count = count.to_token_stream();
                let add = add.unwrap_or(false);
                quote! { LootFunctionTypes::SetCount { count: #count, add: #add } }
            }
            Self::SetOminousBottleAmplifier => {
                quote! { LootFunctionTypes::SetOminousBottleAmplifier }
            }
            Self::FurnaceSmelt => {
                quote! { LootFunctionTypes::FurnaceSmelt }
            }
            Self::SetPotion => {
                quote! { LootFunctionTypes::SetPotion }
            }
            Self::EnchantedCountIncrease => {
                quote! { LootFunctionTypes::EnchantedCountIncrease }
            }
            Self::LimitCount { limit } => {
                let min = if let Some(min) = limit.min {
                    quote! { Some(#min) }
                } else {
                    quote! { None }
                };
                let max = if let Some(max) = limit.max {
                    quote! { Some(#max) }
                } else {
                    quote! { None }
                };
                quote! { LootFunctionTypes::LimitCount { min: #min, max: #max } }
            }
            Self::ApplyBonus {
                enchantment,
                formula,
                parameters,
            } => {
                let parameters = if let Some(params) = parameters {
                    let params = params.to_token_stream();
                    quote! { Some(#params) }
                } else {
                    quote! { None }
                };

                quote! {
                    LootFunctionTypes::ApplyBonus {
                        enchantment: #enchantment,
                        formula: #formula,
                        parameters: #parameters,
                    }
                }
            }
            Self::CopyComponents { source, include } => {
                quote! {
                    LootFunctionTypes::CopyComponents {
                        source: #source,
                        include: &[#(#include),*],
                    }
                }
            }
            Self::CopyState { block, properties } => {
                quote! {
                    LootFunctionTypes::CopyState {
                        block: #block,
                        properties: &[#(#properties),*],
                    }
                }
            }
            Self::ExplosionDecay => {
                quote! { LootFunctionTypes::ExplosionDecay }
            }
        };

        tokens.extend(name);
    }
}

/// Deserialized number provider for loot function count values.
#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum LootFunctionNumberProviderStruct {
    /// Draws a uniformly random value between `min` and `max` (inclusive).
    #[serde(rename = "minecraft:uniform")]
    Uniform {
        /// Lower bound of the uniform range.
        min: f32,
        /// Upper bound of the uniform range.
        max: f32,
    },
    /// Draws from a binomial distribution with `n` trials and success probability `p`.
    #[serde(rename = "minecraft:binomial")]
    Binomial {
        /// Number of trials.
        n: f32,
        /// Probability of success per trial.
        p: f32,
    },
    /// Always returns the fixed value.
    #[serde(rename = "minecraft:constant", untagged)]
    Constant(f32),
}

impl ToTokens for LootFunctionNumberProviderStruct {
    /// Emits the matching `LootFunctionNumberProvider::*` token stream for code generation.
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

/// Deserialized min/max bounds used by the `LimitCount` loot function.
#[derive(Deserialize, Clone, Debug)]
pub struct LootFunctionLimitCountStruct {
    /// Inclusive lower bound; count will not go below this value.
    min: Option<f32>,
    /// Inclusive upper bound; count will not exceed this value.
    max: Option<f32>,
}

/// Deserialized bonus parameters for the `ApplyBonus` loot function.
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum LootFunctionBonusParameterStruct {
    /// A flat multiplier applied to the enchantment level.
    Multiplier {
        /// The bonus multiplier value.
        #[serde(rename = "bonusMultiplier")]
        bonus_multiplier: i32,
    },
    /// Probability-based bonus using extra attempts per enchantment level.
    Probability {
        /// Extra drop attempts added per enchantment level.
        extra: i32,
        /// Per-attempt success probability.
        probability: f32,
    },
}

impl ToTokens for LootFunctionBonusParameterStruct {
    /// Emits the matching `LootFunctionBonusParameter::*` token stream for code generation.
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

/// Deserialized loot pool entry combining an entry type with optional conditions and functions.
#[derive(Deserialize, Clone, Debug)]
pub struct LootPoolEntryStruct {
    /// The concrete entry type (item, alternatives, etc.).
    #[serde(flatten)]
    content: LootPoolEntryTypesStruct,
    /// Conditions that must all pass for this entry to be evaluated.
    conditions: Option<Vec<LootConditionStruct>>,
    /// Functions applied to the item if this entry is selected.
    functions: Option<Vec<LootFunctionStruct>>,
}

impl ToTokens for LootPoolEntryStruct {
    /// Emits a `LootPoolEntry { … }` struct literal token stream for code generation.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let content = &self.content;
        let conditions_tokens = if let Some(conds) = &self.conditions {
            let cond_tokens: Vec<_> = conds.iter().map(ToTokens::to_token_stream).collect();
            quote! { Some(&[#(#cond_tokens),*]) }
        } else {
            quote! { None }
        };
        let functions_tokens = if let Some(fns) = &self.functions {
            let cond_tokens: Vec<_> = fns.iter().map(ToTokens::to_token_stream).collect();
            quote! { Some(&[#(#cond_tokens),*]) }
        } else {
            quote! { None }
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

/// Deserialized loot table category, tagged by its `"type"` field.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename = "snake_case")]
pub enum LootTableTypeStruct {
    /// Nothing will be dropped.
    #[serde(rename = "minecraft:empty")]
    Empty,
    /// Entity loot will be dropped.
    #[serde(rename = "minecraft:entity")]
    Entity,
    /// Block drops will be generated.
    #[serde(rename = "minecraft:block")]
    Block,
    /// Chest loot will be generated.
    #[serde(rename = "minecraft:chest")]
    Chest,
}

impl ToTokens for LootTableTypeStruct {
    /// Emits the matching `LootTableType::*` token stream for code generation.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Self::Empty => quote! { LootTableType::Empty },
            Self::Entity => quote! { LootTableType::Entity },
            Self::Block => quote! { LootTableType::Block },
            Self::Chest => quote! { LootTableType::Chest },
        };

        tokens.extend(name);
    }
}
