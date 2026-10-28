//! Evaluation of loot-table style predicates (`execute if predicate`, advancement conditions).
//!
//! Predicates are evaluated on their NBT form so that inline SNBT from commands and
//! JSON from datapack files share one implementation. Unsupported conditions or
//! sub-predicates never match, so a datapack can't be tricked into a false positive.

use pumpkin_data::{item::Item, tag::Taggable};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::java::server::play::SPlayerInput;
use std::sync::atomic::Ordering;
use tracing::debug;

use crate::command::{
    argument_types::nbt_path::compare_nbt, snbt::SnbtParser, string_reader::StringReader,
};
use crate::entity::EntityBase;

/// Tests a predicate condition (the `condition` object) against `this` entity.
#[must_use]
pub fn test_condition(condition: &NbtCompound, this: Option<&dyn EntityBase>) -> bool {
    let Some(kind) = condition.get_string("condition") else {
        return false;
    };
    match strip_ns(kind) {
        "inverted" => condition
            .get_compound("term")
            .is_some_and(|term| !test_condition(term, this)),
        "all_of" => list_compounds(condition.get("terms"))
            .iter()
            .all(|term| test_condition(term, this)),
        "any_of" => list_compounds(condition.get("terms"))
            .iter()
            .any(|term| test_condition(term, this)),
        "entity_properties" => {
            if condition.get_string("entity").is_some_and(|e| e != "this") {
                debug!(
                    "Unsupported entity_properties target {:?}",
                    condition.get_string("entity")
                );
                return false;
            }
            match (this, condition.get_compound("predicate")) {
                (Some(entity), Some(predicate)) => test_entity(predicate, entity),
                (Some(_), None) => true,
                (None, _) => false,
            }
        }
        other => {
            debug!("Unsupported predicate condition {other}");
            false
        }
    }
}

/// Tests an entity predicate against an entity.
#[must_use]
pub fn test_entity(predicate: &NbtCompound, entity: &dyn EntityBase) -> bool {
    let base = entity.get_entity();
    for (key, value) in &predicate.child_tags {
        let ok = match key.as_ref() {
            "type" => matches_registry(value, base.entity_type.resource_name, |tag| {
                base.entity_type.is_tagged_with(tag).unwrap_or(false)
            }),
            "nbt" => value.extract_string().is_some_and(|snbt| {
                parse_snbt(snbt).is_some_and(|pattern| {
                    let mut nbt = NbtCompound::new();
                    entity.write_nbt(&mut nbt);
                    compare_nbt(&pattern, &NbtTag::Compound(nbt))
                })
            }),
            "flags" => value
                .extract_compound()
                .is_some_and(|flags| test_flags(flags, entity)),
            "vehicle" => value.extract_compound().is_some_and(|vehicle| {
                base.get_vehicle()
                    .is_some_and(|v| test_entity(vehicle, v.as_ref()))
            }),
            "passenger" => value.extract_compound().is_some_and(|passenger| {
                base.passengers
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .iter()
                    .any(|p| test_entity(passenger, p.as_ref()))
            }),
            "type_specific" => value
                .extract_compound()
                .is_some_and(|sub| test_type_specific(sub, entity)),
            "slots" => value.extract_compound().is_some_and(|slots| {
                slots.child_tags.iter().all(|(slot, item_predicate)| {
                    item_predicate
                        .extract_compound()
                        .is_some_and(|item_predicate| {
                            slot_items(entity, slot)
                                .iter()
                                .any(|item| test_item(item_predicate, item))
                        })
                })
            }),
            other => {
                debug!("Unsupported entity predicate field {other}");
                false
            }
        };
        if !ok {
            return false;
        }
    }
    true
}

