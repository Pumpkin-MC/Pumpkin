use crate::entity::EntityBase;
use crate::entity::living::LivingEntity;
use pumpkin_data::effect::StatusEffect;
use pumpkin_world::item::ItemStack;

/// Utilities for reading potion contents from an `ItemStack` and applying effects.
pub struct PotionContents;

/// Source context for applying potion effects (affects scaling rules).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PotionApplicationSource {
    /// Normal application (drinking / splash)
    Normal,
    /// `AreaEffectCloud` application (shorter durations and weaker instant potency)
    AreaEffectCloud,
}

impl PotionContents {
    /// Read effects from an `ItemStack`'s `PotionContents` data component.
    #[must_use]
    pub fn read_potion_effects(
        stack: &ItemStack,
    ) -> Vec<(&'static StatusEffect, i32, u8, bool, bool, bool)> {
        // Prefer generated potion id if present, otherwise use custom_effects
        if let Some(pc) =
            stack.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
        {
            // Custom effects present
            let mut out = Vec::new();
            if let Some(potion_id) = pc.potion_id {
                // Map potion id to generated Potion if possible
                macro_rules! try_push_potion {
                    ($p:expr) => {
                        if $p.id as i32 == potion_id {
                            for e in $p.effects {
                                out.push((
                                    e.effect_type,
                                    e.duration,
                                    e.amplifier,
                                    e.ambient,
                                    e.show_particles,
                                    e.show_icon,
                                ));
                            }
                        }
                    };
                }
                try_push_potion!(pumpkin_data::potion::Potion::AWKWARD);
                try_push_potion!(pumpkin_data::potion::Potion::FIRE_RESISTANCE);
                try_push_potion!(pumpkin_data::potion::Potion::HARMING);
                try_push_potion!(pumpkin_data::potion::Potion::HEALING);
                try_push_potion!(pumpkin_data::potion::Potion::INFESTED);
                try_push_potion!(pumpkin_data::potion::Potion::INVISIBILITY);
                try_push_potion!(pumpkin_data::potion::Potion::LEAPING);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_FIRE_RESISTANCE);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_INVISIBILITY);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_LEAPING);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_NIGHT_VISION);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_POISON);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_REGENERATION);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_SLOW_FALLING);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_SLOWNESS);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_STRENGTH);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_SWIFTNESS);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_TURTLE_MASTER);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_WATER_BREATHING);
                try_push_potion!(pumpkin_data::potion::Potion::LONG_WEAKNESS);
                try_push_potion!(pumpkin_data::potion::Potion::LUCK);
                try_push_potion!(pumpkin_data::potion::Potion::MUNDANE);
                try_push_potion!(pumpkin_data::potion::Potion::NIGHT_VISION);
                try_push_potion!(pumpkin_data::potion::Potion::OOZING);
                try_push_potion!(pumpkin_data::potion::Potion::POISON);
                try_push_potion!(pumpkin_data::potion::Potion::REGENERATION);
                try_push_potion!(pumpkin_data::potion::Potion::SLOW_FALLING);
                try_push_potion!(pumpkin_data::potion::Potion::SLOWNESS);
                try_push_potion!(pumpkin_data::potion::Potion::STRENGTH);
                try_push_potion!(pumpkin_data::potion::Potion::STRONG_HARMING);
                try_push_potion!(pumpkin_data::potion::Potion::STRONG_HEALING);
                try_push_potion!(pumpkin_data::potion::Potion::STRONG_LEAPING);
                try_push_potion!(pumpkin_data::potion::Potion::STRONG_POISON);
                try_push_potion!(pumpkin_data::potion::Potion::STRONG_REGENERATION);
                try_push_potion!(pumpkin_data::potion::Potion::STRONG_SLOWNESS);
                try_push_potion!(pumpkin_data::potion::Potion::STRONG_STRENGTH);
                try_push_potion!(pumpkin_data::potion::Potion::STRONG_SWIFTNESS);
                try_push_potion!(pumpkin_data::potion::Potion::STRONG_TURTLE_MASTER);
                try_push_potion!(pumpkin_data::potion::Potion::SWIFTNESS);
                try_push_potion!(pumpkin_data::potion::Potion::THICK);
                try_push_potion!(pumpkin_data::potion::Potion::TURTLE_MASTER);
                try_push_potion!(pumpkin_data::potion::Potion::WATER);
                try_push_potion!(pumpkin_data::potion::Potion::WATER_BREATHING);
                try_push_potion!(pumpkin_data::potion::Potion::WEAKNESS);
                try_push_potion!(pumpkin_data::potion::Potion::WEAVING);
                try_push_potion!(pumpkin_data::potion::Potion::WIND_CHARGED);
            }

            // Custom effects appended
            for ce in &pc.custom_effects {
                if let Some(se) = status_effect_from_id(ce.effect_id) {
                    out.push((
                        se,
                        ce.duration,
                        ce.amplifier as u8,
                        ce.ambient,
                        ce.show_particles,
                        ce.show_icon,
                    ));
                }
            }

            return out;
        }

        Vec::new()
    }

    /// Apply instant or duration effects to a target living entity.
    pub async fn apply_effects_to(
        target: &LivingEntity,
        effects: Vec<(&'static StatusEffect, i32, u8, bool, bool, bool)>,
        scale: f32,
        source: PotionApplicationSource,
    ) {
        for (effect_type, duration, amplifier, ambient, show_particles, show_icon) in effects {
            // Instant effects should apply immediately
            let is_instant = effect_type.id
                == pumpkin_data::effect::StatusEffect::INSTANT_HEALTH.id
                || effect_type.id == pumpkin_data::effect::StatusEffect::INSTANT_DAMAGE.id;

            if is_instant {
                // Instant potency scaling
                let instant_scale = if source == PotionApplicationSource::AreaEffectCloud {
                    scale * 0.5
                } else {
                    scale
                };

                // Apply instant effects logic directly as they don't tick
                if effect_type.id == pumpkin_data::effect::StatusEffect::INSTANT_HEALTH.id {
                    let amount = (4 * ((amplifier as i32) + 1)) as f32 * instant_scale;
                    target.heal(amount).await;
                } else if effect_type.id == pumpkin_data::effect::StatusEffect::INSTANT_DAMAGE.id {
                    let amount = (6 * ((amplifier as i32) + 1)) as f32 * instant_scale;

                    target
                        .damage(
                            target.get_entity(),
                            amount,
                            pumpkin_data::damage::DamageType::MAGIC,
                        )
                        .await;
                }

                // For instant effects, still add a short visual effect entry as before
                let eff = pumpkin_data::potion::Effect {
                    effect_type,
                    duration: 1,
                    amplifier,
                    ambient,
                    show_particles,
                    show_icon,
                    blend: false,
                };
                target.add_effect(eff).await;
            } else {
                // Duration scaling
                let duration_scale = if source == PotionApplicationSource::AreaEffectCloud {
                    scale * 0.25
                } else {
                    scale
                };

                let dur = ((duration as f32) * duration_scale).max(1.0) as i32;
                let eff = pumpkin_data::potion::Effect {
                    effect_type,
                    duration: dur,
                    amplifier,
                    ambient,
                    show_particles,
                    show_icon,
                    blend: false,
                };
                target.add_effect(eff).await;
            }
        }
    }
}

