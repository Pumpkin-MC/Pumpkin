use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::loot_table::{LootBonusFormula, LootCondition, LootEntry, LootTable};
use pumpkin_util::random::{RandomImpl, xoroshiro128::Xoroshiro};

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

fn check_condition(
    cond: LootCondition,
    has_silk_touch: bool,
    has_shears: bool,
    fortune_level: i32,
    params: &LootContextParameters,
    rng: &mut Xoroshiro,
) -> bool {
    match cond {
        LootCondition::None => true,
        LootCondition::SilkTouch => has_silk_touch,
        LootCondition::NoSilkTouch => !has_silk_touch,
        LootCondition::Shears => has_shears,
        LootCondition::SilkTouchOrShears => has_silk_touch || has_shears,
        LootCondition::NoSilkTouchOrShears => !has_silk_touch && !has_shears,
        LootCondition::KilledByPlayer => params.killed_by_player.unwrap_or(false),
        LootCondition::SurvivesExplosion => params
            .explosion_radius
            .is_none_or(|radius| rng.next_f32() <= 1.0 / radius),
        LootCondition::RandomChance { chance } => rng.next_f32() < chance,
        LootCondition::RandomChanceWithEnchantedBonus {
            unenchanted_chance,
            enchanted_chance_base,
            enchanted_chance_per_level_above_first,
        } => {
            let chance = if fortune_level > 0 {
                enchanted_chance_base
                    + enchanted_chance_per_level_above_first * (fortune_level - 1) as f32
            } else {
                unenchanted_chance
            };
            rng.next_f32() < chance
        }
        LootCondition::TableBonus { chances } => {
            let index = (fortune_level.max(0) as usize).min(chances.len().saturating_sub(1));
            chances
                .get(index)
                .is_some_and(|chance| rng.next_f32() < *chance)
        }
        LootCondition::BlockStateProperty { properties } => {
            params.block_state.is_some_and(|state| {
                Block::from_state_id(state.id)
                    .properties(state.id)
                    .is_some_and(|props| {
                        let props = props.to_props();
                        properties.iter().all(|wanted| props.contains(wanted))
                    })
            })
        }
        LootCondition::Inverted(inner) => !check_condition(
            *inner,
            has_silk_touch,
            has_shears,
            fortune_level,
            params,
            rng,
        ),
        LootCondition::AnyOf(conditions) => conditions
            .iter()
            .any(|c| check_condition(*c, has_silk_touch, has_shears, fortune_level, params, rng)),
        LootCondition::ThisOnFire => params.is_on_fire.unwrap_or(false),
        LootCondition::KillerType { types } => params.killer_entity.is_some_and(|killer| {
            types
                .iter()
                .any(|name| EntityType::from_name(name).is_some_and(|t| t.id == killer.id))
        }),
        LootCondition::ToolEnchanted { enchantments } => params.tool.as_ref().is_some_and(|tool| {
            enchantments.iter().any(|name| {
                pumpkin_data::Enchantment::from_name(name)
                    .is_some_and(|e| tool.get_enchantment_level(e) > 0)
            })
        }),
        LootCondition::AllOf(conditions) => conditions
            .iter()
            .all(|c| check_condition(*c, has_silk_touch, has_shears, fortune_level, params, rng)),
    }
}

/// Applies `limit_count` and `explosion_decay` to a rolled count.
fn apply_limit_and_explosion_decay(
    entry: &LootEntry,
    mut count: i32,
    params: &LootContextParameters,
    rng: &mut Xoroshiro,
) -> i32 {
    if let Some((min, max)) = entry.limit {
        count = count.max(min).min(max);
    }
    if entry.explosion_decay
        && let Some(radius) = params.explosion_radius
    {
        // Like vanilla, each item survives the explosion with a 1 / radius chance.
        let chance = 1.0 / radius;
        let mut kept = 0;
        for _ in 0..count {
            if rng.next_f32() <= chance {
                kept += 1;
            }
        }
        count = kept;
    }
    count
}

