/* This file is generated. Do not edit manually. */
use crate::attributes::Attributes;
use crate::damage::DamageType;
use crate::data_component_impl::Operation;
use crate::sound::Sound;
use crate::tag::Tag;
use pumpkin_util::math::float_provider::FloatProvider;
pub struct SulfurCubeAttributeModifier {
    pub attribute: &'static Attributes,
    pub id: &'static str,
    pub amount: f64,
    pub operation: Operation,
}
pub struct SulfurCubeExplosion {
    pub power: i32,
    pub causes_fire: bool,
    pub fuse: i32,
}
pub struct SulfurCubeContactDamage {
    pub damage_type: &'static DamageType,
    pub amount: FloatProvider,
    pub attribute_to_source: bool,
}
#[derive(Clone, Copy)]
pub struct SulfurCubeKnockbackModifiers {
    pub horizontal_power: f32,
    pub vertical_power: f32,
}
#[derive(Clone, Copy)]
pub struct SulfurCubeSoundSettings {
    pub hit_sound: Sound,
    pub push_sound: Sound,
    pub push_sound_impulse_threshold: f32,
    pub push_sound_cooldown: f32,
}
pub struct SulfurCubeArchetype {
    pub name: &'static str,
    pub items: &'static Tag,
    pub attribute_modifiers: &'static [SulfurCubeAttributeModifier],
    pub buoyant: bool,
    pub explosion: Option<SulfurCubeExplosion>,
    pub contact_damage: Option<SulfurCubeContactDamage>,
    pub knockback_modifiers: SulfurCubeKnockbackModifiers,
    pub sound_settings: SulfurCubeSoundSettings,
}
impl SulfurCubeArchetype {
    pub const BOUNCY: Self = Self {
        name: "bouncy",
        items: &crate::tag::Item::MINECRAFT_SULFUR_CUBE_ARCHETYPE_BOUNCY,
        attribute_modifiers: &[
            SulfurCubeAttributeModifier {
                attribute: &Attributes::KNOCKBACK_RESISTANCE,
                id: "minecraft:bouncy_add_knockback_resistance",
                amount: -2f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
                id: "minecraft:bouncy_add_explosion_knockback_resistance",
                amount: -2f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::BOUNCINESS,
                id: "minecraft:bouncy_add_bounciness",
                amount: 0.8999999761581421f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::FRICTION_MODIFIER,
                id: "minecraft:bouncy_mul_friction_modifier",
                amount: -0.699999988079071f64,
                operation: Operation::AddMultipliedTotal,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::AIR_DRAG_MODIFIER,
                id: "minecraft:bouncy_mul_air_drag_modifier",
                amount: -0.9900000002235174f64,
                operation: Operation::AddMultipliedTotal,
            },
        ],
        buoyant: true,
        explosion: None,
        contact_damage: None,
        knockback_modifiers: SulfurCubeKnockbackModifiers {
            horizontal_power: 0.4125f32,
            vertical_power: 0.105f32,
        },
        sound_settings: SulfurCubeSoundSettings {
            hit_sound: Sound::EntitySulfurCubeBouncyHit,
            push_sound: Sound::EntitySulfurCubeBouncyPush,
            push_sound_impulse_threshold: 0.3f32,
            push_sound_cooldown: 0.7f32,
        },
    };
    pub const EXPLOSIVE: Self = Self {
        name: "explosive",
        items: &crate::tag::Item::MINECRAFT_SULFUR_CUBE_ARCHETYPE_EXPLOSIVE,
        attribute_modifiers: &[
            SulfurCubeAttributeModifier {
                attribute: &Attributes::KNOCKBACK_RESISTANCE,
                id: "minecraft:explosive_add_knockback_resistance",
                amount: -1f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
                id: "minecraft:explosive_add_explosion_knockback_resistance",
                amount: -1f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::BOUNCINESS,
                id: "minecraft:explosive_add_bounciness",
                amount: 0.5f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::FRICTION_MODIFIER,
                id: "minecraft:explosive_mul_friction_modifier",
                amount: -0.699999988079071f64,
                operation: Operation::AddMultipliedTotal,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::AIR_DRAG_MODIFIER,
                id: "minecraft:explosive_mul_air_drag_modifier",
                amount: -0.699999988079071f64,
                operation: Operation::AddMultipliedTotal,
            },
        ],
        buoyant: true,
        explosion: Some(SulfurCubeExplosion {
            power: 3i32,
            causes_fire: false,
            fuse: 120i32,
        }),
        contact_damage: None,
        knockback_modifiers: SulfurCubeKnockbackModifiers {
            horizontal_power: 0.4125f32,
            vertical_power: 0.09f32,
        },
        sound_settings: SulfurCubeSoundSettings {
            hit_sound: Sound::EntitySulfurCubeExplosiveHit,
            push_sound: Sound::EntitySulfurCubeExplosivePush,
            push_sound_impulse_threshold: 0.1f32,
            push_sound_cooldown: 0.7f32,
        },
    };
    pub const FAST_FLAT: Self = Self {
        name: "fast_flat",
        items: &crate::tag::Item::MINECRAFT_SULFUR_CUBE_ARCHETYPE_FAST_FLAT,
        attribute_modifiers: &[
            SulfurCubeAttributeModifier {
                attribute: &Attributes::KNOCKBACK_RESISTANCE,
                id: "minecraft:fast_flat_add_knockback_resistance",
                amount: -1f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
                id: "minecraft:fast_flat_add_explosion_knockback_resistance",
                amount: -1f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::BOUNCINESS,
                id: "minecraft:fast_flat_add_bounciness",
                amount: 0.5f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::FRICTION_MODIFIER,
                id: "minecraft:fast_flat_mul_friction_modifier",
                amount: -0.7999999970197678f64,
                operation: Operation::AddMultipliedTotal,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::AIR_DRAG_MODIFIER,
                id: "minecraft:fast_flat_mul_air_drag_modifier",
                amount: -0.9900000002235174f64,
                operation: Operation::AddMultipliedTotal,
            },
        ],
        buoyant: false,
        explosion: None,
        contact_damage: None,
        knockback_modifiers: SulfurCubeKnockbackModifiers {
            horizontal_power: 0.9125f32,
            vertical_power: 0.09f32,
        },
        sound_settings: SulfurCubeSoundSettings {
            hit_sound: Sound::EntitySulfurCubeFastFlatHit,
            push_sound: Sound::EntitySulfurCubeFastFlatPush,
            push_sound_impulse_threshold: 0.03f32,
            push_sound_cooldown: 0.9f32,
        },
    };
    pub const FAST_SLIDING: Self = Self {
        name: "fast_sliding",
        items: &crate::tag::Item::MINECRAFT_SULFUR_CUBE_ARCHETYPE_FAST_SLIDING,
        attribute_modifiers: &[
            SulfurCubeAttributeModifier {
                attribute: &Attributes::KNOCKBACK_RESISTANCE,
                id: "minecraft:fast_sliding_add_knockback_resistance",
                amount: 0.5f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
                id: "minecraft:fast_sliding_add_explosion_knockback_resistance",
                amount: 0.5f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::BOUNCINESS,
                id: "minecraft:fast_sliding_add_bounciness",
                amount: 0.10000000149011612f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::FRICTION_MODIFIER,
                id: "minecraft:fast_sliding_mul_friction_modifier",
                amount: -0.9499999992549419f64,
                operation: Operation::AddMultipliedTotal,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::AIR_DRAG_MODIFIER,
                id: "minecraft:fast_sliding_mul_air_drag_modifier",
                amount: -0.9900000002235174f64,
                operation: Operation::AddMultipliedTotal,
            },
        ],
        buoyant: false,
        explosion: None,
        contact_damage: None,
        knockback_modifiers: SulfurCubeKnockbackModifiers {
            horizontal_power: 0.6625f32,
            vertical_power: 0.09f32,
        },
        sound_settings: SulfurCubeSoundSettings {
            hit_sound: Sound::EntitySulfurCubeFastSlidingHit,
            push_sound: Sound::EntitySulfurCubeFastSlidingPush,
            push_sound_impulse_threshold: 0.05f32,
            push_sound_cooldown: 1f32,
        },
    };
    pub const HIGH_RESISTANCE: Self = Self {
        name: "high_resistance",
        items: &crate::tag::Item::MINECRAFT_SULFUR_CUBE_ARCHETYPE_HIGH_RESISTANCE,
        attribute_modifiers: &[
            SulfurCubeAttributeModifier {
                attribute: &Attributes::KNOCKBACK_RESISTANCE,
                id: "minecraft:high_resistance_add_knockback_resistance",
                amount: 0.699999988079071f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
                id: "minecraft:high_resistance_add_explosion_knockback_resistance",
                amount: 0.699999988079071f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::BOUNCINESS,
                id: "minecraft:high_resistance_add_bounciness",
                amount: 0.20000000298023224f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::FRICTION_MODIFIER,
                id: "minecraft:high_resistance_mul_friction_modifier",
                amount: 0f64,
                operation: Operation::AddMultipliedTotal,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::AIR_DRAG_MODIFIER,
                id: "minecraft:high_resistance_mul_air_drag_modifier",
                amount: -0.9900000002235174f64,
                operation: Operation::AddMultipliedTotal,
            },
        ],
        buoyant: false,
        explosion: None,
        contact_damage: None,
        knockback_modifiers: SulfurCubeKnockbackModifiers {
            horizontal_power: 0.4125f32,
            vertical_power: 0.09f32,
        },
        sound_settings: SulfurCubeSoundSettings {
            hit_sound: Sound::EntitySulfurCubeHighResistanceHit,
            push_sound: Sound::EntitySulfurCubeHighResistancePush,
            push_sound_impulse_threshold: 0.03f32,
            push_sound_cooldown: 0.7f32,
        },
    };
    pub const HOT: Self = Self {
        name: "hot",
        items: &crate::tag::Item::MINECRAFT_SULFUR_CUBE_ARCHETYPE_HOT,
        attribute_modifiers: &[
            SulfurCubeAttributeModifier {
                attribute: &Attributes::KNOCKBACK_RESISTANCE,
                id: "minecraft:hot_add_knockback_resistance",
                amount: -1f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
                id: "minecraft:hot_add_explosion_knockback_resistance",
                amount: -1f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::BOUNCINESS,
                id: "minecraft:hot_add_bounciness",
                amount: 0.5f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::FRICTION_MODIFIER,
                id: "minecraft:hot_mul_friction_modifier",
                amount: -0.699999988079071f64,
                operation: Operation::AddMultipliedTotal,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::AIR_DRAG_MODIFIER,
                id: "minecraft:hot_mul_air_drag_modifier",
                amount: -0.8999999985098839f64,
                operation: Operation::AddMultipliedTotal,
            },
        ],
        buoyant: true,
        explosion: None,
        contact_damage: Some(SulfurCubeContactDamage {
            damage_type: &DamageType::SULFUR_CUBE_HOT,
            amount: FloatProvider::Constant(1f32),
            attribute_to_source: false,
        }),
        knockback_modifiers: SulfurCubeKnockbackModifiers {
            horizontal_power: 0.4125f32,
            vertical_power: 0.09f32,
        },
        sound_settings: SulfurCubeSoundSettings {
            hit_sound: Sound::EntitySulfurCubeHotHit,
            push_sound: Sound::EntitySulfurCubeHotPush,
            push_sound_impulse_threshold: 0.2f32,
            push_sound_cooldown: 0.7f32,
        },
    };
    pub const LIGHT: Self = Self {
        name: "light",
        items: &crate::tag::Item::MINECRAFT_SULFUR_CUBE_ARCHETYPE_LIGHT,
        attribute_modifiers: &[
            SulfurCubeAttributeModifier {
                attribute: &Attributes::KNOCKBACK_RESISTANCE,
                id: "minecraft:light_add_knockback_resistance",
                amount: -1f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
                id: "minecraft:light_add_explosion_knockback_resistance",
                amount: -1f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::BOUNCINESS,
                id: "minecraft:light_add_bounciness",
                amount: 1f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::FRICTION_MODIFIER,
                id: "minecraft:light_mul_friction_modifier",
                amount: -0.699999988079071f64,
                operation: Operation::AddMultipliedTotal,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::AIR_DRAG_MODIFIER,
                id: "minecraft:light_mul_air_drag_modifier",
                amount: 0.7999999523162842f64,
                operation: Operation::AddMultipliedTotal,
            },
        ],
        buoyant: true,
        explosion: None,
        contact_damage: None,
        knockback_modifiers: SulfurCubeKnockbackModifiers {
            horizontal_power: 0.4125f32,
            vertical_power: 0.18f32,
        },
        sound_settings: SulfurCubeSoundSettings {
            hit_sound: Sound::EntitySulfurCubeLightHit,
            push_sound: Sound::EntitySulfurCubeLightPush,
            push_sound_impulse_threshold: 0.2f32,
            push_sound_cooldown: 0.7f32,
        },
    };
    pub const REGULAR: Self = Self {
        name: "regular",
        items: &crate::tag::Item::MINECRAFT_SULFUR_CUBE_ARCHETYPE_REGULAR,
        attribute_modifiers: &[
            SulfurCubeAttributeModifier {
                attribute: &Attributes::KNOCKBACK_RESISTANCE,
                id: "minecraft:regular_add_knockback_resistance",
                amount: -1f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
                id: "minecraft:regular_add_explosion_knockback_resistance",
                amount: -1f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::BOUNCINESS,
                id: "minecraft:regular_add_bounciness",
                amount: 0.5f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::FRICTION_MODIFIER,
                id: "minecraft:regular_mul_friction_modifier",
                amount: -0.699999988079071f64,
                operation: Operation::AddMultipliedTotal,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::AIR_DRAG_MODIFIER,
                id: "minecraft:regular_mul_air_drag_modifier",
                amount: -0.8999999985098839f64,
                operation: Operation::AddMultipliedTotal,
            },
        ],
        buoyant: true,
        explosion: None,
        contact_damage: None,
        knockback_modifiers: SulfurCubeKnockbackModifiers {
            horizontal_power: 0.4125f32,
            vertical_power: 0.09f32,
        },
        sound_settings: SulfurCubeSoundSettings {
            hit_sound: Sound::EntitySulfurCubeRegularHit,
            push_sound: Sound::EntitySulfurCubeRegularPush,
            push_sound_impulse_threshold: 0.2f32,
            push_sound_cooldown: 0.5f32,
        },
    };
    pub const SLOW_BOUNCY: Self = Self {
        name: "slow_bouncy",
        items: &crate::tag::Item::MINECRAFT_SULFUR_CUBE_ARCHETYPE_SLOW_BOUNCY,
        attribute_modifiers: &[
            SulfurCubeAttributeModifier {
                attribute: &Attributes::KNOCKBACK_RESISTANCE,
                id: "minecraft:slow_bouncy_add_knockback_resistance",
                amount: 0.4000000059604645f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
                id: "minecraft:slow_bouncy_add_explosion_knockback_resistance",
                amount: 0.4000000059604645f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::BOUNCINESS,
                id: "minecraft:slow_bouncy_add_bounciness",
                amount: 0.6000000238418579f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::FRICTION_MODIFIER,
                id: "minecraft:slow_bouncy_mul_friction_modifier",
                amount: -0.699999988079071f64,
                operation: Operation::AddMultipliedTotal,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::AIR_DRAG_MODIFIER,
                id: "minecraft:slow_bouncy_mul_air_drag_modifier",
                amount: -0.9499999992549419f64,
                operation: Operation::AddMultipliedTotal,
            },
        ],
        buoyant: false,
        explosion: None,
        contact_damage: None,
        knockback_modifiers: SulfurCubeKnockbackModifiers {
            horizontal_power: 0.4125f32,
            vertical_power: 0.24f32,
        },
        sound_settings: SulfurCubeSoundSettings {
            hit_sound: Sound::EntitySulfurCubeSlowBouncyHit,
            push_sound: Sound::EntitySulfurCubeSlowBouncyPush,
            push_sound_impulse_threshold: 0.05f32,
            push_sound_cooldown: 0.5f32,
        },
    };
    pub const SLOW_FLAT: Self = Self {
        name: "slow_flat",
        items: &crate::tag::Item::MINECRAFT_SULFUR_CUBE_ARCHETYPE_SLOW_FLAT,
        attribute_modifiers: &[
            SulfurCubeAttributeModifier {
                attribute: &Attributes::KNOCKBACK_RESISTANCE,
                id: "minecraft:slow_flat_add_knockback_resistance",
                amount: 0.5f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
                id: "minecraft:slow_flat_add_explosion_knockback_resistance",
                amount: 0.5f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::BOUNCINESS,
                id: "minecraft:slow_flat_add_bounciness",
                amount: 0.4000000059604645f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::FRICTION_MODIFIER,
                id: "minecraft:slow_flat_mul_friction_modifier",
                amount: -0.5999999940395355f64,
                operation: Operation::AddMultipliedTotal,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::AIR_DRAG_MODIFIER,
                id: "minecraft:slow_flat_mul_air_drag_modifier",
                amount: -0.8999999985098839f64,
                operation: Operation::AddMultipliedTotal,
            },
        ],
        buoyant: false,
        explosion: None,
        contact_damage: None,
        knockback_modifiers: SulfurCubeKnockbackModifiers {
            horizontal_power: 0.4125f32,
            vertical_power: 0.105f32,
        },
        sound_settings: SulfurCubeSoundSettings {
            hit_sound: Sound::EntitySulfurCubeSlowFlatHit,
            push_sound: Sound::EntitySulfurCubeSlowFlatPush,
            push_sound_impulse_threshold: 0.03f32,
            push_sound_cooldown: 0.9f32,
        },
    };
    pub const SLOW_SLIDING: Self = Self {
        name: "slow_sliding",
        items: &crate::tag::Item::MINECRAFT_SULFUR_CUBE_ARCHETYPE_SLOW_SLIDING,
        attribute_modifiers: &[
            SulfurCubeAttributeModifier {
                attribute: &Attributes::KNOCKBACK_RESISTANCE,
                id: "minecraft:slow_sliding_add_knockback_resistance",
                amount: 0.800000011920929f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
                id: "minecraft:slow_sliding_add_explosion_knockback_resistance",
                amount: 0.800000011920929f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::BOUNCINESS,
                id: "minecraft:slow_sliding_add_bounciness",
                amount: 0.10000000149011612f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::FRICTION_MODIFIER,
                id: "minecraft:slow_sliding_mul_friction_modifier",
                amount: -0.9499999992549419f64,
                operation: Operation::AddMultipliedTotal,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::AIR_DRAG_MODIFIER,
                id: "minecraft:slow_sliding_mul_air_drag_modifier",
                amount: -0.9900000002235174f64,
                operation: Operation::AddMultipliedTotal,
            },
        ],
        buoyant: false,
        explosion: None,
        contact_damage: None,
        knockback_modifiers: SulfurCubeKnockbackModifiers {
            horizontal_power: 0.4125f32,
            vertical_power: 0.09f32,
        },
        sound_settings: SulfurCubeSoundSettings {
            hit_sound: Sound::EntitySulfurCubeSlowSlidingHit,
            push_sound: Sound::EntitySulfurCubeSlowSlidingPush,
            push_sound_impulse_threshold: 0.02f32,
            push_sound_cooldown: 1f32,
        },
    };
    pub const STICKY: Self = Self {
        name: "sticky",
        items: &crate::tag::Item::MINECRAFT_SULFUR_CUBE_ARCHETYPE_STICKY,
        attribute_modifiers: &[
            SulfurCubeAttributeModifier {
                attribute: &Attributes::KNOCKBACK_RESISTANCE,
                id: "minecraft:sticky_add_knockback_resistance",
                amount: -2f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
                id: "minecraft:sticky_add_explosion_knockback_resistance",
                amount: -2f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::BOUNCINESS,
                id: "minecraft:sticky_add_bounciness",
                amount: 0f64,
                operation: Operation::AddValue,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::FRICTION_MODIFIER,
                id: "minecraft:sticky_mul_friction_modifier",
                amount: 1f64,
                operation: Operation::AddMultipliedTotal,
            },
            SulfurCubeAttributeModifier {
                attribute: &Attributes::AIR_DRAG_MODIFIER,
                id: "minecraft:sticky_mul_air_drag_modifier",
                amount: -0.9900000002235174f64,
                operation: Operation::AddMultipliedTotal,
            },
        ],
        buoyant: false,
        explosion: None,
        contact_damage: None,
        knockback_modifiers: SulfurCubeKnockbackModifiers {
            horizontal_power: 0.4125f32,
            vertical_power: 0.09f32,
        },
        sound_settings: SulfurCubeSoundSettings {
            hit_sound: Sound::EntitySulfurCubeStickyHit,
            push_sound: Sound::EntitySulfurCubeStickyPush,
            push_sound_impulse_threshold: 0.05f32,
            push_sound_cooldown: 0.5f32,
        },
    };
    #[must_use]
    pub const fn all() -> &'static [&'static Self] {
        &[
            &Self::BOUNCY,
            &Self::EXPLOSIVE,
            &Self::FAST_FLAT,
            &Self::FAST_SLIDING,
            &Self::HIGH_RESISTANCE,
            &Self::HOT,
            &Self::LIGHT,
            &Self::REGULAR,
            &Self::SLOW_BOUNCY,
            &Self::SLOW_FLAT,
            &Self::SLOW_SLIDING,
            &Self::STICKY,
        ]
    }
}
