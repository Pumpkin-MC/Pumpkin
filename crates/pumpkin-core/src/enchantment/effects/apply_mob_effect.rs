use std::sync::Arc;

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::enchantment::LevelBasedValue;
use pumpkin_data::potion::Effect;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro};

use super::EnchantmentEntityEffectExt;
use crate::entity::Entity;
use crate::entity::player::Player;
use crate::world::World;

/// Enchantment entity effect that applies a status effect.
#[derive(Clone, Debug, PartialEq)]
pub struct ApplyMobEffect {
    pub to_apply: Vec<&'static StatusEffect>,
    pub min_duration: LevelBasedValue,
    pub max_duration: LevelBasedValue,
    pub min_amplifier: LevelBasedValue,
    pub max_amplifier: LevelBasedValue,
}

impl ApplyMobEffect {
    #[must_use]
    pub const fn new(
        to_apply: Vec<&'static StatusEffect>,
        min_duration: LevelBasedValue,
        max_duration: LevelBasedValue,
        min_amplifier: LevelBasedValue,
        max_amplifier: LevelBasedValue,
    ) -> Self {
        Self {
            to_apply,
            min_duration,
            max_duration,
            min_amplifier,
            max_amplifier,
        }
    }

    pub fn apply(
        &self,
        world: &Arc<World>,
        enchantment_level: i32,
        _owner: Option<&Arc<Player>>,
        entity: Option<&Entity>,
    ) {
        if self.to_apply.is_empty() {
            return;
        }

        let Some(entity) = entity else {
            return;
        };

        let mut rng = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        let selected_idx = (rng.next_f32() * self.to_apply.len() as f32) as usize;
        let selected = self.to_apply[selected_idx.min(self.to_apply.len() - 1)];

        let min_dur = self.min_duration.calculate(enchantment_level);
        let max_dur = self.max_duration.calculate(enchantment_level);
        let dur_secs = if (max_dur - min_dur).abs() < f32::EPSILON || min_dur >= max_dur {
            min_dur
        } else {
            rng.next_f32().mul_add(max_dur - min_dur, min_dur)
        };
        let ticks = (dur_secs * 20.0).round() as i32;

        let min_amp = self.min_amplifier.calculate(enchantment_level);
        let max_amp = self.max_amplifier.calculate(enchantment_level);
        let amp_f = if (max_amp - min_amp).abs() < f32::EPSILON || min_amp >= max_amp {
            min_amp
        } else {
            rng.next_f32().mul_add(max_amp - min_amp, min_amp)
        };
        let amplifier = (amp_f.round() as i32).max(0) as u8;

        let effect = Effect {
            effect_type: selected,
            duration: ticks,
            amplifier,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        };

        let target_player = world.get_player_by_id(entity.entity_id);
        if let Some(target_player) = target_player {
            target_player.add_effect(effect);
        } else if let Some(living) = world
            .get_entity_by_id(entity.entity_id)
            .as_deref()
            .and_then(crate::entity::EntityBase::get_living_entity)
        {
            living.add_effect(effect);
        }
    }
}

impl EnchantmentEntityEffectExt for ApplyMobEffect {
    fn apply(
        &self,
        world: &Arc<World>,
        enchantment_level: i32,
        owner: Option<&Arc<Player>>,
        entity: Option<&Entity>,
        _position: Vector3<f64>,
    ) {
        self.apply(world, enchantment_level, owner, entity);
    }
}
