use crate::entity::EntityBase;
use crate::world::World;
use crate::world::explosion::Explosion;
use pumpkin_data::fluid::FluidState;
use pumpkin_data::{Block, BlockState};
use pumpkin_util::math::position::BlockPos;

/// A trait to implement a calculator for certain numerical values,
/// like entity damage, when an explosion occurs.
pub trait ExplosionDamageCalculator: Send + Sync {
    /// Calculates the block *blast resistance* of a particular position in a world
    /// using an [`Explosion`].
    fn block_blast_resistance(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: BlockPos,
        block: &Block,
        block_state: &BlockState,
        fluid_state: &FluidState,
    ) -> Option<f32> {
        (!(block_state.is_air() && fluid_state.is_empty))
            .then(|| block.blast_resistance.max(fluid_state.blast_resistance))
    }

    /// Returns whether a block should explode.
    fn block_should_explode(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: BlockPos,
        _block: &Block,
        _block_state: &BlockState,
        _f: f32,
    ) -> bool {
        true
    }

    /// Returns whether an entity should be dealt damage.
    fn entity_takes_damage(&self, _explosion: &Explosion, _entity: &dyn EntityBase) -> bool {
        true
    }

    /// Returns the amount of damage to deal to an entity with the
    /// provided explosion.
    fn compute_entity_damage(&self, explosion: &Explosion, entity: &dyn EntityBase, f: f32) -> f32 {
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
    fn knockback_multiplier(&self, _entity: &dyn EntityBase) -> f32 {
        1.0
    }
}
