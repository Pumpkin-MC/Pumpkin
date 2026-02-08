use pumpkin_data::{Block, BlockState, Enchantment, item::Item};
use pumpkin_data::recipes::{RECIPES_COOKING, CookingRecipeType};
use pumpkin_util::{
    loot_table::{
        LootCondition, LootFunctionBonusParameter, LootFunctionNumberProvider, LootFunctionTypes,
        LootPoolEntry, LootPoolEntryTypes, LootTable,
    },
    random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use pumpkin_world::item::ItemStack;
use rand::RngExt;

#[derive(Default)]
pub struct LootContextParameters {
    pub explosion_radius: Option<f32>,
    pub block_state: Option<&'static BlockState>,
    pub killed_by_player: Option<bool>,
    /// The tool used (for enchantment-based loot functions like `ApplyBonus`).
    /// Callers should set this to the player's held item when breaking blocks.
    pub tool: Option<ItemStack>,
}

pub trait LootTableExt {
    fn get_loot(&self, params: LootContextParameters) -> Vec<ItemStack>;
}

impl LootTableExt for LootTable {
    fn get_loot(&self, params: LootContextParameters) -> Vec<ItemStack> {
        let mut stacks = Vec::new();
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        if let Some(pools) = self.pools {
            for pool in pools {
                if let Some(conditions) = pool.conditions
                    && !conditions.iter().all(|cond| cond.is_fulfilled(&params))
                {
                    continue;
                }

                let rolls = pool.rolls.get(&mut random).round() as i32;

                for _ in 0..rolls {
                    let mut total_weight = 0;
                    let mut valid_entries = Vec::new();

                    for entry in pool.entries {
                        if entry
                            .conditions
                            .as_ref()
                            .is_none_or(|c| c.iter().all(|cond| cond.is_fulfilled(&params)))
                        {
                            let w = 1; // TODO: weight
                            total_weight += w;
                            valid_entries.push((entry, w));
                        }
                    }

                    if total_weight == 0 || valid_entries.is_empty() {
                        continue;
                    }

                    let mut r = random.next_bounded_i32(total_weight);

                    for (entry, weight) in valid_entries {
                        r -= weight;
                        if r < 0 {
                            if let Some(loot) = entry.get_loot(&params) {
                                for stack in loot {
                                    if stack.item_count > 0 {
                                        stacks.push(stack);
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
        stacks
    }
}

trait LootPoolEntryExt {
    fn get_loot(&self, params: &LootContextParameters) -> Option<Vec<ItemStack>>;
}

impl LootPoolEntryExt for LootPoolEntry {
    fn get_loot(&self, params: &LootContextParameters) -> Option<Vec<ItemStack>> {
        if let Some(conditions) = self.conditions
            && !conditions.iter().all(|cond| cond.is_fulfilled(params))
        {
            return None;
        }

        let mut stacks = self.content.get_stacks(params);

        if let Some(functions) = self.functions {
            for function in functions {
                if let Some(conditions) = function.conditions
                    && !conditions.iter().all(|cond| cond.is_fulfilled(params))
                {
                    continue;
                }
                apply_loot_function(&function.content, &mut stacks, params);
            }
        }

        Some(stacks)
    }
}

fn apply_loot_function(
    function: &LootFunctionTypes,
    stacks: &mut Vec<ItemStack>,
    params: &LootContextParameters,
) {
    match function {
        LootFunctionTypes::SetCount { count, add } => {
            for stack in stacks {
                if *add {
                    stack.item_count += count.generate().round() as u8;
                } else {
                    stack.item_count = count.generate().round() as u8;
                }
            }
        }
        LootFunctionTypes::LimitCount { min, max } => {
            if let Some(min) = min.map(|min| min.round() as u8) {
                for stack in &mut *stacks {
                    if stack.item_count < min {
                        stack.item_count = min;
                    }
                }
            }

            if let Some(max) = max.map(|max| max.round() as u8) {
                for stack in &mut *stacks {
                    if stack.item_count > max {
                        stack.item_count = max;
                    }
                }
            }
        }
        LootFunctionTypes::ExplosionDecay => {
            if let Some(radius) = params.explosion_radius {
                for stack in stacks {
                    // Each item in the stack has a 1/radius chance of surviving
                    let mut surviving = 0u8;
                    for _ in 0..stack.item_count {
                        if rand::rng().random::<f32>() <= 1.0 / radius {
                            surviving += 1;
                        }
                    }
                    stack.item_count = surviving;
                }
            }
        }
        LootFunctionTypes::ApplyBonus {
            enchantment,
            formula,
            parameters,
        } => {
            apply_bonus(stacks, params, enchantment, formula, parameters.as_ref());
        }
        LootFunctionTypes::FurnaceSmelt => {
            for stack in stacks {
                // Look up smelting recipe matching this item
                for recipe in RECIPES_COOKING {
                    if let CookingRecipeType::Smelting(cooking) = recipe
                        && cooking.ingredient.match_item(stack.item)
                    {
                        let key = cooking
                            .result
                            .id
                            .strip_prefix("minecraft:")
                            .unwrap_or(cooking.result.id);
                        if let Some(result_item) = Item::from_registry_key(key) {
                            stack.item = result_item;
                        }
                        break;
                    }
                }
            }
        }
        // These functions need data that the codegen doesn't yet provide,
        // or require component system support not available.
        LootFunctionTypes::CopyComponents {
            source: _,
            include: _,
        }
        | LootFunctionTypes::CopyState {
            block: _,
            properties: _,
        }
        | LootFunctionTypes::EnchantedCountIncrease
        | LootFunctionTypes::SetOminousBottleAmplifier
        | LootFunctionTypes::SetPotion => {}
    }
}

fn apply_bonus(
    stacks: &mut [ItemStack],
    params: &LootContextParameters,
    enchantment: &str,
    formula: &str,
    parameters: Option<&LootFunctionBonusParameter>,
) {
    let level = params
        .tool
        .as_ref()
        .and_then(|tool| {
            let key = enchantment
                .strip_prefix("minecraft:")
                .unwrap_or(enchantment);
            Enchantment::from_name(key).map(|ench| tool.get_enchantment_level(ench))
        })
        .unwrap_or(0);

    if level <= 0 {
        return;
    }

    for stack in stacks {
        match formula {
            "minecraft:uniform_bonus_count" => {
                if let Some(LootFunctionBonusParameter::Multiplier { bonus_multiplier }) =
                    parameters
                {
                    // count + random(0..=level*multiplier)
                    let max = level * bonus_multiplier;
                    if max > 0 {
                        let bonus = rand::rng().random_range(0..=(max as u8));
                        stack.item_count = stack.item_count.saturating_add(bonus);
                    }
                }
            }
            "minecraft:binomial_with_bonus_count" => {
                if let Some(LootFunctionBonusParameter::Probability { extra, probability }) =
                    parameters
                {
                    // Binomial(n=extra+level, p=probability)
                    let n = *extra + level;
                    let mut count = 0u8;
                    for _ in 0..n {
                        if rand::rng().random_bool(f64::from(*probability)) {
                            count += 1;
                        }
                    }
                    stack.item_count = count;
                }
            }
            "minecraft:ore_drops" => {
                // count * max(1, random(0..=level+1))
                let roll = rand::rng().random_range(0..=(level + 1));
                let multiplier = roll.max(1) as u8;
                stack.item_count = stack.item_count.saturating_mul(multiplier);
            }
            _ => {}
        }
    }
}

trait LootPoolEntryTypesExt {
    fn get_stacks(&self, params: &LootContextParameters) -> Vec<ItemStack>;
}

impl LootPoolEntryTypesExt for LootPoolEntryTypes {
    fn get_stacks(&self, params: &LootContextParameters) -> Vec<ItemStack> {
        match self {
            Self::Empty => Vec::new(),
            Self::Item(item_entry) => {
                let key = &item_entry.name.strip_prefix("minecraft:").unwrap();
                vec![ItemStack::new(1, Item::from_registry_key(key).unwrap())]
            }
            // These entry types need data fields (nested table name, tag name,
            // children list) that are not yet parsed by the codegen in pumpkin-data.
            // Return empty instead of crashing. Raise to Architect if needed.
            Self::LootTable | Self::Dynamic | Self::Tag | Self::Sequence | Self::Group => {
                Vec::new()
            }
            Self::Alternatives(alternative_entry) => {
                for entry in alternative_entry.children {
                    if let Some(loot) = entry.get_loot(params) {
                        return loot;
                    }
                }
                Vec::new()
            }
        }
    }
}

trait LootConditionExt {
    fn is_fulfilled(&self, params: &LootContextParameters) -> bool;
}

impl LootConditionExt for LootCondition {
    fn is_fulfilled(&self, params: &LootContextParameters) -> bool {
        match self {
            Self::SurvivesExplosion => {
                if let Some(radius) = params.explosion_radius {
                    return rand::rng().random::<f32>() <= 1.0 / radius;
                }
                true
            }
            Self::KilledByPlayer => params.killed_by_player.unwrap_or(false),
            Self::BlockStateProperty {
                block: _,
                properties,
            } => {
                if let Some(state) = &params.block_state {
                    let block_actual_properties =
                        match Block::properties(Block::from_state_id(state.id), state.id) {
                            Some(props_data) => props_data.to_props(),
                            None => {
                                return properties.is_empty();
                            }
                        };

                    return properties.iter().all(|(expected_key, expected_value)| {
                        block_actual_properties
                            .iter()
                            .find(|(actual_key, _)| actual_key == expected_key)
                            .is_some_and(|(_, actual_value_string)| {
                                actual_value_string == expected_value
                            })
                    });
                }
                false
            }
            // These conditions need data fields (chance values, predicates, term lists)
            // that are not yet parsed by the codegen. Default to permissive (true) so
            // loot drops work rather than being silently blocked. Over-dropping is less
            // harmful than no drops at all for gameplay.
            Self::Inverted
            | Self::AnyOf
            | Self::AllOf
            | Self::RandomChance
            | Self::RandomChanceWithEnchantedBonus
            | Self::EntityProperties
            | Self::EntityScores
            | Self::MatchTool
            | Self::TableBonus
            | Self::DamageSourceProperties
            | Self::LocationCheck
            | Self::WeatherCheck
            | Self::Reference
            | Self::TimeCheck
            | Self::ValueCheck
            | Self::EnchantmentActiveCheck => true,
        }
    }
}

trait LootFunctionNumberProviderExt {
    fn generate(&self) -> f32;
}

impl LootFunctionNumberProviderExt for LootFunctionNumberProvider {
    fn generate(&self) -> f32 {
        match self {
            Self::Constant { value } => *value,
            Self::Uniform { min, max } => rand::random::<f32>() * (max - min) + min,
            Self::Binomial { n, p } => (0..n.floor() as u32).fold(0.0, |c, _| {
                if rand::rng().random_bool(f64::from(*p)) {
                    c + 1.0
                } else {
                    c
                }
            }),
        }
    }
}