fn apply_bonus_formula(
    base_count: i32,
    bonus: LootBonusFormula,
    fortune_level: i32,
    rng: &mut Xoroshiro,
) -> i32 {
    match bonus {
        LootBonusFormula::OreDrops => {
            if fortune_level > 0 {
                let bonus = (rng.next_bounded_i32(fortune_level + 2) - 1).max(0);
                base_count * (bonus + 1)
            } else {
                base_count
            }
        }
        LootBonusFormula::UniformBonusCount(bonus_multiplier) => {
            let max_bonus = fortune_level * bonus_multiplier;
            let extra = if max_bonus > 0 {
                rng.next_bounded_i32(max_bonus + 1)
            } else {
                0
            };
            base_count + extra
        }
        LootBonusFormula::BinomialWithBonusCount { extra, probability } => {
            let n = fortune_level + extra;
            let mut bonus_count = 0;
            for _ in 0..n {
                if rng.next_f32() < probability {
                    bonus_count += 1;
                }
            }
            base_count + bonus_count
        }
    }
}

#[must_use]
pub fn generate_loot(table: &LootTable, seed: i64) -> Vec<ItemStack> {
    generate_loot_with_context(table, seed, &LootContextParameters::default())
}

#[must_use]
pub fn generate_loot_with_context(
    table: &LootTable,
    seed: i64,
    params: &LootContextParameters,
) -> Vec<ItemStack> {
    let mut rng = Xoroshiro::from_seed(seed as u64);
    let mut items_to_place: Vec<ItemStack> = Vec::new();

    let has_silk_touch = params.tool.as_ref().is_some_and(|tool| {
        pumpkin_data::Enchantment::from_name("silk_touch")
            .is_some_and(|e| tool.get_enchantment_level(e) > 0)
    });

    let has_shears = params.tool.as_ref().is_some_and(|tool| {
        let name = tool
            .item
            .registry_key
            .strip_prefix("minecraft:")
            .unwrap_or(tool.item.registry_key);
        name == "shears"
    });

    let fortune_level = params.tool.as_ref().map_or(0, |tool| {
        let fortune = pumpkin_data::Enchantment::from_name("fortune")
            .map_or(0, |e| tool.get_enchantment_level(e));
        let looting = pumpkin_data::Enchantment::from_name("looting")
            .map_or(0, |e| tool.get_enchantment_level(e));
        fortune.max(looting)
    });

    for pool in table.pools {
        if !check_condition(
            pool.condition,
            has_silk_touch,
            has_shears,
            fortune_level,
            params,
            &mut rng,
        ) {
            continue;
        }

        let eligible_entries: Vec<&LootEntry> = pool
            .entries
            .iter()
            .filter(|e| {
                check_condition(
                    e.condition,
                    has_silk_touch,
                    has_shears,
                    fortune_level,
                    params,
                    &mut rng,
                )
            })
            .collect();

        if eligible_entries.is_empty() && pool.empty_weight == 0 {
            continue;
        }

        let range = pool.max_rolls - pool.min_rolls;
        let rolls = pool.min_rolls
            + if range > 0 {
                rng.next_bounded_i32(range + 1)
            } else {
                0
            };

        for _ in 0..rolls {
            let entry_weight: i32 = eligible_entries.iter().map(|e| e.weight).sum();
            let total_weight = entry_weight + pool.empty_weight;
            if total_weight == 0 {
                continue;
            }

            let mut pick = rng.next_bounded_i32(total_weight);

            pick -= pool.empty_weight;
            if pick < 0 {
                continue;
            }

            for entry in &eligible_entries {
                pick -= entry.weight;
                if pick < 0 {
                    let count_range = entry.max_count - entry.min_count;
                    let base_count = entry.min_count
                        + if count_range > 0 {
                            rng.next_bounded_i32(count_range + 1)
                        } else {
                            0
                        };

                    let mut final_count = base_count;
                    if let Some(bonus) = entry.bonus_formula {
                        final_count =
                            apply_bonus_formula(final_count, bonus, fortune_level, &mut rng);
                    }
                    final_count =
                        apply_limit_and_explosion_decay(entry, final_count, params, &mut rng);

                    if final_count > 0 {
                        let item_key = entry.item.strip_prefix("minecraft:").unwrap_or(entry.item);

                        if let Some(item) = Item::from_registry_key(item_key) {
                            items_to_place.push(ItemStack::new(final_count as u8, item));
                        }
                    }
                    break;
                }
            }
        }
    }

    items_to_place
}

pub use generate_loot as generate_chest_loot;