/// Map numeric effect id to generated `StatusEffect` const. This is a manual mapping because the generated
/// code does not provide a `from_id` helper. TODO: fix this?
const fn status_effect_from_id(id: i32) -> Option<&'static StatusEffect> {
    match id as u8 {
        x if x == StatusEffect::ABSORPTION.id => Some(&StatusEffect::ABSORPTION),
        x if x == StatusEffect::BAD_OMEN.id => Some(&StatusEffect::BAD_OMEN),
        x if x == StatusEffect::BLINDNESS.id => Some(&StatusEffect::BLINDNESS),
        x if x == StatusEffect::CONDUIT_POWER.id => Some(&StatusEffect::CONDUIT_POWER),
        x if x == StatusEffect::DARKNESS.id => Some(&StatusEffect::DARKNESS),
        x if x == StatusEffect::DOLPHINS_GRACE.id => Some(&StatusEffect::DOLPHINS_GRACE),
        x if x == StatusEffect::FIRE_RESISTANCE.id => Some(&StatusEffect::FIRE_RESISTANCE),
        x if x == StatusEffect::GLOWING.id => Some(&StatusEffect::GLOWING),
        x if x == StatusEffect::HASTE.id => Some(&StatusEffect::HASTE),
        x if x == StatusEffect::HEALTH_BOOST.id => Some(&StatusEffect::HEALTH_BOOST),
        x if x == StatusEffect::HERO_OF_THE_VILLAGE.id => Some(&StatusEffect::HERO_OF_THE_VILLAGE),
        x if x == StatusEffect::HUNGER.id => Some(&StatusEffect::HUNGER),
        x if x == StatusEffect::INFESTED.id => Some(&StatusEffect::INFESTED),
        x if x == StatusEffect::INSTANT_DAMAGE.id => Some(&StatusEffect::INSTANT_DAMAGE),
        x if x == StatusEffect::INSTANT_HEALTH.id => Some(&StatusEffect::INSTANT_HEALTH),
        x if x == StatusEffect::INVISIBILITY.id => Some(&StatusEffect::INVISIBILITY),
        x if x == StatusEffect::JUMP_BOOST.id => Some(&StatusEffect::JUMP_BOOST),
        x if x == StatusEffect::LEVITATION.id => Some(&StatusEffect::LEVITATION),
        x if x == StatusEffect::LUCK.id => Some(&StatusEffect::LUCK),
        x if x == StatusEffect::MINING_FATIGUE.id => Some(&StatusEffect::MINING_FATIGUE),
        x if x == StatusEffect::NAUSEA.id => Some(&StatusEffect::NAUSEA),
        x if x == StatusEffect::NIGHT_VISION.id => Some(&StatusEffect::NIGHT_VISION),
        x if x == StatusEffect::OOZING.id => Some(&StatusEffect::OOZING),
        x if x == StatusEffect::POISON.id => Some(&StatusEffect::POISON),
        x if x == StatusEffect::RAID_OMEN.id => Some(&StatusEffect::RAID_OMEN),
        x if x == StatusEffect::REGENERATION.id => Some(&StatusEffect::REGENERATION),
        x if x == StatusEffect::RESISTANCE.id => Some(&StatusEffect::RESISTANCE),
        x if x == StatusEffect::SATURATION.id => Some(&StatusEffect::SATURATION),
        x if x == StatusEffect::SLOW_FALLING.id => Some(&StatusEffect::SLOW_FALLING),
        x if x == StatusEffect::SLOWNESS.id => Some(&StatusEffect::SLOWNESS),
        x if x == StatusEffect::SPEED.id => Some(&StatusEffect::SPEED),
        x if x == StatusEffect::STRENGTH.id => Some(&StatusEffect::STRENGTH),
        x if x == StatusEffect::UNLUCK.id => Some(&StatusEffect::UNLUCK),
        x if x == StatusEffect::WATER_BREATHING.id => Some(&StatusEffect::WATER_BREATHING),
        x if x == StatusEffect::WEAKNESS.id => Some(&StatusEffect::WEAKNESS),
        x if x == StatusEffect::WEAVING.id => Some(&StatusEffect::WEAVING),
        x if x == StatusEffect::WIND_CHARGED.id => Some(&StatusEffect::WIND_CHARGED),
        x if x == StatusEffect::WITHER.id => Some(&StatusEffect::WITHER),
        _ => None,
    }
}
