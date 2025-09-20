use crate::entity::mob::Mob;
use async_trait::async_trait;
use std::{any::TypeId, ops::BitOr, ptr};

pub mod active_target_goal;
pub mod ambient_stand_goal;
pub mod goal_selector;
pub mod look_around_goal;
pub mod look_at_entity;
mod melee_attack_goal;
pub mod move_to_target_pos_goal;
pub mod step_and_destroy_block_goal;
mod track_target_goal;
pub mod zombie_attack_goal;

#[must_use]
pub fn to_goal_ticks(server_ticks: i32) -> i32 {
    -(-server_ticks).div_euclid(2)
}

#[async_trait]
pub trait Goal: Send + Sync {
    /// How should the `Goal` initially start?
    async fn can_start(&mut self, mob: &dyn Mob) -> bool;
    /// When it's started, how should it continue to run?
    async fn should_continue(&self, mob: &dyn Mob) -> bool;
    /// Call when goal start
    async fn start(&mut self, _: &dyn Mob) {}
    /// Call when goal stop
    async fn stop(&mut self, _: &dyn Mob) {}
    /// If the `Goal` is running, this gets called every tick.
    async fn tick(&mut self, _: &dyn Mob) {}

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

#[derive(Default, Clone, Copy)]
// We actualy only use the first 4 bits ;)
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

    pub fn set(&mut self, control: Self, val: bool) {
        if val {
            self.0 |= control.0;
        } else {
            self.0 &= !control.0;
        }
    }

    #[must_use]
    pub fn get(&self, control: Self) -> bool {
        self.0 & control.0 != 0
    }

    #[must_use]
    pub fn idx(&self) -> usize {
        for (i, control) in Self::ITER.into_iter().enumerate() {
            if self.get(control) {
                return i;
            }
        }
        unreachable!()
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

#[async_trait]
impl Goal for PrioritizedGoal {
    async fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.goal.can_start(mob).await
    }

    async fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.goal.should_continue(mob).await
    }

    async fn start(&mut self, mob: &dyn Mob) {
        if !self.running {
            self.running = true;
            self.goal.start(mob).await;
        }
    }

    async fn stop(&mut self, mob: &dyn Mob) {
        if self.running {
            self.running = false;
            self.goal.stop(mob).await;
        }
    }

    async fn tick(&mut self, mob: &dyn Mob) {
        self.goal.tick(mob).await;
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
