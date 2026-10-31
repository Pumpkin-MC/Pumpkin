use std::sync::atomic::Ordering;

use pumpkin_data::{
    data_component::DataComponent,
    data_component_impl::{
        DataComponentImpl, EnchantmentsImpl, EquipmentSlot,
        basic::{CustomDataImpl, DamageImpl},
        combat::StoredEnchantmentsImpl,
        read_data,
    },
    item_stack::ItemStack,
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::loot_table::LootNumberProvider;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomImpl;
use serde_json::Value;

use super::{EntityBase, LootContextParameters, LootEntityTarget, Taggable, World, Xoroshiro};

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
    entity.is_some_and(|entity| entity_matches(entity, predicate, params))
}

#[allow(clippy::too_many_lines)]
fn entity_matches(
    entity: &dyn EntityBase,
    predicate: &Value,
    params: &LootContextParameters<'_>,
) -> bool {
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
                holder_set_matches(expected, matches)
            }
            "distance" => {
                let Some(origin) = params.position else {
                    return false;
                };
                let pos = entity.get_entity().pos.load();
                let dx = pos.x - origin.x;
                let dy = pos.y - origin.y;
                let dz = pos.z - origin.z;
                let horiz = dx.hypot(dz);
                let abs = (dx * dx + dy * dy + dz * dz).sqrt();
                expected.as_object().is_some_and(|bounds| {
                    bounds
                        .iter()
                        .all(|(k, v)| match k.strip_prefix("minecraft:").unwrap_or(k) {
                            "x" => float_matches(dx, v),
                            "y" => float_matches(dy, v),
                            "z" => float_matches(dz, v),
                            "horizontal" => float_matches(horiz, v),
                            "absolute" => float_matches(abs, v),
                            _ => true,
                        })
                })
            }
            "movement" => {
                let vel = entity.get_entity().velocity.load();
                let horiz = vel.x.hypot(vel.z);
                let vert = vel.y.abs();
                let fall = f64::from(
                    entity
                        .get_living_entity()
                        .map_or(0.0, |l| l.fall_distance.load()),
                );
                expected.as_object().is_some_and(|bounds| {
                    bounds
                        .iter()
                        .all(|(k, v)| match k.strip_prefix("minecraft:").unwrap_or(k) {
                            "x" => float_matches(vel.x, v),
                            "y" => float_matches(vel.y, v),
                            "z" => float_matches(vel.z, v),
                            "horizontal" => float_matches(horiz, v),
                            "vertical" => float_matches(vert, v),
                            "fall_distance" => float_matches(fall, v),
                            _ => true,
                        })
                })
            }
            "flags" => expected.as_object().is_some_and(|flags| {
                flags.iter().all(|(flag, expected_val)| {
                    let Some(expected_val) = expected_val.as_bool() else {
                        return false;
                    };
                    let actual = match flag.strip_prefix("minecraft:").unwrap_or(flag) {
                        "is_on_fire" => params
                            .is_on_fire
                            .unwrap_or_else(|| entity.get_entity().is_on_fire()),
                        "is_sneaking" => entity.get_entity().sneaking.load(Ordering::Relaxed),
                        "is_sprinting" => entity.get_entity().sprinting.load(Ordering::Relaxed),
                        "is_swimming" => entity.get_entity().swimming.load(Ordering::Relaxed),
                        "is_baby" => {
                            entity.get_entity().age.load(Ordering::Relaxed) < 0
                                || entity.get_mob().is_some_and(|m| {
                                    m.as_ageable()
                                        .is_some_and(crate::entity::ageable::AgeableMob::is_baby)
                                })
                        }
                        "is_on_ground" => entity.get_entity().on_ground.load(Ordering::Relaxed),
                        _ => true,
                    };
                    actual == expected_val
                })
            }),
            "equipment" => expected.as_object().is_some_and(|slots| {
                slots.iter().all(|(slot_name, item_pred)| {
                    let stack = get_entity_equipment_stack(
                        entity,
                        slot_name.strip_prefix("minecraft:").unwrap_or(slot_name),
                    );
                    item_matches(&stack, item_pred)
                })
            }),
            "slots" => expected.as_object().is_some_and(|slots| {
                slots.iter().all(|(slot_name, item_pred)| {
                    let stack = get_entity_equipment_stack(
                        entity,
                        slot_name.strip_prefix("minecraft:").unwrap_or(slot_name),
                    );
                    item_matches(&stack, item_pred)
                })
            }),
            "location" => {
                let pos = entity.get_entity().pos.load();
                let world_arc = entity.get_entity().world.load();
                let world = params.world.unwrap_or(&world_arc);
                matches_location(
                    [0, 0, 0],
                    expected,
                    &LootContextParameters {
                        position: Some(pos),
                        world: Some(world),
                        ..params.clone()
                    },
                )
            }
            "stepping_on" => {
                let pos = entity.get_entity().pos.load();
                let stepping_pos = pos.add_raw(0.0, -0.200_000_002_980_232_24, 0.0);
                let world_arc = entity.get_entity().world.load();
                let world = params.world.unwrap_or(&world_arc);
                matches_location(
                    [0, 0, 0],
                    expected,
                    &LootContextParameters {
                        position: Some(stepping_pos),
                        world: Some(world),
                        ..params.clone()
                    },
                )
            }
            "vehicle" => {
                let guard = entity
                    .get_entity()
                    .vehicle
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                guard
                    .as_ref()
                    .is_some_and(|v| entity_matches(v.as_ref(), expected, params))
            }
            "passenger" => {
                let guard = entity
                    .get_entity()
                    .passengers
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                guard
                    .iter()
                    .any(|p| entity_matches(p.as_ref(), expected, params))
            }
            "targeted_entity" => {
                let target = entity
                    .get_mob()
                    .and_then(|m| m.get_mob_entity().get_target());
                target.is_some_and(|t| entity_matches(t.as_ref(), expected, params))
            }
            "team" => expected.as_str().is_some_and(|name| {
                entity
                    .get_team_name()
                    .is_some_and(|actual| actual == name.strip_prefix("minecraft:").unwrap_or(name))
            }),
            "nbt" => {
                let Some(expected_tag) = json_to_nbt(expected) else {
                    return false;
                };
                let mut actual_nbt = NbtCompound::new();
                entity.write_nbt(&mut actual_nbt);
                matches_nbt(&expected_tag, &NbtTag::Compound(actual_nbt))
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
            "type_specific" | "sub_predicate" => expected.as_object().is_some_and(|obj| {
                let kind = obj
                    .get("type")
                    .and_then(Value::as_str)
                    .map_or("", |s| s.strip_prefix("minecraft:").unwrap_or(s));
                match_type_specific(kind, obj, entity, params)
            }),
            sub if sub.starts_with("type_specific/") || sub.starts_with("sub_predicate/") => {
                let kind = sub
                    .strip_prefix("type_specific/")
                    .or_else(|| sub.strip_prefix("sub_predicate/"))
                    .unwrap_or("");
                expected
                    .as_object()
                    .is_some_and(|obj| match_type_specific(kind, obj, entity, params))
            }
            _ => false,
        },
    )
}

