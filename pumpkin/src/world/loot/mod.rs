//! Loot table evaluation, split into cohesive submodules:
//! - `condition`: loot predicate (`LootCondition`) evaluation
//! - `entry`: loot pool entry expansion into item stacks
//! - `function`: loot function application and number providers
//! - `chest`: chest loot generation and inventory filling
use pumpkin_data::BlockState;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::loot_table::LootTable;
use pumpkin_util::random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro};

mod chest;
mod condition;
mod entry;
mod function;

pub use chest::{fill_chest_inventory, generate_chest_loot};

use condition::LootConditionExt;
use entry::LootPoolEntryExt;

#[derive(Default, Clone)]
pub struct LootContextParameters {
    pub explosion_radius: Option<f32>,
    pub block_state: Option<&'static BlockState>,
    pub killed_by_player: Option<bool>,
    pub luck: f32,
    pub this_entity: Option<&'static EntityType>,
    pub killer_entity: Option<&'static EntityType>,
    pub direct_killer_entity: Option<&'static EntityType>,
    pub position: Option<pumpkin_util::math::vector3::Vector3<f64>>,
    pub world_time: u64,
    pub damage_type: Option<DamageType>,
    pub tool: Option<ItemStack>,
    pub is_raining: Option<bool>,
    pub is_thundering: Option<bool>,
    /// Whether the killed entity was on fire at death time.
    /// Computed from `Entity.fire_ticks > 0`.
    pub is_on_fire: Option<bool>,
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

                let rolls = pool.rolls.get(&mut random) as i32
                    + (pool.bonus_rolls.get(&mut random) * params.luck).floor() as i32;

                for _ in 0..rolls {
                    let mut total_weight = 0;
                    let mut valid_entries = Vec::new();

                    for entry in pool.entries {
                        if entry
                            .conditions
                            .as_ref()
                            .is_none_or(|c| c.iter().all(|cond| cond.is_fulfilled(&params)))
                        {
                            let weight = (entry.weight as f32 + entry.quality as f32 * params.luck)
                                .floor() as i32;
                            let weight = weight.max(0);
                            total_weight += weight;
                            valid_entries.push((entry, weight));
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

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::loot_table::LootTableType;

    // Compile-time assertions that the public paths and signatures survived the
    // module split (re-exported through `crate::world::loot`).
    const _: fn(&LootTable, LootContextParameters) -> Vec<ItemStack> =
        <LootTable as crate::world::loot::LootTableExt>::get_loot;
    const _: fn(&pumpkin_util::chest_loot_table::ChestLootTable, i64) -> Vec<ItemStack> =
        crate::world::loot::generate_chest_loot;

    #[test]
    fn loot_table_without_pools_yields_nothing() {
        let table = LootTable {
            r#type: LootTableType::Empty,
            random_sequence: None,
            pools: None,
        };
        assert!(table.get_loot(LootContextParameters::default()).is_empty());
    }
}
