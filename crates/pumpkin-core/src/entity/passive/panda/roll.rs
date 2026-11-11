//! Vanilla `Panda` rolling: `Panda.roll`/`handleRoll`, which drive the roll while the flag is set,
//! and the `PandaRollGoal` that sets it.

use std::sync::atomic::Ordering::Relaxed;

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{cos, sin};
use rand::RngExt;

use super::{PandaEntity, as_panda};
use crate::entity::EntityBase;
use crate::entity::ageable::AgeableMob;
use crate::entity::ai::goal::{Controls, Goal, GoalReach, to_goal_ticks};
use crate::entity::mob::Mob;

/// Vanilla `Panda.handleRoll` ends the roll once the roll counter has passed this value.
const TOTAL_ROLL_STEPS: i32 = 32;
/// Roll counter values at which the panda stops and hops instead of carrying on.
const ROLL_HOP_STEPS: [i32; 3] = [7, 15, 23];
/// Upward speed of the first roll step and of the hops.
const ROLL_JUMP_SPEED: f64 = 0.27;
/// Horizontal speed the first roll step adds along the facing direction.
const ADULT_ROLL_SPEED: f32 = 0.2;
const BABY_ROLL_SPEED: f32 = 0.1;
/// `PandaRollGoal.canUse` rolls a playful panda on one in this many `nextInt` rolls.
const PLAYFUL_ROLL_BOUND: i32 = 60;
/// `PandaRollGoal.canUse` rolls any panda on one in this many `nextInt` rolls.
const ROLL_BOUND: i32 = 500;
/// A look direction component has to pass this to count as a step towards that axis.
const STEP_THRESHOLD: f32 = 0.5;
/// Vanilla converts degrees with this `float` constant.
const DEGREES_TO_RADIANS: f32 = (std::f64::consts::PI / 180.0) as f32;

/// Vanilla `Mth.sign` of a look direction component, zero while it does not pass the threshold.
fn step_towards(component: f32) -> i32 {
    if component.abs() <= STEP_THRESHOLD {
        0
    } else if component > 0.0 {
        1
    } else {
        -1
    }
}

fn facing_step(yaw: f32) -> (i32, i32) {
    let angle = yaw * DEGREES_TO_RADIANS;
    (step_towards(-sin(angle)), step_towards(cos(angle)))
}

/// Vanilla `PandaRollGoal.canUse` once the panda faces a block that is not air. Both bounds go
/// through `reducedTickDelay` because the selector asks every second tick.
fn rolls_roll(playful: bool, mut next_int: impl FnMut(i32) -> i32) -> bool {
    (playful && next_int(to_goal_ticks(PLAYFUL_ROLL_BOUND)) == 1)
        || next_int(to_goal_ticks(ROLL_BOUND)) == 1
}

/// The velocity of one `handleRoll` step for a roll counter that already counts this tick, or `None`
/// once the roll is over.
fn roll_velocity(
    counter: i32,
    baby: bool,
    yaw: f32,
    on_ground: bool,
    movement: Vector3<f64>,
    roll_delta: &mut Vector3<f64>,
) -> Option<Vector3<f64>> {
    if counter > TOTAL_ROLL_STEPS {
        return None;
    }
    Some(if counter == 1 {
        let angle = yaw * DEGREES_TO_RADIANS;
        let multiplier = if baby {
            BABY_ROLL_SPEED
        } else {
            ADULT_ROLL_SPEED
        };
        *roll_delta = Vector3::new(
            movement.x + f64::from(-sin(angle) * multiplier),
            0.0,
            movement.z + f64::from(cos(angle) * multiplier),
        );
        *roll_delta + Vector3::new(0.0, ROLL_JUMP_SPEED, 0.0)
    } else if ROLL_HOP_STEPS.contains(&counter) {
        Vector3::new(
            0.0,
            if on_ground {
                ROLL_JUMP_SPEED
            } else {
                movement.y
            },
            0.0,
        )
    } else {
        Vector3::new(roll_delta.x, movement.y, roll_delta.z)
    })
}

impl PandaEntity {
    /// Vanilla `Panda.roll`: starts or ends the roll flag.
    pub fn roll(&self, value: bool) {
        self.set_rolling(value);
    }

    pub(super) fn tick_roll(&self) {
        if self.is_rolling() {
            self.handle_roll();
        } else {
            self.roll_counter.store(0, Relaxed);
        }
    }

    /// Vanilla `Panda.handleRoll`: steps 7, 15 and 23 stop the push and hop.
    fn handle_roll(&self) {
        let counter = self.roll_counter.fetch_add(1, Relaxed) + 1;
        let entity = self.get_entity();
        let movement = entity.velocity.load();
        let mut roll_delta = self.roll_delta.load();
        let velocity = roll_velocity(
            counter,
            self.is_baby(),
            entity.yaw.load(),
            entity.on_ground.load(Relaxed),
            movement,
            &mut roll_delta,
        );
        self.roll_delta.store(roll_delta);
        let Some(velocity) = velocity else {
            self.roll(false);
            return;
        };
        if velocity != movement {
            entity.velocity.store(velocity);
            entity.velocity_dirty.store(true, Relaxed);
        }
    }

