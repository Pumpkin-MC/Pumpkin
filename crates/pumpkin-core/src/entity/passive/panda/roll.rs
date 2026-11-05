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

/// The (x, z) block offset of the block the panda faces: vanilla `PandaRollGoal.canUse` turns
/// the yaw into a direction and steps along each axis it clearly points to.
fn facing_step(yaw: f32) -> (i32, i32) {
    let angle = yaw * DEGREES_TO_RADIANS;
    (step_towards(-sin(angle)), step_towards(cos(angle)))
}

/// Vanilla `PandaRollGoal.canUse` once the panda faces a block that is not air: a playful
/// panda first rolls the short odds and every panda falls back to the long odds.
/// `next_int(bound)` stands for vanilla's `RandomSource.nextInt`, and both bounds go through
/// `reducedTickDelay` because the selector asks every second tick.
fn rolls_roll(playful: bool, mut next_int: impl FnMut(i32) -> i32) -> bool {
    (playful && next_int(to_goal_ticks(PLAYFUL_ROLL_BOUND)) == 1)
        || next_int(to_goal_ticks(ROLL_BOUND)) == 1
}

/// The velocity of one `handleRoll` step for a roll counter that already counts this tick, or
/// `None` once the roll is over. `roll_delta` is the horizontal push of the first step and is
/// replaced by it.
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

    /// The rolling part of vanilla `Panda.tick`: while the flag is set `handleRoll` runs,
    /// otherwise the roll counter is reset.
    pub(super) fn tick_roll(&self) {
        if self.is_rolling() {
            self.handle_roll();
        } else {
            self.roll_counter.store(0, Relaxed);
        }
    }

    /// Vanilla `Panda.handleRoll`: the counter goes up each tick and the roll ends after 32. The
    /// first step pushes the panda forwards and up, later steps keep that push, and steps 7, 15
    /// and 23 stop it and hop.
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

    /// The part of vanilla `PandaRollGoal.canUse` before the random rolls: a baby or playful panda
    /// on the ground that can act and whose facing block is air to roll off.
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

impl PandaRollGoal {
    /// Vanilla `PandaRollGoal.canUse` with the random source given by `next_int`.
    fn can_start_with(panda: &PandaEntity, next_int: impl FnMut(i32) -> i32) -> bool {
        match panda.rolls_at_edge() {
            None => false,
            Some(true) => true,
            Some(false) => rolls_roll(panda.is_playful(), next_int),
        }
    }
}

impl Goal for PandaRollGoal {
    /// Vanilla `PandaRollGoal.canUse`.
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(panda) = as_panda(mob) else {
            return false;
        };
        let mut rng = mob.get_random();
        Self::can_start_with(panda, |bound| rng.random_range(0..bound))
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

