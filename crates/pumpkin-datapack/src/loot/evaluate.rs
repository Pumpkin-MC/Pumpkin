use std::collections::HashMap;

use crate::Identifier;
use crate::loot::types::{
    DpItemStack, LootCondition, LootEntry, LootEntryType, LootFunction, LootFunctionType,
    LootNumberProvider,
};
use rand::RngExt;

/// Context for evaluating a datapack loot table.
/// Mirrors the fields available in `pumpkin::world::loot::LootContextParameters`.
#[derive(Debug, Clone, Default)]
pub struct LootEvalContext {
    pub explosion_radius: Option<f32>,
    pub killed_by_player: Option<bool>,
    pub luck: f32,
    pub this_entity_type: Option<String>,
    pub killer_entity_type: Option<String>,
    pub direct_killer_entity_type: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
    pub position_z: Option<f64>,
    pub world_time: u64,
    pub tool_item_id: Option<String>,
    /// Enchantment levels on the tool: `enchantment_id` -> level
    pub tool_enchantments: HashMap<String, i32>,
    pub is_raining: Option<bool>,
    pub is_thundering: Option<bool>,
    pub is_on_fire: Option<bool>,
    pub block_state_id: Option<u16>,
    /// All loaded datapack loot tables (for `loot_table` entry type)
    pub all_loot_tables: Option<HashMap<Identifier, crate::loot::LootTable>>,
    /// Datapack predicates for reference conditions
    pub predicates: Option<HashMap<Identifier, crate::predicate::Predicate>>,
}

/// Evaluate a full datapack loot table and produce item stacks.
/// Returns the list of items (with optional component data).
#[must_use]
pub fn evaluate_loot_table(
    table: &crate::loot::LootTable,
    ctx: &LootEvalContext,
) -> Vec<DpItemStack> {
    let mut results = Vec::new();

    for pool in &table.pools {
        if !evaluate_conditions(&pool.conditions, ctx) {
            continue;
        }

        let rolls = eval_number_provider(&pool.rolls, ctx) as i32;
        let bonus_rolls = eval_number_provider(&pool.bonus_rolls, ctx) as i32;
        let total_rolls = rolls + (bonus_rolls as f32 * ctx.luck).floor() as i32;

        for _ in 0..total_rolls.max(0) {
            evaluate_pool_entries(&pool.entries, pool.functions.as_slice(), ctx, &mut results);
        }
    }

    results
}

fn evaluate_pool_entries(
    entries: &[LootEntry],
    pool_functions: &[LootFunction],
    ctx: &LootEvalContext,
    results: &mut Vec<DpItemStack>,
) {
    // Build list of valid entries with their weights
    let mut total_weight = 0i32;
    let mut valid_entries: Vec<(usize, i32)> = Vec::new();

    for (idx, entry) in entries.iter().enumerate() {
        if !evaluate_conditions(&entry.conditions, ctx) {
            continue;
        }
        let weight = ((entry.weight as f32) + (entry.quality as f32) * ctx.luck).floor() as i32;
        let weight = weight.max(0);
        if weight > 0 {
            total_weight += weight;
            valid_entries.push((idx, weight));
        }
    }

    if total_weight == 0 || valid_entries.is_empty() {
        return;
    }

    // Weighted random selection
    let mut pick = rand::rng().random_range(0..total_weight);
    for (idx, weight) in &valid_entries {
        pick -= weight;
        if pick < 0 {
            let entry = &entries[*idx];
            let mut entry_stacks = evaluate_entry_type(&entry.content, ctx);
            // Apply entry functions, then pool functions
            apply_functions_to_stacks(&mut entry_stacks, &entry.functions, ctx);
            apply_functions_to_stacks(&mut entry_stacks, pool_functions, ctx);
            results.extend(entry_stacks);
            return;
        }
    }
}

