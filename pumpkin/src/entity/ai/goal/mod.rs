use crate::entity::mob::Mob;
use std::{any::TypeId, ops::BitOr, pin::Pin, ptr};

pub mod active_target;
pub mod ambient_stand;
pub mod avoid_entity;
pub mod beg;
pub mod blaze_attack;
pub mod breed;
pub mod chase_player;
pub mod creeper_ignite;
pub mod destroy_egg;
pub mod eat_grass;
pub mod escape_danger;
pub mod follow_owner;
pub mod follow_parent;
pub mod goal_selector;
pub mod look_around;
pub mod look_at_entity;
pub mod melee_attack;
pub mod move_to_target_pos;
pub mod owner_hurt_by_target;
pub mod owner_hurt_target;
pub mod pick_up_block;
pub mod place_block;
pub mod revenge;
pub mod step_and_destroy_block;
pub mod swim;
pub mod teleport_towards_player;
pub mod tempt;
pub(crate) mod track_target;
pub mod wander_around;
pub mod zombie_attack;

#[must_use]
pub const fn to_goal_ticks(server_ticks: i32) -> i32 {
    -(-server_ticks).div_euclid(2)
}

/// Whether this tick should run the full goal-selector pass, which re-evaluates
/// `can_start`/`should_continue` and starts and stops goals, as opposed to only
/// ticking the goals that are already running.
///
/// Mirrors vanilla `Mob#serverAiStep`, which computes a single
/// `idBasedTickCount = this.tickCount + this.getId()` and takes the reduced pass
/// only when that is odd *and* `this.tickCount > 1`. So both terms come off the
/// entity's own tick count: it drives the alternation, and `entity_id` merely
/// staggers it so that not every mob re-plans on the same tick. A mob that has
/// just spawned always takes the full pass, so its goals can start without
/// waiting for the alternation.
///
/// Keying the alternation off the *server* tick instead would look equivalent —
/// the two differ by a per-entity constant — but it is not: an entity that
/// misses server ticks keeps its own counter contiguous while the server's runs
/// on, so its parity would shift for reasons that have nothing to do with how
/// often it has actually ticked.
///
/// Using the ageable age for either term, as this did before, means babies —
/// whose age is negative for their whole 20 minutes of childhood — never satisfy
/// the warm-up check and so re-plan twice as often as vanilla, which also halves
/// every interval that [`to_goal_ticks`] computes.
#[must_use]
pub const fn runs_full_goal_pass(entity_id: i32, entity_tick_count: i32) -> bool {
    entity_tick_count <= 1 || entity_tick_count.wrapping_add(entity_id) % 2 == 0
}

pub type GoalFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Goal: Send + Sync {
    /// How should the `Goal` initially start?
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { false })
    }

    /// When it's started, how should it continue to run?
    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { false })
    }

    /// Call when goal start
    fn start<'a>(&'a mut self, _: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {})
    }

    /// Call when goal stop
    fn stop<'a>(&'a mut self, _: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {})
    }

    /// If the `Goal` is running, this gets called every tick.
    fn tick<'a>(&'a mut self, _: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {})
    }

    fn should_run_every_tick(&self) -> bool {
        false
    }

    fn can_stop(&self) -> bool {
        true
    }

    fn get_tick_count(&self, ticks: i32) -> i32 {
        if self.should_run_every_tick() {
            ticks
        } else {
            to_goal_ticks(ticks)
        }
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}

#[derive(Clone, Copy, Default)]
// We actually only use the first 4 bits ;)
pub struct Controls(u8);

impl Controls {
    pub const MOVE: Self = Self(1);
    pub const LOOK: Self = Self(2);
    pub const JUMP: Self = Self(4);
    pub const TARGET: Self = Self(8);

    pub const ITER: [Self; 4] = [Self::MOVE, Self::LOOK, Self::JUMP, Self::TARGET];

    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    pub const fn set(&mut self, control: Self, val: bool) {
        if val {
            self.0 |= control.0;
        } else {
            self.0 &= !control.0;
        }
    }

    #[must_use]
    pub const fn get(&self, control: Self) -> bool {
        self.0 & control.0 != 0
    }

    #[must_use]
    pub fn idx(&self) -> usize {
        for (i, control) in Self::ITER.into_iter().enumerate() {
            if self.get(control) {
                return i;
            }
        }
        tracing::error!("Controls::idx called with no controls set");
        0
    }
}

impl BitOr for Controls {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

pub struct PrioritizedGoal {
    pub goal: Box<dyn Goal>,
    pub running: bool,
    pub priority: u8,
    /// Used to compare goals of the same type.
    /// Always set to `TypeId::of::<G>()` where `G: Goal`.
    type_id: TypeId,
}

impl PrioritizedGoal {
    #[must_use]
    pub fn new(type_id: TypeId, priority: u8, goal: Box<dyn Goal>) -> Self {
        Self {
            goal,
            running: false,
            priority,
            type_id,
        }
    }

    fn can_be_replaced_by(&self, goal: &Self) -> bool {
        self.can_stop() && goal.priority < self.priority
    }
}

impl Goal for PrioritizedGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.goal.can_start(mob).await })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.goal.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            if !self.running {
                self.running = true;
                self.goal.start(mob).await;
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            if self.running {
                self.running = false;
                self.goal.stop(mob).await;
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.goal.tick(mob).await;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        self.goal.should_run_every_tick()
    }

    fn get_tick_count(&self, ticks: i32) -> i32 {
        self.goal.get_tick_count(ticks)
    }

    fn controls(&self) -> Controls {
        self.goal.controls()
    }
}