fn test_flags(flags: &NbtCompound, entity: &dyn EntityBase) -> bool {
    let base = entity.get_entity();
    flags.child_tags.iter().all(|(key, value)| {
        let Some(expected) = value.extract_bool() else {
            return false;
        };
        let actual = match key.as_ref() {
            "is_sneaking" => base.is_sneaking(),
            "is_sprinting" => base.is_sprinting(),
            "is_swimming" => base.is_swimming(),
            "is_on_fire" => base.is_on_fire(),
            "is_on_ground" => base.on_ground.load(Ordering::Relaxed),
            other => {
                debug!("Unsupported entity flag {other}");
                return false;
            }
        };
        actual == expected
    })
}

fn test_type_specific(sub: &NbtCompound, entity: &dyn EntityBase) -> bool {
    let Some(kind) = sub.get_string("type") else {
        return false;
    };
    if strip_ns(kind) != "player" {
        debug!("Unsupported type_specific predicate {kind}");
        return false;
    }
    let Some(player) = entity.get_player() else {
        return false;
    };
    sub.child_tags
        .iter()
        .all(|(key, value)| match key.as_ref() {
            "type" => true,
            "input" => value.extract_compound().is_some_and(|input| {
                let bits = player.last_input.load(Ordering::Relaxed);
                input.child_tags.iter().all(|(name, expected)| {
                    let mask = match name.as_ref() {
                        "forward" => SPlayerInput::FORWARD,
                        "backward" => SPlayerInput::BACKWARD,
                        "left" => SPlayerInput::LEFT,
                        "right" => SPlayerInput::RIGHT,
                        "jump" => SPlayerInput::JUMP,
                        "sneak" => SPlayerInput::SNEAK,
                        "sprint" => SPlayerInput::SPRINT,
                        _ => return false,
                    };
                    expected
                        .extract_bool()
                        .is_some_and(|expected| (bits & mask != 0) == expected)
                })
            }),
            other => {
                debug!("Unsupported player predicate field {other}");
                false
            }
        })
}

/// Returns the items in the slot range named by `slot`, as item NBT.
fn slot_items(entity: &dyn EntityBase, slot: &str) -> Vec<NbtCompound> {
    let write = |stack: &pumpkin_data::item_stack::ItemStack| {
        let mut nbt = NbtCompound::new();
        stack.write_item_stack(&mut nbt);
        nbt
    };
    match slot {
        "weapon" | "weapon.mainhand" => entity
            .get_living_entity()
            .map(|living| vec![write(&living.held_item(entity))])
            .unwrap_or_default(),
        "weapon.offhand" => entity
            .get_living_entity()
            .map(|living| {
                vec![write(
                    &living.get_stack_in_hand(entity, pumpkin_util::Hand::Left),
                )]
            })
            .unwrap_or_default(),
        // Item displays expose their single item through the container and contents slots.
        "contents" | "container.*" | "container.0" if entity.get_living_entity().is_none() => {
            let mut nbt = NbtCompound::new();
            entity.write_nbt(&mut nbt);
            nbt.get_compound("item").cloned().into_iter().collect()
        }
        other => {
            debug!("Unsupported slot range {other}");
            Vec::new()
        }
    }
}

