use pumpkin_data::item::Item;
use pumpkin_util::loot_table::{
    LootCondition, LootFunctionNumberProvider, LootFunctionTypes, LootPoolEntry,
    LootPoolEntryTypes, LootTable,
};
use pumpkin_world::item::ItemStack;
use rand::Rng;

pub(super) trait LootTableExt {
    fn get_loot(&self, block_props: &[(String, String)]) -> Vec<ItemStack>;
}

impl LootTableExt for LootTable {
    fn get_loot(&self, block_props: &[(String, String)]) -> Vec<ItemStack> {
        let mut stacks = Vec::new();

        if let Some(pools) = self.pools {
            for pool in pools {
                let rolls = pool.rolls.round() + pool.bonus_rolls.floor(); // TODO: multiply by luck

                for _ in 0..(rolls as i32) {
                    for entry in pool.entries {
                        if let Some(loot) = entry.get_loot(block_props) {
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
    fn get_loot(&self, block_props: &[(String, String)]) -> Option<Vec<ItemStack>>;
}

impl LootPoolEntryExt for LootPoolEntry {
    fn get_loot(&self, block_props: &[(String, String)]) -> Option<Vec<ItemStack>> {
        if let Some(conditions) = self.conditions {
            if !conditions.iter().all(|cond| cond.is_fulfilled(block_props)) {
                return None;
            }
        }

        let mut stacks = self.content.get_stacks(block_props);

        if let Some(functions) = self.functions {
            for function in functions {
                if let Some(conditions) = function.conditions {
                    if !conditions.iter().all(|cond| cond.is_fulfilled(block_props)) {
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
    fn get_stacks(&self, block_props: &[(String, String)]) -> Vec<ItemStack>;
}

impl LootPoolEntryTypesExt for LootPoolEntryTypes {
    fn get_stacks(&self, block_props: &[(String, String)]) -> Vec<ItemStack> {
        match self {
            Self::Empty => Vec::new(),
            Self::Item(item_entry) => {
                let key = &item_entry.name.replace("minecraft:", "");
                vec![ItemStack::new(1, Item::from_registry_key(key).unwrap())]
            }
            Self::LootTable => todo!(),
            Self::Dynamic => todo!(),
            Self::Tag => todo!(),
            Self::Alternatives(alternative_entry) => alternative_entry
                .children
                .iter()
                .filter_map(|entry| entry.get_loot(block_props))
                .flatten()
                .collect(),
            Self::Sequence => todo!(),
            Self::Group => todo!(),
        }
    }
}

trait LootConditionExt {
    fn is_fulfilled(&self, block_props: &[(String, String)]) -> bool;
}

impl LootConditionExt for LootCondition {
    // TODO: This is trash. Make this right
    fn is_fulfilled(&self, block_props: &[(String, String)]) -> bool {
        match self {
            Self::SurvivesExplosion => true,
            Self::BlockStateProperty {
                block: _,
                properties,
            } => properties
                .iter()
                .all(|(key, value)| block_props.iter().any(|(k, v)| k == key && v == value)),
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
            Self::Uniform { min, max } => rand::thread_rng().gen_range(*min..=*max),
            Self::Binomial { n, p } => (0..n.floor() as u32).fold(0.0, |c, _| {
                if rand::thread_rng().gen_bool(f64::from(*p)) {
                    c + 1.0
                } else {
                    c
                }
            }),
        }
    }
}
