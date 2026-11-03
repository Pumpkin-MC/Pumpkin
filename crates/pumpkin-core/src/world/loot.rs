use crate::{entity::EntityBase, world::World};
use pumpkin_data::BlockState;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EnchantmentsImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::{CookingRecipeKind, get_cooking_recipe_with_ingredient};
use pumpkin_data::tag::Taggable;
pub use pumpkin_util::loot_table::{
    DynamicLootCondition, DynamicLootEntry, DynamicLootEntryKind, DynamicLootFunction,
    DynamicLootPool, DynamicLootTable, LootBonusFormula, LootCondition, LootEntityTarget,
    LootEntry, LootEntryKind, LootFunction, LootFunctionKind, LootNumberProvider, LootPool,
    LootTable,
};
use pumpkin_util::random::{RandomImpl, xoroshiro128::Xoroshiro};

mod generation;
mod predicates;

#[derive(Default, Clone)]
pub struct LootContextParameters<'a> {
    pub world: Option<&'a World>,
    pub explosion_radius: Option<f32>,
    pub block_state: Option<&'static BlockState>,
    pub killed_by_player: Option<bool>,
    pub luck: f32,
    pub this_entity: Option<&'a dyn EntityBase>,
    pub killer_entity: Option<&'a dyn EntityBase>,
    pub direct_killer_entity: Option<&'a dyn EntityBase>,
    pub position: Option<pumpkin_util::math::vector3::Vector3<f64>>,
    pub world_time: u64,
    pub damage_type: Option<DamageType>,
    pub tool: Option<ItemStack>,
    pub is_raining: Option<bool>,
    pub is_thundering: Option<bool>,
    /// Whether the killed entity was on fire at death time.
    /// Computed from `Entity.fire_ticks > 0`.
    pub is_on_fire: Option<bool>,
    pub direct_attacker_weapon: Option<ItemStack>,
}

fn attacker_enchantment_level(params: &LootContextParameters<'_>, name: &str) -> i32 {
    let enchantment = pumpkin_data::Enchantment::from_name(name);
    let Some(enchantment) = enchantment else {
        return 0;
    };
    let entity_level = params
        .killer_entity
        .or(params.direct_killer_entity)
        .map_or(0, |entity| {
            crate::enchantment::helper::EnchantmentHelper::get_enchantment_level(
                enchantment,
                entity,
            )
        });
    let weapon_level = params
        .direct_attacker_weapon
        .as_ref()
        .filter(|weapon| !weapon.is_empty())
        .map_or(0, |weapon| weapon.get_enchantment_level(enchantment));
    entity_level.max(weapon_level)
}

fn tool_enchantment_level(params: &LootContextParameters<'_>, name: &str) -> i32 {
    pumpkin_data::Enchantment::from_name(name)
        .and_then(|enchantment| {
            params
                .tool
                .as_ref()
                .filter(|tool| !tool.is_empty())
                .map(|tool| tool.get_enchantment_level(enchantment))
        })
        .unwrap_or(0)
}

fn direct_attacker_weapon_has_enchantment(params: &LootContextParameters, tag: &str) -> bool {
    params
        .direct_attacker_weapon
        .as_ref()
        .and_then(|weapon| weapon.get_data_component::<EnchantmentsImpl>())
        .is_some_and(|enchantments| {
            enchantments
                .enchantment
                .iter()
                .any(|(enchantment, _)| enchantment.is_tagged_with(tag) == Some(true))
        })
}

fn smelt(item: &'static Item, count: i32) -> (&'static Item, i32) {
    let result =
        get_cooking_recipe_with_ingredient(item, CookingRecipeKind::Smelting).and_then(|recipe| {
            let id = recipe
                .result
                .id
                .strip_prefix("minecraft:")
                .unwrap_or(recipe.result.id);
            Item::from_registry_key(id).map(|item| (item, recipe.result.count))
        });
    let Some((result, multiplier)) = result else {
        tracing::warn!(
            "Couldn't smelt {} because there is no smelting recipe",
            item.registry_key
        );
        return (item, count);
    };
    (result, count.saturating_mul(i32::from(multiplier)))
}

fn weather_matches(
    params: &LootContextParameters<'_>,
    raining: Option<bool>,
    thundering: Option<bool>,
) -> bool {
    raining.is_none_or(|value| {
        params
            .is_raining
            .or_else(|| params.world.map(World::is_raining))
            .unwrap_or(false)
            == value
    }) && thundering.is_none_or(|value| {
        params
            .is_thundering
            .or_else(|| params.world.map(World::is_thundering))
            .unwrap_or(false)
            == value
    })
}

