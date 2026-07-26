use super::LootContextParameters;
use super::chest::generate_chest_loot;
use super::condition::LootConditionExt;
use super::function::LootFunctionExt;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag;
use pumpkin_util::loot_table::{LootPoolEntry, LootPoolEntryTypes};

pub(super) trait LootPoolEntryExt {
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
                function.apply(&mut stacks, params);
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
            Self::Empty | Self::Dynamic(_) => Vec::new(),
            Self::LootTable(entry) => {
                let key = entry
                    .value
                    .strip_prefix("minecraft:")
                    .unwrap_or(entry.value);
                // First try chest loot tables.
                pumpkin_data::chest_loot_table::get_chest_loot_table(&format!("minecraft:{key}"))
                    .map_or_else(Vec::new, |chest_table| {
                        // We don't have a seed here, but we can generate a random one.
                        let seed: i64 = rand::random();
                        generate_chest_loot(chest_table, seed)
                    })
            }
            Self::Item(item_entry) => {
                let key = &item_entry.name.strip_prefix("minecraft:").unwrap();
                vec![ItemStack::new(1, Item::from_registry_key(key).unwrap())]
            }
            Self::Tag(tag) => {
                let key = tag.name.strip_prefix("minecraft:").unwrap_or(tag.name);

                let items = pumpkin_data::tag::get_tag_values(tag::RegistryKey::Item, key)
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|registry_key| {
                        let item_key = registry_key
                            .strip_prefix("minecraft:")
                            .unwrap_or(registry_key);
                        Item::from_registry_key(item_key)
                    })
                    .collect::<Vec<_>>();

                if items.is_empty() {
                    return Vec::new();
                }

                if tag.expand {
                    // Pick one random item from the tag
                    let index = rand::random_range(0..items.len() as i32) as usize;
                    vec![ItemStack::new(1, items[index])]
                } else {
                    // Yield one stack of every item in the tag
                    items.iter().map(|&item| ItemStack::new(1, item)).collect()
                }
            }
            Self::Alternatives(alternative_entry) => {
                for entry in alternative_entry.children {
                    if let Some(loot) = entry.get_loot(params) {
                        return loot;
                    }
                }
                Vec::new()
            }
            Self::Sequence(sequence_entry) => {
                let mut stacks = Vec::new();
                for entry in sequence_entry.children {
                    if entry
                        .conditions
                        .as_ref()
                        .is_some_and(|c| !c.iter().all(|cond| cond.is_fulfilled(params)))
                    {
                        break;
                    }

                    match entry.get_loot(params) {
                        Some(loot) => stacks.extend(loot),
                        // get_loot returning None also signals failure — stop.
                        None => break,
                    }
                }
                stacks
            }

            Self::Group(group_entry) => {
                let mut stacks = Vec::new();
                for entry in group_entry.children {
                    if let Some(loot) = entry.get_loot(params) {
                        stacks.extend(loot);
                    }
                }
                stacks
            }
        }
    }
}
