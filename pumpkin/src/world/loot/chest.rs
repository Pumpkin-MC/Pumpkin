use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::random::{RandomImpl, xoroshiro128::Xoroshiro};

/// Generates a list of items from a `ChestLootTable` using a deterministic seed.
#[must_use]
pub fn generate_chest_loot(
    table: &pumpkin_util::chest_loot_table::ChestLootTable,
    seed: i64,
) -> Vec<ItemStack> {
    let mut rng = Xoroshiro::from_seed(seed as u64);
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

    items_to_place
}

/// Items are scattered randomly across the 27 chest slots.
pub async fn fill_chest_inventory(
    inventory: &std::sync::Arc<dyn pumpkin_world::inventory::Inventory>,
    table: &pumpkin_util::chest_loot_table::ChestLootTable,
    seed: i64,
) {
    let mut items_to_place = generate_chest_loot(table, seed);

    if items_to_place.is_empty() {
        return;
    }

    let inv_size = inventory.size(); // 27 for a normal chest
    let mut rng = Xoroshiro::from_seed(seed as u64);
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

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::chest_loot_table::{ChestLootEntry, ChestLootPool, ChestLootTable};

    const DIAMOND_ONLY: ChestLootTable = ChestLootTable {
        pools: &[ChestLootPool {
            entries: &[ChestLootEntry {
                item: "minecraft:diamond",
                weight: 1,
                min_count: 2,
                max_count: 2,
            }],
            min_rolls: 3,
            max_rolls: 3,
            empty_weight: 0,
        }],
    };

    #[test]
    fn chest_loot_is_deterministic_for_a_seed() {
        let first = generate_chest_loot(&DIAMOND_ONLY, 12345);
        let second = generate_chest_loot(&DIAMOND_ONLY, 12345);
        assert_eq!(first.len(), second.len());
        for (a, b) in first.iter().zip(&second) {
            assert_eq!(a.item.id, b.item.id);
            assert_eq!(a.item_count, b.item_count);
        }
    }

    #[test]
    fn single_entry_pool_rolls_exact_counts() {
        let items = generate_chest_loot(&DIAMOND_ONLY, 999);
        assert_eq!(items.len(), 3);
        for stack in &items {
            assert_eq!(stack.item_count, 2);
            assert_eq!(stack.item.registry_key, "diamond");
        }
    }

    #[test]
    fn splitting_preserves_total_item_count() {
        let mut rng = Xoroshiro::from_seed(42);
        let mut items = vec![ItemStack::new(8, &Item::DIAMOND)];
        shuffle_and_split_items(&mut items, 27, &mut rng);
        let total: u32 = items.iter().map(|stack| u32::from(stack.item_count)).sum();
        assert_eq!(total, 8);
        assert!(items.iter().all(|stack| stack.item_count >= 1));
    }
}