fn match_type_specific(
    kind: &str,
    obj: &serde_json::Map<String, Value>,
    entity: &dyn EntityBase,
    params: &LootContextParameters<'_>,
) -> bool {
    match kind {
        "cube_mob" | "slime" => obj.get("size").is_none_or(|size_exp| {
            get_cube_mob_size(entity).is_some_and(|s| integer_matches(s, size_exp))
        }),
        "sheep" => obj
            .get("sheared")
            .and_then(Value::as_bool)
            .is_none_or(|sheared_exp| get_sheep_sheared(entity).is_some_and(|s| s == sheared_exp)),
        "raider" => {
            let captain_ok = obj
                .get("is_captain")
                .and_then(Value::as_bool)
                .is_none_or(|exp| get_raider_is_captain(entity).is_some_and(|cap| cap == exp));
            let raid_ok = obj
                .get("has_raid")
                .and_then(Value::as_bool)
                .is_none_or(|exp| get_raider_has_raid(entity).is_some_and(|raid| raid == exp));
            captain_ok && raid_ok
        }
        "fishing_hook" => obj
            .get("in_open_water")
            .and_then(Value::as_bool)
            .is_none_or(|exp| {
                get_fishing_hook_in_open_water(entity, params.world).is_some_and(|open| open == exp)
            }),
        "player" => {
            let Some(player) = entity.get_player() else {
                return false;
            };
            let gm_ok = obj
                .get("gamemode")
                .and_then(Value::as_str)
                .is_none_or(|gm| {
                    player.gamemode.load().name() == gm.strip_prefix("minecraft:").unwrap_or(gm)
                });
            let lvl_ok = obj.get("level").is_none_or(|lvl| {
                integer_matches(player.experience_level.load(Ordering::Relaxed), lvl)
            });
            let look_ok = obj.get("looking_at").is_none_or(|look_pred| {
                let target = player.get_target_entity(&player.world(), 64.0);
                target.is_some_and(|t| entity_matches(t.as_ref(), look_pred, params))
            });
            gm_ok && lvl_ok && look_ok
        }
        "lightning" => obj
            .get("blocks_set_on_fire")
            .is_none_or(|b| integer_matches(0, b)),
        _ => true,
    }
}

