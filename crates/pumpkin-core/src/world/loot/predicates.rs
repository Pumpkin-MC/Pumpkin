use pumpkin_data::{data_component::DataComponent, data_component_impl::read_data};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use serde_json::Value;

use super::{EntityBase, LootContextParameters, LootEntityTarget, Taggable};

pub(super) fn matches_entity(
    target: LootEntityTarget,
    predicate: &Value,
    params: &LootContextParameters<'_>,
) -> bool {
    let entity = match target {
        LootEntityTarget::This => params.this_entity,
        LootEntityTarget::Attacker => params.killer_entity,
        LootEntityTarget::DirectAttacker => params.direct_killer_entity,
    };
    entity.is_some_and(|entity| entity_matches(entity, predicate))
}

fn entity_matches(entity: &dyn EntityBase, predicate: &Value) -> bool {
    let Some(properties) = predicate.as_object() else {
        return false;
    };
    properties.iter().all(
        |(key, expected)| match key.strip_prefix("minecraft:").unwrap_or(key) {
            "entity_type" | "type" => {
                let matches = |name: &str| {
                    let kind = entity.get_entity().entity_type;
                    name.strip_prefix('#').map_or_else(
                        || kind.resource_name == name.strip_prefix("minecraft:").unwrap_or(name),
                        |tag| kind.is_tagged_with(tag) == Some(true),
                    )
                };
                expected.as_str().map_or_else(
                    || {
                        expected.as_array().is_some_and(|names| {
                            names.iter().any(|name| name.as_str().is_some_and(matches))
                        })
                    },
                    matches,
                )
            }
            "components" => expected.as_object().is_some_and(|components| {
                components.iter().all(|(key, expected)| {
                    let Some(component) = DataComponent::try_from_name(key) else {
                        return false;
                    };
                    let Some(actual) = entity.get_component(component) else {
                        return false;
                    };
                    let Some(tag) = json_to_nbt(expected) else {
                        return false;
                    };
                    read_data(component, &tag)
                        .is_some_and(|expected| actual.equal(expected.as_ref()))
                })
            }),
            // Other entity subpredicates are not implemented yet.
            _ => true,
        },
    )
}

fn json_to_nbt(value: &Value) -> Option<NbtTag> {
    Some(match value {
        Value::Null => return None,
        Value::Bool(value) => NbtTag::Byte(i8::from(*value)),
        Value::String(value) => NbtTag::String(value.clone().into()),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                i32::try_from(value).map_or_else(|_| NbtTag::Long(value), NbtTag::Int)
            } else if value.is_f64() {
                NbtTag::Double(value.as_f64()?)
            } else {
                return None;
            }
        }
        Value::Array(values) => {
            NbtTag::List(values.iter().map(json_to_nbt).collect::<Option<_>>()?)
        }
        Value::Object(values) => {
            let mut compound = NbtCompound::new();
            for (key, value) in values {
                compound.put(key, json_to_nbt(value)?);
            }
            NbtTag::Compound(compound)
        }
    })
}

fn holder_set_matches(expected: &Value, matches: impl Fn(&str) -> bool) -> bool {
    expected.as_str().map_or_else(
        || {
            expected
                .as_array()
                .is_some_and(|names| names.iter().any(|name| name.as_str().is_some_and(&matches)))
        },
        &matches,
    )
}

fn integer_matches(actual: i32, expected: &Value) -> bool {
    expected.as_i64().map_or_else(
        || {
            expected.as_object().is_some_and(|bounds| {
                bounds.iter().all(|(key, value)| {
                    value.as_i64().is_some_and(|value| match key.as_str() {
                        "min" => i64::from(actual) >= value,
                        "max" => i64::from(actual) <= value,
                        _ => false,
                    })
                })
            })
        },
        |value| i64::from(actual) == value,
    )
}