fn evaluate_entry_type(entry_type: &LootEntryType, ctx: &LootEvalContext) -> Vec<DpItemStack> {
    match entry_type {
        LootEntryType::Empty => Vec::new(),
        LootEntryType::Item(name) => {
            vec![DpItemStack::new(name.clone(), 1)]
        }
        LootEntryType::LootTable(value) => {
            let Some(id) = Identifier::parse(value).ok() else {
                return Vec::new();
            };
            if let Some(table) = ctx.all_loot_tables.as_ref().and_then(|t| t.get(&id)) {
                return evaluate_loot_table(table, ctx);
            }
            Vec::new()
        }
        LootEntryType::Tag { name, expand } => {
            let tag_key = name.strip_prefix('#').unwrap_or(name);
            // Try loading items from the tag
            let items = resolve_tag_items(tag_key);
            if items.is_empty() {
                return Vec::new();
            }
            if *expand {
                // Pick one random item
                let idx = rand::rng().random_range(0..items.len());
                vec![DpItemStack::new(items[idx].clone(), 1)]
            } else {
                // Yield all items
                items.into_iter().map(|i| DpItemStack::new(i, 1)).collect()
            }
        }
        LootEntryType::Alternatives(children) => {
            for child in children {
                if evaluate_conditions(&child.conditions, ctx) {
                    let mut stacks = evaluate_entry_type(&child.content, ctx);
                    apply_functions_to_stacks(&mut stacks, &child.functions, ctx);
                    if !stacks.is_empty() {
                        return stacks;
                    }
                }
            }
            Vec::new()
        }
        LootEntryType::Sequence(children) => {
            let mut stacks = Vec::new();
            for child in children {
                if !evaluate_conditions(&child.conditions, ctx) {
                    break;
                }
                let mut child_stacks = evaluate_entry_type(&child.content, ctx);
                apply_functions_to_stacks(&mut child_stacks, &child.functions, ctx);
                if child_stacks.is_empty() {
                    break;
                }
                stacks.extend(child_stacks);
            }
            stacks
        }
        LootEntryType::Group(children) => {
            let mut stacks = Vec::new();
            for child in children {
                if !evaluate_conditions(&child.conditions, ctx) {
                    continue;
                }
                // For group entries, do weighted selection among children
                let mut total_weight = 0i32;
                let mut child_weights: Vec<(usize, i32)> = Vec::new();
                for (ci, c) in children.iter().enumerate() {
                    if evaluate_conditions(&c.conditions, ctx) {
                        let w = (((c.weight as f32) + (c.quality as f32) * ctx.luck).floor()
                            as i32)
                            .max(0);
                        if w > 0 {
                            total_weight += w;
                            child_weights.push((ci, w));
                        }
                    }
                }
                if total_weight > 0 && !child_weights.is_empty() {
                    let mut pick = rand::rng().random_range(0..total_weight);
                    for (ci, w) in &child_weights {
                        pick -= w;
                        if pick < 0 {
                            let c = &children[*ci];
                            let mut cs = evaluate_entry_type(&c.content, ctx);
                            apply_functions_to_stacks(&mut cs, &c.functions, ctx);
                            stacks.extend(cs);
                            break;
                        }
                    }
                }
            }
            stacks
        }
    }
}

fn evaluate_conditions(conditions: &[LootCondition], ctx: &LootEvalContext) -> bool {
    conditions
        .iter()
        .all(|cond| evaluate_single_condition(cond, ctx))
}