fn check_dynamic_condition(
    cond: &DynamicLootCondition,
    has_silk_touch: bool,
    has_shears: bool,
    params: &LootContextParameters,
    rng: &mut Xoroshiro,
) -> bool {
    match cond {
        DynamicLootCondition::None => true,
        DynamicLootCondition::Never => false,
        DynamicLootCondition::LocationCheck { offset, predicate } => {
            predicates::matches_location(*offset, predicate, params)
        }
        DynamicLootCondition::MatchBlock(predicate) => predicates::matches_block(predicate, params),
        DynamicLootCondition::MatchTool(predicate) => predicates::matches_tool(predicate, params),
        DynamicLootCondition::WeatherCheck {
            raining,
            thundering,
        } => weather_matches(params, *raining, *thundering),
        DynamicLootCondition::EntityProperties { target, predicate } => {
            predicates::matches_entity(*target, predicate, params)
        }
        DynamicLootCondition::DamageSourceTag { tag, expected } => params
            .damage_type
            .is_some_and(|damage| damage.is_tagged_with(tag) == Some(*expected)),
        DynamicLootCondition::SilkTouch => has_silk_touch,
        DynamicLootCondition::NoSilkTouch => !has_silk_touch,
        DynamicLootCondition::Shears => has_shears,
        DynamicLootCondition::SilkTouchOrShears => has_silk_touch || has_shears,
        DynamicLootCondition::NoSilkTouchOrShears => !has_silk_touch && !has_shears,
        DynamicLootCondition::KilledByPlayer => params.killed_by_player.unwrap_or(false),
        DynamicLootCondition::SurvivesExplosion => params
            .explosion_radius
            .is_none_or(|radius| radius <= 1.0 || rng.next_f32() <= 1.0 / radius),
        DynamicLootCondition::RandomChance { chance } => rng.next_f32() < *chance,
        DynamicLootCondition::RandomChanceWithEnchantedBonus {
            enchantment,
            unenchanted_chance,
            enchanted_chance_base,
            enchanted_chance_per_level_above_first,
        } => {
            let level = attacker_enchantment_level(params, enchantment);
            let chance = if level > 0 {
                enchanted_chance_base + enchanted_chance_per_level_above_first * (level - 1) as f32
            } else {
                *unenchanted_chance
            };
            rng.next_f32() < chance
        }
        DynamicLootCondition::RandomChanceWithLooting {
            chance,
            looting_multiplier,
        } => {
            let level = attacker_enchantment_level(params, "minecraft:looting");
            let chance = *chance + (level as f32) * (*looting_multiplier);
            rng.next_f32() < chance
        }
        DynamicLootCondition::TableBonus {
            enchantment,
            chances,
        } => {
            let index = (tool_enchantment_level(params, enchantment).max(0) as usize)
                .min(chances.len().saturating_sub(1));
            chances
                .get(index)
                .is_some_and(|chance| rng.next_f32() < *chance)
        }
        DynamicLootCondition::AllOf(conditions) => conditions
            .iter()
            .all(|c| check_dynamic_condition(c, has_silk_touch, has_shears, params, rng)),
        DynamicLootCondition::AnyOf(conditions) => conditions
            .iter()
            .any(|c| check_dynamic_condition(c, has_silk_touch, has_shears, params, rng)),
        DynamicLootCondition::Inverted(cond) => {
            !check_dynamic_condition(cond, has_silk_touch, has_shears, params, rng)
        }
        DynamicLootCondition::EntityOnFire => params.is_on_fire.unwrap_or_else(|| {
            params
                .this_entity
                .is_some_and(|entity| entity.get_entity().is_on_fire())
        }),
        DynamicLootCondition::DirectAttackerWeaponEnchantment(tag) => {
            direct_attacker_weapon_has_enchantment(params, tag)
        }
        DynamicLootCondition::DamageSourceProperties(predicate) => {
            predicates::matches_damage_source(predicate, params)
        }
        DynamicLootCondition::TimeCheck { value, period } => {
            predicates::matches_time(value, *period, params)
        }
        DynamicLootCondition::ValueCheck { value, range } => {
            predicates::matches_value(*value, range, params, rng)
        }
        DynamicLootCondition::Reference(name) => evaluate_predicate(name, params),
    }
}

