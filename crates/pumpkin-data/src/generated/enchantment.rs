/* This file is generated. Do not edit manually. */
use crate::data_component_impl::EnchantmentsImpl;
use crate::item::Item;
use crate::tag::Enchantment as EnchantmentTag;
use crate::tag::Item as ItemTag;
use crate::tag::{RegistryKey, Tag, Taggable};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use std::hash::{Hash, Hasher};
use std::slice::Iter;
#[derive(Clone, Debug, PartialEq)]
pub enum LevelBasedValue {
    Constant(f32),
    Linear {
        base: f32,
        per_level_above_first: f32,
    },
    Clamped {
        value: &'static Self,
        min: f32,
        max: f32,
    },
    Fraction {
        numerator: &'static Self,
        denominator: &'static Self,
    },
    LevelsSquared {
        added: f32,
    },
    Lookup {
        values: &'static [f32],
        fallback: &'static Self,
    },
}
impl LevelBasedValue {
    #[must_use]
    pub fn calculate(&self, level: i32) -> f32 {
        match self {
            Self::Constant(val) => *val,
            Self::Linear {
                base,
                per_level_above_first,
            } => base + (level.max(1) - 1) as f32 * per_level_above_first,
            Self::Clamped { value, min, max } => value.calculate(level).clamp(*min, *max),
            Self::Fraction {
                numerator,
                denominator,
            } => {
                let denom = denominator.calculate(level);
                if denom == 0.0 {
                    0.0
                } else {
                    numerator.calculate(level) / denom
                }
            }
            Self::LevelsSquared { added } => ((level * level) as f32) + added,
            Self::Lookup { values, fallback } => {
                let idx = (level - 1) as usize;
                values
                    .get(idx)
                    .copied()
                    .unwrap_or_else(|| fallback.calculate(level))
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnchantmentTarget {
    Attacker,
    DamagingEntity,
    Victim,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetedConditionalEffect<T> {
    pub enchanted: Option<EnchantmentTarget>,
    pub affected: Option<EnchantmentTarget>,
    pub effect: T,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionalEffect<T> {
    pub effect: T,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ReplaceDiskPredicate {
    MatchingBlockTag {
        offset: pumpkin_util::math::vector3::Vector3<i32>,
        tag: &'static crate::tag::Tag,
    },
    MatchingBlocks {
        offset: pumpkin_util::math::vector3::Vector3<i32>,
        blocks: &'static [&'static str],
    },
    MatchingFluids {
        offset: pumpkin_util::math::vector3::Vector3<i32>,
        fluids: &'static [&'static str],
    },
    Unobstructed,
    AllOf(&'static [ReplaceDiskPredicate]),
}
#[derive(Clone, Debug, PartialEq)]
pub enum EnchantmentEntityEffect {
    Ignite {
        duration: LevelBasedValue,
    },
    DamageEntity {
        min_damage: LevelBasedValue,
        max_damage: LevelBasedValue,
        damage_type: Option<&'static crate::damage::DamageType>,
    },
    ChangeItemDamage {
        amount: LevelBasedValue,
    },
    PlaySound {
        sound: &'static str,
    },
    ReplaceBlock {
        offset_x: i32,
        offset_y: i32,
        offset_z: i32,
        trigger_game_event: Option<crate::game_event::GameEvent>,
    },
    SetBlockProperties {
        properties: &'static [(&'static str, &'static str)],
        offset_x: i32,
        offset_y: i32,
        offset_z: i32,
        trigger_game_event: Option<crate::game_event::GameEvent>,
    },
    ReplaceDisk {
        radius: LevelBasedValue,
        height: LevelBasedValue,
        offset_x: i32,
        offset_y: i32,
        offset_z: i32,
        predicate: Option<ReplaceDiskPredicate>,
        block_state: &'static crate::block_state::BlockState,
        trigger_game_event: Option<crate::game_event::GameEvent>,
    },
    SummonEntity {
        entity_types: &'static [&'static crate::entity::EntityType],
        join_team: bool,
    },
    SpawnParticles {
        particle: crate::particle::Particle,
        horizontal_position: PositionSource,
        vertical_position: PositionSource,
        horizontal_velocity: VelocitySource,
        vertical_velocity: VelocitySource,
        speed: pumpkin_util::math::float_provider::FloatProvider,
    },
    RunFunction {
        function: &'static str,
    },
    ApplyExhaustion {
        amount: LevelBasedValue,
    },
    ApplyImpulse {
        direction: pumpkin_util::math::vector3::Vector3<f64>,
        coordinate_scale: pumpkin_util::math::vector3::Vector3<f64>,
        magnitude: LevelBasedValue,
    },
    ApplyMobEffect {
        to_apply: &'static [&'static crate::effect::StatusEffect],
        min_duration: LevelBasedValue,
        max_duration: LevelBasedValue,
        min_amplifier: LevelBasedValue,
        max_amplifier: LevelBasedValue,
    },
    Explode {
        attribute_to_user: bool,
        damage_type: Option<&'static crate::damage::DamageType>,
        knockback_multiplier: Option<LevelBasedValue>,
        immune_blocks: Option<&'static str>,
        offset_x: f64,
        offset_y: f64,
        offset_z: f64,
        radius: LevelBasedValue,
        create_fire: bool,
        block_interaction: &'static str,
        small_particle: Option<crate::particle::Particle>,
        large_particle: Option<crate::particle::Particle>,
        sound: Option<crate::sound::Sound>,
    },
    AllOf(&'static [EnchantmentEntityEffect]),
    Other,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum PositionSourceType {
    #[default]
    EntityPosition,
    InBoundingBox,
}
impl PositionSourceType {
    #[must_use]
    pub fn get_coordinate(
        self,
        position: f64,
        center: f64,
        bounding_box_span: f32,
        random: &mut impl pumpkin_util::random::RandomImpl,
    ) -> f64 {
        match self {
            Self::EntityPosition => position,
            Self::InBoundingBox => {
                let random_offset = f64::from(random.next_f32()) - 0.5;
                center + random_offset * f64::from(bounding_box_span)
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionSource {
    pub source_type: PositionSourceType,
    pub offset: f32,
    pub scale: f32,
}
impl PositionSource {
    #[must_use]
    pub const fn new(source_type: PositionSourceType, offset: f32, scale: f32) -> Self {
        Self {
            source_type,
            offset,
            scale,
        }
    }
    #[must_use]
    pub const fn offset_from_entity_position(offset: f32) -> Self {
        Self {
            source_type: PositionSourceType::EntityPosition,
            offset,
            scale: 1.0,
        }
    }
    #[must_use]
    pub const fn in_bounding_box() -> Self {
        Self {
            source_type: PositionSourceType::InBoundingBox,
            offset: 0.0,
            scale: 1.0,
        }
    }
    #[must_use]
    pub fn get_coordinate(
        &self,
        position: f64,
        center: f64,
        bounding_box_span: f32,
        random: &mut impl pumpkin_util::random::RandomImpl,
    ) -> f64 {
        self.source_type
            .get_coordinate(position, center, bounding_box_span * self.scale, random)
            + f64::from(self.offset)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct VelocitySource {
    pub movement_scale: f32,
    pub base: pumpkin_util::math::float_provider::FloatProvider,
}
impl VelocitySource {
    #[must_use]
    pub const fn new(
        movement_scale: f32,
        base: pumpkin_util::math::float_provider::FloatProvider,
    ) -> Self {
        Self {
            movement_scale,
            base,
        }
    }
    #[must_use]
    pub const fn movement_scaled(scale: f32) -> Self {
        Self {
            movement_scale: scale,
            base: pumpkin_util::math::float_provider::FloatProvider::Constant(0.0),
        }
    }
    #[must_use]
    pub const fn fixed_velocity(
        provider: pumpkin_util::math::float_provider::FloatProvider,
    ) -> Self {
        Self {
            movement_scale: 0.0,
            base: provider,
        }
    }
    pub fn get_velocity(
        &self,
        movement: f64,
        random: &mut impl pumpkin_util::random::RandomImpl,
    ) -> f64 {
        f64::from(self.movement_scale).mul_add(movement, f64::from(self.base.get(random)))
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum EnchantmentValueEffect {
    Add(LevelBasedValue),
    Multiply(LevelBasedValue),
    Set(LevelBasedValue),
    RemoveBinomial(LevelBasedValue),
    Other,
}
impl EnchantmentValueEffect {
    #[must_use]
    pub fn process(&self, level: i32, current_value: f32) -> f32 {
        match self {
            Self::Add(val) => current_value + val.calculate(level),
            Self::Multiply(val) => current_value * val.calculate(level),
            Self::Set(val) => val.calculate(level),
            Self::RemoveBinomial(val) => {
                let prob = val.calculate(level);
                if rand::random::<f32>() < prob {
                    0.0
                } else {
                    current_value
                }
            }
            Self::Other => current_value,
        }
    }
}
#[derive(Clone, Debug)]
pub struct EnchantmentEffects {
    pub projectile_spawned: &'static [ConditionalEffect<EnchantmentEntityEffect>],
    pub post_attack: &'static [TargetedConditionalEffect<EnchantmentEntityEffect>],
    pub projectile_count: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub projectile_spread: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub projectile_piercing: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub ammo_use: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub damage: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub knockback: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub armor_effectiveness: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub damage_protection: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub hit_block: &'static [ConditionalEffect<EnchantmentEntityEffect>],
    pub item_damage: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub equipment_drops: &'static [TargetedConditionalEffect<EnchantmentValueEffect>],
    pub fishing_time_reduction: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub fishing_luck_bonus: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub block_experience: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub mob_experience: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub repair_with_xp: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub smash_damage_per_fallen_block: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub trident_return_acceleration: &'static [ConditionalEffect<EnchantmentValueEffect>],
    pub trident_spin_attack_strength: Option<EnchantmentValueEffect>,
    pub crossbow_charge_time: Option<EnchantmentValueEffect>,
    pub location_changed: &'static [ConditionalEffect<EnchantmentEntityEffect>],
    pub prevent_armor_change: bool,
    pub prevent_equipment_drop: bool,
}
pub struct Enchantment {
    pub id: u8,
    pub name: &'static str,
    pub registry_key: &'static str,
    pub description: &'static str,
    pub anvil_cost: u32,
    pub supported_items: &'static Tag,
    pub exclusive_set: Option<&'static Tag>,
    pub max_level: i32,
    pub slots: &'static [AttributeModifierSlot],
    pub weight: i32,
    pub min_cost: Cost,
    pub max_cost: Cost,
    pub effects: EnchantmentEffects,
}
#[derive(Clone, Copy, Debug)]
pub struct Cost {
    pub base: i32,
    pub per_level_above_first: i32,
}
impl Cost {
    pub fn calculate(&self, level: i32) -> i32 {
        self.base + self.per_level_above_first * (level - 1)
    }
}
impl Taggable for Enchantment {
    #[inline]
    fn tag_key() -> RegistryKey {
        RegistryKey::Enchantment
    }
    #[inline]
    fn registry_key(&self) -> &str {
        self.registry_key
    }
    #[inline]
    fn registry_id(&self) -> u16 {
        self.id as u16
    }
}
impl PartialEq for Enchantment {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Enchantment {}
impl Hash for Enchantment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
#[derive(Debug, Clone, Hash, PartialEq)]
pub enum AttributeModifierSlot {
    Any,
    MainHand,
    OffHand,
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Armor,
    Body,
    Saddle,
}
impl Enchantment {
    pub const ALL: &'static [&'static Self] = &[
        &Self::AQUA_AFFINITY,
        &Self::BANE_OF_ARTHROPODS,
        &Self::BINDING_CURSE,
        &Self::BLAST_PROTECTION,
        &Self::BREACH,
        &Self::CHANNELING,
        &Self::DENSITY,
        &Self::DEPTH_STRIDER,
        &Self::EFFICIENCY,
        &Self::FEATHER_FALLING,
        &Self::FIRE_ASPECT,
        &Self::FIRE_PROTECTION,
        &Self::FLAME,
        &Self::FORTUNE,
        &Self::FROST_WALKER,
        &Self::IMPALING,
        &Self::INFINITY,
        &Self::KNOCKBACK,
        &Self::LOOTING,
        &Self::LOYALTY,
        &Self::LUCK_OF_THE_SEA,
        &Self::LUNGE,
        &Self::LURE,
        &Self::MENDING,
        &Self::MULTISHOT,
        &Self::PIERCING,
        &Self::POWER,
        &Self::PROJECTILE_PROTECTION,
        &Self::PROTECTION,
        &Self::PUNCH,
        &Self::QUICK_CHARGE,
        &Self::RESPIRATION,
        &Self::RIPTIDE,
        &Self::SHARPNESS,
        &Self::SILK_TOUCH,
        &Self::SMITE,
        &Self::SOUL_SPEED,
        &Self::SWEEPING_EDGE,
        &Self::SWIFT_SNEAK,
        &Self::THORNS,
        &Self::UNBREAKING,
        &Self::VANISHING_CURSE,
        &Self::WIND_BURST,
    ];
    pub fn all() -> Iter<'static, &'static Self> {
        Self::ALL.iter()
    }
    pub const AQUA_AFFINITY: Self = Self {
        id: 0u8,
        name: "minecraft:aqua_affinity",
        description: "enchantment.minecraft.aqua_affinity",
        registry_key: "aqua_affinity",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_HEAD_ARMOR,
        exclusive_set: None,
        max_level: 1i32,
        slots: &[AttributeModifierSlot::Head],
        weight: 2i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 41i32,
            per_level_above_first: 0i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const BANE_OF_ARTHROPODS: Self = Self {
        id: 1u8,
        name: "minecraft:bane_of_arthropods",
        registry_key: "bane_of_arthropods",
        description: "enchantment.minecraft.bane_of_arthropods",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_WEAPON,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_DAMAGE),
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 8i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[TargetedConditionalEffect {
                enchanted: Some(EnchantmentTarget::Attacker),
                affected: Some(EnchantmentTarget::Victim),
                effect: EnchantmentEntityEffect::ApplyMobEffect {
                    to_apply: &[&crate::effect::StatusEffect::SLOWNESS],
                    min_duration: LevelBasedValue::Constant(1.5f32),
                    max_duration: LevelBasedValue::Linear {
                        base: 1.5f32,
                        per_level_above_first: 0.5f32,
                    },
                    min_amplifier: LevelBasedValue::Constant(3f32),
                    max_amplifier: LevelBasedValue::Constant(3f32),
                },
            }],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 2.5f32,
                    per_level_above_first: 2.5f32,
                }),
            }],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const BINDING_CURSE: Self = Self {
        id: 2u8,
        name: "minecraft:binding_curse",
        description: "enchantment.minecraft.binding_curse",
        registry_key: "binding_curse",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_EQUIPPABLE,
        exclusive_set: None,
        max_level: 1i32,
        slots: &[AttributeModifierSlot::Armor],
        weight: 1i32,
        min_cost: Cost {
            base: 25i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: true,
            prevent_equipment_drop: false,
        },
    };
    pub const BLAST_PROTECTION: Self = Self {
        id: 3u8,
        name: "minecraft:blast_protection",
        registry_key: "blast_protection",
        description: "enchantment.minecraft.blast_protection",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_ARMOR,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_ARMOR),
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
        weight: 2i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 13i32,
            per_level_above_first: 8i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 2f32,
                    per_level_above_first: 2f32,
                }),
            }],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const BREACH: Self = Self {
        id: 4u8,
        name: "minecraft:breach",
        registry_key: "breach",
        description: "enchantment.minecraft.breach",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MACE,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_DAMAGE),
        max_level: 4i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 9i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: -0.15f32,
                    per_level_above_first: -0.15f32,
                }),
            }],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const CHANNELING: Self = Self {
        id: 5u8,
        name: "minecraft:channeling",
        description: "enchantment.minecraft.channeling",
        registry_key: "channeling",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_TRIDENT,
        exclusive_set: None,
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 1i32,
        min_cost: Cost {
            base: 25i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[TargetedConditionalEffect {
                enchanted: Some(EnchantmentTarget::Attacker),
                affected: Some(EnchantmentTarget::Victim),
                effect: EnchantmentEntityEffect::AllOf(&[
                    EnchantmentEntityEffect::SummonEntity {
                        entity_types: &[&crate::entity::EntityType::LIGHTNING_BOLT],
                        join_team: false,
                    },
                    EnchantmentEntityEffect::PlaySound {
                        sound: "minecraft:item.trident.thunder",
                    },
                ]),
            }],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[ConditionalEffect {
                effect: EnchantmentEntityEffect::AllOf(&[
                    EnchantmentEntityEffect::SummonEntity {
                        entity_types: &[&crate::entity::EntityType::LIGHTNING_BOLT],
                        join_team: false,
                    },
                    EnchantmentEntityEffect::PlaySound {
                        sound: "minecraft:item.trident.thunder",
                    },
                ]),
            }],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const DENSITY: Self = Self {
        id: 6u8,
        name: "minecraft:density",
        registry_key: "density",
        description: "enchantment.minecraft.density",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MACE,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_DAMAGE),
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 8i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 0.5f32,
                    per_level_above_first: 0.5f32,
                }),
            }],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const DEPTH_STRIDER: Self = Self {
        id: 7u8,
        name: "minecraft:depth_strider",
        registry_key: "depth_strider",
        description: "enchantment.minecraft.depth_strider",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FOOT_ARMOR,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_BOOTS),
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Feet],
        weight: 2i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 10i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const EFFICIENCY: Self = Self {
        id: 8u8,
        name: "minecraft:efficiency",
        description: "enchantment.minecraft.efficiency",
        registry_key: "efficiency",
        anvil_cost: 1u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MINING,
        exclusive_set: None,
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 10i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 51i32,
            per_level_above_first: 10i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const FEATHER_FALLING: Self = Self {
        id: 9u8,
        name: "minecraft:feather_falling",
        description: "enchantment.minecraft.feather_falling",
        registry_key: "feather_falling",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FOOT_ARMOR,
        exclusive_set: None,
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 6i32,
        },
        max_cost: Cost {
            base: 11i32,
            per_level_above_first: 6i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 3f32,
                    per_level_above_first: 3f32,
                }),
            }],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const FIRE_ASPECT: Self = Self {
        id: 10u8,
        name: "minecraft:fire_aspect",
        description: "enchantment.minecraft.fire_aspect",
        registry_key: "fire_aspect",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FIRE_ASPECT,
        exclusive_set: None,
        max_level: 2i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 20i32,
        },
        max_cost: Cost {
            base: 60i32,
            per_level_above_first: 20i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[TargetedConditionalEffect {
                enchanted: Some(EnchantmentTarget::Attacker),
                affected: Some(EnchantmentTarget::Victim),
                effect: EnchantmentEntityEffect::Ignite {
                    duration: LevelBasedValue::Linear {
                        base: 4f32,
                        per_level_above_first: 4f32,
                    },
                },
            }],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const FIRE_PROTECTION: Self = Self {
        id: 11u8,
        name: "minecraft:fire_protection",
        registry_key: "fire_protection",
        description: "enchantment.minecraft.fire_protection",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_ARMOR,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_ARMOR),
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
        weight: 5i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 18i32,
            per_level_above_first: 8i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 2f32,
                    per_level_above_first: 2f32,
                }),
            }],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const FLAME: Self = Self {
        id: 12u8,
        name: "minecraft:flame",
        description: "enchantment.minecraft.flame",
        registry_key: "flame",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_BOW,
        exclusive_set: None,
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 20i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[ConditionalEffect {
                effect: EnchantmentEntityEffect::Ignite {
                    duration: LevelBasedValue::Constant(100f32),
                },
            }],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const FORTUNE: Self = Self {
        id: 13u8,
        name: "minecraft:fortune",
        registry_key: "fortune",
        description: "enchantment.minecraft.fortune",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MINING_LOOT,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_MINING),
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 9i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const FROST_WALKER: Self = Self {
        id: 14u8,
        name: "minecraft:frost_walker",
        registry_key: "frost_walker",
        description: "enchantment.minecraft.frost_walker",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FOOT_ARMOR,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_BOOTS),
        max_level: 2i32,
        slots: &[AttributeModifierSlot::Feet],
        weight: 2i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 10i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[ConditionalEffect {
                effect: EnchantmentEntityEffect::ReplaceDisk {
                    radius: LevelBasedValue::Clamped {
                        value: &LevelBasedValue::Linear {
                            base: 3f32,
                            per_level_above_first: 1f32,
                        },
                        min: 0f32,
                        max: 16f32,
                    },
                    height: LevelBasedValue::Constant(1f32),
                    offset_x: 0i32,
                    offset_y: -1i32,
                    offset_z: 0i32,
                    predicate: Some(ReplaceDiskPredicate::AllOf(&[
                        ReplaceDiskPredicate::MatchingBlockTag {
                            offset: pumpkin_util::math::vector3::Vector3::new(0i32, 1i32, 0i32),
                            tag: &crate::tag::Block::MINECRAFT_AIR,
                        },
                        ReplaceDiskPredicate::MatchingBlocks {
                            offset: pumpkin_util::math::vector3::Vector3::new(0i32, 0i32, 0i32),
                            blocks: &["minecraft:water"],
                        },
                        ReplaceDiskPredicate::MatchingFluids {
                            offset: pumpkin_util::math::vector3::Vector3::new(0i32, 0i32, 0i32),
                            fluids: &["minecraft:water"],
                        },
                        ReplaceDiskPredicate::Unobstructed,
                    ])),
                    block_state: crate::Block::FROSTED_ICE.default_state,
                    trigger_game_event: Some(crate::game_event::GameEvent::BlockPlace),
                },
            }],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const IMPALING: Self = Self {
        id: 15u8,
        name: "minecraft:impaling",
        registry_key: "impaling",
        description: "enchantment.minecraft.impaling",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_TRIDENT,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_DAMAGE),
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 21i32,
            per_level_above_first: 8i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 2.5f32,
                    per_level_above_first: 2.5f32,
                }),
            }],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const INFINITY: Self = Self {
        id: 16u8,
        name: "minecraft:infinity",
        registry_key: "infinity",
        description: "enchantment.minecraft.infinity",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_BOW,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_BOW),
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 1i32,
        min_cost: Cost {
            base: 20i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Set(LevelBasedValue::Constant(0f32)),
            }],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const KNOCKBACK: Self = Self {
        id: 17u8,
        name: "minecraft:knockback",
        description: "enchantment.minecraft.knockback",
        registry_key: "knockback",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MELEE_WEAPON,
        exclusive_set: None,
        max_level: 2i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 20i32,
        },
        max_cost: Cost {
            base: 55i32,
            per_level_above_first: 20i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 1f32,
                    per_level_above_first: 1f32,
                }),
            }],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const LOOTING: Self = Self {
        id: 18u8,
        name: "minecraft:looting",
        description: "enchantment.minecraft.looting",
        registry_key: "looting",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MELEE_WEAPON,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 9i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[TargetedConditionalEffect {
                enchanted: Some(EnchantmentTarget::Attacker),
                affected: None,
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 0.01f32,
                    per_level_above_first: 0.01f32,
                }),
            }],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const LOYALTY: Self = Self {
        id: 19u8,
        name: "minecraft:loyalty",
        description: "enchantment.minecraft.loyalty",
        registry_key: "loyalty",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_TRIDENT,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 5i32,
        min_cost: Cost {
            base: 12i32,
            per_level_above_first: 7i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 1f32,
                    per_level_above_first: 1f32,
                }),
            }],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const LUCK_OF_THE_SEA: Self = Self {
        id: 20u8,
        name: "minecraft:luck_of_the_sea",
        description: "enchantment.minecraft.luck_of_the_sea",
        registry_key: "luck_of_the_sea",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FISHING,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 9i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 1f32,
                    per_level_above_first: 1f32,
                }),
            }],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const LUNGE: Self = Self {
        id: 21u8,
        name: "minecraft:lunge",
        description: "enchantment.minecraft.lunge",
        registry_key: "lunge",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_LUNGE,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Hand],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 8i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const LURE: Self = Self {
        id: 22u8,
        name: "minecraft:lure",
        description: "enchantment.minecraft.lure",
        registry_key: "lure",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FISHING,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 9i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 5f32,
                    per_level_above_first: 5f32,
                }),
            }],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const MENDING: Self = Self {
        id: 23u8,
        name: "minecraft:mending",
        description: "enchantment.minecraft.mending",
        registry_key: "mending",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_DURABILITY,
        exclusive_set: None,
        max_level: 1i32,
        slots: &[AttributeModifierSlot::Any],
        weight: 2i32,
        min_cost: Cost {
            base: 25i32,
            per_level_above_first: 25i32,
        },
        max_cost: Cost {
            base: 75i32,
            per_level_above_first: 25i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Multiply(LevelBasedValue::Constant(2f32)),
            }],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const MULTISHOT: Self = Self {
        id: 24u8,
        name: "minecraft:multishot",
        registry_key: "multishot",
        description: "enchantment.minecraft.multishot",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_CROSSBOW,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_CROSSBOW),
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 20i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 2f32,
                    per_level_above_first: 2f32,
                }),
            }],
            projectile_spread: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 10f32,
                    per_level_above_first: 10f32,
                }),
            }],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const PIERCING: Self = Self {
        id: 25u8,
        name: "minecraft:piercing",
        registry_key: "piercing",
        description: "enchantment.minecraft.piercing",
        anvil_cost: 1u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_CROSSBOW,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_CROSSBOW),
        max_level: 4i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 10i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 1f32,
                    per_level_above_first: 1f32,
                }),
            }],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const POWER: Self = Self {
        id: 26u8,
        name: "minecraft:power",
        description: "enchantment.minecraft.power",
        registry_key: "power",
        anvil_cost: 1u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_BOW,
        exclusive_set: None,
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 10i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 16i32,
            per_level_above_first: 10i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 1f32,
                    per_level_above_first: 0.5f32,
                }),
            }],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const PROJECTILE_PROTECTION: Self = Self {
        id: 27u8,
        name: "minecraft:projectile_protection",
        registry_key: "projectile_protection",
        description: "enchantment.minecraft.projectile_protection",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_ARMOR,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_ARMOR),
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
        weight: 5i32,
        min_cost: Cost {
            base: 3i32,
            per_level_above_first: 6i32,
        },
        max_cost: Cost {
            base: 9i32,
            per_level_above_first: 6i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 2f32,
                    per_level_above_first: 2f32,
                }),
            }],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const PROTECTION: Self = Self {
        id: 28u8,
        name: "minecraft:protection",
        registry_key: "protection",
        description: "enchantment.minecraft.protection",
        anvil_cost: 1u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_ARMOR,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_ARMOR),
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
        weight: 10i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 11i32,
        },
        max_cost: Cost {
            base: 12i32,
            per_level_above_first: 11i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 1f32,
                    per_level_above_first: 1f32,
                }),
            }],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const PUNCH: Self = Self {
        id: 29u8,
        name: "minecraft:punch",
        description: "enchantment.minecraft.punch",
        registry_key: "punch",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_BOW,
        exclusive_set: None,
        max_level: 2i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 12i32,
            per_level_above_first: 20i32,
        },
        max_cost: Cost {
            base: 37i32,
            per_level_above_first: 20i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 1f32,
                    per_level_above_first: 1f32,
                }),
            }],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const QUICK_CHARGE: Self = Self {
        id: 30u8,
        name: "minecraft:quick_charge",
        description: "enchantment.minecraft.quick_charge",
        registry_key: "quick_charge",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_CROSSBOW,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[
            AttributeModifierSlot::MainHand,
            AttributeModifierSlot::OffHand,
        ],
        weight: 5i32,
        min_cost: Cost {
            base: 12i32,
            per_level_above_first: 20i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: Some(EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                base: -0.25f32,
                per_level_above_first: -0.25f32,
            })),
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const RESPIRATION: Self = Self {
        id: 31u8,
        name: "minecraft:respiration",
        description: "enchantment.minecraft.respiration",
        registry_key: "respiration",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_HEAD_ARMOR,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Head],
        weight: 2i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 40i32,
            per_level_above_first: 10i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const RIPTIDE: Self = Self {
        id: 32u8,
        name: "minecraft:riptide",
        registry_key: "riptide",
        description: "enchantment.minecraft.riptide",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_TRIDENT,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_RIPTIDE),
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Hand],
        weight: 2i32,
        min_cost: Cost {
            base: 17i32,
            per_level_above_first: 7i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: Some(EnchantmentValueEffect::Add(
                LevelBasedValue::Linear {
                    base: 1.5f32,
                    per_level_above_first: 0.75f32,
                },
            )),
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const SHARPNESS: Self = Self {
        id: 33u8,
        name: "minecraft:sharpness",
        registry_key: "sharpness",
        description: "enchantment.minecraft.sharpness",
        anvil_cost: 1u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_SHARP_WEAPON,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_DAMAGE),
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 10i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 11i32,
        },
        max_cost: Cost {
            base: 21i32,
            per_level_above_first: 11i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 1f32,
                    per_level_above_first: 0.5f32,
                }),
            }],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const SILK_TOUCH: Self = Self {
        id: 34u8,
        name: "minecraft:silk_touch",
        registry_key: "silk_touch",
        description: "enchantment.minecraft.silk_touch",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MINING_LOOT,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_MINING),
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 1i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 0i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Set(LevelBasedValue::Constant(0f32)),
            }],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const SMITE: Self = Self {
        id: 35u8,
        name: "minecraft:smite",
        registry_key: "smite",
        description: "enchantment.minecraft.smite",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_WEAPON,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_DAMAGE),
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 8i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[ConditionalEffect {
                effect: EnchantmentValueEffect::Add(LevelBasedValue::Linear {
                    base: 2.5f32,
                    per_level_above_first: 2.5f32,
                }),
            }],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const SOUL_SPEED: Self = Self {
        id: 36u8,
        name: "minecraft:soul_speed",
        description: "enchantment.minecraft.soul_speed",
        registry_key: "soul_speed",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FOOT_ARMOR,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Feet],
        weight: 1i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 10i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[
                ConditionalEffect {
                    effect: EnchantmentEntityEffect::AllOf(&[
                        EnchantmentEntityEffect::Other,
                        EnchantmentEntityEffect::Other,
                    ]),
                },
                ConditionalEffect {
                    effect: EnchantmentEntityEffect::ChangeItemDamage {
                        amount: LevelBasedValue::Constant(1f32),
                    },
                },
            ],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const SWEEPING_EDGE: Self = Self {
        id: 37u8,
        name: "minecraft:sweeping_edge",
        description: "enchantment.minecraft.sweeping_edge",
        registry_key: "sweeping_edge",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_SWEEPING,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 20i32,
            per_level_above_first: 9i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const SWIFT_SNEAK: Self = Self {
        id: 38u8,
        name: "minecraft:swift_sneak",
        description: "enchantment.minecraft.swift_sneak",
        registry_key: "swift_sneak",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_LEG_ARMOR,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Legs],
        weight: 1i32,
        min_cost: Cost {
            base: 25i32,
            per_level_above_first: 25i32,
        },
        max_cost: Cost {
            base: 75i32,
            per_level_above_first: 25i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const THORNS: Self = Self {
        id: 39u8,
        name: "minecraft:thorns",
        description: "enchantment.minecraft.thorns",
        registry_key: "thorns",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_ARMOR,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Any],
        weight: 1i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 20i32,
        },
        max_cost: Cost {
            base: 60i32,
            per_level_above_first: 20i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[TargetedConditionalEffect {
                enchanted: Some(EnchantmentTarget::Victim),
                affected: Some(EnchantmentTarget::Attacker),
                effect: EnchantmentEntityEffect::AllOf(&[
                    EnchantmentEntityEffect::DamageEntity {
                        min_damage: LevelBasedValue::Constant(1f32),
                        max_damage: LevelBasedValue::Constant(5f32),
                        damage_type: Some(&crate::damage::DamageType::THORNS),
                    },
                    EnchantmentEntityEffect::ChangeItemDamage {
                        amount: LevelBasedValue::Constant(2f32),
                    },
                ]),
            }],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const UNBREAKING: Self = Self {
        id: 40u8,
        name: "minecraft:unbreaking",
        description: "enchantment.minecraft.unbreaking",
        registry_key: "unbreaking",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_DURABILITY,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Any],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 55i32,
            per_level_above_first: 8i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[
                ConditionalEffect {
                    effect: EnchantmentValueEffect::RemoveBinomial(LevelBasedValue::Fraction {
                        numerator: &LevelBasedValue::Linear {
                            base: 2f32,
                            per_level_above_first: 2f32,
                        },
                        denominator: &LevelBasedValue::Linear {
                            base: 10f32,
                            per_level_above_first: 5f32,
                        },
                    }),
                },
                ConditionalEffect {
                    effect: EnchantmentValueEffect::RemoveBinomial(LevelBasedValue::Fraction {
                        numerator: &LevelBasedValue::Linear {
                            base: 1f32,
                            per_level_above_first: 1f32,
                        },
                        denominator: &LevelBasedValue::Linear {
                            base: 2f32,
                            per_level_above_first: 1f32,
                        },
                    }),
                },
            ],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub const VANISHING_CURSE: Self = Self {
        id: 41u8,
        name: "minecraft:vanishing_curse",
        description: "enchantment.minecraft.vanishing_curse",
        registry_key: "vanishing_curse",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_VANISHING,
        exclusive_set: None,
        max_level: 1i32,
        slots: &[AttributeModifierSlot::Any],
        weight: 1i32,
        min_cost: Cost {
            base: 25i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: true,
        },
    };
    pub const WIND_BURST: Self = Self {
        id: 42u8,
        name: "minecraft:wind_burst",
        description: "enchantment.minecraft.wind_burst",
        registry_key: "wind_burst",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MACE,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 9i32,
        },
        effects: EnchantmentEffects {
            projectile_spawned: &[],
            post_attack: &[TargetedConditionalEffect {
                enchanted: Some(EnchantmentTarget::Attacker),
                affected: Some(EnchantmentTarget::Attacker),
                effect: EnchantmentEntityEffect::Explode {
                    attribute_to_user: false,
                    damage_type: None,
                    knockback_multiplier: Some(LevelBasedValue::Lookup {
                        values: &[1.2f32, 1.75f32, 2.2f32],
                        fallback: &LevelBasedValue::Linear {
                            base: 1.5f32,
                            per_level_above_first: 0.35f32,
                        },
                    }),
                    immune_blocks: Some("#minecraft:blocks_wind_charge_explosions"),
                    offset_x: 0f64,
                    offset_y: 0f64,
                    offset_z: 0f64,
                    radius: LevelBasedValue::Constant(3.5f32),
                    create_fire: false,
                    block_interaction: "trigger",
                    small_particle: Some(crate::particle::Particle::GustEmitterSmall),
                    large_particle: Some(crate::particle::Particle::GustEmitterLarge),
                    sound: Some(crate::sound::Sound::EntityWindChargeWindBurst),
                },
            }],
            projectile_count: &[],
            projectile_spread: &[],
            projectile_piercing: &[],
            ammo_use: &[],
            damage: &[],
            knockback: &[],
            armor_effectiveness: &[],
            damage_protection: &[],
            hit_block: &[],
            item_damage: &[],
            equipment_drops: &[],
            fishing_time_reduction: &[],
            fishing_luck_bonus: &[],
            block_experience: &[],
            mob_experience: &[],
            repair_with_xp: &[],
            smash_damage_per_fallen_block: &[],
            trident_return_acceleration: &[],
            trident_spin_attack_strength: None,
            crossbow_charge_time: None,
            location_changed: &[],
            prevent_armor_change: false,
            prevent_equipment_drop: false,
        },
    };
    pub fn from_name(name: &str) -> Option<&'static Self> {
        match name {
            "minecraft:aqua_affinity" | "aqua_affinity" => Some(&Self::AQUA_AFFINITY),
            "minecraft:bane_of_arthropods" | "bane_of_arthropods" => {
                Some(&Self::BANE_OF_ARTHROPODS)
            }
            "minecraft:binding_curse" | "binding_curse" => Some(&Self::BINDING_CURSE),
            "minecraft:blast_protection" | "blast_protection" => Some(&Self::BLAST_PROTECTION),
            "minecraft:breach" | "breach" => Some(&Self::BREACH),
            "minecraft:channeling" | "channeling" => Some(&Self::CHANNELING),
            "minecraft:density" | "density" => Some(&Self::DENSITY),
            "minecraft:depth_strider" | "depth_strider" => Some(&Self::DEPTH_STRIDER),
            "minecraft:efficiency" | "efficiency" => Some(&Self::EFFICIENCY),
            "minecraft:feather_falling" | "feather_falling" => Some(&Self::FEATHER_FALLING),
            "minecraft:fire_aspect" | "fire_aspect" => Some(&Self::FIRE_ASPECT),
            "minecraft:fire_protection" | "fire_protection" => Some(&Self::FIRE_PROTECTION),
            "minecraft:flame" | "flame" => Some(&Self::FLAME),
            "minecraft:fortune" | "fortune" => Some(&Self::FORTUNE),
            "minecraft:frost_walker" | "frost_walker" => Some(&Self::FROST_WALKER),
            "minecraft:impaling" | "impaling" => Some(&Self::IMPALING),
            "minecraft:infinity" | "infinity" => Some(&Self::INFINITY),
            "minecraft:knockback" | "knockback" => Some(&Self::KNOCKBACK),
            "minecraft:looting" | "looting" => Some(&Self::LOOTING),
            "minecraft:loyalty" | "loyalty" => Some(&Self::LOYALTY),
            "minecraft:luck_of_the_sea" | "luck_of_the_sea" => Some(&Self::LUCK_OF_THE_SEA),
            "minecraft:lunge" | "lunge" => Some(&Self::LUNGE),
            "minecraft:lure" | "lure" => Some(&Self::LURE),
            "minecraft:mending" | "mending" => Some(&Self::MENDING),
            "minecraft:multishot" | "multishot" => Some(&Self::MULTISHOT),
            "minecraft:piercing" | "piercing" => Some(&Self::PIERCING),
            "minecraft:power" | "power" => Some(&Self::POWER),
            "minecraft:projectile_protection" | "projectile_protection" => {
                Some(&Self::PROJECTILE_PROTECTION)
            }
            "minecraft:protection" | "protection" => Some(&Self::PROTECTION),
            "minecraft:punch" | "punch" => Some(&Self::PUNCH),
            "minecraft:quick_charge" | "quick_charge" => Some(&Self::QUICK_CHARGE),
            "minecraft:respiration" | "respiration" => Some(&Self::RESPIRATION),
            "minecraft:riptide" | "riptide" => Some(&Self::RIPTIDE),
            "minecraft:sharpness" | "sharpness" => Some(&Self::SHARPNESS),
            "minecraft:silk_touch" | "silk_touch" => Some(&Self::SILK_TOUCH),
            "minecraft:smite" | "smite" => Some(&Self::SMITE),
            "minecraft:soul_speed" | "soul_speed" => Some(&Self::SOUL_SPEED),
            "minecraft:sweeping_edge" | "sweeping_edge" => Some(&Self::SWEEPING_EDGE),
            "minecraft:swift_sneak" | "swift_sneak" => Some(&Self::SWIFT_SNEAK),
            "minecraft:thorns" | "thorns" => Some(&Self::THORNS),
            "minecraft:unbreaking" | "unbreaking" => Some(&Self::UNBREAKING),
            "minecraft:vanishing_curse" | "vanishing_curse" => Some(&Self::VANISHING_CURSE),
            "minecraft:wind_burst" | "wind_burst" => Some(&Self::WIND_BURST),
            _ => None,
        }
    }
    pub fn from_id(id: u8) -> Option<&'static Self> {
        match id {
            0u8 => Some(&Self::AQUA_AFFINITY),
            1u8 => Some(&Self::BANE_OF_ARTHROPODS),
            2u8 => Some(&Self::BINDING_CURSE),
            3u8 => Some(&Self::BLAST_PROTECTION),
            4u8 => Some(&Self::BREACH),
            5u8 => Some(&Self::CHANNELING),
            6u8 => Some(&Self::DENSITY),
            7u8 => Some(&Self::DEPTH_STRIDER),
            8u8 => Some(&Self::EFFICIENCY),
            9u8 => Some(&Self::FEATHER_FALLING),
            10u8 => Some(&Self::FIRE_ASPECT),
            11u8 => Some(&Self::FIRE_PROTECTION),
            12u8 => Some(&Self::FLAME),
            13u8 => Some(&Self::FORTUNE),
            14u8 => Some(&Self::FROST_WALKER),
            15u8 => Some(&Self::IMPALING),
            16u8 => Some(&Self::INFINITY),
            17u8 => Some(&Self::KNOCKBACK),
            18u8 => Some(&Self::LOOTING),
            19u8 => Some(&Self::LOYALTY),
            20u8 => Some(&Self::LUCK_OF_THE_SEA),
            21u8 => Some(&Self::LUNGE),
            22u8 => Some(&Self::LURE),
            23u8 => Some(&Self::MENDING),
            24u8 => Some(&Self::MULTISHOT),
            25u8 => Some(&Self::PIERCING),
            26u8 => Some(&Self::POWER),
            27u8 => Some(&Self::PROJECTILE_PROTECTION),
            28u8 => Some(&Self::PROTECTION),
            29u8 => Some(&Self::PUNCH),
            30u8 => Some(&Self::QUICK_CHARGE),
            31u8 => Some(&Self::RESPIRATION),
            32u8 => Some(&Self::RIPTIDE),
            33u8 => Some(&Self::SHARPNESS),
            34u8 => Some(&Self::SILK_TOUCH),
            35u8 => Some(&Self::SMITE),
            36u8 => Some(&Self::SOUL_SPEED),
            37u8 => Some(&Self::SWEEPING_EDGE),
            38u8 => Some(&Self::SWIFT_SNEAK),
            39u8 => Some(&Self::THORNS),
            40u8 => Some(&Self::UNBREAKING),
            41u8 => Some(&Self::VANISHING_CURSE),
            42u8 => Some(&Self::WIND_BURST),
            _ => None,
        }
    }
    pub fn can_enchant(&self, item: &'static Item) -> bool {
        self.supported_items.1.contains(&item.id)
    }
    pub fn are_compatible(&self, other: &'static Enchantment) -> bool {
        if self == other {
            return false;
        }
        if let Some(tag) = self.exclusive_set
            && tag.1.contains(&(other.id as u16))
        {
            return false;
        }
        if let Some(tag) = other.exclusive_set
            && tag.1.contains(&(self.id as u16))
        {
            return false;
        }
        true
    }
    pub fn is_enchantment_compatible(&self, other: &EnchantmentsImpl) -> bool {
        for (i, _) in other.enchantment.iter() {
            if !self.are_compatible(i) {
                return false;
            }
        }
        true
    }
    #[allow(deprecated)]
    pub fn get_fullname(&self, level: i32) -> TextComponent {
        let mut ret = TextComponent::translate(self.description, []).color_named(
            if self.has_tag(&EnchantmentTag::MINECRAFT_CURSE) {
                NamedColor::Red
            } else {
                NamedColor::Gray
            },
        );
        if level != 1 || self.max_level != 1 {
            ret = ret.add_text(" ").add_child(TextComponent::translate(
                "enchantment.level.".to_string() + &level.to_string(),
                [],
            ));
        }
        ret
    }
    pub fn modify_damage_protection(&self, level: i32, protection: &mut f32) {
        for effect in self.effects.damage_protection {
            *protection = effect.effect.process(level, *protection);
        }
    }
    pub fn modify_damage(&self, level: i32, amount: &mut f64) {
        for effect in self.effects.damage {
            *amount = f64::from(effect.effect.process(level, *amount as f32));
        }
    }
    pub fn modify_fall_based_damage(&self, level: i32, amount: &mut f64) {
        for effect in self.effects.smash_damage_per_fallen_block {
            *amount = f64::from(effect.effect.process(level, *amount as f32));
        }
    }
    pub fn modify_knockback(&self, level: i32, amount: &mut f32) {
        for effect in self.effects.knockback {
            *amount = effect.effect.process(level, *amount);
        }
    }
    pub fn modify_armor_effectiveness(&self, level: i32, amount: &mut f32) {
        for effect in self.effects.armor_effectiveness {
            *amount = effect.effect.process(level, *amount);
        }
    }
    pub fn modify_durability_change(&self, level: i32, change: &mut f32) {
        for effect in self.effects.item_damage {
            *change = effect.effect.process(level, *change);
        }
    }
    pub fn modify_ammo_count(&self, level: i32, change: &mut f32) {
        for effect in self.effects.ammo_use {
            *change = effect.effect.process(level, *change);
        }
    }
    pub fn modify_piercing_count(&self, level: i32, count: &mut f32) {
        for effect in self.effects.projectile_piercing {
            *count = effect.effect.process(level, *count);
        }
    }
    pub fn modify_block_experience(&self, level: i32, count: &mut f32) {
        for effect in self.effects.block_experience {
            *count = effect.effect.process(level, *count);
        }
    }
    pub fn modify_mob_experience(&self, level: i32, experience: &mut f32) {
        for effect in self.effects.mob_experience {
            *experience = effect.effect.process(level, *experience);
        }
    }
    pub fn modify_durability_to_repair_from_xp(&self, level: i32, change: &mut f32) {
        for effect in self.effects.repair_with_xp {
            *change = effect.effect.process(level, *change);
        }
    }
    pub fn modify_trident_return_to_owner_acceleration(&self, level: i32, count: &mut f32) {
        for effect in self.effects.trident_return_acceleration {
            *count = effect.effect.process(level, *count);
        }
    }
    pub fn modify_trident_spin_attack_strength(&self, level: i32, strength: &mut f32) {
        if let Some(effect) = &self.effects.trident_spin_attack_strength {
            *strength = effect.process(level, *strength);
        }
    }
    pub fn modify_fishing_time_reduction(&self, level: i32, time_reduction: &mut f32) {
        for effect in self.effects.fishing_time_reduction {
            *time_reduction = effect.effect.process(level, *time_reduction);
        }
    }
    pub fn modify_fishing_luck_bonus(&self, level: i32, luck: &mut f32) {
        for effect in self.effects.fishing_luck_bonus {
            *luck = effect.effect.process(level, *luck);
        }
    }
    pub fn modify_projectile_count(&self, level: i32, count: &mut f32) {
        for effect in self.effects.projectile_count {
            *count = effect.effect.process(level, *count);
        }
    }
    pub fn modify_projectile_spread(&self, level: i32, angle: &mut f32) {
        for effect in self.effects.projectile_spread {
            *angle = effect.effect.process(level, *angle);
        }
    }
    pub fn modify_crossbow_charge_time(&self, level: i32, time: &mut f32) {
        if let Some(effect) = &self.effects.crossbow_charge_time {
            *time = effect.process(level, *time);
        }
    }
    pub fn get_projectile_spawned_effects(
        &self,
    ) -> &'static [ConditionalEffect<EnchantmentEntityEffect>] {
        self.effects.projectile_spawned
    }
    pub fn get_location_changed_effects(
        &self,
    ) -> &'static [ConditionalEffect<EnchantmentEntityEffect>] {
        self.effects.location_changed
    }
}
