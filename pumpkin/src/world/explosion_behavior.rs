use futures::future::BoxFuture;
use pumpkin_data::{Block, fluid::FluidState};
use pumpkin_util::math::position::BlockPos;

use crate::{
    entity::{Entity, EntityBase},
    world::World,
};

/// How an explosion interacts with blocks.
/// Can be derived from `ExplosionInteraction` via `resolve()`.
#[derive(PartialEq, Eq)]
pub enum ExplosionBlockInteraction {
    Keep,
    Destroy,
    DestroyWithDecay,
    TriggerBlock,
}

pub enum ExplosionInteraction {
    None,
    Block,
    Mob,
    TNT,
    Trigger,
}

impl ExplosionInteraction {
    /// Resolve to `ExplosionBlockInteraction` based on gamerules.
    pub fn resolve(self, world: &World) -> ExplosionBlockInteraction {
        let game_rules = world.level_info.load().game_rules.clone();
        match self {
            Self::None => ExplosionBlockInteraction::Keep,
            Self::Block => {
                if game_rules.block_explosion_drop_decay {
                    ExplosionBlockInteraction::DestroyWithDecay
                } else {
                    ExplosionBlockInteraction::Destroy
                }
            }
            Self::Mob => {
                if game_rules.mob_griefing {
                    if game_rules.mob_explosion_drop_decay {
                        ExplosionBlockInteraction::DestroyWithDecay
                    } else {
                        ExplosionBlockInteraction::Destroy
                    }
                } else {
                    ExplosionBlockInteraction::Keep
                }
            }
            Self::TNT => {
                if game_rules.tnt_explosion_drop_decay {
                    ExplosionBlockInteraction::DestroyWithDecay
                } else {
                    ExplosionBlockInteraction::Destroy
                }
            }
            Self::Trigger => ExplosionBlockInteraction::TriggerBlock,
        }
    }
}

/// Strategy trait for customizing explosion behavior.
/// Replaces vanilla's `ExplosionDamageCalculator`.
pub trait ExplosionBehavior: Send + Sync {
    fn get_block_explosion_resistance<'a>(
        &'a self,
        block: &'a Block,
        fluid_state: &'a FluidState,
        _pos: &'a BlockPos,
    ) -> BoxFuture<'a, Option<f32>> {
        Box::pin(async move {
            (!block.is_air() || !fluid_state.is_empty)
                .then(|| fluid_state.blast_resistance.max(block.blast_resistance))
        })
    }

    fn should_affect_block<'a>(&'a self, _block: &'a Block) -> BoxFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn should_damage_entity<'a>(&'a self, _entity: &'a Entity) -> BoxFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn get_knockback_multiplier<'a>(&'a self, _entity: &'a Entity) -> BoxFuture<'a, f64> {
        Box::pin(async move { 1.0 })
    }
}

pub struct DefaultExplosion;

impl ExplosionBehavior for DefaultExplosion {}

pub struct EntityBasedExplosion;

impl ExplosionBehavior for EntityBasedExplosion {}

/// Used by wind charges
pub struct SimpleExplosion {
    pub affect_blocks: bool,
    pub damage_entities: bool,
    pub knockback_multiplier: Option<f64>,
    pub immune_block_ids: Vec<u16>,
}

impl ExplosionBehavior for SimpleExplosion {
    fn get_block_explosion_resistance<'a>(
        &'a self,
        block: &'a Block,
        fluid_state: &'a FluidState,
        _pos: &'a BlockPos,
    ) -> BoxFuture<'a, Option<f32>> {
        let result = if self.immune_block_ids.is_empty() {
            (!block.is_air() || !fluid_state.is_empty)
                .then(|| fluid_state.blast_resistance.max(block.blast_resistance))
        } else {
            self.immune_block_ids
                .contains(&block.id)
                .then_some(3_600_000.0)
        };
        Box::pin(async move { result })
    }

    fn should_affect_block<'a>(&'a self, _block: &'a Block) -> BoxFuture<'a, bool> {
        Box::pin(async move { self.affect_blocks })
    }

    fn should_damage_entity<'a>(&'a self, _entity: &'a Entity) -> BoxFuture<'a, bool> {
        Box::pin(async move { self.damage_entities })
    }

    fn get_knockback_multiplier<'a>(&'a self, entity: &'a Entity) -> BoxFuture<'a, f64> {
        Box::pin(async move {
            let is_flying = if let Some(player) = entity.get_player() {
                player.abilities.lock().await.flying
            } else {
                false
            };
            if is_flying {
                0.0
            } else {
                self.knockback_multiplier.unwrap_or(1.0)
            }
        })
    }
}
