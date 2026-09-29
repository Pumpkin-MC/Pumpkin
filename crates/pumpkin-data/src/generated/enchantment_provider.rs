/* This file is generated. Do not edit manually. */
use crate::enchantment::Enchantment;
use crate::item::Item;
use crate::item_stack::ItemStack;
use rand::RngExt;
#[derive(Clone, Copy, PartialEq)]
pub enum EnchantmentProviderKind {
    Single {
        enchantment: &'static Enchantment,
        level: i32,
    },
    ByCostWithDifficulty {
        enchantments: &'static [&'static Enchantment],
        min_cost: i32,
        max_cost_span: i32,
    },
    ByCost {
        enchantments: &'static [&'static Enchantment],
        cost: i32,
    },
}
impl core::fmt::Debug for EnchantmentProviderKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Single { enchantment, level } => f
                .debug_struct("Single")
                .field("enchantment", &enchantment.registry_key)
                .field("level", level)
                .finish(),
            Self::ByCostWithDifficulty {
                enchantments,
                min_cost,
                max_cost_span,
            } => {
                let keys: Vec<&str> = enchantments.iter().map(|e| e.registry_key).collect();
                f.debug_struct("ByCostWithDifficulty")
                    .field("enchantments", &keys)
                    .field("min_cost", min_cost)
                    .field("max_cost_span", max_cost_span)
                    .finish()
            }
            Self::ByCost { enchantments, cost } => {
                let keys: Vec<&str> = enchantments.iter().map(|e| e.registry_key).collect();
                f.debug_struct("ByCost")
                    .field("enchantments", &keys)
                    .field("cost", cost)
                    .finish()
            }
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
pub struct EnchantmentProvider {
    pub name: &'static str,
    pub kind: EnchantmentProviderKind,
}
impl core::fmt::Debug for EnchantmentProvider {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EnchantmentProvider")
            .field("name", &self.name)
            .field("kind", &self.kind)
            .finish()
    }
}
impl EnchantmentProvider {
    pub const ENDERMAN_LOOT_DROP: Self = Self {
        name: "minecraft:enderman_loot_drop",
        kind: EnchantmentProviderKind::Single {
            enchantment: &Enchantment::SILK_TOUCH,
            level: 1i32,
        },
    };
    pub const MOB_SPAWN_EQUIPMENT: Self = Self {
        name: "minecraft:mob_spawn_equipment",
        kind: EnchantmentProviderKind::ByCostWithDifficulty {
            enchantments: &[
                &Enchantment::AQUA_AFFINITY,
                &Enchantment::BANE_OF_ARTHROPODS,
                &Enchantment::BLAST_PROTECTION,
                &Enchantment::BREACH,
                &Enchantment::CHANNELING,
                &Enchantment::DENSITY,
                &Enchantment::DEPTH_STRIDER,
                &Enchantment::EFFICIENCY,
                &Enchantment::FEATHER_FALLING,
                &Enchantment::FIRE_ASPECT,
                &Enchantment::FIRE_PROTECTION,
                &Enchantment::FLAME,
                &Enchantment::FORTUNE,
                &Enchantment::IMPALING,
                &Enchantment::INFINITY,
                &Enchantment::KNOCKBACK,
                &Enchantment::LOOTING,
                &Enchantment::LOYALTY,
                &Enchantment::LUCK_OF_THE_SEA,
                &Enchantment::LUNGE,
                &Enchantment::LURE,
                &Enchantment::MULTISHOT,
                &Enchantment::PIERCING,
                &Enchantment::POWER,
                &Enchantment::PROJECTILE_PROTECTION,
                &Enchantment::PROTECTION,
                &Enchantment::PUNCH,
                &Enchantment::QUICK_CHARGE,
                &Enchantment::RESPIRATION,
                &Enchantment::RIPTIDE,
                &Enchantment::SHARPNESS,
                &Enchantment::SILK_TOUCH,
                &Enchantment::SMITE,
                &Enchantment::SWEEPING_EDGE,
                &Enchantment::THORNS,
                &Enchantment::UNBREAKING,
            ],
            min_cost: 5i32,
            max_cost_span: 17i32,
        },
    };
    pub const PILLAGER_SPAWN_CROSSBOW: Self = Self {
        name: "minecraft:pillager_spawn_crossbow",
        kind: EnchantmentProviderKind::Single {
            enchantment: &Enchantment::PIERCING,
            level: 1i32,
        },
    };
    pub const RAID_PILLAGER_POST_WAVE_3: Self = Self {
        name: "minecraft:raid/pillager_post_wave_3",
        kind: EnchantmentProviderKind::Single {
            enchantment: &Enchantment::QUICK_CHARGE,
            level: 1i32,
        },
    };
    pub const RAID_PILLAGER_POST_WAVE_5: Self = Self {
        name: "minecraft:raid/pillager_post_wave_5",
        kind: EnchantmentProviderKind::Single {
            enchantment: &Enchantment::QUICK_CHARGE,
            level: 2i32,
        },
    };
    pub const RAID_VINDICATOR: Self = Self {
        name: "minecraft:raid/vindicator",
        kind: EnchantmentProviderKind::Single {
            enchantment: &Enchantment::SHARPNESS,
            level: 1i32,
        },
    };
    pub const RAID_VINDICATOR_POST_WAVE_5: Self = Self {
        name: "minecraft:raid/vindicator_post_wave_5",
        kind: EnchantmentProviderKind::Single {
            enchantment: &Enchantment::SHARPNESS,
            level: 2i32,
        },
    };
    pub const ALL: &'static [Self] = &[
        Self::ENDERMAN_LOOT_DROP,
        Self::MOB_SPAWN_EQUIPMENT,
        Self::PILLAGER_SPAWN_CROSSBOW,
        Self::RAID_PILLAGER_POST_WAVE_3,
        Self::RAID_PILLAGER_POST_WAVE_5,
        Self::RAID_VINDICATOR,
        Self::RAID_VINDICATOR_POST_WAVE_5,
    ];
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }
    #[must_use]
    pub const fn kind(&self) -> &EnchantmentProviderKind {
        &self.kind
    }
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::match_same_arms)]
    #[must_use]
    pub fn from_name(name: &str) -> Option<&'static Self> {
        match name {
            "minecraft:enderman_loot_drop" | "enderman_loot_drop" => {
                Some(&Self::ENDERMAN_LOOT_DROP)
            }
            "minecraft:mob_spawn_equipment" | "mob_spawn_equipment" => {
                Some(&Self::MOB_SPAWN_EQUIPMENT)
            }
            "minecraft:pillager_spawn_crossbow" | "pillager_spawn_crossbow" => {
                Some(&Self::PILLAGER_SPAWN_CROSSBOW)
            }
            "minecraft:raid/pillager_post_wave_3" | "raid/pillager_post_wave_3" => {
                Some(&Self::RAID_PILLAGER_POST_WAVE_3)
            }
            "minecraft:raid/pillager_post_wave_5" | "raid/pillager_post_wave_5" => {
                Some(&Self::RAID_PILLAGER_POST_WAVE_5)
            }
            "minecraft:raid/vindicator" | "raid/vindicator" => Some(&Self::RAID_VINDICATOR),
            "minecraft:raid/vindicator_post_wave_5" | "raid/vindicator_post_wave_5" => {
                Some(&Self::RAID_VINDICATOR_POST_WAVE_5)
            }
            _ => None,
        }
    }
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::match_same_arms)]
    #[must_use]
    pub fn get_raw_json(name: &str) -> Option<&'static str> {
        match name {
            "minecraft:enderman_loot_drop" | "enderman_loot_drop" => Some(include_str!(
                "../../../../assets/datapack/data/minecraft/enchantment_provider/enderman_loot_drop.json"
            )),
            "minecraft:mob_spawn_equipment" | "mob_spawn_equipment" => Some(include_str!(
                "../../../../assets/datapack/data/minecraft/enchantment_provider/mob_spawn_equipment.json"
            )),
            "minecraft:pillager_spawn_crossbow" | "pillager_spawn_crossbow" => Some(include_str!(
                "../../../../assets/datapack/data/minecraft/enchantment_provider/pillager_spawn_crossbow.json"
            )),
            "minecraft:raid/pillager_post_wave_3" | "raid/pillager_post_wave_3" => {
                Some(include_str!(
                    "../../../../assets/datapack/data/minecraft/enchantment_provider/raid/pillager_post_wave_3.json"
                ))
            }
            "minecraft:raid/pillager_post_wave_5" | "raid/pillager_post_wave_5" => {
                Some(include_str!(
                    "../../../../assets/datapack/data/minecraft/enchantment_provider/raid/pillager_post_wave_5.json"
                ))
            }
            "minecraft:raid/vindicator" | "raid/vindicator" => Some(include_str!(
                "../../../../assets/datapack/data/minecraft/enchantment_provider/raid/vindicator.json"
            )),
            "minecraft:raid/vindicator_post_wave_5" | "raid/vindicator_post_wave_5" => {
                Some(include_str!(
                    "../../../../assets/datapack/data/minecraft/enchantment_provider/raid/vindicator_post_wave_5.json"
                ))
            }
            _ => None,
        }
    }
    #[must_use]
    pub const fn all() -> &'static [Self] {
        Self::ALL
    }
    #[must_use]
    pub const fn all_provider_names() -> &'static [&'static str] {
        &[
            "minecraft:enderman_loot_drop",
            "minecraft:mob_spawn_equipment",
            "minecraft:pillager_spawn_crossbow",
            "minecraft:raid/pillager_post_wave_3",
            "minecraft:raid/pillager_post_wave_5",
            "minecraft:raid/vindicator",
            "minecraft:raid/vindicator_post_wave_5",
        ]
    }
    #[doc = r" Evaluates the provider for the given item and difficulty multiplier, returning"]
    #[doc = r" the selected enchantments and their levels."]
    #[must_use]
    pub fn select_enchantments(
        &self,
        item: &'static Item,
        special_multiplier: f32,
    ) -> Vec<(&'static Enchantment, i32)> {
        match &self.kind {
            EnchantmentProviderKind::Single { enchantment, level } => {
                vec![(*enchantment, *level)]
            }
            EnchantmentProviderKind::ByCostWithDifficulty {
                enchantments,
                min_cost,
                max_cost_span,
            } => {
                let cost = *min_cost + (special_multiplier * *max_cost_span as f32).round() as i32;
                Self::select_by_cost(item, enchantments, cost)
            }
            EnchantmentProviderKind::ByCost { enchantments, cost } => {
                Self::select_by_cost(item, enchantments, *cost)
            }
        }
    }
    #[doc = r" Evaluates and applies the enchantments directly to an `ItemStack`."]
    pub fn apply_to_stack(&self, stack: &mut ItemStack, special_multiplier: f32) {
        let enchantments = self.select_enchantments(stack.item, special_multiplier);
        for (enchantment, level) in enchantments {
            stack.enchant(enchantment, level);
        }
    }
    #[doc = r" Selects enchantments by cost with conflict resolution, matching vanilla"]
    #[doc = r" `EnchantmentHelper.enchantItem`."]
    #[must_use]
    pub fn select_by_cost(
        item: &'static Item,
        possible_enchantments: &[&'static Enchantment],
        initial_cost: i32,
    ) -> Vec<(&'static Enchantment, i32)> {
        let candidates: Vec<&'static Enchantment> = possible_enchantments
            .iter()
            .copied()
            .filter(|e| e.can_enchant(item))
            .collect();
        if candidates.is_empty() || initial_cost <= 0 {
            return Vec::new();
        }
        let mut cost = initial_cost;
        let mut applied: Vec<(&'static Enchantment, i32)> = Vec::new();
        let mut rng = rand::rng();
        loop {
            let available: Vec<&'static Enchantment> = candidates
                .iter()
                .copied()
                .filter(|cand| {
                    !applied.iter().any(|(app, _)| app.id == cand.id)
                        && applied.iter().all(|(app, _)| cand.are_compatible(app))
                })
                .collect();
            if available.is_empty() {
                break;
            }
            let total_weight: f32 = available.iter().map(|e| e.weight as f32).sum();
            if total_weight <= 0.0 {
                break;
            }
            let mut roll = rng.random_range(0.0..total_weight);
            let mut selected: Option<&'static Enchantment> = None;
            for e in &available {
                roll -= e.weight as f32;
                if roll <= 0.0 {
                    selected = Some(e);
                    break;
                }
            }
            let Some(&fallback) = available.last() else {
                break;
            };
            let selected = selected.unwrap_or(fallback);
            let mut level = 1;
            for lvl in (1..=selected.max_level).rev() {
                if cost >= selected.min_cost.calculate(lvl) {
                    level = lvl;
                    break;
                }
            }
            applied.push((selected, level.clamp(1, selected.max_level)));
            cost /= 2;
            if cost < 1 {
                break;
            }
        }
        applied
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_providers_count() {
        assert!(EnchantmentProvider::ALL.len() >= 7);
    }
    #[test]
    fn lookup_by_name() {
        assert_eq!(
            EnchantmentProvider::from_name("minecraft:mob_spawn_equipment"),
            Some(&EnchantmentProvider::MOB_SPAWN_EQUIPMENT)
        );
        assert_eq!(
            EnchantmentProvider::from_name("mob_spawn_equipment"),
            Some(&EnchantmentProvider::MOB_SPAWN_EQUIPMENT)
        );
        assert_eq!(
            EnchantmentProvider::from_name("enderman_loot_drop"),
            Some(&EnchantmentProvider::ENDERMAN_LOOT_DROP)
        );
        assert_eq!(
            EnchantmentProvider::from_name("raid/vindicator"),
            Some(&EnchantmentProvider::RAID_VINDICATOR)
        );
        assert_eq!(
            EnchantmentProvider::from_name("minecraft:raid/pillager_post_wave_5"),
            Some(&EnchantmentProvider::RAID_PILLAGER_POST_WAVE_5)
        );
    }
    #[test]
    fn raw_json_retrieval() {
        assert!(EnchantmentProvider::get_raw_json("mob_spawn_equipment").is_some());
        assert!(EnchantmentProvider::get_raw_json("minecraft:enderman_loot_drop").is_some());
        assert!(EnchantmentProvider::get_raw_json("nonexistent").is_none());
    }
    #[test]
    fn single_enchantment_provider() {
        let encs =
            EnchantmentProvider::ENDERMAN_LOOT_DROP.select_enchantments(&Item::DIAMOND_SWORD, 0.0);
        assert_eq!(encs.len(), 1);
        assert_eq!(encs[0].0.registry_key, "silk_touch");
        assert_eq!(encs[0].1, 1);
    }
    #[test]
    fn mob_spawn_equipment_application() {
        let mut stack = ItemStack::new(1, &Item::DIAMOND_SWORD);
        EnchantmentProvider::MOB_SPAWN_EQUIPMENT.apply_to_stack(&mut stack, 1.0);
        assert!(stack.has_enchantments());
    }
}
