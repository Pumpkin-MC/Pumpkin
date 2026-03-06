use crate::entity::EntityBase;
use crate::world::World;
use crate::world::explosion::Explosion;
use pumpkin_data::fluid::FluidState;
use pumpkin_data::{Block, BlockState};
use pumpkin_util::math::position::BlockPos;
use std::collections::HashSet;
use std::sync::Arc;

/// A trait to implement a calculator for certain numerical values,
/// like entity damage, when an explosion occurs.
#[derive(Default, Clone)]
pub enum ExplosionDamageCalculator {
    #[default]
    Default,

    /// A simple [`ExplosionDamageCalculator`] implementation.
    Simple {
        /// Whether to explode blocks.
        explodes_blocks: bool,
        /// Whether to damage entities.
        damages_entities: bool,
        /// The knockback multiplier of this calculator.
        knockback_multiplier: Option<f32>,
        /// Blocks which are not affected by any explosion for
        /// which this calculator calculates.
        immune_blocks: Option<HashSet<&'static Block>>,
    },
    /// An [`ExplosionDamageCalculator`] based on an entity.
    EntityBased { source_entity: Arc<dyn EntityBase> },
}

fn default_block_blast_resistance(
    block: &Block,
    block_state: &BlockState,
    fluid_state: &FluidState,
) -> Option<f32> {
    (!(block_state.is_air() && fluid_state.is_empty))
        .then(|| block.blast_resistance.max(fluid_state.blast_resistance))
}

impl ExplosionDamageCalculator {
    /// Calculates the block *blast resistance* of a particular position in a world
    /// using an [`Explosion`].
    pub fn block_blast_resistance(
        &self,
        explosion: &Explosion,
        world: &World,
        _pos: BlockPos,
        block: &Block,
        block_state: &BlockState,
        fluid_state: &FluidState,
    ) -> Option<f32> {
        match self {
            Self::Simple {
                immune_blocks: Some(immune_blocks),
                ..
            } => immune_blocks.contains(&block).then_some(3600000.0),
            Self::EntityBased { source_entity } => {
                default_block_blast_resistance(block, block_state, fluid_state).map(|r| {
                    source_entity.block_blast_resistance_with(
                        explosion,
                        world,
                        block,
                        block_state,
                        r,
                    )
                })
            }
            _ => default_block_blast_resistance(block, block_state, fluid_state),
        }
    }

    /// Returns whether a block should explode.
    pub fn block_should_explode(
        &self,
        explosion: &Explosion,
        world: &World,
        _pos: BlockPos,
        block: &Block,
        block_state: &BlockState,
        _f: f32,
    ) -> bool {
        match self {
            Self::Simple {
                explodes_blocks, ..
            } => *explodes_blocks,
            Self::EntityBased { source_entity } => {
                source_entity.block_should_explode_with(explosion, world, block, block_state)
            }
            Self::Default => true,
        }
    }

    /// Returns whether an entity should be dealt damage.
    pub fn entity_takes_damage(&self, _explosion: &Explosion, _entity: &dyn EntityBase) -> bool {
        match self {
            Self::Simple {
                damages_entities, ..
            } => *damages_entities,
            _ => true,
        }
    }

    /// Returns the amount of damage to deal to an entity with the
    /// provided explosion.
    pub fn compute_entity_damage(
        &self,
        explosion: &Explosion,
        entity: &dyn EntityBase,
        f: f32,
    ) -> f32 {
        let g = (explosion.power() * 2.0) as f64;
        let center = explosion.pos();

        // Normalized distance from the entity.
        let d = entity
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&center)
            .sqrt()
            / g;

        // 'Exposure' factor.
        let e = (1.0 - d) * (f as f64);

        (e.midpoint(e * e) * 7.0 * g + 1.0) as f32
    }

    /// Returns the knockback multiplier to apply to an entity.
    pub async fn knockback_multiplier(&self, entity: &dyn EntityBase) -> f32 {
        match self {
            Self::Simple {
                knockback_multiplier,
                ..
            } => {
                if let Some(player) = entity.get_player()
                    && player.is_flying().await
                {
                    0.0
                } else {
                    knockback_multiplier.unwrap_or(1.0)
                }
            }
            _ => 1.0,
        }
    }
}