fn get_entity_equipment_stack(entity: &dyn EntityBase, slot: &str) -> ItemStack {
    let slot_key = slot.strip_prefix("minecraft:").unwrap_or(slot);
    let target_slot = match slot_key {
        "head" | "helmet" | "armor.head" => EquipmentSlot::HEAD,
        "chest" | "chestplate" | "armor.chest" => EquipmentSlot::CHEST,
        "legs" | "leggings" | "armor.legs" => EquipmentSlot::LEGS,
        "feet" | "boots" | "armor.feet" => EquipmentSlot::FEET,
        "mainhand" | "weapon" | "weapon.mainhand" => EquipmentSlot::MAIN_HAND,
        "offhand" | "weapon.offhand" => EquipmentSlot::OFF_HAND,
        "body" => EquipmentSlot::BODY,
        _ => return ItemStack::EMPTY.clone(),
    };

    if let Some(living) = entity.get_living_entity() {
        if target_slot == EquipmentSlot::MAIN_HAND {
            return living.held_item(entity);
        }
        let guard = living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        return guard.get(&target_slot);
    }

    ItemStack::EMPTY.clone()
}

fn get_cube_mob_size(entity: &dyn EntityBase) -> Option<i32> {
    if let Some(slime) = entity
        .cast_any()
        .downcast_ref::<crate::entity::mob::slime::SlimeEntity>()
    {
        return Some(slime.get_size());
    }
    if let Some(cube) = entity
        .cast_any()
        .downcast_ref::<crate::entity::mob::magma_cube::MagmaCubeEntity>()
    {
        return Some(cube.slime.get_size());
    }
    None
}

fn get_sheep_sheared(entity: &dyn EntityBase) -> Option<bool> {
    entity
        .cast_any()
        .downcast_ref::<crate::entity::passive::sheep::SheepEntity>()
        .map(crate::entity::passive::sheep::SheepEntity::is_sheared)
}

fn get_raider_is_captain(entity: &dyn EntityBase) -> Option<bool> {
    entity
        .get_mob()
        .and_then(crate::entity::mob::Mob::as_raider)
        .map(crate::entity::mob::raider::Raider::is_captain)
}

fn get_raider_has_raid(entity: &dyn EntityBase) -> Option<bool> {
    entity
        .get_mob()
        .and_then(crate::entity::mob::Mob::as_raider)
        .map(crate::entity::mob::raider::Raider::has_active_raid)
}