    /// The goal reads one block next to the panda and raises its roll flag, so it reaches no
    /// other entity. The roll movement belongs to the panda tick, not to this goal.
    fn tick_reach(&self, _mob: &dyn Mob, _running: bool) -> Option<GoalReach> {
        Some(GoalReach::default())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pumpkin_data::Block;
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::entity_variant::PandaGene;
    use pumpkin_util::math::vector2::Vector2;
    use pumpkin_world::chunk::ChunkData;
    use pumpkin_world::world::BlockFlags;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::test_support::world;
    use crate::world::World;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    /// Server ticks the selector test lets the look goals of higher priorities finish.
    const SELECTOR_TICKS: i32 = 200;

    /// A panda standing on the stone floor at `(x, z)`, facing south (yaw 0, towards +z).
    fn spawn_panda(world: &Arc<World>, x: f64, z: f64) -> Arc<PandaEntity> {
        let panda = PandaEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, z),
            &EntityType::PANDA,
        ));
        panda.get_entity().on_ground.store(true, Relaxed);
        world.spawn_entity(panda.clone());
        panda
    }

    fn stone_floor(world: &Arc<World>) {
        for chunk_x in -2..=1 {
            for chunk_z in -2..=1 {
                world.level.loaded_chunks.insert(
                    Vector2::new(chunk_x, chunk_z),
                    ChunkData::empty_sync(chunk_x, chunk_z),
                );
            }
        }
        let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
        for x in -20..=20 {
            for z in -20..=20 {
                world.set_block_state(
                    &BlockPos::new(x, 63, z),
                    Block::STONE.default_state.id,
                    flags,
                );
            }
        }
    }

    fn velocity(panda: &PandaEntity) -> Vector3<f64> {
        panda.get_entity().velocity.load()
    }

    fn assert_velocity(panda: &PandaEntity, expected: (f64, f64, f64), message: &str) {
        let velocity = velocity(panda);
        assert!(
            (velocity.x - expected.0).abs() < 1e-6
                && (velocity.y - expected.1).abs() < 1e-6
                && (velocity.z - expected.2).abs() < 1e-6,
            "{message}: {velocity:?}"
        );
    }

    /// Runs `can_start_with` against scripted `(bound, result)` random calls.
    fn can_start(panda: &PandaEntity, calls: &[(i32, i32)]) -> bool {
        let mut script = calls.iter();
        let starts = PandaRollGoal::can_start_with(panda, |bound| {
            let (expected_bound, result) = script.next().copied().unwrap_or((-1, -1));
            assert_eq!(bound, expected_bound);
            result
        });
        assert!(script.next().is_none(), "unused scripted random calls");
        starts
    }

    #[test]
    fn panda_steps_only_along_the_axes_it_clearly_faces() {
        // Yaw 0 faces +z, 90 faces -x, 180 faces -z and -90 faces +x.
        assert_eq!(facing_step(0.0), (0, 1));
        assert_eq!(facing_step(90.0), (-1, 0));
        assert_eq!(facing_step(180.0), (0, -1));
        assert_eq!(facing_step(-90.0), (1, 0));
        // Diagonals step on both axes, and a direction right at the threshold on neither.
        assert_eq!(facing_step(45.0), (-1, 1));
        assert_eq!(facing_step(-135.0), (1, -1));
        assert_eq!(step_towards(0.5), 0);
        assert_eq!(step_towards(-0.5), 0);
        assert_eq!(step_towards(0.51), 1);
        assert_eq!(step_towards(-0.51), -1);
    }

    /// Runs `rolls_roll` against scripted `(bound, result)` random calls.
    fn roll_odds(playful: bool, calls: &[(i32, i32)]) -> bool {
        let mut script = calls.iter();
        let rolls = rolls_roll(playful, |bound| {
            let (expected_bound, result) = script.next().copied().unwrap_or((-1, -1));
            assert_eq!(bound, expected_bound);
            result
        });
        assert!(script.next().is_none(), "unused scripted random calls");
        rolls
    }

    #[test]
    fn roll_odds_try_the_short_roll_for_playful_pandas_only() {
        // Reduced tick delay halves the vanilla bounds 60 and 500.
        assert!(roll_odds(true, &[(30, 1)]));
        assert!(roll_odds(true, &[(30, 0), (250, 1)]));
        assert!(!roll_odds(true, &[(30, 0), (250, 0)]));
        assert!(!roll_odds(true, &[(30, 2), (250, 2)]));
        assert!(roll_odds(false, &[(250, 1)]));
        assert!(!roll_odds(false, &[(250, 0)]));
    }

    #[test]
    fn first_step_pushes_forwards_and_up_and_the_next_steps_keep_the_push() -> TestResult {
        let mut delta = Vector3::new(0.0, 0.0, 0.0);
        let rest = Vector3::new(0.0, 0.0, 0.0);
        // Yaw 0 pushes towards +z, an adult 0.2 and a baby 0.1.
        let first = roll_velocity(1, false, 0.0, true, rest, &mut delta);
        assert_eq!(
            first,
            Some(Vector3::new(0.0, 0.27, f64::from(ADULT_ROLL_SPEED)))
        );
        assert_eq!(delta, Vector3::new(0.0, 0.0, f64::from(ADULT_ROLL_SPEED)));
        let mut baby_delta = Vector3::new(0.0, 0.0, 0.0);
        let baby = roll_velocity(1, true, 0.0, true, rest, &mut baby_delta);
        assert_eq!(
            baby,
            Some(Vector3::new(0.0, 0.27, f64::from(BABY_ROLL_SPEED)))
        );
        // Facing -x (yaw 90) the push has no z part, and it adds to the current movement.
        let mut sideways = Vector3::new(0.0, 0.0, 0.0);
        let moving = Vector3::new(0.05, 0.1, -0.02);
        let side = roll_velocity(1, false, 90.0, true, moving, &mut sideways)
            .ok_or("the first step is inside the roll")?;
        assert!((side.x - (0.05 - 0.2)).abs() < 1e-6);
        assert!((side.y - 0.27).abs() < 1e-9);
        assert!((side.z - -0.02).abs() < 1e-6);

        // A later step keeps the push and the vertical speed the physics gave the panda.
        let falling = Vector3::new(0.3, -0.4, 0.3);
        let later = roll_velocity(2, false, 33.0, true, falling, &mut delta);
        assert_eq!(
            later,
            Some(Vector3::new(0.0, -0.4, f64::from(ADULT_ROLL_SPEED)))
        );
        Ok(())
    }

    #[test]
    fn hop_steps_stop_the_push_and_only_jump_from_the_ground() {
        let mut delta = Vector3::new(0.1, 0.0, 0.1);
        let movement = Vector3::new(0.1, -0.3, 0.1);
        for counter in [7, 15, 23] {
            assert_eq!(
                roll_velocity(counter, false, 0.0, true, movement, &mut delta),
                Some(Vector3::new(0.0, 0.27, 0.0)),
                "hop {counter} on the ground"
            );
            assert_eq!(
                roll_velocity(counter, false, 0.0, false, movement, &mut delta),
                Some(Vector3::new(0.0, -0.3, 0.0)),
                "hop {counter} in the air"
            );
        }
        // The hops leave the stored push alone, so the next step carries on with it.
        assert_eq!(delta, Vector3::new(0.1, 0.0, 0.1));
        assert_eq!(
            roll_velocity(8, false, 0.0, true, movement, &mut delta),
            Some(Vector3::new(0.1, -0.3, 0.1))
        );
    }

    #[test]
    fn roll_is_over_after_thirty_two_steps() {
        let mut delta = Vector3::new(0.0, 0.0, 0.0);
        let rest = Vector3::new(0.0, 0.0, 0.0);
        assert!(roll_velocity(32, false, 0.0, true, rest, &mut delta).is_some());
        assert!(roll_velocity(33, false, 0.0, true, rest, &mut delta).is_none());
    }

    #[tokio::test]
    async fn rolling_panda_counts_steps_moves_and_stops_after_thirty_two_ticks() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = spawn_panda(&world, 0.5, 0.5);
        panda.mob_init_data_tracker();
        let synched = &panda.get_entity().synched_data;

        // Without the flag the counter stays at zero and nothing moves.
        panda.roll_counter.store(9, Relaxed);
        panda.tick_roll();
        assert_eq!(panda.roll_counter.load(Relaxed), 0);
        assert_velocity(&panda, (0.0, 0.0, 0.0), "moved without rolling");

        synched.clear_dirty();
        panda.roll(true);
        assert!(panda.is_rolling());
        assert!(synched.is_dirty(), "the roll flag was not synched");
        assert!(!panda.can_perform_action());

        panda.tick_roll();
        assert_eq!(panda.roll_counter.load(Relaxed), 1);
        assert_velocity(&panda, (0.0, 0.27, 0.2), "first step");
        for counter in 2..=TOTAL_ROLL_STEPS {
            panda.tick_roll();
            assert_eq!(panda.roll_counter.load(Relaxed), counter);
            assert!(panda.is_rolling(), "the roll ended at step {counter}");
            let expected = if ROLL_HOP_STEPS.contains(&counter) {
                (0.0, 0.27, 0.0)
            } else {
                (0.0, 0.27, 0.2)
            };
            // The scripted velocity keeps the vertical speed it had, apart from the hops.
            assert_velocity(&panda, expected, &format!("step {counter}"));
        }
        panda.tick_roll();
        assert!(!panda.is_rolling(), "the roll never ended");
        assert!(panda.can_perform_action());
        panda.tick_roll();
        assert_eq!(panda.roll_counter.load(Relaxed), 0);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn goal_needs_a_baby_or_playful_panda_on_the_ground_that_can_act() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);

        // An adult that is not playful never rolls and draws no random number.
        assert!(!can_start(&panda, &[]));

        panda.set_main_gene(PandaGene::Playful);
        panda.set_hidden_gene(PandaGene::Playful);
        assert!(can_start(&panda, &[(30, 1)]));
        panda.get_entity().on_ground.store(false, Relaxed);
        assert!(!can_start(&panda, &[]), "a panda in the air rolled");
        panda.get_entity().on_ground.store(true, Relaxed);
        panda.set_sitting(true);
        assert!(!can_start(&panda, &[]), "a sitting panda rolled");
        panda.set_sitting(false);
        panda.set_on_back(true);
        assert!(!can_start(&panda, &[]), "a panda on its back rolled");
        panda.set_on_back(false);

        panda.set_main_gene(PandaGene::Normal);
        panda.set_hidden_gene(PandaGene::Normal);
        panda.set_age(-24000);
        assert!(panda.is_baby());
        // A baby rolls only the long odds.
        assert!(can_start(&panda, &[(250, 1)]));
        assert!(!can_start(&panda, &[(250, 0)]));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn panda_always_rolls_off_a_drop_it_faces() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        panda.set_age(-24000);
        assert!(!can_start(&panda, &[(250, 0)]));

        // Facing +z: the block diagonally below ahead is (0, 63, 1).
        let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
        world.set_block_state(&BlockPos::new(0, 63, 1), Block::AIR.default_state.id, flags);
        assert!(can_start(&panda, &[]), "the drop ahead was not seen");

        // A drop behind the panda does not count.
        world.set_block_state(
            &BlockPos::new(0, 63, 1),
            Block::STONE.default_state.id,
            flags,
        );
        world.set_block_state(
            &BlockPos::new(0, 63, -1),
            Block::AIR.default_state.id,
            flags,
        );
        assert!(!can_start(&panda, &[(250, 0)]));
        // Turned around, the same drop is ahead.
        panda.get_entity().yaw.store(180.0);
        assert!(can_start(&panda, &[]));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn panda_tick_runs_the_roll_and_the_selector_starts_it_at_a_drop() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
        // Air all around the block the panda stands on, so every facing leads to a drop.
        for (x, z) in [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ] {
            world.set_block_state(&BlockPos::new(x, 63, z), Block::AIR.default_state.id, flags);
        }
        let panda = spawn_panda(&world, 0.5, 0.5);
        panda.set_age(-24000);

        // The registered goal starts the roll for a baby that faces a drop. A look goal of a
        // higher priority may hold the look control for a while first, so allow some ticks.
        let started = (1..=SELECTOR_TICKS).any(|tick| {
            panda.get_entity().tick_count.store(tick, Relaxed);
            panda
                .mob_entity
                .server_ai_step(panda.as_ref(), panda.as_ref());
            panda.is_rolling()
        });
        assert!(started, "the selector never started the roll");

        // The panda tick then takes the first roll step.
        panda.post_tick();
        assert_eq!(panda.roll_counter.load(Relaxed), 1);
        assert!(velocity(&panda).y > 0.2, "the roll did not jump");
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn goal_rolls_once_and_cannot_be_interrupted() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let mut goal = PandaRollGoal;

        assert!(!goal.can_stop());
        assert_eq!(
            goal.controls(),
            Controls::MOVE | Controls::LOOK | Controls::JUMP
        );
        goal.start(panda.as_ref());
        assert!(panda.is_rolling());
        assert!(!goal.should_continue(panda.as_ref()));
        // The goal leaves the roll to the panda tick.
        goal.stop(panda.as_ref());
        assert!(panda.is_rolling());
        world.level.shutdown().await;
        Ok(())
    }
}
