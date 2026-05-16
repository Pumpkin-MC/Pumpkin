use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag;
use pumpkin_data::{Block, BlockState, item::Item};
use pumpkin_util::{
    loot_table::{
        LootCondition, LootFunctionNumberProvider, LootFunctionTypes, LootPoolEntry,
        LootPoolEntryTypes, LootTable,
    },
    random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use rand::RngExt;

#[derive(Default, Copy, Clone)]
pub struct LootContextParameters {
    pub explosion_radius: Option<f32>,
    pub block_state: Option<&'static BlockState>,
    pub killed_by_player: Option<bool>,
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
            Self::RandomChance { chance } => rand::rng().random::<f32>() < *chance,
            Self::KilledByPlayer => params.killed_by_player.unwrap_or(false),
            Self::BlockStateProperty {
                block: _,
                properties,
            } => {
                if let Some(state) = &params.block_state {
                    let block_actual_properties =
                        match Block::properties(Block::from_state_id(state.id), state.id) {
                            Some(props_data) => props_data.to_props(), // Assuming to_props() returns HashMap<String, String>
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

/// Fills a chest inventory with items generated from a static `ChestLootTable`, using a
/// deterministic seed for deferred loot chests.
///
/// Items are scattered randomly across the 27 chest slots.
pub async fn fill_chest_inventory(
    inventory: &std::sync::Arc<dyn pumpkin_world::inventory::Inventory>,
    table: &pumpkin_util::chest_loot_table::ChestLootTable,
    seed: i64,
) {
    use pumpkin_util::random::RandomImpl;

    let mut rng = Xoroshiro::from_seed(seed as u64);
    let inv_size = inventory.size(); // 27 for a normal chest

    let mut items_to_place: Vec<ItemStack> = Vec::new();

    for pool in table.pools {
        let range = pool.max_rolls - pool.min_rolls;
        let rolls = pool.min_rolls
            + if range > 0 {
                rng.next_bounded_i32(range + 1)
            } else {
                0
            };

        for _ in 0..rolls {
            let entry_weight: i32 = pool.entries.iter().map(|e| e.weight).sum();
            let total_weight = entry_weight + pool.empty_weight;
            if total_weight == 0 {
                continue;
            }

            let mut pick = rng.next_bounded_i32(total_weight);

            // Subtract empty weight first (if the pick lands here, it yields nothing).
            pick -= pool.empty_weight;
            if pick < 0 {
                continue;
            }

            for entry in pool.entries {
                pick -= entry.weight;
                if pick < 0 {
                    let count_range = entry.max_count - entry.min_count;
                    let count = entry.min_count
                        + if count_range > 0 {
                            rng.next_bounded_i32(count_range + 1)
                        } else {
                            0
                        };

                    // Strip "minecraft:" prefix because from_registry_key uses short keys.
                    let item_key = entry.item.strip_prefix("minecraft:").unwrap_or(entry.item);

                    if let Some(item) = Item::from_registry_key(item_key) {
                        items_to_place.push(ItemStack::new(count as u8, item));
                    }
                    break;
                }
            }
        }
    }

    if items_to_place.is_empty() {
        return;
    }

    // Count free slots in the inventory.
    let free_slots = inv_size;

    // Split large stacks across extra slots then shuffle.
    shuffle_and_split_items(&mut items_to_place, free_slots, &mut rng);

    // Pick random distinct slots and place each item.
    let mut available_slots: Vec<usize> = (0..inv_size).collect();
    // Shuffle available slots using Fisher-Yates so item order from above maps to random slots.
    for i in (1..available_slots.len()).rev() {
        let j = rng.next_bounded_i32((i + 1) as i32) as usize;
        available_slots.swap(i, j);
    }

    for item in items_to_place {
        if available_slots.is_empty() {
            break;
        }
        let slot = available_slots.pop().unwrap();
        inventory.set_stack(slot, item).await;
    }
}

/// Stacks with count > 1 are split at a random midpoint and redistributed while
/// there are more free slots than total items. Then everything is shuffled.
fn shuffle_and_split_items(
    result: &mut Vec<ItemStack>,
    available_slots: usize,
    rng: &mut Xoroshiro,
) {
    use pumpkin_util::random::RandomImpl;

    // Drain all items with count > 1 into a splittable list.
    let mut splittable: Vec<ItemStack> = Vec::new();
    let mut i = 0;
    while i < result.len() {
        if result[i].item_count > 1 {
            splittable.push(result.swap_remove(i));
        } else {
            i += 1;
        }
    }

    // While there are more free slots than total items, split a random stack.
    while available_slots > result.len() + splittable.len() && !splittable.is_empty() {
        let idx = rng.next_bounded_i32(splittable.len() as i32) as usize;
        let mut stack = splittable.swap_remove(idx);

        let count = stack.item_count as i32;
        // Split off [1, count/2] items.
        let split_off = 1 + rng.next_bounded_i32(count / 2);
        stack.item_count = (count - split_off) as u8;
        let mut copy = stack.clone();
        copy.item_count = split_off as u8;

        if stack.item_count > 1 {
            splittable.push(stack);
        } else {
            result.push(stack);
        }
        if copy.item_count > 1 {
            splittable.push(copy);
        } else {
            result.push(copy);
        }
    }

    // Remaining unsplit multis go straight into result.
    result.extend(splittable);

    // Fisher-Yates shuffle with our RNG.
    let n = result.len();
    for i in (1..n).rev() {
        let j = rng.next_bounded_i32((i + 1) as i32) as usize;
        result.swap(i, j);
    }
}