fn check_condition(
    cond: LootCondition,
    has_silk_touch: bool,
    has_shears: bool,
    params: &LootContextParameters,
    rng: &mut Xoroshiro,
) -> bool {
    match cond {
        LootCondition::None => true,
        LootCondition::Never => false,
        LootCondition::LocationCheck { offset, predicate } => {
            predicates::matches_location(offset, predicate, params)
        }
        LootCondition::MatchBlock(predicate) => predicates::matches_block(predicate, params),
        LootCondition::MatchTool(predicate) => predicates::matches_tool(predicate, params),
        LootCondition::WeatherCheck {
            raining,
            thundering,
        } => weather_matches(params, raining, thundering),
        LootCondition::EntityProperties { target, predicate } => {
            predicates::matches_entity(target, predicate, params)
        }
        LootCondition::DamageSourceTag { tag, expected } => params
            .damage_type
            .is_some_and(|damage| damage.is_tagged_with(tag) == Some(expected)),
        LootCondition::Inverted(condition) => {
            !check_condition(*condition, has_silk_touch, has_shears, params, rng)
        }
        LootCondition::SilkTouch => has_silk_touch,
        LootCondition::NoSilkTouch => !has_silk_touch,
        LootCondition::Shears => has_shears,
        LootCondition::SilkTouchOrShears => has_silk_touch || has_shears,
        LootCondition::NoSilkTouchOrShears => !has_silk_touch && !has_shears,
        LootCondition::KilledByPlayer => params.killed_by_player.unwrap_or(false),
        LootCondition::SurvivesExplosion => params
            .explosion_radius
            .is_none_or(|radius| radius <= 1.0 || rng.next_f32() <= 1.0 / radius),
        LootCondition::RandomChance { chance } => rng.next_f32() < chance,
        LootCondition::RandomChanceWithEnchantedBonus {
            enchantment,
            unenchanted_chance,
            enchanted_chance_base,
            enchanted_chance_per_level_above_first,
        } => {
            let level = attacker_enchantment_level(params, enchantment);
            let chance = if level > 0 {
                enchanted_chance_base + enchanted_chance_per_level_above_first * (level - 1) as f32
            } else {
                unenchanted_chance
            };
            rng.next_f32() < chance
        }
        LootCondition::RandomChanceWithLooting {
            chance,
            looting_multiplier,
        } => {
            let level = attacker_enchantment_level(params, "minecraft:looting");
            let chance = chance + (level as f32) * looting_multiplier;
            rng.next_f32() < chance
        }
        LootCondition::TableBonus {
            enchantment,
            chances,
        } => {
            let index = (tool_enchantment_level(params, enchantment).max(0) as usize)
                .min(chances.len().saturating_sub(1));
            chances
                .get(index)
                .is_some_and(|chance| rng.next_f32() < *chance)
        }
        LootCondition::AllOf(conditions) => conditions
            .iter()
            .all(|c| check_condition(*c, has_silk_touch, has_shears, params, rng)),
        LootCondition::AnyOf(conditions) => conditions
            .iter()
            .any(|c| check_condition(*c, has_silk_touch, has_shears, params, rng)),
        LootCondition::EntityOnFire => params.is_on_fire.unwrap_or_else(|| {
            params
                .this_entity
                .is_some_and(|entity| entity.get_entity().is_on_fire())
        }),
        LootCondition::DirectAttackerWeaponEnchantment(tag) => {
            direct_attacker_weapon_has_enchantment(params, tag)
        }
        LootCondition::DamageSourceProperties(predicate) => {
            predicates::matches_damage_source(predicate, params)
        }
        LootCondition::TimeCheck { value, period } => {
            predicates::matches_time(value, period, params)
        }
        LootCondition::ValueCheck { value, range } => {
            predicates::matches_value(value, range, params, rng)
        }
        LootCondition::Reference(name) => evaluate_predicate(name, params),
    }
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
            let bound = fortune_level
                .saturating_mul(bonus_multiplier)
                .saturating_add(1);
            if bound > 0 {
                base_count.saturating_add(rng.next_bounded_i32(bound))
            } else {
                base_count
            }
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
    params: &LootContextParameters<'_>,
) -> Vec<ItemStack> {
    generation::generate(generation::Table::Static(table), seed, params)
}
pub use generate_loot as generate_chest_loot;
#[must_use]
pub fn generate_dynamic_loot(table: &DynamicLootTable, seed: i64) -> Vec<ItemStack> {
    generate_dynamic_loot_with_context(table, seed, &LootContextParameters::default())
}
#[must_use]
pub fn generate_dynamic_loot_with_context(
    table: &DynamicLootTable,
    seed: i64,
    params: &LootContextParameters<'_>,
) -> Vec<ItemStack> {
    generation::generate(generation::Table::Dynamic(table), seed, params)
}

/// A handle to either a compile-time static loot table or a datapack dynamic loot table.
#[derive(Clone, Debug)]
pub enum LootTableHandle {
    Static(&'static LootTable),
    Dynamic(std::sync::Arc<DynamicLootTable>),
}

impl From<&'static LootTable> for LootTableHandle {
    fn from(t: &'static LootTable) -> Self {
        Self::Static(t)
    }
}

impl From<std::sync::Arc<DynamicLootTable>> for LootTableHandle {
    fn from(t: std::sync::Arc<DynamicLootTable>) -> Self {
        Self::Dynamic(t)
    }
}

