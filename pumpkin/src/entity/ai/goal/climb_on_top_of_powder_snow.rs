use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::tag::{self, Taggable};

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;

/// Makes a mob standing in powder snow jump onto solid-looking powder snow directly above it.
///
/// Mirrors vanilla's ability for small mobs to walk across the top layer of a powder snow block
/// instead of sinking in. Vanilla source:
/// `net/minecraft/world/entity/ai/goal/ClimbOnTopOfPowderSnowGoal.java`. Registered (per-mob, not
/// via the tag) on Silverfish, Endermite, Rabbit and Fox.
pub struct ClimbOnTopOfPowderSnowGoal {
    goal_control: Controls,
}

impl ClimbOnTopOfPowderSnowGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::JUMP,
        })
    }

    fn wants_to_climb(mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let in_powder_snow =
            entity.was_in_powder_snow.load(Relaxed) || entity.is_in_powder_snow.load(Relaxed);
        if !in_powder_snow
            || !entity
                .entity_type
                .has_tag(&tag::EntityType::MINECRAFT_POWDER_SNOW_WALKABLE_MOBS)
        {
            return false;
        }

        let world = entity.world.load();
        let above = entity.block_pos.load().up();
        let (block, state) = world.get_block_and_state(&above);
        // Vanilla checks `POWDER_SNOW` or an empty collision shape above; Pumpkin doesn't
        // expose per-block collision shapes generically here, so approximate "empty shape"
        // with "not solid", which covers the common air/foliage/snow-layer cases.
        block == &pumpkin_data::Block::POWDER_SNOW || !state.is_solid()
    }
}

impl Default for ClimbOnTopOfPowderSnowGoal {
    fn default() -> Self {
        *Self::new()
    }
}

impl Goal for ClimbOnTopOfPowderSnowGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { Self::wants_to_climb(mob) })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        // Vanilla has no `canContinueToUse` override, so it defaults to re-running `canUse`.
        Box::pin(async move { Self::wants_to_climb(mob) })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            // Vanilla: `this.mob.getJumpControl().jump()`, which just arms the jump flag for the
            // next physics tick.
            mob.get_mob_entity()
                .living_entity
                .jumping
                .store(true, std::sync::atomic::Ordering::SeqCst);
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
