use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use serde::Deserialize;

use crate::random::RandomImpl;

/// Represents a loot table used to generate items or rewards.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LootTable {
    /// The type of loot table (e.g., `Entity`, `Block`, `Chest`).
    pub r#type: LootTableType,
    /// Optional random sequence identifier for deterministic rolls.
    pub random_sequence: Option<&'static str>,
    /// Optional array of loot pools that define item generation.
    pub pools: Option<&'static [LootPool]>,
}

/// Defines a loot pool containing entries, rolls, and conditions.
#[derive(Clone, PartialEq, Debug)]
pub struct LootPool {
    /// Entries contained in this loot pool.
    pub entries: &'static [LootPoolEntry],
    /// Number of rolls, specified using a number provider.
    pub rolls: LootNumberProviderTypes,
    /// Additional bonus rolls to apply.
    pub bonus_rolls: f32,
    /// Optional conditions that must be met for this pool to be applied.
    pub conditions: Option<&'static [LootCondition]>,
    /// Optional functions applied to each entry in the pool.
    pub functions: Option<&'static [LootFunction]>,
}

/// Represents an individual item entry in a loot pool.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ItemEntry {
    /// Name of the item.
    pub name: &'static str,
}

/// Represents an alternative loot entry, which chooses one of its children.
#[derive(Clone, PartialEq, Debug)]
pub struct AlternativeEntry {
    /// Child entries for this alternative.
    pub children: &'static [LootPoolEntry],
}

/// Types of entries that can exist in a loot pool.
#[derive(Clone, PartialEq, Debug)]
pub enum LootPoolEntryTypes {
    /// Empty entry; no item is generated.
    Empty,
    /// Entry that generates a specific item.
    Item(ItemEntry),
    /// Entry that references another loot table.
    LootTable,
    /// Dynamic entry, resolved at runtime.
    Dynamic,
    /// Entry that references a tag.
    Tag,
    /// Entry that provides alternatives.
    Alternatives(AlternativeEntry),
    /// Entry that executes sequentially.
    Sequence,
    /// Entry that groups multiple entries.
    Group,
}

/// Conditions that can modify whether loot entries are applied.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LootCondition {
    /// Inverts the result of another condition.
    Inverted,
    /// Passes if any of the given conditions are met.
    AnyOf,
    /// Passes only if all the given conditions are met.
    AllOf,
    /// Passes based on a random chance.
    RandomChance,
    /// Passes based on a random chance modified by item enchantments.
    RandomChanceWithEnchantedBonus,
    /// Checks properties of the entity (e.g. type, attributes).
    EntityProperties,
    /// Requires the entity to have been killed by a player.
    KilledByPlayer,
    /// Checks entity scores against specified criteria.
    EntityScores,
    /// Checks block state properties.
    BlockStateProperty {
        /// The block to check.
        block: &'static str,
        /// Key-value pairs of properties and their expected values.
        properties: &'static [(&'static str, &'static str)],
    },
    /// Requires a specific tool to match certain criteria.
    MatchTool,
    /// Applies a bonus based on a table.
    TableBonus,
    /// Survives explosion damage.
    SurvivesExplosion,
    /// Checks properties of the damage source.
    DamageSourceProperties,
    /// Checks the location of the entity.
    LocationCheck,
    /// Checks weather conditions.
    WeatherCheck,
    /// References to another condition or table.
    Reference,
    /// Checks the in-game time.
    TimeCheck,
    /// Checks against a specific value or range.
    ValueCheck,
    /// Checks if a specific enchantment is active.
    EnchantmentActiveCheck,
}

/// Functions applied to loot entries.
#[derive(Clone, PartialEq, Debug)]
pub struct LootFunction {
    /// The type of function to apply.
    pub content: LootFunctionTypes,
    /// Optional conditions for the function.
    pub conditions: Option<&'static [LootCondition]>,
}

