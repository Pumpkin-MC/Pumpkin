use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use serde::Deserialize;

use crate::random::RandomImpl;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LootTable {
    pub r#type: LootTableType,
    pub random_sequence: Option<&'static str>,
    pub pools: Option<&'static [LootPool]>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LootPool {
    pub entries: &'static [LootPoolEntry],
    pub rolls: LootNumberProviderTypes,
    pub bonus_rolls: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ItemEntry {
    pub name: &'static str,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AlternativeEntry {
    pub children: &'static [LootPoolEntry],
}

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
pub struct LootFunction {
    pub content: LootFunctionTypes,
    pub conditions: Option<&'static [LootCondition]>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LootFunctionTypes {
    SetCount {
        count: LootFunctionNumberProvider,
        add: bool,
    },
    EnchantedCountIncrease,
    FurnaceSmelt,
    SetPotion,
    SetOminousBottleAmplifier,
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

#[derive(Clone, PartialEq, Debug)]
pub enum LootFunctionNumberProvider {
    Constant { value: f32 },
    Uniform { min: f32, max: f32 },
    Binomial { n: f32, p: f32 },
}

#[derive(Clone, PartialEq, Debug)]
pub enum LootFunctionBonusParameter {
    Multiplier { bonus_multiplier: i32 },
    Probability { extra: i32, probability: f32 },
}

#[derive(Clone, PartialEq, Debug)]
pub struct LootPoolEntry {
    pub content: LootPoolEntryTypes,
    pub conditions: Option<&'static [LootCondition]>,
    pub functions: Option<&'static [LootFunction]>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LootTableType {
    Empty,
    Entity,
    Block,
    Chest,
}

#[derive(Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(tag = "type")]
pub enum LootNumberProviderTypesProvider {
    #[serde(rename = "minecraft:uniform")]
    Uniform(UniformLootNumberProvider),
}
impl ToTokens for LootNumberProviderTypesProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            LootNumberProviderTypesProvider::Uniform(uniform) => {
                tokens.extend(quote! {
                    LootNumberProviderTypesProvider::Uniform(#uniform)
                });
            }
        }
    }
}

#[derive(Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct UniformLootNumberProvider {
    pub min: f32,
    pub max: f32,
}

impl ToTokens for UniformLootNumberProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min_inclusive = self.min;
        let max_inclusive = self.max;

        tokens.extend(quote! {
            UniformLootNumberProvider { min: #min_inclusive, max: #max_inclusive }
        });
    }
}

impl UniformLootNumberProvider {
    pub fn get_min(&self) -> f32 {
        self.min
    }

    pub fn get(&self, random: &mut impl RandomImpl) -> f32 {
        // TODO
        random.next_f32()
    }

    pub fn get_max(&self) -> f32 {
        self.max
    }
}

#[derive(Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(untagged)]
pub enum LootNumberProviderTypes {
    Object(LootNumberProviderTypesProvider),
    Constant(f32),
}

impl ToTokens for LootNumberProviderTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            LootNumberProviderTypes::Object(provider) => {
                tokens.extend(quote! {
                    LootNumberProviderTypes::Object(#provider)
                });
            }
            LootNumberProviderTypes::Constant(i) => tokens.extend(quote! {
                LootNumberProviderTypes::Constant(#i)
            }),
        }
    }
}

impl LootNumberProviderTypes {
    pub fn get_min(&self) -> f32 {
        match self {
            LootNumberProviderTypes::Object(int_provider) => match int_provider {
                LootNumberProviderTypesProvider::Uniform(uniform) => uniform.get_min(),
            },
            LootNumberProviderTypes::Constant(i) => *i,
        }
    }

    pub fn get(&self, random: &mut impl RandomImpl) -> f32 {
        match self {
            LootNumberProviderTypes::Object(int_provider) => match int_provider {
                LootNumberProviderTypesProvider::Uniform(uniform) => uniform.get(random),
            },
            LootNumberProviderTypes::Constant(i) => *i,
        }
    }

    pub fn get_max(&self) -> f32 {
        match self {
            LootNumberProviderTypes::Object(int_provider) => match int_provider {
                LootNumberProviderTypesProvider::Uniform(uniform) => uniform.get_max(),
            },
            LootNumberProviderTypes::Constant(i) => *i,
        }
    }
}
