use pumpkin_data::tag::{RegistryKey, get_tag_values};
use serde_json::Value;

pub type ItemEnchantment<'a> = (&'a str, u32);
pub type InventoryItemSnapshot<'a> = (&'a str, u32, &'a [ItemEnchantment<'a>]);

/// Checks if an expected identifier equals the actual identifier, ignoring "minecraft:" prefix.
#[must_use]
pub fn matches_id(expected: &str, actual: &str) -> bool {
    let exp_clean = expected.strip_prefix("minecraft:").unwrap_or(expected);
    let act_clean = actual.strip_prefix("minecraft:").unwrap_or(actual);
    exp_clean == act_clean
}

/// Matches an identifier or tag against a registry tag collection.
#[must_use]
pub fn matches_tag_or_id(registry_key: RegistryKey, expected: &str, actual: &str) -> bool {
    if let Some(tag_name) = expected.strip_prefix('#') {
        if let Some(values) = get_tag_values(registry_key, tag_name) {
            let act_clean = actual.strip_prefix("minecraft:").unwrap_or(actual);
            return values.iter().any(|v| {
                let v_clean = v.strip_prefix("minecraft:").unwrap_or(v);
                v_clean == act_clean
            });
        }
        if !tag_name.contains(':') {
            let full_tag = format!("minecraft:{tag_name}");
            if let Some(values) = get_tag_values(registry_key, &full_tag) {
                let act_clean = actual.strip_prefix("minecraft:").unwrap_or(actual);
                return values.iter().any(|v| {
                    let v_clean = v.strip_prefix("minecraft:").unwrap_or(v);
                    v_clean == act_clean
                });
            }
        }
        false
    } else {
        matches_id(expected, actual)
    }
}

/// Matches against a string, tag, or array of strings/tags in JSON.
#[must_use]
pub fn matches_any_tag_or_id(registry_key: RegistryKey, expected: &Value, actual: &str) -> bool {
    match expected {
        Value::String(s) => matches_tag_or_id(registry_key, s, actual),
        Value::Array(arr) => arr.iter().any(|item| {
            item.as_str()
                .is_some_and(|s| matches_tag_or_id(registry_key, s, actual))
        }),
        _ => false,
    }
}

/// Checks if integer value is within the specified bounds (single number or `{"min": ..., "max": ...}`).
#[must_use]
pub fn matches_int_bounds(bounds: &Value, val: i64) -> bool {
    IntBounds::from_value(bounds).is_none_or(|b| b.matches(val))
}

/// Checks if float value is within the specified bounds (single number or `{"min": ..., "max": ...}`).
#[must_use]
pub fn matches_float_bounds(bounds: &Value, val: f64) -> bool {
    FloatBounds::from_value(bounds).is_none_or(|b| b.matches(val))
}

/// Strongly-typed bounds for integer numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntBounds {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

impl IntBounds {
    #[must_use]
    pub fn matches(&self, val: i64) -> bool {
        self.min.is_none_or(|min| val >= min) && self.max.is_none_or(|max| val <= max)
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Option<Self> {
        match val {
            Value::Number(n) => n.as_i64().map(|v| Self {
                min: Some(v),
                max: Some(v),
            }),
            Value::Object(map) => {
                let min = map.get("min").and_then(Value::as_i64);
                let max = map.get("max").and_then(Value::as_i64);
                Some(Self { min, max })
            }
            _ => None,
        }
    }
}

/// Strongly-typed bounds for floating point numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct FloatBounds {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl FloatBounds {
    #[must_use]
    pub fn matches(&self, val: f64) -> bool {
        self.min.is_none_or(|min| val >= min) && self.max.is_none_or(|max| val <= max)
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Option<Self> {
        match val {
            Value::Number(n) => n.as_f64().map(|v| Self {
                min: Some(v),
                max: Some(v),
            }),
            Value::Object(map) => {
                let min = map.get("min").and_then(Value::as_f64);
                let max = map.get("max").and_then(Value::as_f64);
                Some(Self { min, max })
            }
            _ => None,
        }
    }
}

/// Matches against one or more item/block/entity tags or IDs.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TagOrIdMatcher {
    pub patterns: Vec<String>,
}

impl TagOrIdMatcher {
    #[must_use]
    pub fn matches(&self, registry_key: RegistryKey, actual: &str) -> bool {
        if self.patterns.is_empty() {
            return true;
        }
        self.patterns
            .iter()
            .any(|pattern| matches_tag_or_id(registry_key, pattern, actual))
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Option<Self> {
        match val {
            Value::String(s) => Some(Self {
                patterns: vec![s.clone()],
            }),
            Value::Array(arr) => {
                let patterns = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect();
                Some(Self { patterns })
            }
            _ => None,
        }
    }
}

/// Enchantment requirement for an item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnchantmentPredicate {
    pub enchantment: String,
    pub min_level: i64,
}

/// Strongly-typed condition for item matching.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ItemPredicate {
    pub items: Option<TagOrIdMatcher>,
    pub count: Option<IntBounds>,
    pub durability: Option<IntBounds>,
    pub enchantments: Vec<EnchantmentPredicate>,
}