#[derive(Clone)]
pub struct ParentHandle<P> {
    ptr: *const P,
}

impl<P> ParentHandle<P> {
    /// This wrapper allows a child struct to hold a reference to its parent
    /// without making the code overly verbose.
    ///
    /// # Safety
    /// - The parent must outlive this handle.
    /// - The parent must be inside a smart pointer; otherwise it
    ///   will move in memory and cause undefined behavior!
    ///
    /// # Example
    /// ```
    /// use pumpkin::entity::ai::goal::ParentHandle;
    ///
    /// struct Parent {
    ///     child: Child,
    ///     value: i32
    /// }
    ///
    /// struct Child {
    ///     parent: ParentHandle<Parent>,
    /// }
    ///
    /// impl Child {
    ///    fn value(&self) -> i32 {
    ///        self.parent.get().unwrap().value
    ///    }
    /// }
    ///
    /// let mut parent = Box::new(Parent {
    ///     child: Child {parent: ParentHandle::none()},
    ///     value: 7,
    /// });
    /// parent.child.parent = unsafe { ParentHandle::new(&parent) };
    ///
    /// assert_eq!(parent.child.value(), 7);
    /// ```
    pub const unsafe fn new(parent: &P) -> Self {
        Self {
            ptr: ptr::from_ref(parent),
        }
    }

    #[must_use]
    /// Creates an empty handle (equivalent to `Option::None`).
    // We can use null as None because we handle it in get.
    pub const fn none() -> Self {
        Self { ptr: ptr::null() }
    }

    #[must_use]
    /// Returns a reference to the parent if available.
    /// This will cause undefined behavior if #Safety rules in new aren't followed
    pub const fn get(&self) -> Option<&P> {
        if self.ptr.is_null() {
            None
        } else {
            unsafe { Some(&*self.ptr) }
        }
    }
}

impl<P> Default for ParentHandle<P> {
    fn default() -> Self {
        Self::none()
    }
}

// This is safe since we own everything.
unsafe impl<P> Sync for ParentHandle<P> {}
unsafe impl<P> Send for ParentHandle<P> {}

#[cfg(test)]
mod tests {
    use super::{runs_full_goal_pass, to_goal_ticks};

    #[test]
    fn goal_ticks_halve_and_round_up() {
        // `to_goal_ticks` assumes the caller is only re-evaluated every other
        // tick, so an interval of n server ticks becomes ceil(n / 2) passes.
        assert_eq!(to_goal_ticks(120), 60);
        assert_eq!(to_goal_ticks(10), 5);
        assert_eq!(to_goal_ticks(1), 1);
        assert_eq!(to_goal_ticks(0), 0);
    }

    #[test]
    fn full_goal_pass_alternates_every_other_tick() {
        let entity_id = 0;
        let passes: Vec<bool> = (100..106)
            .map(|entity_tick_count| runs_full_goal_pass(entity_id, entity_tick_count))
            .collect();
        assert_eq!(passes, vec![true, false, true, false, true, false]);
    }

    #[test]
    fn full_goal_pass_is_staggered_by_entity_id() {
        // Two mobs with adjacent ids must not re-plan on the same tick, which is
        // the whole point of folding the id into the parity.
        for entity_tick_count in 100..108 {
            assert_ne!(
                runs_full_goal_pass(7, entity_tick_count),
                runs_full_goal_pass(8, entity_tick_count),
                "ids 7 and 8 collided on entity tick {entity_tick_count}"
            );
        }
    }

    #[test]
    fn a_freshly_spawned_mob_always_takes_the_full_pass() {
        // Vanilla's `tickCount > 1` warm-up: goals must be able to start on the
        // first ticks without waiting for the alternation to come round. Both
        // parities of `entity_id` have to pass, since the id is the only other
        // term and the warm-up must win regardless of it.
        for entity_id in 0..4 {
            assert!(runs_full_goal_pass(entity_id, 0));
            assert!(runs_full_goal_pass(entity_id, 1));
        }
    }

    #[test]
    fn a_settled_mob_skips_half_of_the_full_passes() {
        // Regression: this gate used to key off the ageable age instead of a
        // tick count. A baby's age stays negative for its whole 20 minutes of
        // childhood, so the warm-up check never stopped applying and babies took
        // the full pass on *every* tick — twice vanilla's decision rate, and
        // twice the goal-selection work. Any mob past the warm-up must skip half
        // the passes, which is also what every `to_goal_ticks` interval assumes.
        let full_passes = (500..600)
            .filter(|&entity_tick_count| runs_full_goal_pass(3, entity_tick_count))
            .count();
        assert_eq!(full_passes, 50);
    }

    #[test]
    fn the_alternation_follows_the_entity_not_the_server() {
        // Vanilla's `idBasedTickCount` is `this.tickCount + this.getId()`, so a
        // mob that has ticked n times is at the same point in the alternation
        // however long the server has been up. Keying off the server tick would
        // shift an entity's parity whenever it missed a tick the server did not.
        for entity_id in 0..4 {
            for entity_tick_count in 2..20 {
                assert_eq!(
                    runs_full_goal_pass(entity_id, entity_tick_count),
                    (entity_tick_count + entity_id) % 2 == 0,
                    "id {entity_id} at entity tick {entity_tick_count}"
                );
            }
        }
    }

    #[test]
    fn full_goal_pass_survives_tick_counter_wraparound() {
        // The entity tick count is an i32 that is only ever incremented, so the
        // parity add must wrap rather than panic in a debug build.
        assert!(runs_full_goal_pass(1, i32::MAX));
        let _ = runs_full_goal_pass(i32::MAX, i32::MAX);
    }
}
