use std::sync::Arc;

use pumpkin_data::block_properties::{
    BlockProperties, CaveVinesLikeProperties, CaveVinesPlantLikeProperties,
};
use pumpkin_data::{Block, BlockId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, RandomTickArgs};
use crate::world::World;

/// `CaveVinesBlock`'s `growPerTickProbability`, passed to the `GrowingPlantHeadBlock`
/// constructor: `super(properties, Direction.DOWN, SHAPE, false, 0.1)`.
const GROW_PER_TICK_PROBABILITY: f64 = 0.1;

/// `CaveVinesBlock.CHANCE_OF_BERRIES_ON_GROWTH`, rolled in `getGrowIntoState` for the
/// newly grown segment: `random.nextFloat() < 0.11F`.
const CHANCE_OF_BERRIES_ON_GROWTH: f32 = 0.11;

/// Vanilla's `GrowingPlantHeadBlock.MAX_AGE`.
const MAX_AGE: u8 = 25;

/// Growth/spread `random_tick` for cave vines (glow berries).
///
/// Only `CAVE_VINES` (the head block) grows on a random tick. `CAVE_VINES_PLANT` (the
/// body block, `CaveVinesPlantBlock`) never gets `.randomTicks()` in vanilla's
/// `Blocks.java` registration and `GrowingPlantBodyBlock` never overrides `randomTick`,
/// so the body segment is inert here — matching vanilla.
pub struct CaveVinesBlock;

impl BlockMetadata for CaveVinesBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::CAVE_VINES, BlockId::CAVE_VINES_PLANT].into()
    }
}

impl BlockBehaviour for CaveVinesBlock {
    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if args.block != &Block::CAVE_VINES {
                return;
            }
            grow(args.world.clone(), args.position).await;
        })
    }
}

/// Mirrors `GrowingPlantHeadBlock.randomTick` + `CaveVinesBlock.getGrowIntoState`.
///
/// Vanilla:
/// ```java
/// if (state.getValue(AGE) < 25 && random.nextDouble() < this.growPerTickProbability) {
///    BlockPos growthPos = pos.relative(this.growthDirection); // DOWN
///    if (this.canGrowInto(level.getBlockState(growthPos))) {  // isAir()
///       level.setBlockAndUpdate(growthPos, this.getGrowIntoState(state, level.getRandom()));
///    }
/// }
/// ```
/// `setBlockAndUpdate` at `growthPos` cascades a neighbor update back onto the old head
/// at `pos`, which `GrowingPlantHeadBlock.updateShape` turns into the body block via
/// `updateBodyAfterConvertedFromHead`, which `CaveVinesBlock` overrides to carry the old
/// head's berries onto the new body state. Both writes are applied explicitly below
/// since pumpkin has no registered neighbor-update chain for cave vines to rely on.
async fn grow(world: Arc<World>, pos: &BlockPos) {
    let (block, state_id) = world.get_block_and_state_id(pos);
    if block != &Block::CAVE_VINES {
        return;
    }
    let props = CaveVinesLikeProperties::from_state_id(state_id, block);
    if !should_attempt_growth(props.age, rand::rng().random::<f64>()) {
        return;
    }

    let grow_pos = pos.down();
    if !world.get_block_state(&grow_pos).is_air() {
        return;
    }

    let new_age = next_age(props.age);
    let new_berries = rand::rng().random::<f32>() < CHANCE_OF_BERRIES_ON_GROWTH;
    let new_head_props = CaveVinesLikeProperties {
        age: new_age,
        berries: new_berries,
    };
    world
        .set_block_state(
            &grow_pos,
            new_head_props.to_state_id(&Block::CAVE_VINES),
            BlockFlags::NOTIFY_NEIGHBORS,
        )
        .await;

    // Old head converts into the body block, carrying over its own (pre-growth) berries.
    let new_body_props = CaveVinesPlantLikeProperties {
        berries: props.berries,
    };
    world
        .set_block_state(
            pos,
            new_body_props.to_state_id(&Block::CAVE_VINES_PLANT),
            BlockFlags::NOTIFY_NEIGHBORS,
        )
        .await;
}

/// Pure growth gate mirroring `state.getValue(AGE) < 25 && random.nextDouble() < growPerTickProbability`.
const fn should_attempt_growth(age: u8, roll: f64) -> bool {
    age < MAX_AGE && roll < GROW_PER_TICK_PROBABILITY
}

/// Mirrors `BlockState#cycle(AGE)` used by `getGrowIntoState`. Only ever called when
/// `age < MAX_AGE` (see `should_attempt_growth`), so the wrap-to-0 branch is unreachable
/// in practice; it is kept to faithfully mirror `cycle`'s general wraparound semantics.
const fn next_age(age: u8) -> u8 {
    if age >= MAX_AGE { 0 } else { age + 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_growing_at_max_age() {
        assert!(!should_attempt_growth(MAX_AGE, 0.0));
    }

    #[test]
    fn growth_gate_respects_probability() {
        assert!(should_attempt_growth(0, 0.0));
        assert!(should_attempt_growth(
            0,
            GROW_PER_TICK_PROBABILITY - f64::EPSILON
        ));
        assert!(!should_attempt_growth(0, GROW_PER_TICK_PROBABILITY));
        assert!(!should_attempt_growth(0, 1.0));
    }

    #[test]
    fn next_age_increments_then_wraps() {
        assert_eq!(next_age(0), 1);
        assert_eq!(next_age(24), 25);
        assert_eq!(next_age(25), 0);
    }
}
