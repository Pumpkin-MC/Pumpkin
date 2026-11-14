/* This file is generated. Do not edit manually. */
use crate::effect::StatusEffect;
pub struct SuspiciousEffect {
    pub effect: &'static StatusEffect,
    pub duration: i32,
}
#[must_use]
pub const fn get_suspicious_effects(item_id: u16) -> Option<&'static [SuspiciousEffect]> {
    match item_id {
        304 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::FIRE_RESISTANCE,
            duration: 60i32,
        }]),
        305 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::BLINDNESS,
            duration: 220i32,
        }]),
        303 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::SATURATION,
            duration: 7i32,
        }]),
        301 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::NAUSEA,
            duration: 140i32,
        }]),
        311 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::JUMP_BOOST,
            duration: 100i32,
        }]),
        298 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::SATURATION,
            duration: 7i32,
        }]),
        299 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::SATURATION,
            duration: 7i32,
        }]),
        312 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::POISON,
            duration: 220i32,
        }]),
        300 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::BLINDNESS,
            duration: 220i32,
        }]),
        307 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::WEAKNESS,
            duration: 140i32,
        }]),
        310 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::REGENERATION,
            duration: 140i32,
        }]),
        309 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::WEAKNESS,
            duration: 140i32,
        }]),
        302 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::NIGHT_VISION,
            duration: 100i32,
        }]),
        306 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::WEAKNESS,
            duration: 140i32,
        }]),
        314 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::NIGHT_VISION,
            duration: 100i32,
        }]),
        308 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::WEAKNESS,
            duration: 140i32,
        }]),
        313 => Some(&[SuspiciousEffect {
            effect: &StatusEffect::WITHER,
            duration: 140i32,
        }]),
        _ => None,
    }
}
