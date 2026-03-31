use crate::entity::EntityBase;
use crate::entity::living::LivingEntity;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::item_stack::ItemStack;

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
                if let Some(se) = StatusEffect::from_minecraft_name(&ce.effect_id) {
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