pub fn fill_chest_inventory(
    inventory: &std::sync::Arc<dyn pumpkin_inventory::Inventory>,
    table: &LootTable,
    seed: i64,
) {
    let mut items_to_place = generate_loot(table, seed);

    if items_to_place.is_empty() {
        return;
    }

    let inv_size = inventory.size();
    let mut rng = Xoroshiro::from_seed(seed as u64);

    let mut available_slots: Vec<usize> = (0..inv_size)
        .filter(|&slot| inventory.get_stack(slot).is_empty())
        .collect();

    for i in (1..available_slots.len()).rev() {
        let j = rng.next_bounded_i32((i + 1) as i32) as usize;
        available_slots.swap(i, j);
    }

    shuffle_and_split_items(&mut items_to_place, available_slots.len(), &mut rng);

    for item in items_to_place {
        let Some(slot) = available_slots.pop() else {
            tracing::warn!("Tried to over-fill a container");
            return;
        };
        inventory.set_stack(slot, item);
    }
}

fn shuffle_and_split_items(
    result: &mut Vec<ItemStack>,
    available_slots: usize,
    rng: &mut Xoroshiro,
) {
    let mut splittable: Vec<ItemStack> = Vec::new();
    let mut i = 0;
    while i < result.len() {
        if result[i].is_empty() {
            result.swap_remove(i);
        } else if result[i].item_count > 1 {
            splittable.push(result.swap_remove(i));
        } else {
            i += 1;
        }
    }

    while available_slots > result.len() + splittable.len() && !splittable.is_empty() {
        let idx = rng.next_bounded_i32(splittable.len() as i32) as usize;
        let mut stack = splittable.swap_remove(idx);

        let count = stack.item_count as i32;
        let split_off = 1 + rng.next_bounded_i32(count / 2);
        stack.item_count = (count - split_off) as u8;
        let mut copy = stack.clone();
        copy.item_count = split_off as u8;

        if stack.item_count > 1 && rng.next_bool() {
            splittable.push(stack);
        } else {
            result.push(stack);
        }
        if copy.item_count > 1 && rng.next_bool() {
            splittable.push(copy);
        } else {
            result.push(copy);
        }
    }

    result.extend(splittable);

    let n = result.len();
    for i in (1..n).rev() {
        let j = rng.next_bounded_i32((i + 1) as i32) as usize;
        result.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::Block;
    use pumpkin_data::loot_table::get_loot_table;

    fn state_with(block: &Block, name: &str, value: &str) -> &'static BlockState {
        block
            .states
            .iter()
            .find(|state| {
                block.properties(state.id).is_some_and(|props| {
                    props
                        .to_props()
                        .iter()
                        .any(|(key, val)| *key == name && *val == value)
                })
            })
            .expect("block has a state with this property value")
    }

    /// Returns `(item id, count)` for every stack the block drops in `state`, over 16 seeds.
    fn drops(block: &Block, state: &'static BlockState) -> Vec<(u16, u8)> {
        let table = get_loot_table(&format!("minecraft:blocks/{}", block.name))
            .expect("block has a loot table");
        let params = LootContextParameters {
            block_state: Some(state),
            ..Default::default()
        };
        (0..16)
            .flat_map(|seed| generate_loot_with_context(table, seed, &params))
            .map(|stack| (stack.item.id, stack.item_count))
            .collect()
    }

    #[test]
    fn composter_drops_bone_meal_only_when_full() {
        let composter = &Block::COMPOSTER;
        let empty = drops(composter, state_with(composter, "level", "0"));
        assert!(empty.iter().all(|(id, _)| *id == Item::COMPOSTER.id));

        let full = drops(composter, state_with(composter, "level", "8"));
        let bone_meal = full
            .iter()
            .filter(|(id, _)| *id == Item::BONE_MEAL.id)
            .count();
        assert_eq!(bone_meal, 16);
    }

    #[test]
    fn sweet_berry_bush_drops_depend_on_age() {
        let bush = &Block::SWEET_BERRY_BUSH;
        assert!(drops(bush, state_with(bush, "age", "0")).is_empty());

        let ripe = drops(bush, state_with(bush, "age", "3"));
        assert_eq!(ripe.len(), 16);
        assert!(
            ripe.iter()
                .all(|(id, count)| *id == Item::SWEET_BERRIES.id && (2..=3).contains(count))
        );
    }

    #[test]
    fn only_double_slabs_drop_two() {
        let slab = &Block::OAK_SLAB;
        let single = drops(slab, state_with(slab, "type", "bottom"));
        assert_eq!(single.len(), 16);
        assert!(single.iter().all(|(_, count)| *count == 1));

        let double = drops(slab, state_with(slab, "type", "double"));
        assert_eq!(double.len(), 16);
        assert!(double.iter().all(|(_, count)| *count == 2));
    }

    #[test]
    fn wheat_drops_wheat_only_when_mature() {
        let wheat = &Block::WHEAT;
        let young = drops(wheat, state_with(wheat, "age", "0"));
        assert!(young.iter().all(|(id, _)| *id != Item::WHEAT.id));

        let mature = drops(wheat, state_with(wheat, "age", "7"));
        let wheat_drops = mature
            .iter()
            .filter(|(id, _)| *id == Item::WHEAT.id)
            .count();
        assert_eq!(wheat_drops, 16);
    }

    /// Items an entity drops over 16 seeds.
    fn entity_drops(key: &str, params: &LootContextParameters) -> Vec<&'static Item> {
        let table = get_loot_table(key).expect("entity has a loot table");
        (0..16)
            .flat_map(|seed| generate_loot_with_context(table, seed, params))
            .map(|stack| stack.item)
            .collect()
    }

    fn is_music_disc(item: &Item) -> bool {
        item.registry_key.contains("music_disc")
    }

    #[test]
    fn creepers_only_drop_music_discs_when_killed_by_skeletons() {
        let by_player = LootContextParameters {
            killer_entity: Some(&EntityType::PLAYER),
            ..Default::default()
        };
        let drops = entity_drops("minecraft:entities/creeper", &by_player);
        assert!(!drops.iter().any(|item| is_music_disc(item)));

        let by_skeleton = LootContextParameters {
            killer_entity: Some(&EntityType::SKELETON),
            ..Default::default()
        };
        let drops = entity_drops("minecraft:entities/creeper", &by_skeleton);
        assert_eq!(drops.iter().filter(|item| is_music_disc(item)).count(), 16);
    }

    #[test]
    fn burning_cows_drop_cooked_beef() {
        let normal = LootContextParameters {
            is_on_fire: Some(false),
            ..Default::default()
        };
        let drops = entity_drops("minecraft:entities/cow", &normal);
        assert_eq!(
            drops.iter().filter(|item| item.id == Item::BEEF.id).count(),
            16
        );
        assert!(!drops.iter().any(|item| item.id == Item::COOKED_BEEF.id));

        let burning = LootContextParameters {
            is_on_fire: Some(true),
            ..Default::default()
        };
        let drops = entity_drops("minecraft:entities/cow", &burning);
        assert_eq!(
            drops
                .iter()
                .filter(|item| item.id == Item::COOKED_BEEF.id)
                .count(),
            16
        );
        assert!(!drops.iter().any(|item| item.id == Item::BEEF.id));
    }

    /// Glowstone dust counts over 16 seeds.
    fn glowstone_drops(params: &LootContextParameters) -> Vec<u8> {
        let table =
            get_loot_table("minecraft:blocks/glowstone").expect("glowstone has a loot table");
        (0..16)
            .flat_map(|seed| generate_loot_with_context(table, seed, params))
            .map(|stack| stack.item_count)
            .collect()
    }

    #[test]
    fn fortune_respects_limit_count() {
        let mut tool = ItemStack::new(1, &Item::DIAMOND_PICKAXE);
        tool.enchant(&pumpkin_data::Enchantment::FORTUNE, 3);
        let params = LootContextParameters {
            tool: Some(tool),
            ..Default::default()
        };
        let counts = glowstone_drops(&params);
        assert_eq!(counts.len(), 16);
        assert!(counts.iter().all(|count| (1..=4).contains(count)));
    }

    #[test]
    fn explosions_decay_block_drops() {
        let params = LootContextParameters {
            explosion_radius: Some(1.0e6),
            ..Default::default()
        };
        let total: u32 = glowstone_drops(&params)
            .iter()
            .map(|count| u32::from(*count))
            .sum();
        assert!(total <= 1);
    }
}
