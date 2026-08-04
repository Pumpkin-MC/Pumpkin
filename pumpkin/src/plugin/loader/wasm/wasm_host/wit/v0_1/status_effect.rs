use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState, wit::v0_1::pumpkin::plugin::status_effect,
};

impl status_effect::Host for PluginHostState {}

#[must_use]
#[allow(clippy::too_many_lines)]
pub const fn from_wasm_status_effect_type(
    t: status_effect::StatusEffectType,
) -> pumpkin_data::status_effect::EffectType {
    match t {
        status_effect::StatusEffectType::Speed => pumpkin_data::status_effect::EffectType::Speed,
        status_effect::StatusEffectType::Slowness => {
            pumpkin_data::status_effect::EffectType::Slowness
        }
        status_effect::StatusEffectType::Haste => pumpkin_data::status_effect::EffectType::Haste,
        status_effect::StatusEffectType::MiningFatigue => {
            pumpkin_data::status_effect::EffectType::MiningFatigue
        }
        status_effect::StatusEffectType::Strength => {
            pumpkin_data::status_effect::EffectType::Strength
        }
        status_effect::StatusEffectType::InstantHealth => {
            pumpkin_data::status_effect::EffectType::InstantHealth
        }
        status_effect::StatusEffectType::InstantDamage => {
            pumpkin_data::status_effect::EffectType::InstantDamage
        }
        status_effect::StatusEffectType::JumpBoost => {
            pumpkin_data::status_effect::EffectType::JumpBoost
        }
        status_effect::StatusEffectType::Nausea => pumpkin_data::status_effect::EffectType::Nausea,
        status_effect::StatusEffectType::Regeneration => {
            pumpkin_data::status_effect::EffectType::Regeneration
        }
        status_effect::StatusEffectType::Resistance => {
            pumpkin_data::status_effect::EffectType::Resistance
        }
        status_effect::StatusEffectType::FireResistance => {
            pumpkin_data::status_effect::EffectType::FireResistance
        }
        status_effect::StatusEffectType::WaterBreathing => {
            pumpkin_data::status_effect::EffectType::WaterBreathing
        }
        status_effect::StatusEffectType::Invisibility => {
            pumpkin_data::status_effect::EffectType::Invisibility
        }
        status_effect::StatusEffectType::Blindness => {
            pumpkin_data::status_effect::EffectType::Blindness
        }
        status_effect::StatusEffectType::NightVision => {
            pumpkin_data::status_effect::EffectType::NightVision
        }
        status_effect::StatusEffectType::Hunger => pumpkin_data::status_effect::EffectType::Hunger,
        status_effect::StatusEffectType::Weakness => {
            pumpkin_data::status_effect::EffectType::Weakness
        }
        status_effect::StatusEffectType::Poison => pumpkin_data::status_effect::EffectType::Poison,
        status_effect::StatusEffectType::Wither => pumpkin_data::status_effect::EffectType::Wither,
        status_effect::StatusEffectType::HealthBoost => {
            pumpkin_data::status_effect::EffectType::HealthBoost
        }
        status_effect::StatusEffectType::Absorption => {
            pumpkin_data::status_effect::EffectType::Absorption
        }
        status_effect::StatusEffectType::Saturation => {
            pumpkin_data::status_effect::EffectType::Saturation
        }
        status_effect::StatusEffectType::Glowing => {
            pumpkin_data::status_effect::EffectType::Glowing
        }
        status_effect::StatusEffectType::Levitation => {
            pumpkin_data::status_effect::EffectType::Levitation
        }
        status_effect::StatusEffectType::Luck => pumpkin_data::status_effect::EffectType::Luck,
        status_effect::StatusEffectType::Unluck => pumpkin_data::status_effect::EffectType::Unluck,
        status_effect::StatusEffectType::SlowFalling => {
            pumpkin_data::status_effect::EffectType::SlowFalling
        }
        status_effect::StatusEffectType::ConduitPower => {
            pumpkin_data::status_effect::EffectType::ConduitPower
        }
        status_effect::StatusEffectType::DolphinsGrace => {
            pumpkin_data::status_effect::EffectType::DolphinsGrace
        }
        status_effect::StatusEffectType::BadOmen => {
            pumpkin_data::status_effect::EffectType::BadOmen
        }
        status_effect::StatusEffectType::HeroOfTheVillage => {
            pumpkin_data::status_effect::EffectType::HeroOfTheVillage
        }
        status_effect::StatusEffectType::Darkness => {
            pumpkin_data::status_effect::EffectType::Darkness
        }
        status_effect::StatusEffectType::TrialOmen => {
            pumpkin_data::status_effect::EffectType::TrialOmen
        }
        status_effect::StatusEffectType::RaidOmen => {
            pumpkin_data::status_effect::EffectType::RaidOmen
        }
        status_effect::StatusEffectType::WindCharged => {
            pumpkin_data::status_effect::EffectType::WindCharged
        }
        status_effect::StatusEffectType::Weaving => {
            pumpkin_data::status_effect::EffectType::Weaving
        }
        status_effect::StatusEffectType::Oozing => pumpkin_data::status_effect::EffectType::Oozing,
        status_effect::StatusEffectType::Infested => {
            pumpkin_data::status_effect::EffectType::Infested
        }
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub const fn to_wasm_status_effect_type(
    t: pumpkin_data::status_effect::EffectType,
) -> status_effect::StatusEffectType {
    match t {
        pumpkin_data::status_effect::EffectType::Speed => status_effect::StatusEffectType::Speed,
        pumpkin_data::status_effect::EffectType::Slowness => {
            status_effect::StatusEffectType::Slowness
        }
        pumpkin_data::status_effect::EffectType::Haste => status_effect::StatusEffectType::Haste,
        pumpkin_data::status_effect::EffectType::MiningFatigue => {
            status_effect::StatusEffectType::MiningFatigue
        }
        pumpkin_data::status_effect::EffectType::Strength => {
            status_effect::StatusEffectType::Strength
        }
        pumpkin_data::status_effect::EffectType::InstantHealth => {
            status_effect::StatusEffectType::InstantHealth
        }
        pumpkin_data::status_effect::EffectType::InstantDamage => {
            status_effect::StatusEffectType::InstantDamage
        }
        pumpkin_data::status_effect::EffectType::JumpBoost => {
            status_effect::StatusEffectType::JumpBoost
        }
        pumpkin_data::status_effect::EffectType::Nausea => status_effect::StatusEffectType::Nausea,
        pumpkin_data::status_effect::EffectType::Regeneration => {
            status_effect::StatusEffectType::Regeneration
        }
        pumpkin_data::status_effect::EffectType::Resistance => {
            status_effect::StatusEffectType::Resistance
        }
        pumpkin_data::status_effect::EffectType::FireResistance => {
            status_effect::StatusEffectType::FireResistance
        }
        pumpkin_data::status_effect::EffectType::WaterBreathing => {
            status_effect::StatusEffectType::WaterBreathing
        }
        pumpkin_data::status_effect::EffectType::Invisibility => {
            status_effect::StatusEffectType::Invisibility
        }
        pumpkin_data::status_effect::EffectType::Blindness => {
            status_effect::StatusEffectType::Blindness
        }
        pumpkin_data::status_effect::EffectType::NightVision => {
            status_effect::StatusEffectType::NightVision
        }
        pumpkin_data::status_effect::EffectType::Hunger => status_effect::StatusEffectType::Hunger,
        pumpkin_data::status_effect::EffectType::Weakness => {
            status_effect::StatusEffectType::Weakness
        }
        pumpkin_data::status_effect::EffectType::Poison => status_effect::StatusEffectType::Poison,
        pumpkin_data::status_effect::EffectType::Wither => status_effect::StatusEffectType::Wither,
        pumpkin_data::status_effect::EffectType::HealthBoost => {
            status_effect::StatusEffectType::HealthBoost
        }
        pumpkin_data::status_effect::EffectType::Absorption => {
            status_effect::StatusEffectType::Absorption
        }
        pumpkin_data::status_effect::EffectType::Saturation => {
            status_effect::StatusEffectType::Saturation
        }
        pumpkin_data::status_effect::EffectType::Glowing => {
            status_effect::StatusEffectType::Glowing
        }
        pumpkin_data::status_effect::EffectType::Levitation => {
            status_effect::StatusEffectType::Levitation
        }
        pumpkin_data::status_effect::EffectType::Luck => status_effect::StatusEffectType::Luck,
        pumpkin_data::status_effect::EffectType::Unluck => status_effect::StatusEffectType::Unluck,
        pumpkin_data::status_effect::EffectType::SlowFalling => {
            status_effect::StatusEffectType::SlowFalling
        }
        pumpkin_data::status_effect::EffectType::ConduitPower => {
            status_effect::StatusEffectType::ConduitPower
        }
        pumpkin_data::status_effect::EffectType::DolphinsGrace => {
            status_effect::StatusEffectType::DolphinsGrace
        }
        pumpkin_data::status_effect::EffectType::BadOmen => {
            status_effect::StatusEffectType::BadOmen
        }
        pumpkin_data::status_effect::EffectType::HeroOfTheVillage => {
            status_effect::StatusEffectType::HeroOfTheVillage
        }
        pumpkin_data::status_effect::EffectType::Darkness => {
            status_effect::StatusEffectType::Darkness
        }
        pumpkin_data::status_effect::EffectType::TrialOmen => {
            status_effect::StatusEffectType::TrialOmen
        }
        pumpkin_data::status_effect::EffectType::RaidOmen => {
            status_effect::StatusEffectType::RaidOmen
        }
        pumpkin_data::status_effect::EffectType::WindCharged => {
            status_effect::StatusEffectType::WindCharged
        }
        pumpkin_data::status_effect::EffectType::Weaving => {
            status_effect::StatusEffectType::Weaving
        }
        pumpkin_data::status_effect::EffectType::Oozing => status_effect::StatusEffectType::Oozing,
        pumpkin_data::status_effect::EffectType::Infested => {
            status_effect::StatusEffectType::Infested
        }
    }
}

#[must_use]
pub fn to_wasm_status_effect_instance(
    effect: &pumpkin_data::potion::Effect,
) -> Option<status_effect::StatusEffectInstance> {
    let name = effect
        .effect_type
        .minecraft_name
        .strip_prefix("minecraft:")
        .unwrap_or(effect.effect_type.minecraft_name);
    let effect_type_enum = pumpkin_data::status_effect::EffectType::from_name(name)?;
    let wasm_type = to_wasm_status_effect_type(effect_type_enum);
    Some(status_effect::StatusEffectInstance {
        effect_type: wasm_type,
        duration: u32::try_from(effect.duration).unwrap_or(0),
        amplifier: effect.amplifier,
        ambient: effect.ambient,
        show_particles: effect.show_particles,
        show_icon: effect.show_icon,
    })
}
