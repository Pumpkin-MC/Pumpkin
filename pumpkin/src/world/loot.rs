use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag;
use pumpkin_data::{Block, BlockState, item::Item};
use pumpkin_util::{
    loot_table::{
        LootCondition, LootFunction, LootFunctionBonusParameter, LootFunctionNumberProvider,
        LootFunctionTypes, LootPoolEntry, LootPoolEntryTypes, LootTable,
    },
    random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use rand::RngExt;

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

trait LootPoolEntryExt {
    fn get_loot(&self, params: &LootContextParameters) -> Option<Vec<ItemStack>>;
}

trait LootFunctionExt {
    fn apply(&self, stacks: &mut Vec<ItemStack>, params: &LootContextParameters);
}

fn apply_bonus(
    stacks: &mut [ItemStack],
    enchantment_name: &str,
    formula: &str,
    parameters: Option<&LootFunctionBonusParameter>,
    params: &LootContextParameters,
) {
    let enchantment_level = params.tool.as_ref().map_or(0, |tool| {
        pumpkin_data::Enchantment::from_name(enchantment_name)
            .map_or(0, |enchantment| tool.get_enchantment_level(enchantment))
    });
    if enchantment_level > 0 {
        for stack in stacks {
            match formula {
                "minecraft:binomial_with_bonus_count" => {
                    if let Some(LootFunctionBonusParameter::Probability { extra, probability }) =
                        parameters
                    {
                        let n = enchantment_level + *extra;
                        let mut extra_items = 0;
                        for _ in 0..n {
                            if rand::rng().random::<f32>() < *probability {
                                extra_items += 1;
                            }
                        }
                        stack.item_count = stack.item_count.saturating_add(extra_items as u8);
                    }
                }
                "minecraft:uniform_bonus_count" => {
                    if let Some(LootFunctionBonusParameter::Multiplier { bonus_multiplier }) =
                        parameters
                    {
                        let extra =
                            rand::rng().random_range(0..=(enchantment_level * *bonus_multiplier));
                        stack.item_count = stack.item_count.saturating_add(extra as u8);
                    }
                }
                "minecraft:ore_drops" if enchantment_level > 0 => {
                    let multiplier = rand::rng().random_range(0..=(enchantment_level + 1));
                    if multiplier > 0 {
                        stack.item_count = stack.item_count.saturating_mul(multiplier as u8);
                    }
                }
                _ => {}
            }
        }
    }
}

impl LootFunctionExt for LootFunction {
    #[allow(clippy::too_many_lines)]
    fn apply(&self, stacks: &mut Vec<ItemStack>, params: &LootContextParameters) {
        if let Some(conditions) = self.conditions
            && !conditions.iter().all(|cond| cond.is_fulfilled(params))
        {
            return;
        }

        match &self.content {
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
                    for stack in stacks.iter_mut() {
                        if stack.item_count < min {
                            stack.item_count = min;
                        }
                    }
                }

                if let Some(max) = max.map(|max| max.round() as u8) {
                    for stack in stacks.iter_mut() {
                        if stack.item_count > max {
                            stack.item_count = max;
                        }
                    }
                }
            }
            LootFunctionTypes::ExplosionDecay => {
                if let Some(radius) = params.explosion_radius {
                    let survival_chance = 1.0 / radius;
                    for stack in stacks.iter_mut() {
                        let mut survived = 0;
                        for _ in 0..stack.item_count {
                            if rand::rng().random::<f32>() <= survival_chance {
                                survived += 1;
                            }
                        }
                        stack.item_count = survived;
                    }
                    // Remove empty stacks
                    stacks.retain(|stack| stack.item_count > 0);
                }
            }
            LootFunctionTypes::ApplyBonus {
                enchantment,
                formula,
                parameters,
            } => {
                apply_bonus(stacks, enchantment, formula, parameters.as_ref(), params);
            }
            LootFunctionTypes::EnchantedCountIncrease {
                enchantment,
                count,
                limit,
            } => {
                let level = params.tool.as_ref().map_or(0.0, |tool| {
                    pumpkin_data::Enchantment::from_name(enchantment)
                        .map_or(0.0, |enc| tool.get_enchantment_level(enc) as f32)
                });
                let mut additional = (count.generate() * level).round() as u32;
                if let Some(lim) = limit {
                    let lim_u32 = lim.round() as u32;
                    if additional > lim_u32 {
                        additional = lim_u32;
                    }
                }
                for stack in stacks {
                    stack.item_count = stack.item_count.saturating_add(additional as u8);
                }
            }
            LootFunctionTypes::CopyComponents { source, include } => {
                tracing::warn!(
                    "CopyComponents not supported from source: {} for {:?}",
                    source,
                    include
                );
            }
            LootFunctionTypes::CopyState {
                block: _,
                properties,
            } => {
                if let Some(state) = params.block_state
                    && let Some(props_data) =
                        Block::properties(Block::from_state_id(state.id), state.id)
                {
                    let actual_props = props_data.to_props();
                    let mut properties_to_copy = std::collections::HashMap::new();
                    for &prop_name in *properties {
                        if let Some((_, value)) = actual_props.iter().find(|(k, _)| k == &prop_name)
                        {
                            properties_to_copy.insert(prop_name.to_string(), value.to_string());
                        }
                    }
                    if !properties_to_copy.is_empty() {
                        for stack in stacks.iter_mut() {
                            if let Some(block_state_comp) = stack.get_data_component_mut::<pumpkin_data::data_component_impl::BlockStateImpl>() {
                                    for (k, v) in &properties_to_copy {
                                        block_state_comp.properties.insert(k.clone(), v.clone());
                                    }
                                } else {
                                    stack.patch.push((
                                        pumpkin_data::data_component::DataComponent::BlockState,
                                        Some(Box::new(pumpkin_data::data_component_impl::BlockStateImpl {
                                            properties: properties_to_copy.clone(),
                                        })),
                                    ));
                                }
                        }
                    }
                }
            }
            LootFunctionTypes::SetOminousBottleAmplifier => {
                let amplifier = rand::random_range(0..5); // Random 0 to 4
                for stack in stacks.iter_mut() {
                    if let Some(amplifier_comp) = stack.get_data_component_mut::<pumpkin_data::data_component_impl::OminousBottleAmplifierImpl>() {
                        amplifier_comp.amplifier = amplifier;
                    } else {
                        stack.patch.push((
                            pumpkin_data::data_component::DataComponent::OminousBottleAmplifier,
                            Some(Box::new(pumpkin_data::data_component_impl::OminousBottleAmplifierImpl {
                                amplifier,
                            })),
                        ));
                    }
                }
            }
            LootFunctionTypes::SetPotion { id } => {
                let name = id.strip_prefix("minecraft:").unwrap_or(id);
                if let Some(potion) = pumpkin_data::potion::Potion::from_name(name) {
                    let potion_id = Some(potion.id as i32);
                    for stack in stacks.iter_mut() {
                        if let Some(potion_contents) = stack.get_data_component_mut::<pumpkin_data::data_component_impl::PotionContentsImpl>() {
                            potion_contents.potion_id = potion_id;
                        } else {
                            stack.patch.push((
                                pumpkin_data::data_component::DataComponent::PotionContents,
                                Some(Box::new(pumpkin_data::data_component_impl::PotionContentsImpl {
                                    potion_id,
                                    custom_color: None,
                                    custom_effects: Vec::new(),
                                    custom_name: None,
                                })),
                            ));
                        }
                    }
                }
            }
            LootFunctionTypes::FurnaceSmelt => {
                for stack in stacks.iter_mut() {
                    for recipe_type in pumpkin_data::recipes::RECIPES_COOKING {
                        if let pumpkin_data::recipes::CookingRecipeType::Smelting(recipe) =
                            recipe_type
                            && recipe.ingredient.match_item(stack.item)
                        {
                            let result_key = recipe
                                .result
                                .id
                                .strip_prefix("minecraft:")
                                .unwrap_or(recipe.result.id);
                            if let Some(smelted_item) = Item::from_registry_key(result_key) {
                                stack.item = smelted_item;
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
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

trait LootConditionExt {
    fn is_fulfilled(&self, params: &LootContextParameters) -> bool;
}

fn compare_entity_type(expected_type: &str, actual: &EntityType) -> bool {
    let expected = expected_type
        .strip_prefix("minecraft:")
        .unwrap_or(expected_type);
    let actual = actual
        .resource_name
        .strip_prefix("minecraft:")
        .unwrap_or(actual.resource_name);
    expected == actual
}

fn check_block_state_property(state: &BlockState, properties: &[(&str, &str)]) -> bool {
    let block_actual_properties = match Block::properties(Block::from_state_id(state.id), state.id)
    {
        Some(props_data) => props_data.to_props(), // Assuming to_props() returns HashMap<String, String>
        None => {
            return properties.is_empty();
        }
    };

    properties.iter().all(|(expected_key, expected_value)| {
        block_actual_properties
            .iter()
            .find(|(actual_key, _)| actual_key == expected_key)
            .is_some_and(|(_, actual_value_string)| actual_value_string == expected_value)
    })
}

fn check_damage_source_properties(
    params: &LootContextParameters,
    expected_source_type: Option<&str>,
    expected_direct_type: Option<&str>,
) -> bool {
    if params.damage_type.is_none() {
        return false;
    }
    if let Some(expected) = expected_source_type {
        if let Some(actual) = params.killer_entity {
            if !compare_entity_type(expected, actual) {
                return false;
            }
        } else {
            return false;
        }
    }
    if let Some(expected) = expected_direct_type {
        if let Some(actual) = params.direct_killer_entity {
            if !compare_entity_type(expected, actual) {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

impl LootConditionExt for LootCondition {
    #[allow(clippy::too_many_lines)]
    fn is_fulfilled(&self, params: &LootContextParameters) -> bool {
        match self {
            Self::SurvivesExplosion => {
                if let Some(radius) = params.explosion_radius {
                    return rand::rng().random::<f32>() <= 1.0 / radius;
                }
                true
            }
            Self::RandomChance { chance } => rand::rng().random::<f32>() < *chance,
            Self::EntityProperties {
                entity,
                expected_type,
            } => {
                let target = match *entity {
                    "this" => params.this_entity,
                    "killer" | "direct_killer" => params.killer_entity,
                    _ => None,
                };
                if let Some(target) = target {
                    if let Some(expected_type) = expected_type {
                        return compare_entity_type(expected_type, target);
                    }
                    true
                } else {
                    false
                }
            }
            Self::KilledByPlayer => params.killed_by_player.unwrap_or(false),
            Self::BlockStateProperty {
                block: _,
                properties,
            } => {
                if let Some(state) = &params.block_state {
                    return check_block_state_property(state, properties);
                }
                false
            }
            Self::Inverted(term) => !term.is_fulfilled(params),
            Self::AnyOf(terms) => terms.iter().any(|cond| cond.is_fulfilled(params)),
            Self::AllOf(terms) => terms.iter().all(|cond| cond.is_fulfilled(params)),
            Self::RandomChanceWithEnchantedBonus {
                enchantment,
                chances,
            } => chances.as_ref().is_some_and(|chances| {
                let level = params.tool.as_ref().map_or(0, |tool| {
                    pumpkin_data::Enchantment::from_name(enchantment)
                        .map_or(0, |enc| tool.get_enchantment_level(enc) as usize)
                });
                let chance = chances.get(level).unwrap_or(chances.last().unwrap_or(&0.0));
                rand::rng().random::<f32>() < *chance
            }),
            Self::TableBonus {
                enchantment,
                chances,
            } => {
                let level = params.tool.as_ref().map_or(0, |tool| {
                    pumpkin_data::Enchantment::from_name(enchantment)
                        .map_or(0, |enc| tool.get_enchantment_level(enc) as usize)
                });
                let chance = chances.get(level).unwrap_or(chances.last().unwrap_or(&0.0));
                rand::rng().random::<f32>() < *chance
            }
            Self::TimeCheck { range, period } => {
                let mut time = params.world_time;
                if let Some(period) = period {
                    time %= period;
                }
                let (min, max) = range;
                let val = time as f32;
                min.is_none_or(|min| val >= min) && max.is_none_or(|max| val <= max)
            }
            Self::ValueCheck { value, range } => {
                let mut rng = Xoroshiro::from_seed(get_seed());
                let val = value.get(&mut rng);
                let (min, max) = range;
                min.is_none_or(|min| val >= min) && max.is_none_or(|max| val <= max)
            }
            Self::DamageSourceProperties {
                expected_source_type,
                expected_direct_type,
            } => {
                check_damage_source_properties(params, *expected_source_type, *expected_direct_type)
            }
            Self::WeatherCheck {
                raining,
                thundering,
            } => {
                let r_match = raining.is_none_or(|r| params.is_raining.unwrap_or(false) == r);
                let t_match = thundering.is_none_or(|t| params.is_thundering.unwrap_or(false) == t);
                r_match && t_match
            }
            Self::MatchTool { items } => params.tool.as_ref().is_some_and(|tool| {
                items.as_ref().map_or_else(
                    || {
                        pumpkin_data::Enchantment::from_name("minecraft:silk_touch")
                            .is_some_and(|silk_touch| tool.get_enchantment_level(silk_touch) > 0)
                    },
                    |items| {
                        items.iter().any(|&item_name| {
                            let expected =
                                item_name.strip_prefix("minecraft:").unwrap_or(item_name);
                            let actual = tool
                                .item
                                .registry_key
                                .strip_prefix("minecraft:")
                                .unwrap_or(tool.item.registry_key);
                            expected == actual
                        })
                    },
                )
            }),
            Self::LocationCheck { expected_biome, .. } => expected_biome.is_none(),
            Self::EntityScores { entity } => {
                tracing::warn!("EntityScores check not supported for entity: {}", entity);
                false
            }
            Self::Reference { name } => {
                tracing::warn!("Loot condition reference not supported: {}", name);
                false
            }
            Self::EnchantmentActiveCheck { active } => {
                params.tool.as_ref().map_or(!*active, |tool| {
                    let has_enchantments = tool
                        .get_data_component::<pumpkin_data::data_component_impl::EnchantmentsImpl>()
                        .is_some_and(|e| !e.enchantment.is_empty());
                    has_enchantments == *active
                })
            }
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

/// Generates a list of items from a `ChestLootTable` using a deterministic seed.
#[must_use]
pub fn generate_chest_loot(
    table: &pumpkin_util::chest_loot_table::ChestLootTable,
    seed: i64,
) -> Vec<ItemStack> {
    use pumpkin_util::random::RandomImpl;

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