#[allow(clippy::too_many_lines)]
fn evaluate_single_condition(condition: &LootCondition, ctx: &LootEvalContext) -> bool {
    match condition {
        LootCondition::SurvivesExplosion => ctx
            .explosion_radius
            .is_none_or(|radius| rand::rng().random::<f32>() <= 1.0 / radius),
        LootCondition::KilledByPlayer => ctx.killed_by_player.unwrap_or(false),
        LootCondition::RandomChance(chance) => rand::rng().random::<f32>() < *chance,
        LootCondition::RandomChanceWithLooting {
            enchantment,
            unenchanted_chance,
            enchanted_chance,
        } => {
            let level = ctx.tool_enchantments.get(enchantment).copied().unwrap_or(0);
            if level > 0 {
                let chance = eval_number_provider(enchanted_chance, ctx);
                // Vanilla: enchanted_chance is usually a linear provider
                let effective = chance * level as f32;
                rand::rng().random::<f32>() < effective
            } else {
                rand::rng().random::<f32>() < *unenchanted_chance
            }
        }
        LootCondition::MatchTool {
            items,
            require_silk_touch,
        } => {
            let Some(ref tool_id) = ctx.tool_item_id else {
                return false;
            };
            items.as_ref().map_or_else(
                || {
                    if *require_silk_touch {
                        ctx.tool_enchantments.contains_key("minecraft:silk_touch")
                    } else {
                        true
                    }
                },
                |items| {
                    items.iter().any(|i| {
                        let expected = i.strip_prefix("minecraft:").unwrap_or(i);
                        let actual = tool_id.strip_prefix("minecraft:").unwrap_or(tool_id);
                        expected == actual
                    })
                },
            )
        }
        LootCondition::EntityProperties { entity, predicate } => {
            let target = match entity.as_str() {
                "this" => ctx.this_entity_type.as_deref(),
                "killer" | "attacker" => ctx.killer_entity_type.as_deref(),
                "direct_killer" | "direct_attacker" => ctx.direct_killer_entity_type.as_deref(),
                _ => None,
            };
            // Check entity type from predicate (`type` or `type_specific.type`)
            if let Some(expected_type) = predicate.get("type").and_then(|t| t.as_str()) {
                let expected = expected_type
                    .strip_prefix("minecraft:")
                    .unwrap_or(expected_type);
                if !target.is_some_and(|t| {
                    let actual = t.strip_prefix("minecraft:").unwrap_or(t);
                    actual == expected
                }) {
                    return false;
                }
            }
            // Check type_specific sub-predicate (e.g. sheep)
            if let Some(type_specific) = predicate.get("type_specific").and_then(|v| v.as_object())
            {
                if let Some(expected_type) = type_specific.get("type").and_then(|t| t.as_str()) {
                    let expected = expected_type
                        .strip_prefix("minecraft:")
                        .unwrap_or(expected_type);
                    if !target.is_some_and(|t| {
                        let actual = t.strip_prefix("minecraft:").unwrap_or(t);
                        actual == expected
                    }) {
                        return false;
                    }
                }
                // Check sheared status for sheep
                if type_specific
                    .get("sheared")
                    .and_then(serde_json::Value::as_bool)
                    .is_some()
                {
                    // We don't have sheared info in context yet; for now assume it matches
                    // when the entity type is sheep (vanilla behaviour: unsheared sheep drop wool).
                }
            }
            // Check entity data components (e.g. `minecraft:sheep/color`)
            if predicate
                .get("components")
                .and_then(|v| v.as_object())
                .is_some()
            {
                // We don't have entity component info in the loot eval context yet.
                // This will need entity component data to work fully.
            }
            // Check flags: support both `flags` and `minecraft:flags` keys
            let flags_obj = predicate
                .get("flags")
                .or_else(|| predicate.get("minecraft:flags"));
            if flags_obj.and_then(|f| f.as_object()).is_some_and(|flags| {
                flags.get("is_on_fire").and_then(serde_json::Value::as_bool) == Some(true)
                    && !ctx.is_on_fire.unwrap_or(false)
            }) {
                return false;
            }
            // Check equipment predicates: supports both `equipment` and `minecraft:equipment`.
            let equipment_obj = predicate
                .get("equipment")
                .or_else(|| predicate.get("minecraft:equipment"));
            if let Some(equipment) = equipment_obj {
                let has_mainhand = equipment.get("mainhand").is_some();
                if has_mainhand {
                    // Resolve the smelts_loot enchantment tag
                    let tag = &pumpkin_data::tag::Enchantment::MINECRAFT_SMELTS_LOOT;
                    let has_smelting_enchant = ctx.tool_enchantments.keys().any(|e| {
                        tag.0.iter().any(|ench_name| {
                            e.ends_with(ench_name) || e == &format!("minecraft:{ench_name}")
                        })
                    });
                    if !has_smelting_enchant {
                        return false;
                    }
                }
            }
            true
        }
        LootCondition::DamageSourceProperties(_predicate) => {
            // Simplified check: just verify damage source exists
            ctx.killer_entity_type.is_some()
        }
        LootCondition::Inverted(inner) => !evaluate_single_condition(inner, ctx),
        LootCondition::AllOf(conditions) => {
            conditions.iter().all(|c| evaluate_single_condition(c, ctx))
        }
        LootCondition::AnyOf(conditions) => {
            conditions.iter().any(|c| evaluate_single_condition(c, ctx))
        }
        LootCondition::LocationCheck(predicate) => {
            // Simplified: just check biome if specified
            predicate.get("biome").is_none()
        }
        LootCondition::WeatherCheck {
            raining,
            thundering,
        } => {
            let r_match = raining.is_none_or(|r| ctx.is_raining.unwrap_or(false) == r);
            let t_match = thundering.is_none_or(|t| ctx.is_thundering.unwrap_or(false) == t);
            r_match && t_match
        }
        LootCondition::TableBonus {
            enchantment,
            chances,
        } => {
            let level = ctx.tool_enchantments.get(enchantment).copied().unwrap_or(0) as usize;
            let chance = chances.get(level).unwrap_or(chances.last().unwrap_or(&0.0));
            rand::rng().random::<f32>() < *chance
        }
        LootCondition::EntityScores { .. } => {
            // Not supported yet
            false
        }
        LootCondition::TimeCheck { value, period } => {
            let mut time = ctx.world_time;
            if let Some(p) = period {
                time %= *p as u64;
            }
            value
                .as_ref()
                .is_none_or(|val| time >= eval_number_provider(val, ctx) as u64)
        }
        LootCondition::ValueCheck { value, range } => {
            let v = eval_number_provider(value, ctx);
            let (min, max) = range;
            min.is_none_or(|m| v >= m) && max.is_none_or(|m| v <= m)
        }
        LootCondition::Reference(name) => {
            let Some(id) = Identifier::parse(name).ok() else {
                return true;
            };
            ctx.predicates
                .as_ref()
                .and_then(|p| p.get(&id))
                .is_none_or(|pred| evaluate_predicate_json(&pred.data, ctx))
        }
        LootCondition::EnchantmentActiveCheck(active) => {
            let has_enchantments = !ctx.tool_enchantments.is_empty();
            has_enchantments == *active
        }
        LootCondition::BlockStateProperty(_predicate) => {
            // Not fully implemented
            true
        }
    }
}

