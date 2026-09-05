use std::sync::Arc;

use pumpkin_data::enchantment::EnchantmentEntityEffect;
use pumpkin_util::math::vector3::Vector3;

use super::{
    AllOf, ApplyEntityImpulse, ApplyExhaustion, ApplyMobEffect, ChangeItemDamage, DamageEntity,
    ExplodeEffect, Ignite, PlaySound, ReplaceBlock, ReplaceDisk, RunFunction, SetBlockProperties,
    SpawnParticlesEffect, SummonEntityEffect,
};
use crate::entity::Entity;
use crate::entity::player::Player;
use crate::world::World;

/// Extension trait for enchantment entity effects.
pub trait EnchantmentEntityEffectExt {
    /// Applies this entity effect at the given position in the world.
    fn apply(
        &self,
        world: &Arc<World>,
        enchantment_level: i32,
        owner: Option<&Arc<Player>>,
        entity: Option<&Entity>,
        position: Vector3<f64>,
    );
}

impl EnchantmentEntityEffectExt for EnchantmentEntityEffect {
    #[allow(clippy::too_many_lines)]
    fn apply(
        &self,
        world: &Arc<World>,
        enchantment_level: i32,
        owner: Option<&Arc<Player>>,
        entity: Option<&Entity>,
        position: Vector3<f64>,
    ) {
        match self {
            Self::Ignite { duration } => {
                EnchantmentEntityEffectExt::apply(
                    &Ignite::new(duration.clone()),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::DamageEntity {
                min_damage,
                max_damage,
                damage_type,
            } => {
                EnchantmentEntityEffectExt::apply(
                    &DamageEntity::new(min_damage.clone(), max_damage.clone(), *damage_type),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::ChangeItemDamage { amount } => {
                EnchantmentEntityEffectExt::apply(
                    &ChangeItemDamage::new(amount.clone()),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::PlaySound { sound } => {
                EnchantmentEntityEffectExt::apply(
                    &PlaySound::new(sound),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::ReplaceBlock {
                offset_x,
                offset_y,
                offset_z,
                trigger_game_event,
            } => {
                let offset = Vector3::new(*offset_x, *offset_y, *offset_z);
                EnchantmentEntityEffectExt::apply(
                    &ReplaceBlock::new(offset, *trigger_game_event),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::SetBlockProperties {
                properties,
                offset_x,
                offset_y,
                offset_z,
                trigger_game_event,
            } => {
                let offset = Vector3::new(*offset_x, *offset_y, *offset_z);
                let props = properties
                    .iter()
                    .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                    .collect();
                EnchantmentEntityEffectExt::apply(
                    &SetBlockProperties::new(props, offset, *trigger_game_event),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::ReplaceDisk {
                radius,
                height,
                offset_x,
                offset_y,
                offset_z,
                predicate,
                block_state,
                trigger_game_event,
            } => {
                let offset = Vector3::new(*offset_x, *offset_y, *offset_z);
                EnchantmentEntityEffectExt::apply(
                    &ReplaceDisk {
                        radius: radius.clone(),
                        height: height.clone(),
                        offset,
                        predicate: predicate.clone(),
                        block_state,
                        trigger_game_event: *trigger_game_event,
                    },
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::SummonEntity {
                entity_types,
                join_team,
            } => {
                EnchantmentEntityEffectExt::apply(
                    &SummonEntityEffect::new(entity_types.to_vec(), *join_team),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::SpawnParticles {
                particle,
                horizontal_position,
                vertical_position,
                horizontal_velocity,
                vertical_velocity,
                speed,
            } => {
                let spawn_particles = SpawnParticlesEffect::new(
                    *particle,
                    *horizontal_position,
                    *vertical_position,
                    horizontal_velocity.clone(),
                    vertical_velocity.clone(),
                    speed.clone(),
                );
                EnchantmentEntityEffectExt::apply(
                    &spawn_particles,
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::RunFunction { function } => {
                EnchantmentEntityEffectExt::apply(
                    &RunFunction::new(*function),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::ApplyExhaustion { amount } => {
                EnchantmentEntityEffectExt::apply(
                    &ApplyExhaustion::new(amount.clone()),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::ApplyImpulse {
                direction,
                coordinate_scale,
                magnitude,
            } => {
                EnchantmentEntityEffectExt::apply(
                    &ApplyEntityImpulse::new(*direction, *coordinate_scale, magnitude.clone()),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::ApplyMobEffect {
                to_apply,
                min_duration,
                max_duration,
                min_amplifier,
                max_amplifier,
            } => {
                EnchantmentEntityEffectExt::apply(
                    &ApplyMobEffect::new(
                        to_apply.to_vec(),
                        min_duration.clone(),
                        max_duration.clone(),
                        min_amplifier.clone(),
                        max_amplifier.clone(),
                    ),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::Explode {
                offset_x,
                offset_y,
                offset_z,
                radius,
                create_fire,
                ..
            } => {
                EnchantmentEntityEffectExt::apply(
                    &ExplodeEffect::new(
                        false,
                        None,
                        None,
                        None,
                        Vector3::new(*offset_x, *offset_y, *offset_z),
                        radius.clone(),
                        *create_fire,
                        crate::world::ExplosionInteraction::None,
                        None,
                        None,
                        None,
                    ),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::AllOf(effects) => {
                EnchantmentEntityEffectExt::apply(
                    &AllOf::new(effects),
                    world,
                    enchantment_level,
                    owner,
                    entity,
                    position,
                );
            }
            Self::Other => {}
        }
    }
}