/// Specific types of loot functions.
#[derive(Clone, PartialEq, Debug)]
pub enum LootFunctionTypes {
    /// Sets the number of items in the stack, optionally adding to an existing count.
    SetCount {
        count: LootFunctionNumberProvider,
        add: bool,
    },
    /// Increases the count of items based on the item's enchantments.
    EnchantedCountIncrease,
    /// Smelts the item as if it passed through a furnace.
    FurnaceSmelt,
    /// Sets the potion type for potion items.
    SetPotion,
    /// Sets the amplifier for ominous bottles.
    SetOminousBottleAmplifier,
    /// Limits the count of items to a minimum and/or maximum value.
    LimitCount { min: Option<f32>, max: Option<f32> },
    /// Applies a bonus based on an enchantment and a formula, optionally with extra parameters.
    ApplyBonus {
        enchantment: &'static str,
        formula: &'static str,
        parameters: Option<LootFunctionBonusParameter>,
    },
    /// Copies specified components from a source entity or item.
    CopyComponents {
        source: &'static str,
        include: &'static [&'static str],
    },
    /// Copies state or properties from a block.
    CopyState {
        block: &'static str,
        properties: &'static [&'static str],
    },
    /// Applies decay to items in case of an explosion.
    ExplosionDecay,
}

/// Numeric providers for loot function counts.
#[derive(Clone, PartialEq, Debug)]
pub enum LootFunctionNumberProvider {
    /// Constant value.
    Constant { value: f32 },
    /// Uniform random value between min and max.
    Uniform { min: f32, max: f32 },
    /// Binomial distribution.
    Binomial { n: f32, p: f32 },
}

/// Parameters for loot bonus functions.
#[derive(Clone, PartialEq, Debug)]
pub enum LootFunctionBonusParameter {
    /// Multiplier applied to bonus rolls.
    Multiplier { bonus_multiplier: i32 },
    /// Probability-based bonus application.
    Probability { extra: i32, probability: f32 },
}

/// Single entry in a loot pool with optional conditions and functions.
#[derive(Clone, PartialEq, Debug)]
pub struct LootPoolEntry {
    /// The type of entry.
    pub content: LootPoolEntryTypes,
    /// Optional conditions for this entry.
    pub conditions: Option<&'static [LootCondition]>,
    /// Optional functions for this entry.
    pub functions: Option<&'static [LootFunction]>,
}

/// Categories of loot tables.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LootTableType {
    /// Empty loot table.
    Empty,
    /// Entity loot table.
    Entity,
    /// Block loot table.
    Block,
    /// Chest loot table.
    Chest,
}

/// Represents structured number providers used in loot rolls.
#[derive(Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(tag = "type")]
pub enum LootNumberProviderTypesProvider {
    /// Uniformly distributes a random number between `min` and `max` (inclusive).
    #[serde(rename = "minecraft:uniform")]
    Uniform(UniformLootNumberProvider),
}

impl ToTokens for LootNumberProviderTypesProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Uniform(uniform) => {
                tokens.extend(quote! {
                    LootNumberProviderTypesProvider::Uniform(#uniform)
                });
            }
        }
    }
}

/// Uniform numeric provider for loot rolls.
#[derive(Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct UniformLootNumberProvider {
    /// Minimum value (inclusive).
    pub min: f32,
    /// Maximum value (inclusive).
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
    /// Generates a random number within the uniform range.
    ///
    /// # Parameters
    /// - `random`: A random number generator implementing `RandomImpl`.
    ///
    /// # Returns
    /// A `f32` between `min` and `max`.
    pub fn get(&self, random: &mut impl RandomImpl) -> f32 {
        random.next_f32().mul_add(self.max - self.min, self.min)
    }
}

/// Numeric providers for loot table rolls (either object or constant).
#[derive(Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(untagged)]
pub enum LootNumberProviderTypes {
    /// A structured provider object (e.g., uniform).
    Object(LootNumberProviderTypesProvider),
    /// Constant numeric value.
    Constant(f32),
}

impl ToTokens for LootNumberProviderTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Object(provider) => {
                tokens.extend(quote! {
                    LootNumberProviderTypes::Object(#provider)
                });
            }
            Self::Constant(i) => tokens.extend(quote! {
                LootNumberProviderTypes::Constant(#i)
            }),
        }
    }
}

impl LootNumberProviderTypes {
    /// Computes a numeric value from this provider.
    ///
    /// # Parameters
    /// - `random`: A random number generator implementing `RandomImpl`.
    ///
    /// # Returns
    /// A `f32` value as determined by the provider (constant or computed).
    pub fn get(&self, random: &mut impl RandomImpl) -> f32 {
        match self {
            Self::Object(int_provider) => match int_provider {
                LootNumberProviderTypesProvider::Uniform(uniform) => uniform.get(random),
            },
            Self::Constant(i) => *i,
        }
    }
}