impl LootTableHandle {
    #[must_use]
    pub fn generate_loot(&self, seed: i64) -> Vec<ItemStack> {
        self.generate_loot_with_context(seed, &LootContextParameters::default())
    }

    #[must_use]
    pub fn generate_loot_with_context(
        &self,
        seed: i64,
        params: &LootContextParameters,
    ) -> Vec<ItemStack> {
        generate_loot_from_handle(self, seed, params)
    }
}

thread_local! {
    static ACTIVE_PREDICATES: std::cell::RefCell<std::collections::HashSet<String>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
}

struct PredicateGuard(String);

impl Drop for PredicateGuard {
    fn drop(&mut self) {
        ACTIVE_PREDICATES.with(|set| {
            set.borrow_mut().remove(&self.0);
        });
    }
}

#[must_use]
pub fn evaluate_predicate(predicate_name: &str, params: &LootContextParameters<'_>) -> bool {
    let full_key = if predicate_name.contains(':') {
        predicate_name.to_string()
    } else {
        format!("minecraft:{predicate_name}")
    };
    let inserted = ACTIVE_PREDICATES.with(|set| set.borrow_mut().insert(full_key.clone()));
    if !inserted {
        tracing::warn!("Recursive loot predicate evaluation detected for '{full_key}'");
        return false;
    }
    let _guard = PredicateGuard(full_key.clone());

    if let Some(world) = params.world
        && let Some(server) = world.server.upgrade()
        && let Some(val) = server
            .datapack_manager
            .get_predicate(&full_key)
            .or_else(|| server.datapack_manager.get_predicate(predicate_name))
    {
        return evaluate_predicate_value(&val, params);
    }
    pumpkin_data::loot_table::get_loot_predicate(&full_key)
        .or_else(|| pumpkin_data::loot_table::get_loot_predicate(predicate_name))
        .is_some_and(|val| evaluate_predicate_value(val, params))
}

#[must_use]
pub fn evaluate_predicate_value(
    value: &serde_json::Value,
    params: &LootContextParameters<'_>,
) -> bool {
    pumpkin_util::loot_table::parser::parse_condition_with_predicates(value, &|name| {
        pumpkin_data::loot_table::get_loot_predicate(name).cloned()
    })
    .map_or_else(
        || {
            if value.is_object() && params.this_entity.is_some() {
                predicates::matches_entity(LootEntityTarget::This, value, params)
            } else {
                false
            }
        },
        |cond| {
            let has_silk_touch = params.tool.as_ref().is_some_and(|tool| {
                tool.get_enchantment_level(&pumpkin_data::Enchantment::SILK_TOUCH) > 0
            });
            let has_shears = params
                .tool
                .as_ref()
                .is_some_and(|tool| tool.item.id == pumpkin_data::item::Item::SHEARS.id);
            let mut rng = Xoroshiro::from_seed(rand::random());
            check_dynamic_condition(&cond, has_silk_touch, has_shears, params, &mut rng)
        },
    )
}

#[must_use]
pub fn get_loot_table(key: &str) -> Option<LootTableHandle> {
    let full_key = if key.contains(':') {
        key.to_string()
    } else {
        format!("minecraft:{key}")
    };
    pumpkin_data::loot_table::get_loot_table(key)
        .or_else(|| pumpkin_data::loot_table::get_loot_table(&full_key))
        .map(LootTableHandle::Static)
}

#[must_use]
pub fn generate_loot_from_handle(
    handle: &LootTableHandle,
    seed: i64,
    params: &LootContextParameters,
) -> Vec<ItemStack> {
    match handle {
        LootTableHandle::Static(table) => generate_loot_with_context(table, seed, params),
        LootTableHandle::Dynamic(table) => generate_dynamic_loot_with_context(table, seed, params),
    }
}

fn place_items_in_chest(
    inventory: &std::sync::Arc<dyn pumpkin_inventory::Inventory>,
    mut items_to_place: Vec<ItemStack>,
    seed: i64,
) {
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

pub fn fill_chest_inventory_handle(
    inventory: &std::sync::Arc<dyn pumpkin_inventory::Inventory>,
    handle: &LootTableHandle,
    seed: i64,
    params: &LootContextParameters<'_>,
) {
    let items_to_place = generate_loot_from_handle(handle, seed, params);
    place_items_in_chest(inventory, items_to_place, seed);
}

pub fn fill_chest_inventory(
    inventory: &std::sync::Arc<dyn pumpkin_inventory::Inventory>,
    table: &LootTable,
    seed: i64,
) {
    let items_to_place = generate_loot(table, seed);
    place_items_in_chest(inventory, items_to_place, seed);
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
mod tests;