    fn rolls_at_edge(&self) -> Option<bool> {
        let entity = self.get_entity();
        if !((self.is_baby() || self.is_playful()) && entity.on_ground.load(Relaxed))
            || !self.can_perform_action()
        {
            return None;
        }
        let (x_step, z_step) = facing_step(entity.yaw.load());
        let feet = entity.block_pos.load();
        let ahead = BlockPos::new(feet.0.x + x_step, feet.0.y - 1, feet.0.z + z_step);
        Some(entity.world.load().get_block_state(&ahead).is_air())
    }
}

/// Vanilla `PandaRollGoal` (goal priority 12, move, look and jump flags, not interruptible): a
/// baby or playful panda rolls now and then, and always when it faces a drop.
#[derive(Default)]
pub struct PandaRollGoal;

impl Goal for PandaRollGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(panda) = as_panda(mob) else {
            return false;
        };
        match panda.rolls_at_edge() {
            None => false,
            Some(true) => true,
            Some(false) => {
                let mut rng = mob.get_random();
                rolls_roll(panda.is_playful(), |bound| rng.random_range(0..bound))
            }
        }
    }

    /// Vanilla `PandaRollGoal.canContinueToUse`: never, the goal is over right after `start`.
    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        false
    }

    /// Vanilla `PandaRollGoal.start`: `roll(true)`.
    fn start(&mut self, mob: &dyn Mob) {
        if let Some(panda) = as_panda(mob) {
            panda.roll(true);
        }
    }

    /// Vanilla's interruptible check of the roll goal: false.
    fn can_stop(&self) -> bool {
        false
    }

    /// Vanilla `PandaRollGoal` flags: move, look and jump.
    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP
    }

    /// Reads one block next to the panda and raises its roll flag, so it reaches no other entity.
    fn tick_reach(&self, _mob: &dyn Mob, _running: bool) -> Option<GoalReach> {
        Some(GoalReach::default())
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::Block;
    use pumpkin_world::world::BlockFlags;

    use super::super::test_support::{TestResult, scripted, spawn_panda, stone_floor};
    use super::*;
    use crate::entity::ai::test_support::world;

    #[test]
    fn a_playful_panda_rolls_the_short_odds_first() {
        // `reducedTickDelay` halves the bounds 60 and 500.
        let rolls =
            |playful: bool, calls: &[(i32, i32)]| scripted(calls, |next| rolls_roll(playful, next));
        assert!(rolls(true, &[(30, 1)]));
        assert!(rolls(true, &[(30, 0), (250, 1)]));
        assert!(!rolls(true, &[(30, 0), (250, 0)]));
        assert!(rolls(false, &[(250, 1)]));
        assert!(!rolls(false, &[(250, 0)]));
    }

    #[tokio::test]
    async fn a_rolling_panda_pushes_hops_and_stops_after_thirty_two_steps() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = spawn_panda(&world, 0.5, 0.5);

        // A baby pushes less, and a hop in the air keeps the vertical speed.
        let mut delta = Vector3::new(0.0, 0.0, 0.0);
        let rest = Vector3::new(0.0, 0.0, 0.0);
        assert_eq!(
            roll_velocity(1, true, 0.0, true, rest, &mut delta),
            Some(Vector3::new(
                0.0,
                ROLL_JUMP_SPEED,
                f64::from(BABY_ROLL_SPEED)
            ))
        );
        assert_eq!(
            roll_velocity(
                7,
                false,
                0.0,
                false,
                Vector3::new(0.0, -0.3, 0.0),
                &mut delta
            ),
            Some(Vector3::new(0.0, -0.3, 0.0))
        );

        panda.roll(true);
        for counter in 1..=TOTAL_ROLL_STEPS {
            panda.tick_roll();
            assert!(panda.is_rolling(), "the roll ended at step {counter}");
            let push = if ROLL_HOP_STEPS.contains(&counter) {
                0.0
            } else {
                f64::from(ADULT_ROLL_SPEED)
            };
            assert_eq!(
                panda.get_entity().velocity.load(),
                Vector3::new(0.0, ROLL_JUMP_SPEED, push),
                "step {counter}"
            );
        }
        panda.tick_roll();
        assert!(!panda.is_rolling(), "the roll never ended");
        panda.tick_roll();
        assert_eq!(panda.roll_counter.load(Relaxed), 0);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn a_baby_always_rolls_off_a_drop_it_faces() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let set_floor = |x, z, block: &Block| {
            let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
            world.set_block_state(&BlockPos::new(x, 63, z), block.default_state.id, flags);
        };

        assert_eq!(panda.rolls_at_edge(), None, "an adult that is not playful");
        panda.set_age(-24000);
        assert_eq!(panda.rolls_at_edge(), Some(false));

        // Yaw 0 faces +z, so the block below and ahead is (0, 63, 1).
        set_floor(0, 1, &Block::AIR);
        assert_eq!(panda.rolls_at_edge(), Some(true));
        assert!(PandaRollGoal.can_start(panda.as_ref()));
        set_floor(0, 1, &Block::STONE);
        set_floor(0, -1, &Block::AIR);
        assert_eq!(panda.rolls_at_edge(), Some(false), "a drop behind counts");
        panda.get_entity().yaw.store(180.0);
        assert_eq!(panda.rolls_at_edge(), Some(true));

        // Diagonals step on both axes, a direction right at the threshold on neither.
        assert_eq!(facing_step(45.0), (-1, 1));
        assert_eq!(step_towards(0.5), 0);
        world.level.shutdown().await;
        Ok(())
    }
}