impl ItemPredicate {
    #[must_use]
    pub fn matches(&self, item_key: &str, count: u32, enchantments: &[(&str, u32)]) -> bool {
        if let Some(items) = &self.items
            && !items.matches(RegistryKey::Item, item_key)
        {
            return false;
        }
        if let Some(count_bounds) = &self.count
            && !count_bounds.matches(i64::from(count))
        {
            return false;
        }
        for target_ench in &self.enchantments {
            let matched = enchantments.iter().any(|(name, level)| {
                matches_tag_or_id(RegistryKey::Enchantment, &target_ench.enchantment, name)
                    && i64::from(*level) >= target_ench.min_level
            });
            if !matched {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn from_value(pred_val: &Value) -> Self {
        let Value::Object(pred_map) = pred_val else {
            return TagOrIdMatcher::from_value(pred_val).map_or_else(Self::default, |matcher| {
                Self {
                    items: Some(matcher),
                    count: None,
                    durability: None,
                    enchantments: Vec::new(),
                }
            });
        };

        let items = pred_map.get("items").and_then(TagOrIdMatcher::from_value);
        let count = pred_map.get("count").and_then(IntBounds::from_value);
        let durability = pred_map.get("durability").and_then(IntBounds::from_value);

        let mut enchantments = Vec::new();
        if let Some(predicates) = pred_map.get("predicates").and_then(Value::as_object)
            && let Some(enchants) = predicates
                .get("minecraft:enchantments")
                .and_then(Value::as_array)
        {
            for ench_cond in enchants {
                if let Some(target_ench) = ench_cond.get("enchantments").and_then(Value::as_str) {
                    let min_level = ench_cond
                        .get("levels")
                        .and_then(|l| l.get("min"))
                        .and_then(Value::as_i64)
                        .unwrap_or(1);
                    enchantments.push(EnchantmentPredicate {
                        enchantment: target_ench.to_string(),
                        min_level,
                    });
                }
            }
        }

        Self {
            items,
            count,
            durability,
            enchantments,
        }
    }
}

/// Slot condition for `minecraft:inventory_changed`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventorySlotsPredicate {
    pub empty: Option<IntBounds>,
    pub full: Option<IntBounds>,
    pub occupied: Option<IntBounds>,
}

/// Strongly-typed condition for `minecraft:inventory_changed`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InventoryChangedPredicate {
    pub items: Vec<ItemPredicate>,
    pub slots: Option<InventorySlotsPredicate>,
}

impl InventoryChangedPredicate {
    #[must_use]
    pub fn matches(
        &self,
        inventory: &[InventoryItemSnapshot<'_>],
        slots_full: usize,
        slots_empty: usize,
        slots_occupied: usize,
    ) -> bool {
        if let Some(slots) = &self.slots {
            if let Some(empty) = &slots.empty
                && !empty.matches(slots_empty as i64)
            {
                return false;
            }
            if let Some(full) = &slots.full
                && !full.matches(slots_full as i64)
            {
                return false;
            }
            if let Some(occupied) = &slots.occupied
                && !occupied.matches(slots_occupied as i64)
            {
                return false;
            }
        }

        for item_pred in &self.items {
            let matched = inventory
                .iter()
                .any(|(key, count, enchants)| item_pred.matches(key, *count, enchants));
            if !matched {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };

        let slots = map
            .get("slots")
            .and_then(Value::as_object)
            .map(|slots_map| InventorySlotsPredicate {
                empty: slots_map.get("empty").and_then(IntBounds::from_value),
                full: slots_map.get("full").and_then(IntBounds::from_value),
                occupied: slots_map.get("occupied").and_then(IntBounds::from_value),
            });

        let mut items = Vec::new();
        if let Some(items_val) = map.get("items").and_then(Value::as_array) {
            for item_val in items_val {
                items.push(ItemPredicate::from_value(item_val));
            }
        }

        Self { items, slots }
    }
}

/// Strongly-typed location condition.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct LocationPredicate {
    pub x: Option<FloatBounds>,
    pub y: Option<FloatBounds>,
    pub z: Option<FloatBounds>,
    pub biome: Option<TagOrIdMatcher>,
    pub dimension: Option<String>,
    pub block: Option<TagOrIdMatcher>,
    pub can_see_sky: Option<bool>,
    pub smokey: Option<bool>,
    pub structure: Option<String>,
    pub stepping_on: Option<TagOrIdMatcher>,
    pub feet_item: Option<TagOrIdMatcher>,
}

impl LocationPredicate {
    #[must_use]
    #[expect(clippy::too_many_arguments)]
    pub fn matches(
        &self,
        pos: (f64, f64, f64),
        biome: Option<&str>,
        dimension: Option<&str>,
        step_block: Option<&str>,
        can_see_sky: Option<bool>,
        smokey: Option<bool>,
        stepping_block: Option<&str>,
        feet_item: Option<&str>,
    ) -> bool {
        if let Some(x) = &self.x
            && !x.matches(pos.0)
        {
            return false;
        }
        if let Some(y) = &self.y
            && !y.matches(pos.1)
        {
            return false;
        }
        if let Some(z) = &self.z
            && !z.matches(pos.2)
        {
            return false;
        }
        if let Some(expected_dim) = &self.dimension {
            let Some(dim) = dimension else {
                return false;
            };
            if !matches_id(expected_dim, dim) {
                return false;
            }
        }
        if let Some(biome_matcher) = &self.biome {
            let Some(b) = biome else {
                return false;
            };
            if !biome_matcher.matches(RegistryKey::WorldgenBiome, b) {
                return false;
            }
        }
        if let Some(block_matcher) = &self.block {
            let Some(blk) = step_block else {
                return false;
            };
            if !block_matcher.matches(RegistryKey::Block, blk) {
                return false;
            }
        }
        if let Some(stepping_matcher) = &self.stepping_on {
            let Some(blk) = stepping_block else {
                return false;
            };
            if !stepping_matcher.matches(RegistryKey::Block, blk) {
                return false;
            }
        }
        if let Some(feet_matcher) = &self.feet_item {
            let Some(f) = feet_item else {
                return false;
            };
            if !feet_matcher.matches(RegistryKey::Item, f) {
                return false;
            }
        }
        if let Some(expected_sky) = self.can_see_sky
            && can_see_sky != Some(expected_sky)
        {
            return false;
        }
        if let Some(expected_smokey) = self.smokey
            && smokey != Some(expected_smokey)
        {
            return false;
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };

        let loc_val = map
            .get("player")
            .and_then(|p| p.get("predicate"))
            .and_then(|pred| {
                pred.get("minecraft:location")
                    .or_else(|| pred.get("location"))
            })
            .or_else(|| map.get("location"))
            .unwrap_or(val);
        let loc_map = loc_val.as_object().unwrap_or(map);

        let (x, y, z) = loc_map
            .get("position")
            .map_or((None, None, None), |pos_val| {
                (
                    pos_val.get("x").and_then(FloatBounds::from_value),
                    pos_val.get("y").and_then(FloatBounds::from_value),
                    pos_val.get("z").and_then(FloatBounds::from_value),
                )
            });

        let biome = loc_map
            .get("biomes")
            .or_else(|| loc_map.get("biome"))
            .and_then(TagOrIdMatcher::from_value);

        let dimension = loc_map
            .get("dimension")
            .and_then(Value::as_str)
            .map(ToString::to_string);

        let block = loc_map
            .get("block")
            .or_else(|| loc_map.get("blocks"))
            .and_then(|b_val| {
                let pred = b_val.get("blocks").unwrap_or(b_val);
                TagOrIdMatcher::from_value(pred)
            });

        let can_see_sky = loc_map.get("can_see_sky").and_then(Value::as_bool);
        let smokey = loc_map.get("smokey").and_then(Value::as_bool);
        let structure = loc_map
            .get("structures")
            .or_else(|| loc_map.get("structure"))
            .and_then(Value::as_str)
            .map(ToString::to_string);

        let player_pred = map.get("player").and_then(|p| p.get("predicate"));
        let stepping_on = player_pred
            .and_then(|p| {
                p.get("minecraft:stepping_on")
                    .or_else(|| p.get("stepping_on"))
            })
            .and_then(|s| s.get("block").or_else(|| s.get("blocks")))
            .and_then(|b| b.get("blocks").or(Some(b)))
            .and_then(TagOrIdMatcher::from_value);
        let feet_item = player_pred
            .and_then(|p| p.get("minecraft:equipment").or_else(|| p.get("equipment")))
            .and_then(|eq| eq.get("feet"))
            .and_then(|f| f.get("items"))
            .and_then(TagOrIdMatcher::from_value);

        Self {
            x,
            y,
            z,
            biome,
            dimension,
            block,
            can_see_sky,
            smokey,
            structure,
            stepping_on,
            feet_item,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ItemUsedOnBlockPredicate {
    pub blocks: Option<TagOrIdMatcher>,
    pub items: Option<TagOrIdMatcher>,
    pub state_key: Option<String>,
    pub state_value: Option<String>,
    pub smokey: Option<bool>,
    pub biome: Option<String>,
}

impl ItemUsedOnBlockPredicate {
    #[must_use]
    pub fn matches(
        &self,
        block_id: &str,
        item_id: &str,
        block_state: Option<(&str, &str)>,
        smokey: Option<bool>,
        biome: Option<&str>,
    ) -> bool {
        if let Some(block_matcher) = &self.blocks
            && !block_matcher.matches(RegistryKey::Block, block_id)
        {
            return false;
        }
        if let Some(item_matcher) = &self.items
            && !item_matcher.matches(RegistryKey::Item, item_id)
        {
            return false;
        }
        if let Some(expected_key) = &self.state_key {
            let Some((k, v)) = block_state else {
                return false;
            };
            if expected_key != k {
                return false;
            }
            if let Some(expected_val) = &self.state_value
                && expected_val != v
            {
                return false;
            }
        }
        if let Some(expected_smokey) = self.smokey
            && smokey != Some(expected_smokey)
        {
            return false;
        }
        if let Some(expected_biome) = &self.biome {
            let Some(b) = biome else {
                return false;
            };
            if !matches_id(expected_biome, b) {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let loc = val.get("location").unwrap_or(val);
        let mut block_patterns = Vec::new();
        let mut item_patterns = Vec::new();
        let mut state_key = None;
        let mut state_value = None;
        let mut smokey = None;
        let mut biome = None;

        let mut terms = Vec::new();
        if let Some(arr) = loc.get("terms").and_then(Value::as_array) {
            terms.extend(arr.iter());
        } else {
            terms.push(loc);
        }

        for term in terms {
            let pred = term.get("predicate").unwrap_or(term);
            let blk = pred.get("block").unwrap_or(pred);
            if let Some(b_val) = blk.get("blocks").or_else(|| blk.get("block")) {
                if let Some(s) = b_val.as_str() {
                    block_patterns.push(s.to_string());
                } else if let Some(arr) = b_val.as_array() {
                    for b in arr {
                        if let Some(s) = b.as_str() {
                            block_patterns.push(s.to_string());
                        }
                    }
                }
            }
            if let Some(st) = blk.get("state").and_then(Value::as_object) {
                for (k, v) in st {
                    state_key = Some(k.clone());
                    state_value = v.as_str().map(ToString::to_string);
                }
            }
            if let Some(sm) = pred.get("smokey").and_then(Value::as_bool) {
                smokey = Some(sm);
            }
            if let Some(b) = pred
                .get("biomes")
                .or_else(|| pred.get("biome"))
                .and_then(Value::as_str)
            {
                biome = Some(b.to_string());
            }
            if let Some(i_val) = pred.get("items").or_else(|| pred.get("item")) {
                if let Some(s) = i_val.as_str() {
                    item_patterns.push(s.to_string());
                } else if let Some(arr) = i_val.as_array() {
                    for i in arr {
                        if let Some(s) = i.as_str() {
                            item_patterns.push(s.to_string());
                        }
                    }
                }
            }
        }

        Self {
            blocks: (!block_patterns.is_empty()).then_some(TagOrIdMatcher {
                patterns: block_patterns,
            }),
            items: (!item_patterns.is_empty()).then_some(TagOrIdMatcher {
                patterns: item_patterns,
            }),
            state_key,
            state_value,
            smokey,
            biome,
        }
    }
}

/// Strongly-typed entity condition.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct EntityPredicate {
    pub entity_type: Option<TagOrIdMatcher>,
    pub distance_horizontal: Option<FloatBounds>,
    pub distance_absolute: Option<FloatBounds>,
    pub is_baby: Option<bool>,
    pub vehicle_type: Option<TagOrIdMatcher>,
    pub passenger_type: Option<TagOrIdMatcher>,
    pub stepping_on: Option<String>,
    pub looking_at: Option<TagOrIdMatcher>,
    pub feet_item: Option<TagOrIdMatcher>,
    pub safe_armor_disallowed: Option<bool>,
}

impl EntityPredicate {
    #[must_use]
    pub fn matches(&self, entity_type: &str, distance: Option<f64>, is_baby: Option<bool>) -> bool {
        if let Some(types) = &self.entity_type
            && !types.matches(RegistryKey::EntityType, entity_type)
        {
            return false;
        }
        if let Some(dist_bounds) = &self.distance_horizontal {
            let Some(d) = distance else {
                return false;
            };
            if !dist_bounds.matches(d) {
                return false;
            }
        }
        if let Some(dist_bounds) = &self.distance_absolute {
            let Some(d) = distance else {
                return false;
            };
            if !dist_bounds.matches(d) {
                return false;
            }
        }
        if let Some(expected_baby) = self.is_baby
            && is_baby != Some(expected_baby)
        {
            return false;
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let pred_val = val.get("predicate").unwrap_or(val);
        let Value::Object(map) = pred_val else {
            return Self::default();
        };

        let entity_type = map
            .get("minecraft:entity_type")
            .or_else(|| {
                map.get("type").filter(|t| {
                    t.as_str().is_none_or(|s| {
                        !matches!(
                            s,
                            "minecraft:all_of"
                                | "minecraft:any_of"
                                | "minecraft:entity_properties"
                                | "minecraft:inverted"
                                | "minecraft:location_check"
                                | "minecraft:match_block"
                                | "minecraft:match_tool"
                        )
                    })
                })
            })
            .and_then(TagOrIdMatcher::from_value);

        let (distance_horizontal, distance_absolute) = map
            .get("minecraft:distance")
            .or_else(|| map.get("distance"))
            .map_or((None, None), |dist_val| {
                (
                    dist_val.get("horizontal").and_then(FloatBounds::from_value),
                    dist_val.get("absolute").and_then(FloatBounds::from_value),
                )
            });

        let is_baby = map
            .get("minecraft:flags")
            .or_else(|| map.get("flags"))
            .and_then(|f| f.get("is_baby"))
            .and_then(Value::as_bool);

        let (vehicle_type, passenger_type) = map
            .get("minecraft:vehicle")
            .or_else(|| map.get("vehicle"))
            .map_or((None, None), |v| {
                let vt = v
                    .get("minecraft:entity_type")
                    .or_else(|| v.get("type"))
                    .and_then(TagOrIdMatcher::from_value);
                let pt = v
                    .get("minecraft:passenger")
                    .or_else(|| v.get("passenger"))
                    .and_then(|p| p.get("minecraft:entity_type").or_else(|| p.get("type")))
                    .and_then(TagOrIdMatcher::from_value);
                (vt, pt)
            });

        let stepping_on = map
            .get("minecraft:stepping_on")
            .or_else(|| map.get("stepping_on"))
            .and_then(|s| s.get("block").or_else(|| s.get("blocks")))
            .and_then(|b| b.get("blocks").or(Some(b)))
            .and_then(Value::as_str)
            .map(ToString::to_string);

        let looking_at = map
            .get("minecraft:type_specific/player")
            .and_then(|p| p.get("looking_at"))
            .and_then(|l| l.get("minecraft:entity_type").or_else(|| l.get("type")))
            .and_then(TagOrIdMatcher::from_value);

        let feet_item = map
            .get("minecraft:equipment")
            .or_else(|| map.get("equipment"))
            .and_then(|eq| eq.get("feet"))
            .and_then(|f| f.get("items"))
            .and_then(TagOrIdMatcher::from_value);

        let val_str = val.to_string();
        let safe_armor_disallowed = (val_str.contains("piglin_safe_armor")
            && (val_str.contains("inverted") || val_str.contains("minecraft:inverted")))
        .then_some(true);

        Self {
            entity_type,
            distance_horizontal,
            distance_absolute,
            is_baby,
            vehicle_type,
            passenger_type,
            stepping_on,
            looking_at,
            feet_item,
            safe_armor_disallowed,
        }
    }
}

/// Strongly-typed damage condition.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DamagePredicate {
    pub blocked: Option<bool>,
    pub deflector: Option<bool>,
    pub dealt: Option<FloatBounds>,
    pub taken: Option<FloatBounds>,
}

impl DamagePredicate {
    #[must_use]
    pub fn matches(&self, is_blocked: bool, is_deflector: bool, damage: Option<f64>) -> bool {
        if let Some(blocked) = self.blocked
            && blocked != is_blocked
        {
            return false;
        }
        if let Some(deflector) = self.deflector
            && deflector != is_deflector
        {
            return false;
        }
        if let Some(dealt) = &self.dealt {
            let Some(d) = damage else {
                return false;
            };
            if !dealt.matches(d) {
                return false;
            }
        }
        if let Some(taken) = &self.taken {
            let Some(d) = damage else {
                return false;
            };
            if !taken.matches(d) {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };

        let damage_obj = map.get("damage").unwrap_or(val);
        let blocked = damage_obj.get("blocked").and_then(Value::as_bool);
        let deflector = damage_obj
            .get("type")
            .and_then(|t| t.get("is_projectile"))
            .and_then(Value::as_bool)
            .or_else(|| {
                damage_obj
                    .get("type")
                    .and_then(|t| t.get("tags"))
                    .and_then(Value::as_array)
                    .map(|tags| {
                        tags.iter().any(|entry| {
                            entry
                                .get("id")
                                .and_then(Value::as_str)
                                .is_some_and(|id| id.contains("projectile"))
                        })
                    })
            });
        let dealt = damage_obj.get("dealt").and_then(FloatBounds::from_value);
        let taken = damage_obj.get("taken").and_then(FloatBounds::from_value);

        Self {
            blocked,
            deflector,
            dealt,
            taken,
        }
    }
}

/// Strongly-typed dimension change condition.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DimensionPredicate {
    pub from: Option<String>,
    pub to: Option<String>,
}

impl DimensionPredicate {
    #[must_use]
    pub fn matches(&self, from: Option<&str>, to: &str) -> bool {
        if let Some(expected_from) = &self.from {
            let Some(f) = from else {
                return false;
            };
            if !matches_id(expected_from, f) {
                return false;
            }
        }
        if let Some(expected_to) = &self.to
            && !matches_id(expected_to, to)
        {
            return false;
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        let from = map
            .get("from")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        let to = map
            .get("to")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        Self { from, to }
    }
}

/// Strongly-typed placed block condition.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PlacedBlockPredicate {
    pub block: Option<TagOrIdMatcher>,
    pub item: Option<ItemPredicate>,
}

impl PlacedBlockPredicate {
    #[must_use]
    pub fn matches(&self, block_id: &str, item_id: Option<&str>) -> bool {
        if let Some(b) = &self.block
            && !b.matches(RegistryKey::Block, block_id)
        {
            return false;
        }
        if let Some(item_pred) = &self.item {
            let Some(actual_item) = item_id else {
                return false;
            };
            if !item_pred.matches(actual_item, 1, &[]) {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        let block = map
            .get("block")
            .or_else(|| map.get("blocks"))
            .and_then(TagOrIdMatcher::from_value);
        let item = map.get("item").map(ItemPredicate::from_value);
        Self { block, item }
    }
}

/// Strongly-typed target hit condition.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TargetHitPredicate {
    pub signal_strength: Option<IntBounds>,
    pub distance: Option<FloatBounds>,
}

impl TargetHitPredicate {
    #[must_use]
    pub fn matches(&self, signal: i32, distance: Option<f64>) -> bool {
        if let Some(sig) = &self.signal_strength
            && !sig.matches(i64::from(signal))
        {
            return false;
        }
        if let Some(dist) = &self.distance {
            let Some(d) = distance else {
                return false;
            };
            if !dist.matches(d) {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        let signal_strength = map.get("signal_strength").and_then(IntBounds::from_value);
        let mut distance = None;
        if let Some(proj_val) = map.get("projectile") {
            let pred = proj_val.get("predicate").unwrap_or(proj_val);
            if let Some(dist_val) = pred
                .get("minecraft:distance")
                .or_else(|| pred.get("distance"))
            {
                distance = dist_val.get("horizontal").and_then(FloatBounds::from_value);
            }
        }
        Self {
            signal_strength,
            distance,
        }
    }
}

/// Strongly-typed beacon construction condition.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BeaconPredicate {
    pub level: Option<IntBounds>,
}

impl BeaconPredicate {
    #[must_use]
    pub fn matches(&self, level: i32) -> bool {
        self.level
            .as_ref()
            .is_none_or(|b| b.matches(i64::from(level)))
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        Self {
            level: map.get("level").and_then(IntBounds::from_value),
        }
    }
}

/// Strongly-typed animal breeding condition.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct BredAnimalsPredicate {
    pub parent: Option<EntityPredicate>,
    pub partner: Option<EntityPredicate>,
    pub child: Option<EntityPredicate>,
}

impl BredAnimalsPredicate {
    #[must_use]
    pub fn matches(&self, entity_type: &str) -> bool {
        if let Some(p) = &self.parent
            && p.matches(entity_type, None, None)
        {
            return true;
        }
        if let Some(p) = &self.partner
            && p.matches(entity_type, None, None)
        {
            return true;
        }
        if let Some(c) = &self.child
            && c.matches(entity_type, None, None)
        {
            return true;
        }
        self.parent.is_none() && self.partner.is_none() && self.child.is_none()
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        let parent = map
            .get("parent")
            .map(|p| EntityPredicate::from_value(p.get("predicate").unwrap_or(p)));
        let partner = map
            .get("partner")
            .map(|p| EntityPredicate::from_value(p.get("predicate").unwrap_or(p)));
        let child = map
            .get("child")
            .map(|p| EntityPredicate::from_value(p.get("predicate").unwrap_or(p)));
        Self {
            parent,
            partner,
            child,
        }
    }
}

/// Strongly-typed recipe unlocked/crafted condition.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RecipePredicate {
    pub recipes: Vec<String>,
}

impl RecipePredicate {
    #[must_use]
    pub fn matches(&self, recipe_id: &str) -> bool {
        if self.recipes.is_empty() {
            return true;
        }
        self.recipes.iter().any(|r| matches_id(r, recipe_id))
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        let mut recipes = Vec::new();
        if let Some(r_val) = map.get("recipes").or_else(|| map.get("recipe")) {
            match r_val {
                Value::String(s) => recipes.push(s.clone()),
                Value::Array(arr) => {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            recipes.push(s.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
        Self { recipes }
    }
}

/// Strongly-typed criterion conditions for all advancement trigger types.
#[derive(Clone, Debug, PartialEq)]
pub enum CriterionConditions {
    InventoryChanged(InventoryChangedPredicate),
    Location(LocationPredicate),
    PlayerKilledEntity(EntityPredicate),
    KilledByArrow,
    ChangedDimension(DimensionPredicate),
    SleptInBed,
    ConsumeItem(ItemPredicate),
    PlacedBlock(PlacedBlockPredicate),
    EnterBlock(TagOrIdMatcher),
    SlideDownBlock(TagOrIdMatcher),
    FishingRodHooked(ItemPredicate),
    FilledBucket(ItemPredicate),
    UsingItem {
        item: ItemPredicate,
        looking_at: Option<TagOrIdMatcher>,
    },
    UsedTotem(ItemPredicate),
    EntityHurtPlayer(DamagePredicate),
    EntityKilledPlayer,
    PlayerHurtEntity(DamagePredicate),
    BredAnimals(BredAnimalsPredicate),
    UsedEnderEye,
    ItemUsedOnBlock(ItemUsedOnBlockPredicate),
    TargetHit(TargetHitPredicate),
    CuredZombieVillager,
    VillagerTrade,
    AllayDropItemOnBlock(ItemUsedOnBlockPredicate),
    ThrownItemPickedUpByPlayer(EntityPredicate),
    TameAnimal(EntityPredicate),
    ConstructBeacon(BeaconPredicate),
    RecipeCrafted(RecipePredicate),
    RecipeUnlocked(RecipePredicate),
    PlayerGeneratesContainerLoot(String),
    EnchantedItem,
    BrewedPotion,
    ItemDurabilityChanged {
        item: ItemPredicate,
        vehicle_type: Option<TagOrIdMatcher>,
    },
    Levitation {
        distance_y: Option<FloatBounds>,
    },
    FallFromHeight {
        distance_y: Option<FloatBounds>,
    },
    FallAfterExplosion {
        distance_y: Option<FloatBounds>,
        cause_entity: Option<String>,
    },
    NetherTravel {
        distance_horizontal: Option<FloatBounds>,
    },
    RideEntityInLava {
        distance_horizontal: Option<FloatBounds>,
        dimension: Option<String>,
        vehicle_type: Option<TagOrIdMatcher>,
    },
    ShotCrossbow(ItemPredicate),
    AvoidVibration,
    BeeNestDestroyed {
        block: Option<String>,
        num_bees_inside: Option<i32>,
    },
    ChanneledLightning {
        victim_type: Option<String>,
    },
    LightningStrike {
        bystander_type: Option<String>,
        blocks_set_on_fire: Option<i32>,
        distance_max: Option<f64>,
    },
    KillMobNearSculkCatalyst,
    HeroOfTheVillage,
    PlayerInteractedWithEntity {
        entity: Option<EntityPredicate>,
        item: Option<ItemPredicate>,
        player: Option<EntityPredicate>,
    },
    ThrownItemPickedUpByEntity {
        entity: Option<EntityPredicate>,
        item: Option<ItemPredicate>,
        player: Option<EntityPredicate>,
    },
    PlayerShearedEquipment {
        entity: Option<EntityPredicate>,
        item: Option<ItemPredicate>,
    },
    StartedRiding(Option<EntityPredicate>),
    SummonedEntity(EntityPredicate),
    CrafterRecipeCrafted(String),
    EffectsChanged,
    SpearMobs {
        count: Option<i32>,
    },
    Impossible,
    Tick,
    Custom {
        trigger: String,
        conditions: Value,
    },
}

#[expect(clippy::too_many_lines, reason = "converting from data enum")]
impl From<pumpkin_data::advancement_data::AdvancementCondition> for CriterionConditions {
    fn from(cond: pumpkin_data::advancement_data::AdvancementCondition) -> Self {
        use pumpkin_data::advancement_data::AdvancementCondition as AC;
        match cond {
            AC::InventoryChanged(ic) => {
                let items =
                    ic.items
                        .iter()
                        .map(|item| ItemPredicate {
                            items: if item.items.is_empty() && item.tag.is_none() {
                                None
                            } else {
                                let mut patterns = Vec::new();
                                if let Some(t) = item.tag {
                                    patterns.push(format!("#{t}"));
                                }
                                for i in item.items {
                                    patterns.push((*i).to_string());
                                }
                                Some(TagOrIdMatcher { patterns })
                            },
                            count: (item.count_min.is_some() || item.count_max.is_some())
                                .then_some(IntBounds {
                                    min: item.count_min,
                                    max: item.count_max,
                                }),
                            durability: None,
                            enchantments: item
                                .enchantments
                                .iter()
                                .map(|e| EnchantmentPredicate {
                                    enchantment: e.enchantment.to_string(),
                                    min_level: e.min_level,
                                })
                                .collect(),
                        })
                        .collect();
                let slots = ic.slots.map(|s| InventorySlotsPredicate {
                    empty: (s.empty_min.is_some() || s.empty_max.is_some()).then_some(IntBounds {
                        min: s.empty_min,
                        max: s.empty_max,
                    }),
                    full: (s.full_min.is_some() || s.full_max.is_some()).then_some(IntBounds {
                        min: s.full_min,
                        max: s.full_max,
                    }),
                    occupied: (s.occupied_min.is_some() || s.occupied_max.is_some()).then_some(
                        IntBounds {
                            min: s.occupied_min,
                            max: s.occupied_max,
                        },
                    ),
                });
                Self::InventoryChanged(InventoryChangedPredicate { items, slots })
            }
            AC::RecipeUnlocked(r) => Self::RecipeUnlocked(RecipePredicate {
                recipes: vec![r.to_string()],
            }),
            AC::RecipeCrafted(r) => Self::RecipeCrafted(RecipePredicate {
                recipes: vec![r.to_string()],
            }),
            AC::PlayerGeneratesContainerLoot(lt) => {
                Self::PlayerGeneratesContainerLoot(lt.to_string())
            }
            AC::ChangedDimension { from, to } => Self::ChangedDimension(DimensionPredicate {
                from: from.map(ToString::to_string),
                to: to.map(ToString::to_string),
            }),
            AC::ConsumeItem(item) => Self::ConsumeItem(ItemPredicate {
                items: Some(TagOrIdMatcher {
                    patterns: item.items.iter().map(ToString::to_string).collect(),
                }),
                count: None,
                durability: None,
                enchantments: Vec::new(),
            }),
            AC::PlacedBlock { block, item } => Self::PlacedBlock(PlacedBlockPredicate {
                block: block.map(|b| TagOrIdMatcher {
                    patterns: vec![b.to_string()],
                }),
                item: item.map(|i| ItemPredicate {
                    items: Some(TagOrIdMatcher {
                        patterns: vec![i.to_string()],
                    }),
                    count: None,
                    durability: None,
                    enchantments: Vec::new(),
                }),
            }),
            AC::EnterBlock(b) => Self::EnterBlock(TagOrIdMatcher {
                patterns: vec![b.to_string()],
            }),
            AC::SlideDownBlock(b) => Self::SlideDownBlock(TagOrIdMatcher {
                patterns: vec![b.to_string()],
            }),
            AC::PlayerKilledEntity(e) => Self::PlayerKilledEntity(from_entity_condition(&e)),
            AC::BredAnimals(b) => Self::BredAnimals(BredAnimalsPredicate {
                parent: b.parent.as_ref().map(from_entity_condition),
                partner: b.partner.as_ref().map(from_entity_condition),
                child: b.child.as_ref().map(from_entity_condition),
            }),
            AC::TameAnimal(e) => Self::TameAnimal(from_entity_condition(&e)),
            AC::FilledBucket(item) => Self::FilledBucket(ItemPredicate {
                items: Some(TagOrIdMatcher {
                    patterns: vec![item.to_string()],
                }),
                count: None,
                durability: None,
                enchantments: Vec::new(),
            }),
            AC::FishingRodHooked(item) => Self::FishingRodHooked(ItemPredicate {
                items: Some(TagOrIdMatcher {
                    patterns: item.items.iter().map(ToString::to_string).collect(),
                }),
                count: None,
                durability: None,
                enchantments: Vec::new(),
            }),
            AC::UsingItem { item, looking_at } => Self::UsingItem {
                item: ItemPredicate {
                    items: (!item.items.is_empty() || item.tag.is_some()).then_some(
                        TagOrIdMatcher {
                            patterns: item
                                .items
                                .iter()
                                .map(ToString::to_string)
                                .chain(item.tag.map(|t| {
                                    if t.starts_with('#') {
                                        t.to_string()
                                    } else {
                                        format!("#{t}")
                                    }
                                }))
                                .collect(),
                        },
                    ),
                    count: None,
                    durability: None,
                    enchantments: Vec::new(),
                },
                looking_at: looking_at.map(|l| TagOrIdMatcher {
                    patterns: vec![l.to_string()],
                }),
            },
            AC::UsedTotem(item) => Self::UsedTotem(ItemPredicate {
                items: Some(TagOrIdMatcher {
                    patterns: item.items.iter().map(ToString::to_string).collect(),
                }),
                count: None,
                durability: None,
                enchantments: Vec::new(),
            }),
            AC::EntityHurtPlayer(d) => Self::EntityHurtPlayer(DamagePredicate {
                blocked: d.blocked,
                deflector: d.is_projectile,
                dealt: d.dealt_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
                taken: d.taken_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
            }),
            AC::PlayerHurtEntity(d) => Self::PlayerHurtEntity(DamagePredicate {
                blocked: d.blocked,
                deflector: d.is_projectile,
                dealt: d.dealt_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
                taken: d.taken_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
            }),
            AC::TargetHit(th) => Self::TargetHit(TargetHitPredicate {
                signal_strength: th.signal_strength.map(|sig| IntBounds {
                    min: Some(i64::from(sig)),
                    max: None,
                }),
                distance: th.distance_min.map(|dist| FloatBounds {
                    min: Some(dist),
                    max: None,
                }),
            }),
            AC::ConstructBeacon(b) => Self::ConstructBeacon(BeaconPredicate {
                level: b.level_min.map(|lvl| IntBounds {
                    min: Some(i64::from(lvl)),
                    max: None,
                }),
            }),
            AC::AllayDropItemOnBlock(c) => {
                Self::AllayDropItemOnBlock(from_item_used_on_block_condition(c))
            }
            AC::ThrownItemPickedUpByPlayer(e) => {
                Self::ThrownItemPickedUpByPlayer(from_entity_condition(&e))
            }
            AC::ItemUsedOnBlock(c) => Self::ItemUsedOnBlock(from_item_used_on_block_condition(c)),
            AC::Location(l) => Self::Location(LocationPredicate {
                x: None,
                y: None,
                z: None,
                biome: l.biome.map(|b| TagOrIdMatcher {
                    patterns: vec![b.to_string()],
                }),
                dimension: l.dimension.map(ToString::to_string),
                block: l.block.map(|b| TagOrIdMatcher {
                    patterns: vec![b.to_string()],
                }),
                can_see_sky: l.can_see_sky,
                smokey: l.smokey,
                structure: l.structure.map(ToString::to_string),
                stepping_on: l.stepping_on.map(|b| TagOrIdMatcher {
                    patterns: vec![b.to_string()],
                }),
                feet_item: l.feet_item.map(|f| TagOrIdMatcher {
                    patterns: vec![f.to_string()],
                }),
            }),
            AC::KilledByArrow => Self::KilledByArrow,
            AC::SleptInBed => Self::SleptInBed,
            AC::EntityKilledPlayer => Self::EntityKilledPlayer,
            AC::UsedEnderEye => Self::UsedEnderEye,
            AC::CuredZombieVillager => Self::CuredZombieVillager,
            AC::VillagerTrade => Self::VillagerTrade,
            AC::EnchantedItem => Self::EnchantedItem,
            AC::BrewedPotion => Self::BrewedPotion,
            AC::ItemDurabilityChanged { item, vehicle_type } => Self::ItemDurabilityChanged {
                item: ItemPredicate {
                    items: (!item.items.is_empty() || item.tag.is_some()).then_some(
                        TagOrIdMatcher {
                            patterns: item
                                .items
                                .iter()
                                .map(ToString::to_string)
                                .chain(item.tag.map(|t| {
                                    if t.starts_with('#') {
                                        t.to_string()
                                    } else {
                                        format!("#{t}")
                                    }
                                }))
                                .collect(),
                        },
                    ),
                    count: None,
                    durability: None,
                    enchantments: Vec::new(),
                },
                vehicle_type: vehicle_type.map(|v| TagOrIdMatcher {
                    patterns: vec![v.to_string()],
                }),
            },
            AC::Levitation { distance_y_min } => Self::Levitation {
                distance_y: distance_y_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
            },
            AC::FallFromHeight { distance_y_min } => Self::FallFromHeight {
                distance_y: distance_y_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
            },
            AC::FallAfterExplosion {
                distance_y_min,
                cause_entity,
            } => Self::FallAfterExplosion {
                distance_y: distance_y_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
                cause_entity: cause_entity.map(ToString::to_string),
            },
            AC::NetherTravel {
                distance_horizontal_min,
            } => Self::NetherTravel {
                distance_horizontal: distance_horizontal_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
            },
            AC::RideEntityInLava {
                distance_horizontal_min,
                dimension,
                vehicle_type,
            } => Self::RideEntityInLava {
                distance_horizontal: distance_horizontal_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
                dimension: dimension.map(ToString::to_string),
                vehicle_type: vehicle_type.map(|v| TagOrIdMatcher {
                    patterns: vec![v.to_string()],
                }),
            },
            AC::ShotCrossbow(item) => Self::ShotCrossbow(ItemPredicate {
                items: Some(TagOrIdMatcher {
                    patterns: item.items.iter().map(ToString::to_string).collect(),
                }),
                count: None,
                durability: None,
                enchantments: Vec::new(),
            }),
            AC::AvoidVibration => Self::AvoidVibration,
            AC::BeeNestDestroyed {
                block,
                num_bees_inside,
            } => Self::BeeNestDestroyed {
                block: block.map(ToString::to_string),
                num_bees_inside,
            },
            AC::ChanneledLightning { victim_type } => Self::ChanneledLightning {
                victim_type: victim_type.map(ToString::to_string),
            },
            AC::LightningStrike {
                bystander_type,
                blocks_set_on_fire,
                distance_max,
            } => Self::LightningStrike {
                bystander_type: bystander_type.map(ToString::to_string),
                blocks_set_on_fire,
                distance_max,
            },
            AC::KillMobNearSculkCatalyst => Self::KillMobNearSculkCatalyst,
            AC::HeroOfTheVillage => Self::HeroOfTheVillage,
            AC::PlayerInteractedWithEntity {
                entity,
                item,
                player,
            } => Self::PlayerInteractedWithEntity {
                entity: entity.as_ref().map(from_entity_condition),
                item: item.as_ref().map(from_item_condition),
                player: player.as_ref().map(from_entity_condition),
            },
            AC::ThrownItemPickedUpByEntity {
                entity,
                item,
                player,
            } => Self::ThrownItemPickedUpByEntity {
                entity: entity.as_ref().map(from_entity_condition),
                item: item.as_ref().map(from_item_condition),
                player: player.as_ref().map(from_entity_condition),
            },
            AC::PlayerShearedEquipment { entity, item } => Self::PlayerShearedEquipment {
                entity: entity.as_ref().map(from_entity_condition),
                item: item.as_ref().map(from_item_condition),
            },
            AC::StartedRiding(p) => Self::StartedRiding(p.as_ref().map(from_entity_condition)),
            AC::SummonedEntity(e) => Self::SummonedEntity(from_entity_condition(&e)),
            AC::CrafterRecipeCrafted(r) => Self::CrafterRecipeCrafted(r.to_string()),
            AC::EffectsChanged => Self::EffectsChanged,
            AC::SpearMobs { count } => Self::SpearMobs { count },
            AC::Impossible => Self::Impossible,
            AC::Empty | AC::Tick => Self::Tick,
            AC::Custom(raw) => Self::parse("", raw),
        }
    }
}

fn from_entity_condition(e: &pumpkin_data::advancement_data::EntityCondition) -> EntityPredicate {
    let mut patterns = Vec::new();
    if let Some(t) = e.entity_type {
        patterns.push(t.to_string());
    }
    if let Some(t) = e.tag {
        let tag_str = if t.starts_with('#') {
            t.to_string()
        } else {
            format!("#{t}")
        };
        patterns.push(tag_str);
    }
    let mut vpatterns = Vec::new();
    if let Some(vt) = e.vehicle_type {
        vpatterns.push(vt.to_string());
    }
    if let Some(vt) = e.vehicle_tag {
        let tag_str = if vt.starts_with('#') {
            vt.to_string()
        } else {
            format!("#{vt}")
        };
        vpatterns.push(tag_str);
    }
    let passenger_type = e.passenger_type.map(|pt| TagOrIdMatcher {
        patterns: vec![pt.to_string()],
    });
    let looking_at = e.looking_at.map(|l| TagOrIdMatcher {
        patterns: vec![l.to_string()],
    });
    let feet_item = e.feet_item.map(|f| TagOrIdMatcher {
        patterns: vec![f.to_string()],
    });

    EntityPredicate {
        entity_type: (!patterns.is_empty()).then_some(TagOrIdMatcher { patterns }),
        distance_horizontal: e.distance_horizontal_min.map(|min| FloatBounds {
            min: Some(min),
            max: None,
        }),
        distance_absolute: e.distance_absolute_min.map(|min| FloatBounds {
            min: Some(min),
            max: None,
        }),
        is_baby: e.is_baby,
        vehicle_type: (!vpatterns.is_empty()).then_some(TagOrIdMatcher {
            patterns: vpatterns,
        }),
        passenger_type,
        stepping_on: e.stepping_on.map(ToString::to_string),
        looking_at,
        feet_item,
        safe_armor_disallowed: e.safe_armor_disallowed,
    }
}

fn from_item_used_on_block_condition(
    c: &pumpkin_data::advancement_data::ItemUsedOnBlockCondition,
) -> ItemUsedOnBlockPredicate {
    let mut block_patterns: Vec<String> = c.blocks.iter().map(ToString::to_string).collect();
    if let Some(tag) = c.block_tag {
        let tag_str = if tag.starts_with('#') {
            tag.to_string()
        } else {
            format!("#{tag}")
        };
        block_patterns.push(tag_str);
    }
    let mut item_patterns: Vec<String> = c.items.iter().map(ToString::to_string).collect();
    if let Some(tag) = c.item_tag {
        let tag_str = if tag.starts_with('#') {
            tag.to_string()
        } else {
            format!("#{tag}")
        };
        item_patterns.push(tag_str);
    }
    ItemUsedOnBlockPredicate {
        blocks: (!block_patterns.is_empty()).then_some(TagOrIdMatcher {
            patterns: block_patterns,
        }),
        items: (!item_patterns.is_empty()).then_some(TagOrIdMatcher {
            patterns: item_patterns,
        }),
        state_key: c.state_key.map(ToString::to_string),
        state_value: c.state_value.map(ToString::to_string),
        smokey: c.smokey,
        biome: c.biome.map(ToString::to_string),
    }
}

fn from_item_condition(i: &pumpkin_data::advancement_data::ItemCondition) -> ItemPredicate {
    let mut patterns: Vec<String> = i.items.iter().map(ToString::to_string).collect();
    if let Some(t) = i.tag {
        let tag_str = if t.starts_with('#') {
            t.to_string()
        } else {
            format!("#{t}")
        };
        patterns.push(tag_str);
    }
    ItemPredicate {
        items: if patterns.is_empty() {
            None
        } else {
            Some(TagOrIdMatcher { patterns })
        },
        count: (i.count_min.is_some() || i.count_max.is_some()).then_some(IntBounds {
            min: i.count_min,
            max: i.count_max,
        }),
        durability: (i.durability_min.is_some() || i.durability_max.is_some()).then_some(
            IntBounds {
                min: i.durability_min,
                max: i.durability_max,
            },
        ),
        enchantments: Vec::new(),
    }
}

fn parse_entity_pred(val: &Value) -> EntityPredicate {
    val.get("predicate").map_or_else(
        || {
            val.get("entity").map_or_else(
                || EntityPredicate::from_value(val),
                |p| {
                    if p.is_object() {
                        EntityPredicate::from_value(p.get("predicate").unwrap_or(p))
                    } else {
                        EntityPredicate::from_value(val)
                    }
                },
            )
        },
        EntityPredicate::from_value,
    )
}

fn parse_item_pred(val: &Value) -> ItemPredicate {
    val.get("item")
        .map_or_else(|| ItemPredicate::from_value(val), ItemPredicate::from_value)
}

impl CriterionConditions {
    #[must_use]
    #[expect(clippy::too_many_lines)]
    pub fn parse(trigger: &str, json_str: &str) -> Self {
        let val: Value = serde_json::from_str(json_str).unwrap_or(Value::Null);
        match trigger {
            "minecraft:inventory_changed" => {
                Self::InventoryChanged(InventoryChangedPredicate::from_value(&val))
            }
            "minecraft:location" => Self::Location(LocationPredicate::from_value(&val)),
            "minecraft:player_killed_entity" => Self::PlayerKilledEntity(parse_entity_pred(&val)),
            "minecraft:killed_by_arrow" => Self::KilledByArrow,
            "minecraft:changed_dimension" => {
                Self::ChangedDimension(DimensionPredicate::from_value(&val))
            }
            "minecraft:slept_in_bed" => Self::SleptInBed,
            "minecraft:consume_item" => Self::ConsumeItem(parse_item_pred(&val)),
            "minecraft:placed_block" => Self::PlacedBlock(PlacedBlockPredicate::from_value(&val)),
            "minecraft:enter_block" => {
                let block = val
                    .get("blocks")
                    .or_else(|| val.get("block"))
                    .and_then(TagOrIdMatcher::from_value)
                    .unwrap_or_default();
                Self::EnterBlock(block)
            }
            "minecraft:slide_down_block" => {
                let block = val
                    .get("blocks")
                    .or_else(|| val.get("block"))
                    .and_then(TagOrIdMatcher::from_value)
                    .unwrap_or_default();
                Self::SlideDownBlock(block)
            }
            "minecraft:fishing_rod_hooked" => Self::FishingRodHooked(parse_item_pred(&val)),
            "minecraft:filled_bucket" => Self::FilledBucket(parse_item_pred(&val)),
            "minecraft:using_item" => {
                let looking_at = val
                    .get("player")
                    .and_then(|p| p.get("predicate"))
                    .and_then(|pred| pred.get("minecraft:type_specific/player"))
                    .and_then(|p| p.get("looking_at"))
                    .and_then(|l| l.get("minecraft:entity_type").or_else(|| l.get("type")))
                    .and_then(TagOrIdMatcher::from_value);
                Self::UsingItem {
                    item: parse_item_pred(&val),
                    looking_at,
                }
            }
            "minecraft:used_totem" => Self::UsedTotem(parse_item_pred(&val)),
            "minecraft:entity_hurt_player" => {
                Self::EntityHurtPlayer(DamagePredicate::from_value(&val))
            }
            "minecraft:entity_killed_player" => Self::EntityKilledPlayer,
            "minecraft:player_hurt_entity" => {
                Self::PlayerHurtEntity(DamagePredicate::from_value(&val))
            }
            "minecraft:bred_animals" => Self::BredAnimals(BredAnimalsPredicate::from_value(&val)),
            "minecraft:used_ender_eye" => Self::UsedEnderEye,
            "minecraft:item_used_on_block" => {
                Self::ItemUsedOnBlock(ItemUsedOnBlockPredicate::from_value(&val))
            }
            "minecraft:target_hit" => Self::TargetHit(TargetHitPredicate::from_value(&val)),
            "minecraft:cured_zombie_villager" => Self::CuredZombieVillager,
            "minecraft:villager_trade" => Self::VillagerTrade,
            "minecraft:allay_drop_item_on_block" => {
                Self::AllayDropItemOnBlock(ItemUsedOnBlockPredicate::from_value(&val))
            }
            "minecraft:thrown_item_picked_up_by_player" => {
                Self::ThrownItemPickedUpByPlayer(parse_entity_pred(&val))
            }
            "minecraft:tame_animal" => Self::TameAnimal(parse_entity_pred(&val)),
            "minecraft:construct_beacon" => {
                Self::ConstructBeacon(BeaconPredicate::from_value(&val))
            }
            "minecraft:recipe_crafted" => Self::RecipeCrafted(RecipePredicate::from_value(&val)),
            "minecraft:recipe_unlocked" => Self::RecipeUnlocked(RecipePredicate::from_value(&val)),
            "minecraft:player_generates_container_loot" => Self::PlayerGeneratesContainerLoot(
                val.get("loot_tables")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            "minecraft:enchanted_item" => Self::EnchantedItem,
            "minecraft:brewed_potion" => Self::BrewedPotion,
            "minecraft:item_durability_changed" => {
                let vehicle_type = val
                    .get("player")
                    .and_then(|p| p.get("predicate"))
                    .and_then(|pred| {
                        pred.get("minecraft:vehicle")
                            .or_else(|| pred.get("vehicle"))
                    })
                    .and_then(|v| v.get("minecraft:entity_type").or_else(|| v.get("type")))
                    .and_then(TagOrIdMatcher::from_value);
                Self::ItemDurabilityChanged {
                    item: parse_item_pred(&val),
                    vehicle_type,
                }
            }
            "minecraft:levitation" => Self::Levitation {
                distance_y: val
                    .get("distance")
                    .and_then(|d| d.get("y"))
                    .and_then(|y| y.get("min"))
                    .and_then(Value::as_f64)
                    .map(|min| FloatBounds {
                        min: Some(min),
                        max: None,
                    }),
            },
            "minecraft:fall_from_height" => Self::FallFromHeight {
                distance_y: val
                    .get("distance")
                    .and_then(|d| d.get("y"))
                    .and_then(|y| y.get("min"))
                    .and_then(Value::as_f64)
                    .map(|min| FloatBounds {
                        min: Some(min),
                        max: None,
                    }),
            },
            "minecraft:fall_after_explosion" => Self::FallAfterExplosion {
                distance_y: val
                    .get("distance")
                    .and_then(|d| d.get("y"))
                    .and_then(|y| y.get("min"))
                    .and_then(Value::as_f64)
                    .map(|min| FloatBounds {
                        min: Some(min),
                        max: None,
                    }),
                cause_entity: val
                    .get("cause")
                    .and_then(|c| c.get("predicate"))
                    .and_then(|p| {
                        p.get("minecraft:entity_type")
                            .or_else(|| p.get("entity_type"))
                    })
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string),
            },
            "minecraft:nether_travel" => Self::NetherTravel {
                distance_horizontal: val
                    .get("distance")
                    .and_then(|d| d.get("horizontal"))
                    .and_then(|h| h.get("min"))
                    .and_then(Value::as_f64)
                    .map(|min| FloatBounds {
                        min: Some(min),
                        max: None,
                    }),
            },
            "minecraft:ride_entity_in_lava" => {
                let distance_horizontal = val
                    .get("distance")
                    .and_then(|d| d.get("horizontal"))
                    .and_then(|h| h.get("min"))
                    .and_then(Value::as_f64)
                    .map(|min| FloatBounds {
                        min: Some(min),
                        max: None,
                    });
                let dimension = val
                    .get("player")
                    .and_then(|p| p.get("predicate"))
                    .and_then(|pred| {
                        pred.get("minecraft:location")
                            .or_else(|| pred.get("location"))
                    })
                    .and_then(|l| l.get("dimension"))
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let vehicle_type = val
                    .get("player")
                    .and_then(|p| p.get("predicate"))
                    .and_then(|pred| {
                        pred.get("minecraft:vehicle")
                            .or_else(|| pred.get("vehicle"))
                    })
                    .and_then(|v| v.get("minecraft:entity_type").or_else(|| v.get("type")))
                    .and_then(TagOrIdMatcher::from_value);
                Self::RideEntityInLava {
                    distance_horizontal,
                    dimension,
                    vehicle_type,
                }
            }
            "minecraft:shot_crossbow" => Self::ShotCrossbow(parse_item_pred(&val)),
            "minecraft:avoid_vibration" => Self::AvoidVibration,
            "minecraft:bee_nest_destroyed" => Self::BeeNestDestroyed {
                block: val
                    .get("blocks")
                    .or_else(|| val.get("block"))
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string),
                num_bees_inside: val
                    .get("num_bees_inside")
                    .and_then(Value::as_i64)
                    .map(|v| v as i32),
            },
            "minecraft:channeled_lightning" => Self::ChanneledLightning {
                victim_type: val
                    .get("victims")
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())
                    .and_then(|e| e.get("predicate"))
                    .and_then(|p| {
                        p.get("minecraft:entity_type")
                            .or_else(|| p.get("entity_type"))
                    })
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string),
            },
            "minecraft:lightning_strike" => {
                let bystander_type = val
                    .get("bystander")
                    .and_then(|b| b.get("predicate"))
                    .and_then(|p| {
                        p.get("minecraft:entity_type")
                            .or_else(|| p.get("entity_type"))
                    })
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let blocks_set_on_fire = val
                    .get("lightning")
                    .and_then(|l| l.get("predicate"))
                    .and_then(|p| p.get("minecraft:type_specific/lightning"))
                    .and_then(|t| t.get("blocks_set_on_fire"))
                    .and_then(Value::as_i64)
                    .map(|v| v as i32);
                let distance_max = val
                    .get("lightning")
                    .and_then(|l| l.get("predicate"))
                    .and_then(|p| p.get("minecraft:distance"))
                    .and_then(|d| d.get("absolute"))
                    .and_then(|a| a.get("max"))
                    .and_then(Value::as_f64);
                Self::LightningStrike {
                    bystander_type,
                    blocks_set_on_fire,
                    distance_max,
                }
            }
            "minecraft:kill_mob_near_sculk_catalyst" => Self::KillMobNearSculkCatalyst,
            "minecraft:hero_of_the_village" => Self::HeroOfTheVillage,
            "minecraft:player_interacted_with_entity" => Self::PlayerInteractedWithEntity {
                entity: val.get("entity").map(parse_entity_pred),
                item: val.get("item").map(parse_item_pred),
                player: val.get("player").map(parse_entity_pred),
            },
            "minecraft:thrown_item_picked_up_by_entity" => Self::ThrownItemPickedUpByEntity {
                entity: val.get("entity").map(parse_entity_pred),
                item: val.get("item").map(parse_item_pred),
                player: val.get("player").map(parse_entity_pred),
            },
            "minecraft:player_sheared_equipment" => Self::PlayerShearedEquipment {
                entity: val.get("entity").map(parse_entity_pred),
                item: val.get("item").map(parse_item_pred),
            },
            "minecraft:started_riding" => {
                let player = val.get("player").map(parse_entity_pred);
                Self::StartedRiding(player)
            }
            "minecraft:summoned_entity" => Self::SummonedEntity(parse_entity_pred(&val)),
            "minecraft:crafter_recipe_crafted" => Self::CrafterRecipeCrafted(
                val.get("recipes")
                    .or_else(|| val.get("recipe"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            "minecraft:effects_changed" => Self::EffectsChanged,
            "minecraft:spear_mobs" => Self::SpearMobs {
                count: val.get("count").and_then(Value::as_i64).map(|v| v as i32),
            },
            "minecraft:impossible" => Self::Impossible,
            "minecraft:tick" => Self::Tick,
            _ => Self::Custom {
                trigger: trigger.to_string(),
                conditions: val,
            },
        }
    }
}

// Backward-compatible standalone matching functions

/// Matches an individual item predicate.
#[must_use]
pub fn matches_item(
    item_pred: &Value,
    item_key: &str,
    count: u32,
    enchantments: &[(&str, u32)],
) -> bool {
    ItemPredicate::from_value(item_pred).matches(item_key, count, enchantments)
}

/// Evaluates conditions for `minecraft:inventory_changed`.
#[must_use]
pub fn matches_inventory_condition(
    cond: &Value,
    inventory: &[InventoryItemSnapshot<'_>],
    slots_full: usize,
    slots_empty: usize,
    slots_occupied: usize,
) -> bool {
    InventoryChangedPredicate::from_value(cond).matches(
        inventory,
        slots_full,
        slots_empty,
        slots_occupied,
    )
}

/// Evaluates conditions for `minecraft:location`.
#[expect(
    clippy::too_many_arguments,
    reason = "location condition has many properties"
)]
#[must_use]
pub fn matches_location_condition(
    cond: &Value,
    pos: (f64, f64, f64),
    biome: Option<&str>,
    dimension: Option<&str>,
    _features: &[&str],
    can_see_sky: Option<bool>,
    step_block: Option<&str>,
    feet_item: Option<&str>,
    _is_sneaking: bool,
) -> bool {
    LocationPredicate::from_value(cond).matches(
        pos,
        biome,
        dimension,
        step_block,
        can_see_sky,
        None,
        step_block,
        feet_item,
    )
}

/// Evaluates conditions for `minecraft:player_killed_entity`.
#[must_use]
pub fn matches_entity_condition(
    cond: &Value,
    entity_key: &str,
    target_type: &str,
    distance: Option<f64>,
    _location: Option<(&str, (f64, f64, f64))>,
    is_baby: Option<bool>,
    _variant: Option<&str>,
) -> bool {
    let pred_val = cond.get(entity_key).unwrap_or(cond);
    EntityPredicate::from_value(pred_val).matches(target_type, distance, is_baby)
}

/// Evaluates conditions for `minecraft:entity_hurt_player` and `minecraft:player_hurt_entity`.
#[must_use]
pub fn matches_damage_condition(
    cond: &Value,
    is_blocked: bool,
    is_deflector: bool,
    damage: Option<f64>,
) -> bool {
    DamagePredicate::from_value(cond).matches(is_blocked, is_deflector, damage)
}

/// Evaluates conditions for `minecraft:placed_block` and `minecraft:item_used_on_block`.
#[must_use]
pub fn matches_placed_block_condition(cond: &Value, block_id: &str, item_id: Option<&str>) -> bool {
    PlacedBlockPredicate::from_value(cond).matches(block_id, item_id)
}

/// Evaluates conditions for `minecraft:changed_dimension`.
#[must_use]
pub fn matches_dimension_condition(cond: &Value, from: Option<&str>, to: &str) -> bool {
    DimensionPredicate::from_value(cond).matches(from, to)
}

/// Evaluates conditions for `minecraft:recipe_unlocked` and `minecraft:recipe_crafted`.
#[must_use]
pub fn matches_recipe_condition(cond: &Value, recipe_id: &str) -> bool {
    RecipePredicate::from_value(cond).matches(recipe_id)
}

/// Evaluates conditions for `minecraft:consume_item`, `minecraft:filled_bucket`, `minecraft:fishing_rod_hooked`, `minecraft:used_totem`.
#[must_use]
pub fn matches_item_condition(cond: &Value, item_key: &str) -> bool {
    ItemPredicate::from_value(cond).matches(item_key, 1, &[])
}

/// Evaluates conditions for `minecraft:target_hit`.
#[must_use]
pub fn matches_target_hit_condition(cond: &Value, signal: i32, distance: Option<f64>) -> bool {
    TargetHitPredicate::from_value(cond).matches(signal, distance)
}

/// Evaluates conditions for `minecraft:construct_beacon`.
#[must_use]
pub fn matches_beacon_condition(cond: &Value, level: i32) -> bool {
    BeaconPredicate::from_value(cond).matches(level)
}

/// Evaluates conditions for `minecraft:enter_block` and `minecraft:slide_down_block`.
#[must_use]
pub fn matches_enter_block_condition(cond: &Value, block_id: &str) -> bool {
    TagOrIdMatcher::from_value(cond).is_none_or(|m| m.matches(RegistryKey::Block, block_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn matches_id_works() {
        assert!(matches_id("minecraft:stone", "stone"));
        assert!(matches_id("stone", "minecraft:stone"));
        assert!(matches_id("minecraft:stone", "minecraft:stone"));
        assert!(!matches_id("minecraft:stone", "minecraft:dirt"));
    }

    #[test]
    fn matches_int_bounds_works() {
        assert!(matches_int_bounds(&json!(5), 5));
        assert!(!matches_int_bounds(&json!(5), 4));
        assert!(matches_int_bounds(&json!({"min": 1, "max": 10}), 5));
        assert!(matches_int_bounds(&json!({"min": 5}), 5));
        assert!(matches_int_bounds(&json!({"min": 5}), 10));
        assert!(!matches_int_bounds(&json!({"min": 5}), 4));
        assert!(matches_int_bounds(&json!({"max": 5}), 5));
        assert!(!matches_int_bounds(&json!({"max": 5}), 6));
    }

    #[test]
    fn matches_item_predicate_works() {
        let pred = json!({
            "items": "minecraft:stone_pickaxe",
            "count": {"min": 1}
        });
        assert!(matches_item(&pred, "stone_pickaxe", 1, &[]));
        assert!(matches_item(&pred, "minecraft:stone_pickaxe", 2, &[]));
        assert!(!matches_item(&pred, "iron_pickaxe", 1, &[]));

        // Array of items
        let pred_arr = json!({
            "items": ["minecraft:stone", "minecraft:cobblestone"]
        });
        assert!(matches_item(&pred_arr, "stone", 1, &[]));
        assert!(matches_item(&pred_arr, "cobblestone", 1, &[]));
        assert!(!matches_item(&pred_arr, "dirt", 1, &[]));
    }

    #[test]
    fn matches_inventory_condition_works() {
        let cond = json!({
            "items": [
                {"items": "minecraft:crafting_table"}
            ]
        });
        let inv = [("crafting_table", 1, &[][..])];
        assert!(matches_inventory_condition(&cond, &inv, 0, 35, 1));

        let empty_inv = [];
        assert!(!matches_inventory_condition(&cond, &empty_inv, 0, 36, 0));
    }

    #[test]
    fn matches_location_condition_works() {
        let cond = json!({
            "player": {
                "type": "minecraft:entity_properties",
                "entity": "this",
                "predicate": {
                    "minecraft:location": {
                        "biomes": "minecraft:badlands",
                        "dimension": "minecraft:overworld"
                    }
                }
            }
        });
        assert!(matches_location_condition(
            &cond,
            (0.0, 64.0, 0.0),
            Some("badlands"),
            Some("minecraft:overworld"),
            &[],
            None,
            None,
            None,
            false,
        ));
        assert!(!matches_location_condition(
            &cond,
            (0.0, 64.0, 0.0),
            Some("desert"),
            Some("minecraft:overworld"),
            &[],
            None,
            None,
            None,
            false,
        ));
    }

    #[test]
    fn matches_dimension_condition_works() {
        let cond = json!({
            "to": "minecraft:the_nether"
        });
        assert!(matches_dimension_condition(
            &cond,
            None,
            "minecraft:the_nether"
        ));
        assert!(matches_dimension_condition(&cond, None, "the_nether"));
        assert!(!matches_dimension_condition(
            &cond,
            None,
            "minecraft:the_end"
        ));
    }

    #[test]
    fn matches_damage_condition_works() {
        let cond = json!({
            "damage": {
                "blocked": true,
                "type": {
                    "tags": [
                        {"id": "#minecraft:is_projectile", "expected": true}
                    ]
                }
            }
        });
        assert!(matches_damage_condition(&cond, true, true, None));
        assert!(!matches_damage_condition(&cond, false, true, None));
        assert!(!matches_damage_condition(&cond, true, false, None));
    }

    #[test]
    fn typed_criterion_conditions_parsing() {
        let cond = CriterionConditions::parse(
            "minecraft:inventory_changed",
            r#"{"items":[{"items":"minecraft:diamond","count":{"min":1}}]}"#,
        );
        match &cond {
            CriterionConditions::InventoryChanged(pred) => {
                let inv = [("diamond", 2, &[][..])];
                assert!(pred.matches(&inv, 0, 35, 1));
                let inv_wood = [("oak_planks", 2, &[][..])];
                assert!(!pred.matches(&inv_wood, 0, 35, 1));
            }
            _ => panic!("Expected InventoryChanged variant"),
        }
    }

    #[test]
    fn item_used_on_block_complex_predicate() {
        let cond = CriterionConditions::parse(
            "minecraft:item_used_on_block",
            r#"{
                "location": {
                    "type": "minecraft:all_of",
                    "terms": [
                        {
                            "type": "minecraft:location_check",
                            "predicate": {
                                "block": { "blocks": "minecraft:copper_bulb", "state": { "lit": "true" } }
                            }
                        },
                        {
                            "type": "minecraft:match_tool",
                            "predicate": { "items": "minecraft:redstone_torch" }
                        }
                    ]
                }
            }"#,
        );
        match &cond {
            CriterionConditions::ItemUsedOnBlock(pred) => {
                assert!(pred.matches(
                    "minecraft:copper_bulb",
                    "minecraft:redstone_torch",
                    Some(("lit", "true")),
                    None,
                    None
                ));
                assert!(!pred.matches(
                    "minecraft:copper_bulb",
                    "minecraft:redstone_torch",
                    Some(("lit", "false")),
                    None,
                    None
                ));
                assert!(!pred.matches(
                    "minecraft:tuff",
                    "minecraft:redstone_torch",
                    Some(("lit", "true")),
                    None,
                    None
                ));
            }
            _ => panic!("Expected ItemUsedOnBlock variant"),
        }
    }

    #[test]
    fn walk_on_powder_snow_predicate() {
        let cond = CriterionConditions::parse(
            "minecraft:location",
            r#"{
                "player": {
                    "type": "minecraft:entity_properties",
                    "predicate": {
                        "minecraft:equipment": { "feet": { "items": "minecraft:leather_boots" } },
                        "minecraft:stepping_on": { "block": { "blocks": "minecraft:powder_snow" } }
                    }
                }
            }"#,
        );
        match &cond {
            CriterionConditions::Location(pred) => {
                assert!(pred.matches(
                    (0.0, 64.0, 0.0),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some("powder_snow"),
                    Some("leather_boots")
                ));
                assert!(!pred.matches(
                    (0.0, 64.0, 0.0),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some("stone"),
                    Some("leather_boots")
                ));
                assert!(!pred.matches(
                    (0.0, 64.0, 0.0),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some("powder_snow"),
                    Some("diamond_boots")
                ));
            }
            _ => panic!("Expected Location variant"),
        }
    }

    #[test]
    fn ride_boat_with_goat_predicate() {
        let cond = CriterionConditions::parse(
            "minecraft:started_riding",
            r##"{
                "player": {
                    "type": "minecraft:entity_properties",
                    "predicate": {
                        "vehicle": {
                            "type": "#minecraft:boat",
                            "passenger": { "type": "minecraft:goat" }
                        }
                    }
                }
            }"##,
        );
        match &cond {
            CriterionConditions::StartedRiding(Some(pred)) => {
                assert!(
                    pred.vehicle_type
                        .as_ref()
                        .unwrap()
                        .matches(RegistryKey::EntityType, "minecraft:oak_boat")
                );
                assert!(
                    pred.passenger_type
                        .as_ref()
                        .unwrap()
                        .matches(RegistryKey::EntityType, "minecraft:goat")
                );
                assert!(
                    !pred
                        .passenger_type
                        .as_ref()
                        .unwrap()
                        .matches(RegistryKey::EntityType, "minecraft:pig")
                );
            }
            _ => panic!("Expected StartedRiding variant"),
        }
    }

    #[test]
    fn distract_piglin_safe_armor_predicate() {
        let cond = CriterionConditions::parse(
            "minecraft:player_interacted_with_entity",
            r##"{
                "entity": {
                    "predicate": {
                        "type": "minecraft:piglin",
                        "flags": { "is_baby": false }
                    }
                },
                "item": { "items": "minecraft:gold_ingot" },
                "player": {
                    "type": "minecraft:inverted",
                    "term": {
                        "type": "minecraft:any_of",
                        "terms": [
                            {
                                "type": "minecraft:entity_properties",
                                "predicate": {
                                    "equipment": { "head": { "items": "#minecraft:piglin_safe_armor" } }
                                }
                            }
                        ]
                    }
                }
            }"##,
        );
        match &cond {
            CriterionConditions::PlayerInteractedWithEntity {
                entity,
                item,
                player,
            } => {
                assert!(
                    entity
                        .as_ref()
                        .unwrap()
                        .matches("minecraft:piglin", None, Some(false))
                );
                assert!(
                    !entity
                        .as_ref()
                        .unwrap()
                        .matches("minecraft:piglin", None, Some(true))
                );
                assert!(
                    item.as_ref()
                        .unwrap()
                        .matches("minecraft:gold_ingot", 1, &[])
                );
                assert_eq!(player.as_ref().unwrap().safe_armor_disallowed, Some(true));
            }
            _ => panic!("Expected PlayerInteractedWithEntity variant"),
        }
    }
}