fn get_fishing_hook_in_open_water(entity: &dyn EntityBase, world: Option<&World>) -> Option<bool> {
    if entity.get_entity().entity_type.id != pumpkin_data::entity::EntityType::FISHING_BOBBER.id {
        return None;
    }
    let Some(world) = world else {
        return Some(true);
    };
    let pos = BlockPos::containing(
        entity.get_entity().pos.load().x,
        entity.get_entity().pos.load().y,
        entity.get_entity().pos.load().z,
    );
    let state = world.get_block_state(&pos);
    let block = state.id.to_block();
    Some(block == &pumpkin_data::Block::WATER)
}

pub(super) fn json_to_nbt(value: &Value) -> Option<NbtTag> {
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

pub(super) fn matches_nbt(expected: &NbtTag, actual: &NbtTag) -> bool {
    match (expected, actual) {
        (NbtTag::Compound(expected_comp), NbtTag::Compound(actual_comp)) => {
            matches_nbt_compound(expected_comp, actual_comp)
        }
        (NbtTag::List(expected_list), NbtTag::List(actual_list)) => {
            for expected_val in expected_list {
                if !actual_list
                    .iter()
                    .any(|actual_val| matches_nbt(expected_val, actual_val))
                {
                    return false;
                }
            }
            true
        }
        (a, b) => a == b,
    }
}

pub(super) fn matches_nbt_compound(expected: &NbtCompound, actual: &NbtCompound) -> bool {
    for (key, expected_val) in &expected.child_tags {
        if let Some(actual_val) = actual.child_tags.get(key) {
            if !matches_nbt(expected_val, actual_val) {
                return false;
            }
        } else {
            return false;
        }
    }
    true
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
                    let key = key.strip_prefix("minecraft:").unwrap_or(key);
                    value.as_i64().is_some_and(|value| match key {
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

fn float_matches(actual: f64, expected: &Value) -> bool {
    expected.as_f64().map_or_else(
        || {
            expected.as_object().is_some_and(|bounds| {
                bounds.iter().all(|(key, value)| {
                    let key = key.strip_prefix("minecraft:").unwrap_or(key);
                    value.as_f64().is_some_and(|value| match key {
                        "min" => actual >= value,
                        "max" => actual <= value,
                        _ => false,
                    })
                })
            })
        },
        |value| (actual - value).abs() < f64::EPSILON,
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
    predicate.iter().all(
        |(key, expected)| match key.strip_prefix("minecraft:").unwrap_or(key) {
            "type" | "condition" => true,
            "blocks" | "block" => holder_set_matches(expected, |name| {
                name.strip_prefix('#').map_or_else(
                    || block.name == name.strip_prefix("minecraft:").unwrap_or(name),
                    |tag| block.is_tagged_with(tag) == Some(true),
                )
            }),
            "state" | "properties" => expected.as_object().is_some_and(|expected| {
                expected.is_empty()
                    || block.properties(state.id).is_some_and(|properties| {
                        let properties = properties.to_props();
                        expected.iter().all(|(key, expected)| {
                            let key = key.strip_prefix("minecraft:").unwrap_or(key);
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
                                            let bound =
                                                bound.strip_prefix("minecraft:").unwrap_or(bound);
                                            expected
                                                .as_str()
                                                .and_then(|value| value.parse::<i32>().ok())
                                                .or_else(|| {
                                                    expected
                                                        .as_i64()
                                                        .and_then(|v| i32::try_from(v).ok())
                                                })
                                                .is_some_and(|expected| match bound {
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
            "nbt" => {
                if let (Some(world), Some(pos)) = (params.world, params.position) {
                    let block_pos = BlockPos::containing(pos.x, pos.y, pos.z);
                    if let Some(block_entity) = world.get_block_entity(&block_pos) {
                        let mut actual_nbt = NbtCompound::new();
                        block_entity.write_nbt(&mut actual_nbt);
                        if let Some(expected_tag) = json_to_nbt(expected) {
                            return matches_nbt(&expected_tag, &NbtTag::Compound(actual_nbt));
                        }
                    }
                }
                false
            }
            _ => false,
        },
    )
}

pub(super) fn matches_tool(predicate: &Value, params: &LootContextParameters<'_>) -> bool {
    let Some(tool) = params.tool.as_ref() else {
        return false;
    };
    item_matches(tool, predicate)
}

pub(super) fn item_matches(tool: &ItemStack, predicate: &Value) -> bool {
    let Some(predicate) = predicate.as_object() else {
        return false;
    };
    predicate.iter().all(
        |(key, expected)| match key.strip_prefix("minecraft:").unwrap_or(key) {
            "items" => holder_set_matches(expected, |name| {
                let actual_item = if tool.is_empty() {
                    &pumpkin_data::item::Item::AIR
                } else {
                    tool.item
                };
                name.strip_prefix('#').map_or_else(
                    || actual_item.registry_key == name.strip_prefix("minecraft:").unwrap_or(name),
                    |tag| actual_item.is_tagged_with(tag) == Some(true),
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
            "components" => expected.as_object().is_some_and(|components| {
                components.iter().all(|(key, expected)| {
                    let Some(component) = DataComponent::try_from_name(key) else {
                        return false;
                    };
                    let Some(actual) = get_item_data_component(tool, component) else {
                        return false;
                    };
                    let Some(tag) = json_to_nbt(expected) else {
                        return false;
                    };
                    read_data(component, &tag)
                        .is_some_and(|expected| actual.equal(expected.as_ref()))
                })
            }),
            "predicates" => expected.as_object().is_some_and(|predicates| {
                predicates.iter().all(|(key, requirements)| {
                    match key.strip_prefix("minecraft:").unwrap_or(key) {
                        "enchantments" => match_enchantments(tool, requirements, false),
                        "stored_enchantments" => match_enchantments(tool, requirements, true),
                        "damage" => match_damage(tool, requirements),
                        "custom_data" => match_custom_data(tool, requirements),
                        _ => false,
                    }
                })
            }),
            _ => false,
        },
    )
}

fn get_item_data_component(
    item: &ItemStack,
    to_get_id: DataComponent,
) -> Option<&dyn DataComponentImpl> {
    for (id, component) in &item.patch {
        if *id == to_get_id {
            return component.as_deref();
        }
    }
    for (id, component) in item.item.components {
        if *id == to_get_id {
            return Some(*component);
        }
    }
    None
}

fn match_enchantments(tool: &ItemStack, requirements: &Value, stored: bool) -> bool {
    if tool.is_empty() {
        return false;
    }
    let enchantments_list: Vec<(&pumpkin_data::enchantment::Enchantment, i32)> = if stored {
        tool.get_data_component::<StoredEnchantmentsImpl>()
            .map(|data| data.enchantment.to_vec())
            .unwrap_or_default()
    } else {
        tool.get_data_component::<EnchantmentsImpl>()
            .map(|data| data.enchantment.to_vec())
            .unwrap_or_default()
    };

    let check_requirement = |req: &Value| {
        let Some(req) = req.as_object() else {
            return false;
        };
        let expected_encs = req.get("enchantments");
        let expected_levels = req.get("levels");

        enchantments_list.iter().any(|(enchantment, level)| {
            let enc_matches = expected_encs.is_none_or(|expected| {
                holder_set_matches(expected, |name| {
                    name.strip_prefix('#').map_or_else(
                        || {
                            enchantment.registry_key()
                                == name.strip_prefix("minecraft:").unwrap_or(name)
                        },
                        |tag| enchantment.is_tagged_with(tag) == Some(true),
                    )
                })
            });
            let lvl_matches = expected_levels.is_none_or(|lvl| integer_matches(*level, lvl));
            enc_matches && lvl_matches
        })
    };

    requirements.as_array().map_or_else(
        || {
            if requirements.is_object() {
                check_requirement(requirements)
            } else {
                false
            }
        },
        |req_array| req_array.iter().all(check_requirement),
    )
}

fn match_damage(tool: &ItemStack, requirements: &Value) -> bool {
    if tool.is_empty() {
        return false;
    }
    let Some(req) = requirements.as_object() else {
        return false;
    };
    let damage_taken = tool
        .get_data_component::<DamageImpl>()
        .map_or(0, |d| d.damage);
    let max_durability = tool.get_max_damage().unwrap_or(0);
    let durability_left = (max_durability - damage_taken).max(0);

    req.iter()
        .all(|(k, v)| match k.strip_prefix("minecraft:").unwrap_or(k) {
            "durability" => integer_matches(durability_left, v),
            "damage" => integer_matches(damage_taken, v),
            _ => true,
        })
}

fn match_custom_data(tool: &ItemStack, requirements: &Value) -> bool {
    if tool.is_empty() {
        return false;
    }
    let Some(expected_tag) = json_to_nbt(requirements) else {
        return false;
    };
    let Some(custom_data) = tool.get_data_component::<CustomDataImpl>() else {
        return false;
    };
    matches_nbt(&expected_tag, &NbtTag::Compound(custom_data.data.clone()))
}

#[allow(clippy::too_many_lines)]
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
    let target_pos = origin.add_raw(
        f64::from(offset[0]),
        f64::from(offset[1]),
        f64::from(offset[2]),
    );
    let position = BlockPos::containing(target_pos.x, target_pos.y, target_pos.z);
    let loaded = world
        .level
        .loaded_chunks
        .contains_key(&position.chunk_position());
    predicate.iter().all(
        |(key, expected)| match key.strip_prefix("minecraft:").unwrap_or(key) {
            "block" => {
                loaded
                    && matches_block(
                        expected,
                        &LootContextParameters {
                            block_state: Some(world.get_block_state(&position)),
                            position: Some(target_pos),
                            ..params.clone()
                        },
                    )
            }
            "biomes" | "biome" => {
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
            "fluid" => {
                if !loaded {
                    return false;
                }
                let state = world.get_block_state(&position);
                let block = state.id.to_block();
                expected.as_object().is_some_and(|fpred| {
                    fpred
                        .iter()
                        .all(|(k, v)| match k.strip_prefix("minecraft:").unwrap_or(k) {
                            "fluids" | "fluid" => holder_set_matches(v, |name| {
                                let name = name.strip_prefix("minecraft:").unwrap_or(name);
                                block.name == name || block.is_tagged_with(name) == Some(true)
                            }),
                            _ => true,
                        })
                })
            }
            "light" => {
                if !loaded {
                    return false;
                }
                let block_light = i32::from(world.get_block_light_level(&position).unwrap_or(0));
                let sky_light = i32::from(world.get_sky_light_level(&position));
                let total = block_light.max(sky_light);
                if expected.is_number()
                    || expected.get("min").is_some()
                    || expected.get("max").is_some()
                {
                    integer_matches(total, expected)
                } else if let Some(map) = expected.as_object() {
                    map.iter()
                        .all(|(k, v)| match k.strip_prefix("minecraft:").unwrap_or(k) {
                            "light" | "composite" => integer_matches(total, v),
                            "block" => integer_matches(block_light, v),
                            "sky" => integer_matches(sky_light, v),
                            _ => true,
                        })
                } else {
                    false
                }
            }
            "position" => expected.as_object().is_some_and(|pos_pred| {
                pos_pred
                    .iter()
                    .all(|(k, v)| match k.strip_prefix("minecraft:").unwrap_or(k) {
                        "x" => float_matches(target_pos.x, v),
                        "y" => float_matches(target_pos.y, v),
                        "z" => float_matches(target_pos.z, v),
                        _ => true,
                    })
            }),
            "can_see_sky" => {
                if !loaded {
                    return false;
                }
                expected
                    .as_bool()
                    .is_some_and(|exp| world.can_see_sky(&position) == exp)
            }
            "smokey" => {
                if !loaded {
                    return false;
                }
                let is_smokey = (1..=5).any(|dy| {
                    let check_pos = BlockPos::new(position.0.x, position.0.y - dy, position.0.z);
                    let state = world.get_block_state(&check_pos);
                    let block = state.id.to_block();
                    (block == &pumpkin_data::Block::CAMPFIRE
                        || block == &pumpkin_data::Block::SOUL_CAMPFIRE)
                        && block
                            .properties(state.id)
                            .and_then(|p| {
                                p.to_props()
                                    .into_iter()
                                    .find(|prop| prop.key_str() == "lit")
                            })
                            .is_some_and(|prop| prop.value_str() == "true")
                });
                expected.as_bool().is_some_and(|exp| is_smokey == exp)
            }
            "dimension" => expected.as_str().is_some_and(|name| {
                let dim_name = &world.dimension.minecraft_name;
                name.strip_prefix("minecraft:").unwrap_or(name)
                    == dim_name.strip_prefix("minecraft:").unwrap_or(dim_name)
            }),
            _ => false,
        },
    )
}

pub(super) fn matches_damage_source(predicate: &Value, params: &LootContextParameters<'_>) -> bool {
    let Some(predicate) = predicate.as_object() else {
        return false;
    };
    predicate.iter().all(
        |(key, expected)| match key.strip_prefix("minecraft:").unwrap_or(key) {
            "source_entity" => matches_entity(LootEntityTarget::Attacker, expected, params),
            "direct_entity" => matches_entity(LootEntityTarget::DirectAttacker, expected, params),
            "tags" => expected.as_array().is_some_and(|tags| {
                tags.iter().all(|tag| {
                    let Some(name) = tag.get("id").and_then(Value::as_str) else {
                        return false;
                    };
                    let exp = tag.get("expected").and_then(Value::as_bool).unwrap_or(true);
                    let clean_tag = name.trim_start_matches('#');
                    params
                        .damage_type
                        .is_some_and(|dt| dt.is_tagged_with(clean_tag) == Some(exp))
                })
            }),
            "is_projectile" => expected.as_bool().is_some_and(|exp| {
                params
                    .damage_type
                    .is_some_and(|dt| dt.is_tagged_with("minecraft:is_projectile") == Some(exp))
            }),
            "is_explosion" => expected.as_bool().is_some_and(|exp| {
                params
                    .damage_type
                    .is_some_and(|dt| dt.is_tagged_with("minecraft:is_explosion") == Some(exp))
            }),
            "is_fire" => expected.as_bool().is_some_and(|exp| {
                params
                    .damage_type
                    .is_some_and(|dt| dt.is_tagged_with("minecraft:is_fire") == Some(exp))
            }),
            "is_lightning" => expected.as_bool().is_some_and(|exp| {
                params
                    .damage_type
                    .is_some_and(|dt| dt.is_tagged_with("minecraft:is_lightning") == Some(exp))
            }),
            "is_fall" => expected.as_bool().is_some_and(|exp| {
                params
                    .damage_type
                    .is_some_and(|dt| dt.is_tagged_with("minecraft:is_fall") == Some(exp))
            }),
            "is_direct" => expected.as_bool().is_some_and(|exp| {
                let is_direct = match (params.direct_killer_entity, params.killer_entity) {
                    (Some(direct), Some(killer)) => {
                        direct.get_entity().entity_id == killer.get_entity().entity_id
                    }
                    (None, _) => true,
                    _ => false,
                };
                is_direct == exp
            }),
            _ => true,
        },
    )
}

pub(super) fn matches_time(
    value: &Value,
    period: Option<u64>,
    params: &LootContextParameters<'_>,
) -> bool {
    let time = params.world_time;
    let effective_time = match period {
        Some(p) if p > 0 => time % p,
        _ => time,
    };
    integer_matches(effective_time as i32, value)
}

pub(super) fn matches_value(
    value: LootNumberProvider,
    range: &Value,
    _params: &LootContextParameters<'_>,
    rng: &mut Xoroshiro,
) -> bool {
    let num = match value {
        LootNumberProvider::Constant(c) => c as f32,
        LootNumberProvider::Uniform { min, max } => {
            let (min, max) = (min as f32, max as f32);
            if min >= max {
                min
            } else {
                min + rng.next_f32() * (max - min)
            }
        }
        LootNumberProvider::Binomial { n, probability } => {
            (0..n).filter(|_| rng.next_f32() < probability).count() as f32
        }
    };
    float_matches(f64::from(num), range)
}
