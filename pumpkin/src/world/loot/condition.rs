use super::LootContextParameters;
use pumpkin_data::entity::EntityType;
use pumpkin_data::{Block, BlockState};
use pumpkin_util::loot_table::LootCondition;
use pumpkin_util::random::{get_seed, xoroshiro128::Xoroshiro};
use rand::RngExt;

pub(super) trait LootConditionExt {
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
                is_on_fire,
                mainhand_enchantment_tag,
            } => {
                // Mirrors vanilla `EntityTarget` resolution from `LootContext.java:148-186`.
                let target = match *entity {
                    "this" => params.this_entity,
                    "attacker" | "killer" | "attacking_player" => params.killer_entity,
                    "direct_attacker" | "direct_killer" => params.direct_killer_entity,
                    _ => None,
                };
                if let Some(target) = target {
                    if let Some(expected) = expected_type
                        && !compare_entity_type(expected, target)
                    {
                        return false;
                    }
                    // Mirrors vanilla `EntityFlagsPredicate.isOnFire` check.
                    if let Some(expected_fire) = is_on_fire {
                        let actual_fire = params.is_on_fire.unwrap_or(false);
                        if actual_fire != *expected_fire {
                            return false;
                        }
                    }
                    // Mirrors vanilla enchantment tag lookup for smelts_loot.
                    if let Some(tag_name) = mainhand_enchantment_tag {
                        let tag = tag_name.strip_prefix('#').unwrap_or(tag_name);
                        let has_enchant = params.tool.as_ref().is_some_and(|tool| {
                            pumpkin_data::tag::get_tag_ids(
                                pumpkin_data::tag::RegistryKey::Enchantment,
                                tag,
                            )
                            .is_some_and(|tag_ids| {
                                tag_ids.iter().any(|&ench_id| {
                                    pumpkin_data::Enchantment::from_id(ench_id as u8)
                                        .is_some_and(|enc| tool.get_enchantment_level(enc) > 0)
                                })
                            })
                        });
                        if !has_enchant {
                            return false;
                        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::Enchantment;
    use pumpkin_data::damage::DamageType;
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;

    fn base_params() -> LootContextParameters {
        LootContextParameters {
            killed_by_player: Some(true),
            this_entity: Some(&EntityType::PIG),
            killer_entity: Some(&EntityType::PLAYER),
            direct_killer_entity: Some(&EntityType::PLAYER),
            damage_type: Some(DamageType::GENERIC),
            ..Default::default()
        }
    }

    fn fire_aspect_sword(level: i32) -> ItemStack {
        let mut sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
        sword.enchant(&Enchantment::FIRE_ASPECT, level);
        sword
    }

    #[test]
    fn entity_properties_this_matches_expected_type() {
        let params = base_params();
        let cond = LootCondition::EntityProperties {
            entity: "this",
            expected_type: Some("minecraft:pig"),
            is_on_fire: None,
            mainhand_enchantment_tag: None,
        };
        assert!(cond.is_fulfilled(&params));
    }

    #[test]
    fn entity_properties_this_rejects_wrong_type() {
        let params = base_params();
        let cond = LootCondition::EntityProperties {
            entity: "this",
            expected_type: Some("minecraft:cow"),
            is_on_fire: None,
            mainhand_enchantment_tag: None,
        };
        assert!(!cond.is_fulfilled(&params));
    }

    #[test]
    fn entity_properties_direct_attacker_resolves() {
        let params = base_params();
        let cond = LootCondition::EntityProperties {
            entity: "direct_attacker",
            expected_type: None,
            is_on_fire: None,
            mainhand_enchantment_tag: None,
        };
        assert!(cond.is_fulfilled(&params));
    }

    #[test]
    fn entity_properties_direct_attacker_no_direct_killer() {
        let mut params = base_params();
        params.direct_killer_entity = None;
        let cond = LootCondition::EntityProperties {
            entity: "direct_attacker",
            expected_type: None,
            is_on_fire: None,
            mainhand_enchantment_tag: None,
        };
        assert!(!cond.is_fulfilled(&params));
    }

    #[test]
    fn entity_properties_unknown_entity_returns_false() {
        let params = base_params();
        let cond = LootCondition::EntityProperties {
            entity: "target_entity",
            expected_type: None,
            is_on_fire: None,
            mainhand_enchantment_tag: None,
        };
        assert!(!cond.is_fulfilled(&params));
    }

    #[test]
    fn is_on_fire_true_when_burning() {
        let params = LootContextParameters {
            is_on_fire: Some(true),
            ..base_params()
        };
        let cond = LootCondition::EntityProperties {
            entity: "this",
            expected_type: None,
            is_on_fire: Some(true),
            mainhand_enchantment_tag: None,
        };
        assert!(cond.is_fulfilled(&params));
    }

    #[test]
    fn is_on_fire_true_fails_when_not_burning() {
        let params = LootContextParameters {
            is_on_fire: Some(false),
            ..base_params()
        };
        let cond = LootCondition::EntityProperties {
            entity: "this",
            expected_type: None,
            is_on_fire: Some(true),
            mainhand_enchantment_tag: None,
        };
        assert!(!cond.is_fulfilled(&params));
    }

    #[test]
    fn is_on_fire_false_matches_not_burning() {
        let params = LootContextParameters {
            is_on_fire: Some(false),
            ..base_params()
        };
        let cond = LootCondition::EntityProperties {
            entity: "this",
            expected_type: None,
            is_on_fire: Some(false),
            mainhand_enchantment_tag: None,
        };
        assert!(cond.is_fulfilled(&params));
    }

    #[test]
    fn is_on_fire_true_fails_when_context_none() {
        let params = LootContextParameters {
            is_on_fire: None,
            ..base_params()
        };
        let cond = LootCondition::EntityProperties {
            entity: "this",
            expected_type: None,
            is_on_fire: Some(true),
            mainhand_enchantment_tag: None,
        };
        assert!(!cond.is_fulfilled(&params));
    }

    #[test]
    fn none_is_on_fire_skips_check() {
        let params = LootContextParameters {
            is_on_fire: Some(true),
            ..base_params()
        };
        let cond = LootCondition::EntityProperties {
            entity: "this",
            expected_type: None,
            is_on_fire: None,
            mainhand_enchantment_tag: None,
        };
        assert!(cond.is_fulfilled(&params));
    }

    #[test]
    fn enchantment_tag_matches_fire_aspect() {
        let params = LootContextParameters {
            tool: Some(fire_aspect_sword(1)),
            ..base_params()
        };
        let cond = LootCondition::EntityProperties {
            entity: "direct_attacker",
            expected_type: None,
            is_on_fire: None,
            mainhand_enchantment_tag: Some("minecraft:smelts_loot"),
        };
        assert!(cond.is_fulfilled(&params));
    }

    #[test]
    fn enchantment_tag_fails_without_enchantment() {
        let params = LootContextParameters {
            tool: Some(ItemStack::new(1, &Item::DIAMOND_SWORD)),
            ..base_params()
        };
        let cond = LootCondition::EntityProperties {
            entity: "direct_attacker",
            expected_type: None,
            is_on_fire: None,
            mainhand_enchantment_tag: Some("minecraft:smelts_loot"),
        };
        assert!(!cond.is_fulfilled(&params));
    }

    #[test]
    fn enchantment_tag_rejects_unrelated_enchantment() {
        let mut sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
        sword.enchant(&Enchantment::SHARPNESS, 5);
        let params = LootContextParameters {
            tool: Some(sword),
            ..base_params()
        };
        let cond = LootCondition::EntityProperties {
            entity: "direct_attacker",
            expected_type: None,
            is_on_fire: None,
            mainhand_enchantment_tag: Some("minecraft:smelts_loot"),
        };
        assert!(!cond.is_fulfilled(&params));
    }

    #[test]
    fn enchantment_tag_fails_with_no_tool() {
        let params = LootContextParameters {
            tool: None,
            ..base_params()
        };
        let cond = LootCondition::EntityProperties {
            entity: "direct_attacker",
            expected_type: None,
            is_on_fire: None,
            mainhand_enchantment_tag: Some("minecraft:smelts_loot"),
        };
        assert!(!cond.is_fulfilled(&params));
    }

    #[test]
    fn none_enchantment_tag_skips_check() {
        let params = LootContextParameters {
            tool: Some(fire_aspect_sword(2)),
            ..base_params()
        };
        let cond = LootCondition::EntityProperties {
            entity: "direct_attacker",
            expected_type: None,
            is_on_fire: None,
            mainhand_enchantment_tag: None,
        };
        assert!(cond.is_fulfilled(&params));
    }

    #[test]
    fn anyof_passes_when_entity_on_fire() {
        let params = LootContextParameters {
            is_on_fire: Some(true),
            tool: Some(ItemStack::new(1, &Item::DIAMOND_SWORD)),
            ..base_params()
        };
        let cond = LootCondition::AnyOf(&[
            LootCondition::EntityProperties {
                entity: "this",
                expected_type: None,
                is_on_fire: Some(true),
                mainhand_enchantment_tag: None,
            },
            LootCondition::EntityProperties {
                entity: "direct_attacker",
                expected_type: None,
                is_on_fire: None,
                mainhand_enchantment_tag: Some("minecraft:smelts_loot"),
            },
        ]);
        assert!(cond.is_fulfilled(&params));
    }

    #[test]
    fn anyof_passes_when_weapon_has_fire_aspect() {
        let params = LootContextParameters {
            is_on_fire: Some(false),
            tool: Some(fire_aspect_sword(1)),
            ..base_params()
        };
        let cond = LootCondition::AnyOf(&[
            LootCondition::EntityProperties {
                entity: "this",
                expected_type: None,
                is_on_fire: Some(true),
                mainhand_enchantment_tag: None,
            },
            LootCondition::EntityProperties {
                entity: "direct_attacker",
                expected_type: None,
                is_on_fire: None,
                mainhand_enchantment_tag: Some("minecraft:smelts_loot"),
            },
        ]);
        assert!(cond.is_fulfilled(&params));
    }

    #[test]
    fn anyof_fails_without_fire_or_fire_aspect() {
        let params = LootContextParameters {
            is_on_fire: Some(false),
            tool: Some(ItemStack::new(1, &Item::DIAMOND_SWORD)),
            ..base_params()
        };
        let cond = LootCondition::AnyOf(&[
            LootCondition::EntityProperties {
                entity: "this",
                expected_type: None,
                is_on_fire: Some(true),
                mainhand_enchantment_tag: None,
            },
            LootCondition::EntityProperties {
                entity: "direct_attacker",
                expected_type: None,
                is_on_fire: None,
                mainhand_enchantment_tag: Some("minecraft:smelts_loot"),
            },
        ]);
        assert!(!cond.is_fulfilled(&params));
    }

    #[test]
    fn damage_source_requires_damage_type_in_context() {
        let mut params = base_params();
        assert!(check_damage_source_properties(
            &params,
            Some("minecraft:player"),
            None,
        ));
        params.damage_type = None;
        assert!(!check_damage_source_properties(
            &params,
            Some("minecraft:player"),
            None,
        ));
    }

    #[test]
    fn entity_type_comparison_ignores_namespace_prefix() {
        assert!(compare_entity_type("minecraft:pig", &EntityType::PIG));
        assert!(compare_entity_type("pig", &EntityType::PIG));
        assert!(!compare_entity_type("minecraft:cow", &EntityType::PIG));
    }
}
