//! Port of `Phantom.PhantomAttackStrategyGoal` (`Phantom.java:251-297`).
//!
//! Controls `CIRCLE` <-> `SWOOP` phase switching and picks the anchor point above the
//! target that `PhantomCircleAroundAnchorGoal`/`PhantomSweepAttackGoal` fly around/at.
//! Extends `Goal` directly in vanilla (no `setFlags` call), so this has no controls -
//! giving it `Controls::MOVE` would lock the priority-3 circle goal out entirely.

use std::sync::Weak;

use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::chunk::ChunkHeightmapType;
use rand::RngExt;

use crate::entity::ai::goal::{Goal, GoalFuture};
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::mob::phantom::{AttackPhase, PhantomEntity};

pub struct PhantomAttackStrategyGoal {
    phantom: Weak<PhantomEntity>,
    next_sweep_tick: i32,
}

impl PhantomAttackStrategyGoal {
    #[must_use]
    pub const fn new(phantom: Weak<PhantomEntity>) -> Self {
        Self {
            phantom,
            next_sweep_tick: 0,
        }
    }

    async fn set_anchor_above_target(phantom: &PhantomEntity, mob: &dyn Mob) {
        if phantom.anchor_point().is_none() {
            return;
        }
        let target = phantom.mob_entity.target.lock().await.clone();
        let Some(target) = target else {
            return;
        };
        let target_pos = target.get_entity().block_pos.load();
        let random_extra = mob.get_random().random_range(0..20);
        let sea_level = phantom
            .mob_entity
            .living_entity
            .entity
            .world
            .load()
            .sea_level;
        let anchor = anchor_above_target(target_pos, random_extra, sea_level);
        phantom.set_anchor_point(Some(anchor));
    }
}

/// Vanilla `setAnchorAboveTarget`: `targetPos.above(20 + random.nextInt(20))`, clamped to
/// `seaLevel + 1` if it would otherwise land at or below sea level.
#[must_use]
pub fn anchor_above_target(target_pos: BlockPos, random_extra: i32, sea_level: i32) -> BlockPos {
    let anchor = target_pos.up_height(20 + random_extra);
    if anchor.0.y < sea_level {
        BlockPos::new(anchor.0.x, sea_level + 1, anchor.0.z)
    } else {
        anchor
    }
}

impl Goal for PhantomAttackStrategyGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(phantom) = self.phantom.upgrade() else {
                return false;
            };
            let target = phantom.mob_entity.target.lock().await.clone();
            let Some(target) = target else {
                return false;
            };
            let Some(target_living) = target.get_living_entity() else {
                return false;
            };
            let world = phantom.mob_entity.living_entity.entity.world.load_full();
            TargetPredicate::create_attackable()
                .test(
                    &world,
                    Some(&phantom.mob_entity.living_entity),
                    target_living,
                )
                .await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(phantom) = self.phantom.upgrade() else {
                return;
            };
            self.next_sweep_tick = self.get_tick_count(10);
            phantom.set_attack_phase(AttackPhase::Circle);
            Self::set_anchor_above_target(&phantom, mob).await;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(phantom) = self.phantom.upgrade() else {
                return;
            };
            let Some(anchor) = phantom.anchor_point() else {
                return;
            };
            let world = phantom.mob_entity.living_entity.entity.world.load_full();
            let ground_y = world.get_heightmap_height(
                ChunkHeightmapType::MotionBlocking,
                anchor.0.x,
                anchor.0.z,
            );
            let random_extra = mob.get_random().random_range(0..20);
            let new_anchor =
                BlockPos::new(anchor.0.x, ground_y, anchor.0.z).up_height(10 + random_extra);
            phantom.set_anchor_point(Some(new_anchor));
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(phantom) = self.phantom.upgrade() else {
                return;
            };
            if phantom.attack_phase() != AttackPhase::Circle {
                return;
            }
            self.next_sweep_tick -= 1;
            if self.next_sweep_tick <= 0 {
                phantom.set_attack_phase(AttackPhase::Swoop);
                Self::set_anchor_above_target(&phantom, mob).await;
                let extra_seconds = mob.get_random().random_range(0..4);
                self.next_sweep_tick = self.get_tick_count((8 + extra_seconds) * 20);

                let entity = &phantom.mob_entity.living_entity.entity;
                let pos = entity.pos.load();
                let pitch = 0.95 + mob.get_random().random::<f32>() * 0.1;
                entity.world.load().play_sound_fine(
                    Sound::EntityPhantomSwoop,
                    SoundCategory::Hostile,
                    &pos,
                    10.0,
                    pitch,
                );
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_stays_above_sea_level() {
        let target = BlockPos::new(0, -60, 0);
        let anchor = anchor_above_target(target, 0, 63);
        assert_eq!(anchor.0.y, 64);
    }

    #[test]
    fn anchor_above_sea_level_is_unclamped() {
        let target = BlockPos::new(0, 70, 0);
        let anchor = anchor_above_target(target, 5, 63);
        assert_eq!(anchor.0.y, 70 + 20 + 5);
    }

    #[test]
    fn anchor_height_offset_is_twenty_plus_random_extra() {
        let target = BlockPos::new(3, 100, -3);
        let anchor = anchor_above_target(target, 19, 0);
        assert_eq!(anchor.0.x, 3);
        assert_eq!(anchor.0.z, -3);
        assert_eq!(anchor.0.y, 100 + 20 + 19);
    }
}
