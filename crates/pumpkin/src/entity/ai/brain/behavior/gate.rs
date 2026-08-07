//! Port of `behavior/GateBehavior.java` and its `RunOne` specialization
//! (`behavior/RunOne.java`), plus the weighted shuffle from `behavior/ShufflingList.java`.
//!
//! `GateBehavior` implements `BehaviorControl` directly in vanilla, so it does the same here:
//! it is a combinator over child behaviors, not a `Behavior` subclass.

use rand::RngExt;

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::behavior::{Behavior, BehaviorStatus};
use crate::entity::ai::brain::memory::{MemoryKeyId, MemoryStatus};
use crate::entity::mob::Mob;

/// `GateBehavior.OrderPolicy` (`GateBehavior.java:104-117`).
#[derive(Clone, Copy)]
pub enum OrderPolicy {
    Ordered,
    Shuffled,
}

/// `GateBehavior.RunningPolicy` (`GateBehavior.java:119-140`).
#[derive(Clone, Copy)]
pub enum RunningPolicy {
    /// Starts children in order until the first one that succeeds.
    RunOne,
    /// Attempts to start every stopped child.
    TryAll,
}

pub struct GateBehavior {
    entry_condition: Vec<(MemoryKeyId, MemoryStatus)>,
    /// Union of `entry_condition` and every child's required memories
    /// (`GateBehavior.getRequiredMemories`, `:44-53`). Precomputed so `required_memories` can
    /// return a slice.
    required_memories: Vec<(MemoryKeyId, MemoryStatus)>,
    exit_erased_memories: Vec<MemoryKeyId>,
    order_policy: OrderPolicy,
    running_policy: RunningPolicy,
    /// `ShufflingList<BehaviorControl>` (`GateBehavior.java:22`): `(behavior, weight)`.
    behaviors: Vec<(Box<dyn Behavior>, i32)>,
    status: BehaviorStatus,
}

impl GateBehavior {
    #[must_use]
    pub fn new(
        entry_condition: Vec<(MemoryKeyId, MemoryStatus)>,
        exit_erased_memories: Vec<MemoryKeyId>,
        order_policy: OrderPolicy,
        running_policy: RunningPolicy,
        behaviors: Vec<(Box<dyn Behavior>, i32)>,
    ) -> Self {
        let mut required_memories = entry_condition.clone();
        for (behavior, _) in &behaviors {
            for condition in behavior.required_memories() {
                if !required_memories.contains(condition) {
                    required_memories.push(*condition);
                }
            }
        }
        Self {
            entry_condition,
            required_memories,
            exit_erased_memories,
            order_policy,
            running_policy,
            behaviors,
            status: BehaviorStatus::Stopped,
        }
    }

    /// `new RunOne(weightedBehaviors)` (`RunOne.java:13-19`): no entry condition, nothing erased
    /// on exit, `SHUFFLED` order, `RUN_ONE` policy.
    #[must_use]
    pub fn run_one(behaviors: Vec<(Box<dyn Behavior>, i32)>) -> Box<dyn Behavior> {
        Box::new(Self::new(
            Vec::new(),
            Vec::new(),
            OrderPolicy::Shuffled,
            RunningPolicy::RunOne,
            behaviors,
        ))
    }

    /// `ShufflingList.shuffle` (`ShufflingList.java:76-80`): each entry gets a sort key of
    /// `-nextFloat().powf(1.0 / weight)` and the list is sorted ascending by it. That is the
    /// standard weighted-reservoir ordering -- higher weight means a key closer to -1, so it
    /// sorts earlier and is tried first.
    fn shuffle(&mut self) {
        let mut rng = rand::rng();
        let mut keyed: Vec<(f32, usize)> = (0..self.behaviors.len())
            .map(|index| {
                let weight = self.behaviors[index].1;
                let roll: f32 = rng.random::<f32>();
                #[allow(clippy::cast_precision_loss)]
                let key = -roll.powf(1.0 / weight as f32);
                (key, index)
            })
            .collect();
        keyed.sort_by(|a, b| a.0.total_cmp(&b.0));

        let mut reordered: Vec<Option<(Box<dyn Behavior>, i32)>> =
            self.behaviors.drain(..).map(Some).collect();
        self.behaviors = keyed
            .into_iter()
            .filter_map(|(_, index)| reordered[index].take())
            .collect();
    }
}

impl Behavior for GateBehavior {
    fn required_memories(&self) -> &[(MemoryKeyId, MemoryStatus)] {
        &self.required_memories
    }

    fn status(&self) -> BehaviorStatus {
        self.status
    }

    /// `tryStart` (`GateBehavior.java:67-77`).
    ///
    /// Note it gates on `entry_condition` only, NOT on `required_memories` -- vanilla's
    /// `hasRequiredMemories` here reads `this.entryCondition` (`:55-65`) even though
    /// `getRequiredMemories` returns the wider union. The union exists purely so the brain
    /// registers every memory the children touch.
    fn try_start(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) -> bool {
        if !brain.check_memories(&self.entry_condition) {
            return false;
        }
        self.status = BehaviorStatus::Running;
        if matches!(self.order_policy, OrderPolicy::Shuffled) {
            self.shuffle();
        }
        match self.running_policy {
            RunningPolicy::RunOne => {
                for (behavior, _) in &mut self.behaviors {
                    if behavior.status() == BehaviorStatus::Stopped
                        && behavior.try_start(mob, brain, game_time)
                    {
                        break;
                    }
                }
            }
            RunningPolicy::TryAll => {
                for (behavior, _) in &mut self.behaviors {
                    if behavior.status() == BehaviorStatus::Stopped {
                        behavior.try_start(mob, brain, game_time);
                    }
                }
            }
        }
        true
    }

    /// `tickOrStop` (`GateBehavior.java:79-85`): the gate stops itself once no child is running.
    fn tick_or_stop(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) {
        for (behavior, _) in &mut self.behaviors {
            if behavior.status() == BehaviorStatus::Running {
                behavior.tick_or_stop(mob, brain, game_time);
            }
        }
        if self
            .behaviors
            .iter()
            .all(|(behavior, _)| behavior.status() != BehaviorStatus::Running)
        {
            self.do_stop(mob, brain, game_time);
        }
    }

    /// `doStop` (`GateBehavior.java:87-92`).
    fn do_stop(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) {
        self.status = BehaviorStatus::Stopped;
        for (behavior, _) in &mut self.behaviors {
            if behavior.status() == BehaviorStatus::Running {
                behavior.do_stop(mob, brain, game_time);
            }
        }
        if !self.exit_erased_memories.is_empty() {
            let mut store = brain.memory.lock().unwrap();
            for id in &self.exit_erased_memories {
                store.erase_by_id(*id);
            }
        }
    }

    fn debug_name(&self) -> &'static str {
        "GateBehavior"
    }
}
