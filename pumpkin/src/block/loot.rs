use std::collections::HashMap;

use pumpkin_data::{Block, BlockState, block_properties::get_block_by_state_id, item::Item};
use pumpkin_util::{
    loot_table::{
        LootCondition, LootFunctionNumberProvider, LootFunctionTypes, LootPoolEntry,
        LootPoolEntryTypes, LootTable,
    },
    random::{RandomGenerator, xoroshiro128::Xoroshiro},
};
use pumpkin_world::item::ItemStack;
use rand::Rng;

#[derive(Default)]
pub struct LootContextParameters {
    pub explosion_radius: Option<f32>,
    pub block_state: Option<&'static BlockState>,
}

pub trait LootTableExt {
    fn get_loot(&self, params: LootContextParameters) -> Vec<ItemStack>;
}

impl LootTableExt for LootTable {
    fn get_loot(&self, params: LootContextParameters) -> Vec<ItemStack> {
        let mut stacks = Vec::new();

        if let Some(pools) = self.pools {
            for pool in pools {
                // TODO
                let rolls = pool
                    .rolls
                    .get(&mut RandomGenerator::Xoroshiro(Xoroshiro::from_seed(123)))
                    .round()
                    + pool.bonus_rolls.floor(); // TODO: multiply by luck

                for _ in 0..(rolls as i32) {
                    for entry in pool.entries {
                        if let Some(loot) = entry.get_loot(&params) {
                            stacks.extend(loot);
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
        if let Some(conditions) = self.conditions {
            if !conditions.iter().all(|cond| cond.is_fulfilled(params)) {
                return None;
            }
        }

        let mut stacks = self.content.get_stacks(params);

        if let Some(functions) = self.functions {
            for function in functions {
                if let Some(conditions) = function.conditions {
                    if !conditions.iter().all(|cond| cond.is_fulfilled(params)) {
                        continue;
                    }
                }

                match &function.content {
                    LootFunctionTypes::SetCount { count, add } => {
                        for stack in &mut stacks {
                            if *add {
                                stack.item_count += count.generate().round() as u8;
                            } else {
                                stack.item_count = count.generate().round() as u8;
                            }
                        }
                    }
                    LootFunctionTypes::LimitCount { min, max } => {
                        if let Some(min) = min.map(|min| min.round() as u8) {
                            for stack in &mut stacks {
                                if stack.item_count < min {
                                    stack.item_count = min;
                                }
                            }
                        }

                        if let Some(max) = max.map(|max| max.round() as u8) {
                            for stack in &mut stacks {
                                if stack.item_count > max {
                                    stack.item_count = max;
                                }
                            }
                        }
                    }
                    LootFunctionTypes::ApplyBonus {
                        enchantment: _,
                        formula: _,
                        parameters: _,
                    }
                    | LootFunctionTypes::CopyComponents {
                        source: _,
                        include: _,
                    }
                    | LootFunctionTypes::CopyState {
                        block: _,
                        properties: _,
                    }
                    | LootFunctionTypes::EnchantedCountIncrease
                    | LootFunctionTypes::SetOminousBottleAmplifier
                    | LootFunctionTypes::SetPotion
                    | LootFunctionTypes::FurnaceSmelt
                    | LootFunctionTypes::ExplosionDecay => {
                        // TODO: shouldnt crash here but needs to be implemented someday
                    }
                }
            }
        }

        Some(stacks)
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
            Self::LootTable => todo!(),
            Self::Dynamic => todo!(),
            Self::Tag => todo!(),
            Self::Alternatives(alternative_entry) => alternative_entry
                .children
                .iter()
                .filter_map(|entry| entry.get_loot(params))
                .flatten()
                .collect(),
            Self::Sequence => todo!(),
            Self::Group => todo!(),
        }
    }
}

trait LootConditionExt {
    fn is_fulfilled(&self, params: &LootContextParameters) -> bool;
}

impl LootConditionExt for LootCondition {
    // TODO: This is trash. Make this right
    fn is_fulfilled(&self, params: &LootContextParameters) -> bool {
        match self {
            Self::SurvivesExplosion => {
                if let Some(radius) = params.explosion_radius {
                    return rand::rng().random::<f32>() <= 1.0 / radius;
                }
                true
            }
            Self::BlockStateProperty {
                block: _,
                properties,
            } => {
                if let Some(state) = &params.block_state {
                    let block_actual_properties: HashMap<String, String> =
                        match Block::properties(get_block_by_state_id(state.id), state.id) {
                            Some(props_data) => props_data.to_props(), // Assuming to_props() returns HashMap<String, String>
                            None => {
                                return properties.is_empty();
                            }
                        };

                    return properties.iter().all(|&(expected_key, expected_value)| {
                        block_actual_properties.get(expected_key).is_some_and(
                            |actual_value_string| actual_value_string.as_str() == expected_value,
                        )
                    });
                }
                false
            }
            _ => false,
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
            Self::Uniform { min, max } => rand::rng().random_range(*min..=*max),
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