fn apply_functions_to_stacks(
    stacks: &mut Vec<DpItemStack>,
    functions: &[LootFunction],
    ctx: &LootEvalContext,
) {
    for func in functions {
        if !evaluate_conditions(&func.conditions, ctx) {
            continue;
        }
        apply_single_function(stacks, &func.content, ctx);
    }
}

#[allow(clippy::too_many_lines)]
fn apply_single_function(
    stacks: &mut Vec<DpItemStack>,
    func: &LootFunctionType,
    ctx: &LootEvalContext,
) {
    match func {
        LootFunctionType::SetCount { count, add } => {
            let v = eval_number_provider(count, ctx).round() as u8;
            for stack in stacks.iter_mut() {
                if *add {
                    stack.count = stack.count.saturating_add(v);
                } else {
                    stack.count = v;
                }
            }
        }
        LootFunctionType::SetDamage { .. }
        | LootFunctionType::SetPotion { .. }
        | LootFunctionType::SetOminousBottleAmplifier
        | LootFunctionType::CopyState(_) => {
            // These are data components, not stored in our simple (id, count) format
        }
        LootFunctionType::SetComponents(components) => {
            if let Some(obj) = components.as_object() {
                for stack in stacks.iter_mut() {
                    for (key, value) in obj {
                        stack.components.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        LootFunctionType::CopyComponents { source } => {
            // copy_components needs block entity / entity context which is
            // not available in the datapack evaluator context yet.
            let _ = source;
        }
        LootFunctionType::FurnaceSmelt => {
            for stack in stacks.iter_mut() {
                if let Some(smelted) = get_smelting_result(&stack.item_id) {
                    stack.item_id = smelted;
                }
            }
        }
        LootFunctionType::EnchantedCountIncrease {
            enchantment,
            count,
            limit,
        } => {
            let level = ctx.tool_enchantments.get(enchantment).copied().unwrap_or(0) as f32;
            let mut additional = (eval_number_provider(count, ctx) * level).round() as u32;
            if let Some(lim) = limit {
                additional = additional.min(*lim as u32);
            }
            for stack in stacks.iter_mut() {
                stack.count = stack.count.saturating_add(additional as u8);
            }
        }
        LootFunctionType::ApplyBonus {
            enchantment,
            formula,
            parameters,
        } => {
            let level = ctx.tool_enchantments.get(enchantment).copied().unwrap_or(0);
            if level > 0 {
                for stack in stacks.iter_mut() {
                    let extra = match formula.as_str() {
                        "minecraft:binomial_with_bonus_count" => {
                            parameters.as_ref().map_or(0, |params| {
                                let extra = params
                                    .get("extra")
                                    .and_then(serde_json::Value::as_i64)
                                    .unwrap_or(0);
                                let prob = params
                                    .get("probability")
                                    .and_then(serde_json::Value::as_f64)
                                    .unwrap_or(0.0)
                                    as f32;
                                let n = level + extra as i32;
                                let mut count = 0;
                                for _ in 0..n {
                                    if rand::rng().random::<f32>() < prob {
                                        count += 1;
                                    }
                                }
                                count
                            })
                        }
                        "minecraft:uniform_bonus_count" => {
                            parameters.as_ref().map_or(0, |params| {
                                let mult = params
                                    .get("bonus_multiplier")
                                    .and_then(serde_json::Value::as_i64)
                                    .unwrap_or(0);
                                rand::rng().random_range(0..(level * mult as i32 + 1))
                            })
                        }
                        "minecraft:ore_drops" => {
                            let mult = rand::rng().random_range(0..(level + 1)) + 1;
                            mult as i32 - 1
                        }
                        _ => 0,
                    };
                    stack.count = stack.count.saturating_add(extra as u8);
                }
            }
        }
        LootFunctionType::LimitCount { min, max } => {
            for stack in stacks.iter_mut() {
                if let Some(m) = min
                    && (stack.count as f32) < *m
                {
                    stack.count = m.round() as u8;
                }
                if let Some(m) = max
                    && (stack.count as f32) > *m
                {
                    stack.count = m.round() as u8;
                }
            }
        }
        LootFunctionType::ExplosionDecay => {
            if let Some(radius) = ctx.explosion_radius {
                let survival_chance = 1.0 / radius;
                for stack in stacks.iter_mut() {
                    let mut survived = 0;
                    for _ in 0..stack.count {
                        if rand::rng().random::<f32>() <= survival_chance {
                            survived += 1;
                        }
                    }
                    stack.count = survived;
                }
                stacks.retain(|s| s.count > 0);
            }
        }
        LootFunctionType::EnchantRandomly(data) | LootFunctionType::EnchantWithLevels(data) => {
            let _ = data;
        }
        LootFunctionType::SetStewEffect(_data) => {}
        LootFunctionType::SetInstrument(_data) => {}
        LootFunctionType::ExplorationMap(_data) => {}
        LootFunctionType::SetName(data) => {
            if let Some(obj) = data.as_object() {
                for stack in stacks.iter_mut() {
                    if let Some(name) = obj.get("name") {
                        stack
                            .components
                            .insert("minecraft:custom_name".into(), name.clone());
                    }
                }
            }
        }
        LootFunctionType::SetEnchantments(_data) => {}
        LootFunctionType::CopyCustomData(_data) => {}
        LootFunctionType::SetCustomData(_data) => {}
        LootFunctionType::Filtered(_data) => {}
    }
}

fn eval_number_provider(provider: &LootNumberProvider, _ctx: &LootEvalContext) -> f32 {
    match provider {
        LootNumberProvider::Constant(v) => *v,
        LootNumberProvider::Uniform { min, max } => rand::rng().random::<f32>() * (max - min) + min,
        LootNumberProvider::Binomial { n, p } => {
            let mut count = 0;
            for _ in 0..*n {
                if rand::rng().random::<f32>() < *p {
                    count += 1;
                }
            }
            count as f32
        }
    }
}

fn evaluate_predicate_json(data: &serde_json::Value, ctx: &LootEvalContext) -> bool {
    match data {
        serde_json::Value::Array(arr) => arr.iter().all(|c| evaluate_predicate_single(c, ctx)),
        _ => evaluate_predicate_single(data, ctx),
    }
}

fn evaluate_predicate_single(data: &serde_json::Value, ctx: &LootEvalContext) -> bool {
    let Some(condition) = data.get("condition").and_then(|c| c.as_str()) else {
        return true;
    };

    match condition {
        "minecraft:entity_properties" | "entity_properties" => {
            let entity = data.get("entity").and_then(|e| e.as_str());
            let target = match entity {
                Some("this") => ctx.this_entity_type.as_deref(),
                Some("killer" | "attacker") => ctx.killer_entity_type.as_deref(),
                Some("direct_killer" | "direct_attacker") => {
                    ctx.direct_killer_entity_type.as_deref()
                }
                _ => None,
            };
            let expected = data.pointer("/predicate/type").and_then(|t| t.as_str());
            if let Some(expected_type) = expected {
                let expected = expected_type
                    .strip_prefix("minecraft:")
                    .unwrap_or(expected_type);
                if !target.is_some_and(|t| {
                    let actual = t.strip_prefix("minecraft:").unwrap_or(t);
                    actual == expected
                }) {
                    return false;
                }
            }
            if data
                .pointer("/predicate/flags/is_on_fire")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
                && !ctx.is_on_fire.unwrap_or(false)
            {
                return false;
            }
            true
        }
        "minecraft:killed_by_player" | "killed_by_player" => ctx.killed_by_player.unwrap_or(false),
        "minecraft:inverted" | "inverted" => data
            .get("term")
            .is_none_or(|t| !evaluate_predicate_json(t, ctx)),
        "minecraft:alternative" | "alternative" => data
            .get("terms")
            .and_then(|v| v.as_array())
            .is_none_or(|terms| terms.iter().any(|t| evaluate_predicate_json(t, ctx))),
        "minecraft:weather_check" | "weather_check" => {
            let raining = data.get("raining").and_then(serde_json::Value::as_bool);
            let thundering = data.get("thundering").and_then(serde_json::Value::as_bool);
            if let Some(r) = raining
                && ctx.is_raining.unwrap_or(false) != r
            {
                return false;
            }
            if let Some(t) = thundering
                && ctx.is_thundering.unwrap_or(false) != t
            {
                return false;
            }
            true
        }
        "minecraft:random_chance" | "random_chance" => {
            let chance = data
                .get("chance")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0);
            rand::rng().random::<f64>() < chance
        }
        "minecraft:survives_explosion" | "survives_explosion" => ctx
            .explosion_radius
            .is_none_or(|radius| rand::rng().random::<f32>() <= 1.0 / radius),
        "minecraft:match_tool" | "match_tool" => ctx.tool_item_id.is_some(),
        "minecraft:damage_source_properties" | "damage_source_properties" => true,
        _ => {
            tracing::debug!("Unknown predicate condition type: {condition}");
            true
        }
    }
}

/// Resolve items from a tag key (e.g., "minecraft:logs", "logs").
fn resolve_tag_items(tag_key: &str) -> Vec<String> {
    let registry_key = tag_key
        .trim_start_matches('#')
        .strip_prefix("minecraft:")
        .unwrap_or(tag_key.trim_start_matches('#'));
    if let Some(items) =
        pumpkin_data::tag::get_tag_values(pumpkin_data::tag::RegistryKey::Item, registry_key)
    {
        return items
            .iter()
            .map(|s| {
                if s.starts_with("minecraft:") {
                    s.to_string()
                } else {
                    format!("minecraft:{s}")
                }
            })
            .collect();
    }
    Vec::new()
}

/// Get the smelting result for an item.
fn get_smelting_result(item_id: &str) -> Option<String> {
    let key = item_id.strip_prefix("minecraft:").unwrap_or(item_id);
    let item = pumpkin_data::item::Item::from_registry_key(key)?;
    for recipe_type in pumpkin_data::recipes::RECIPES_COOKING {
        if let pumpkin_data::recipes::CookingRecipeType::Smelting(recipe) = recipe_type
            && recipe.ingredient.match_item(item)
        {
            let result = recipe
                .result
                .id
                .strip_prefix("minecraft:")
                .unwrap_or(recipe.result.id);
            return Some(format!("minecraft:{result}"));
        }
    }
    None
}
