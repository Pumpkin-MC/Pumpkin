use std::sync::atomic::Ordering::Relaxed;

use crate::world::World;

/// `LevelReader#isBrightOutside` (`Level.java`), used throughout `Drowned.java`'s
/// `addBehaviourGoals` (`DrownedGoToWaterGoal`, `DrownedGoToBeachGoal`, `DrownedSwimUpGoal`,
/// `Drowned#okTarget`). Duplicated from `fox_sleep.rs`'s private `is_bright_outside` rather
/// than shared -- there is no common water/light module in this codebase yet and the two
/// goal families are otherwise unrelated.
const BRIGHT_OUTSIDE_THRESHOLD: u8 = 4;

pub fn is_bright_outside(world: &World) -> bool {
    world.dimension.has_skylight && world.sky_darken.load(Relaxed) < BRIGHT_OUTSIDE_THRESHOLD
}
