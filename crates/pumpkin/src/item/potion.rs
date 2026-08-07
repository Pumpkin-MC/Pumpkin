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
    Arrow,
}

impl PotionApplicationSource {
    // AreaEffectCloud.java: `effect.getEffect().value().applyInstantaneousEffect(...,
    // effect.getAmplifier(), 0.5)` -- the potency factor is the literal constant 0.5,
    // independent of the cloud's distance-based duration scale, not `scale * 0.5`.
    const fn instant_scale(self, scale: f32) -> f32 {
        match self {
            Self::AreaEffectCloud => 0.5,
            Self::Arrow => 1.0,
            Self::Normal => scale,
        }
    }

    const fn duration_scale(self, scale: f32) -> f32 {
        match self {
            Self::AreaEffectCloud => scale * 0.25,
            Self::Arrow | Self::Normal => scale,
        }
    }
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
                let instant_scale = source.instant_scale(scale);

                // Apply instant effects logic directly as they don't tick
                let inverted = target.is_undead();
                if LivingEntity::instant_effect_is_damage(effect_type, inverted) {
                    let amount = (6 * ((amplifier as i32) + 1)) as f32 * instant_scale;

                    target
                        .damage(
                            target.get_entity(),
                            amount,
                            pumpkin_data::damage::DamageType::MAGIC,
                        )
                        .await;
                } else {
                    let amount = (4 * ((amplifier as i32) + 1)) as f32 * instant_scale;
                    target.heal(amount);
                }
                // Vanilla applies instant effects via a direct heal/damage call only
                // (ThrownSplashPotion#applyInstantaneousEffect / HealOrHarmMobEffect), never
                // through addEffect -- calling `target.add_effect` here as well would apply the
                // heal/damage a second time, since `LivingEntity::add_effect` independently
                // re-implements the instant-effect heal/damage branch.
            } else {
                // Vanilla ThrownSplashPotion#applyEffects: duration is rounded (not truncated),
                // and effects whose scaled duration would end within 20 ticks are dropped
                // entirely rather than clamped to a minimum of 1 tick.
                // Area effect clouds scale through PotionContents#forEachEffect ->
                // MobEffectInstance#withScaledDuration (MobEffectInstance.java:191), which
                // floors and clamps up to 1 tick, and AreaEffectCloud.java:233 then adds the
                // effect unconditionally. The `endsWithin(20)` drop is ThrownSplashPotion-only
                // (ThrownSplashPotion.java:67).
                let is_cloud = source == PotionApplicationSource::AreaEffectCloud;
                let scaled = duration as f32 * source.duration_scale(scale);
                let dur = if is_cloud {
                    (scaled.floor() as i32).max(1)
                } else {
                    (scaled + 0.5) as i32
                };
                if !is_cloud && dur <= 20 {
                    continue;
                }
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

#[cfg(test)]
mod tests {
    use super::PotionApplicationSource;
    use pumpkin_data::data_component_impl::PotionDurationScaleImpl;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;

    #[test]
    fn tipped_arrow_scale_shortens_duration_without_reducing_instant_potency() {
        let tipped_arrow = ItemStack::new(1, &Item::TIPPED_ARROW);
        let scale = tipped_arrow
            .get_data_component::<PotionDurationScaleImpl>()
            .expect("tipped arrows should define a potion duration scale")
            .scale;

        assert_eq!(PotionApplicationSource::Arrow.duration_scale(scale), 0.125);
        assert_eq!(
            (160.0 * PotionApplicationSource::Arrow.duration_scale(scale)) as i32,
            20
        );
        assert_eq!(PotionApplicationSource::Arrow.instant_scale(scale), 1.0);
    }
}