pub(super) fn matches_block(predicate: &Value, params: &LootContextParameters<'_>) -> bool {
    let Some(state) = params.block_state else {
        return false;
    };
    let block = state.id.to_block();
    let Some(predicate) = predicate.as_object() else {
        return false;
    };
    predicate.iter().all(|(key, expected)| match key.as_str() {
        "type" | "condition" => true,
        "blocks" => holder_set_matches(expected, |name| {
            name.strip_prefix('#').map_or_else(
                || block.name == name.strip_prefix("minecraft:").unwrap_or(name),
                |tag| block.is_tagged_with(tag) == Some(true),
            )
        }),
        "state" => expected.as_object().is_some_and(|expected| {
            expected.is_empty()
                || block.properties(state.id).is_some_and(|properties| {
                    let properties = properties.to_props();
                    expected.iter().all(|(key, expected)| {
                        properties
                            .iter()
                            .find(|property| property.key_str() == key)
                            .is_some_and(|actual| {
                                if let Some(expected) = expected.as_str() {
                                    return actual.value_str() == expected;
                                }
                                let Some(bounds) = expected.as_object() else {
                                    return false;
                                };
                                actual.value_str().parse::<i32>().is_ok_and(|actual| {
                                    bounds.iter().all(|(bound, expected)| {
                                        expected
                                            .as_str()
                                            .and_then(|value| value.parse::<i32>().ok())
                                            .is_some_and(|expected| match bound.as_str() {
                                                "min" => actual >= expected,
                                                "max" => actual <= expected,
                                                _ => false,
                                            })
                                    })
                                })
                            })
                    })
                })
        }),
        _ => false,
    })
}

pub(super) fn matches_tool(predicate: &Value, params: &LootContextParameters<'_>) -> bool {
    use pumpkin_data::data_component_impl::EnchantmentsImpl;
    let Some(tool) = params.tool.as_ref() else {
        return false;
    };
    let Some(predicate) = predicate.as_object() else {
        return false;
    };
    predicate.iter().all(|(key, expected)| match key.as_str() {
        "items" => holder_set_matches(expected, |name| {
            name.strip_prefix('#').map_or_else(
                || {
                    (if tool.is_empty() {
                        &pumpkin_data::item::Item::AIR
                    } else {
                        tool.item
                    })
                    .registry_key
                        == name.strip_prefix("minecraft:").unwrap_or(name)
                },
                |tag| {
                    (if tool.is_empty() {
                        &pumpkin_data::item::Item::AIR
                    } else {
                        tool.item
                    })
                    .is_tagged_with(tag)
                        == Some(true)
                },
            )
        }),
        "count" => integer_matches(
            if tool.is_empty() {
                0
            } else {
                i32::from(tool.item_count)
            },
            expected,
        ),
        "predicates" => expected.as_object().is_some_and(|predicates| {
            predicates.iter().all(|(key, requirements)| {
                if key != "minecraft:enchantments" {
                    return false;
                }
                requirements.as_array().is_some_and(|requirements| {
                    requirements.iter().all(|requirement| {
                        let Some(enchantments) = requirement.get("enchantments") else {
                            return false;
                        };
                        tool.get_data_component::<EnchantmentsImpl>()
                            .filter(|_| !tool.is_empty())
                            .is_some_and(|actual| {
                                actual.enchantment.iter().any(|(enchantment, level)| {
                                    holder_set_matches(enchantments, |name| {
                                        name.strip_prefix('#').map_or_else(
                                            || {
                                                enchantment.registry_key()
                                                    == name
                                                        .strip_prefix("minecraft:")
                                                        .unwrap_or(name)
                                            },
                                            |tag| enchantment.is_tagged_with(tag) == Some(true),
                                        )
                                    }) && requirement
                                        .get("levels")
                                        .is_none_or(|expected| integer_matches(*level, expected))
                                })
                            })
                    })
                })
            })
        }),
        _ => false,
    })
}

pub(super) fn matches_location(
    offset: [i32; 3],
    predicate: &Value,
    params: &LootContextParameters<'_>,
) -> bool {
    let (Some(origin), Some(predicate)) = (params.position, predicate.as_object()) else {
        return false;
    };
    if predicate.is_empty() {
        return true;
    }
    let Some(world) = params.world else {
        return false;
    };
    let position = pumpkin_util::math::position::BlockPos(
        origin
            .add_raw(
                f64::from(offset[0]),
                f64::from(offset[1]),
                f64::from(offset[2]),
            )
            .floor_to_i32(),
    );
    let loaded = world
        .level
        .loaded_chunks
        .contains_key(&position.chunk_position());
    predicate.iter().all(|(key, expected)| match key.as_str() {
        "block" => {
            loaded
                && matches_block(
                    expected,
                    &LootContextParameters {
                        block_state: Some(world.get_block_state(&position)),
                        ..params.clone()
                    },
                )
        }
        "biomes" => {
            if !loaded {
                return false;
            }
            let biome = world.get_biome(&position);
            holder_set_matches(expected, |name| {
                name.strip_prefix('#').map_or_else(
                    || {
                        biome
                            .registry_key()
                            .strip_prefix("minecraft:")
                            .unwrap_or(biome.registry_key())
                            == name.strip_prefix("minecraft:").unwrap_or(name)
                    },
                    |tag| biome.is_tagged_with(tag) == Some(true),
                )
            })
        }
        _ => false,
    })
}