/// Tests an item predicate against an item in its NBT form.
#[must_use]
pub fn test_item(predicate: &NbtCompound, item: &NbtCompound) -> bool {
    let id = item.get_string("id").unwrap_or("minecraft:air");
    if strip_ns(id) == "air" {
        return false;
    }
    let components = item.get_compound("components");
    let component = |name: &str| -> Option<&NbtTag> {
        let components = components?;
        components
            .get(name)
            .or_else(|| components.get(strip_ns(name)))
            .or_else(|| components.get(&format!("minecraft:{}", strip_ns(name))))
    };
    predicate
        .child_tags
        .iter()
        .all(|(key, value)| match key.as_ref() {
            "items" => matches_registry(value, strip_ns(id), |tag| {
                Item::from_registry_key(strip_ns(id))
                    .and_then(|item| item.is_tagged_with(tag))
                    .unwrap_or(false)
            }),
            "count" => test_int_range(value, item.get_int("count").unwrap_or(1)),
            "components" => value.extract_compound().is_some_and(|expected| {
                expected
                    .child_tags
                    .iter()
                    .all(|(name, want)| component(name).is_some_and(|have| have == want))
            }),
            "predicates" => value.extract_compound().is_some_and(|predicates| {
                predicates
                    .child_tags
                    .iter()
                    .all(|(name, sub)| match strip_ns(name) {
                        "custom_data" => {
                            let pattern = match sub {
                                NbtTag::String(snbt) => parse_snbt(snbt),
                                other => Some(other.clone()),
                            };
                            let have = component("minecraft:custom_data")
                                .cloned()
                                .unwrap_or(NbtTag::Compound(NbtCompound::new()));
                            pattern.is_some_and(|pattern| compare_nbt(&pattern, &have))
                        }
                        "damage" => sub.extract_compound().is_some_and(|damage| {
                            let current = component("minecraft:damage")
                                .and_then(NbtTag::extract_int)
                                .unwrap_or(0);
                            damage.child_tags.iter().all(|(k, range)| match k.as_ref() {
                                "damage" => test_int_range(range, current),
                                "durability" => Item::from_registry_key(strip_ns(id))
                                    .and_then(|item| {
                                        pumpkin_data::item_stack::ItemStack::new(1, item)
                                            .get_max_damage()
                                    })
                                    .is_some_and(|max| test_int_range(range, max - current)),
                                _ => false,
                            })
                        }),
                        other => {
                            debug!("Unsupported item sub-predicate {other}");
                            false
                        }
                    })
            }),
            other => {
                debug!("Unsupported item predicate field {other}");
                false
            }
        })
}

/// Matches a registry entry against an id, `#tag`, or list of ids.
fn matches_registry(value: &NbtTag, id: &str, is_tagged: impl Fn(&str) -> bool) -> bool {
    match value {
        NbtTag::String(s) => s
            .strip_prefix('#')
            .map_or_else(|| strip_ns(s) == strip_ns(id), is_tagged),
        NbtTag::List(list) => list.iter().any(|v| {
            v.extract_string()
                .is_some_and(|s| strip_ns(s) == strip_ns(id))
        }),
        _ => false,
    }
}

/// Tests a `MinMaxBounds.Ints` value: either an exact int or `{min, max}`.
fn test_int_range(value: &NbtTag, actual: i32) -> bool {
    if let Some(exact) = value.extract_int() {
        return actual == exact;
    }
    value.extract_compound().is_some_and(|range| {
        range.get_int("min").is_none_or(|min| actual >= min)
            && range.get_int("max").is_none_or(|max| actual <= max)
    })
}

fn list_compounds(tag: Option<&NbtTag>) -> Vec<&NbtCompound> {
    match tag {
        Some(NbtTag::List(list)) => list.iter().filter_map(NbtTag::extract_compound).collect(),
        _ => Vec::new(),
    }
}

fn parse_snbt(snbt: &str) -> Option<NbtTag> {
    SnbtParser::parse_for_commands(&mut StringReader::new(snbt)).ok()
}

fn strip_ns(id: &str) -> &str {
    id.strip_prefix("minecraft:").unwrap_or(id)
}

/// Converts datapack JSON into the NBT shape the predicate functions expect.
#[must_use]
pub fn json_to_nbt(value: &serde_json::Value) -> NbtTag {
    use serde_json::Value;
    match value {
        Value::Null => NbtTag::Compound(NbtCompound::new()),
        Value::Bool(b) => NbtTag::Byte(i8::from(*b)),
        Value::Number(n) => n.as_i64().map_or_else(
            || NbtTag::Double(n.as_f64().unwrap_or_default()),
            |i| i32::try_from(i).map_or(NbtTag::Long(i), NbtTag::Int),
        ),
        Value::String(s) => NbtTag::String(s.as_str().into()),
        Value::Array(items) => NbtTag::List(items.iter().map(json_to_nbt).collect()),
        Value::Object(map) => {
            let mut compound = NbtCompound::new();
            for (k, v) in map {
                compound.put(k, json_to_nbt(v));
            }
            NbtTag::Compound(compound)
        }
    }
}
