use rand::Rng;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus};
use super::shuffling_list::ShufflingList;
use super::{BehaviorControl, Status};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrderPolicy {
    Ordered,
    Shuffled,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunningPolicy {
    RunOne,
    TryAll,
}

pub struct GateBehavior {
    name: &'static str,
    entry_condition: Box<[(MemoryModuleId, MemoryStatus)]>,
    exit_erased_memories: Box<[MemoryModuleId]>,
    order_policy: OrderPolicy,
    running_policy: RunningPolicy,
    behaviors: ShufflingList<Box<dyn BehaviorControl>>,
    required: Box<[MemoryModuleId]>,
    status: Status,
}

impl GateBehavior {
    #[must_use]
    pub fn new(
        name: &'static str,
        entry_condition: Vec<(MemoryModuleId, MemoryStatus)>,
        exit_erased_memories: Vec<MemoryModuleId>,
        order_policy: OrderPolicy,
        running_policy: RunningPolicy,
        weighted_behaviors: Vec<(Box<dyn BehaviorControl>, i32)>,
    ) -> Self {
        let mut behaviors = ShufflingList::new();
        let mut required: Vec<MemoryModuleId> = entry_condition.iter().map(|(id, _)| *id).collect();
        for (behavior, weight) in weighted_behaviors {
            for id in behavior.required_memories() {
                if !required.contains(id) {
                    required.push(*id);
                }
            }
            behaviors.add(behavior, weight);
        }
        Self {
            name,
            entry_condition: entry_condition.into_boxed_slice(),
            exit_erased_memories: exit_erased_memories.into_boxed_slice(),
            order_policy,
            running_policy,
            behaviors,
            required: required.into_boxed_slice(),
            status: Status::Stopped,
        }
    }

    pub fn pick_order<R: Rng>(&mut self, rng: &mut R) {
        if self.order_policy == OrderPolicy::Shuffled {
            self.behaviors.shuffle(rng);
        }
    }
}

impl BehaviorControl for GateBehavior {
    fn status(&self) -> Status {
        self.status
    }

    fn required_memories(&self) -> &[MemoryModuleId] {
        &self.required
    }

    fn try_start(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if !tick.brain.check_all(&self.entry_condition) {
            return false;
        }
        self.status = Status::Running;
        let mut rng = tick.mob.get_random();
        self.pick_order(&mut rng);
        for behavior in self.behaviors.iter_mut() {
            if behavior.status() == Status::Stopped
                && behavior.try_start(tick)
                && self.running_policy == RunningPolicy::RunOne
            {
                break;
            }
        }
        true
    }

    fn tick_or_stop(&mut self, tick: &mut BrainTick<'_>) {
        for behavior in self.behaviors.iter_mut() {
            if behavior.status() == Status::Running {
                behavior.tick_or_stop(tick);
            }
        }
        if !self
            .behaviors
            .iter()
            .any(|behavior| behavior.status() == Status::Running)
        {
            self.do_stop(tick);
        }
    }

    fn do_stop(&mut self, tick: &mut BrainTick<'_>) {
        self.status = Status::Stopped;
        for behavior in self.behaviors.iter_mut() {
            if behavior.status() == Status::Running {
                behavior.do_stop(tick);
            }
        }
        for id in &self.exit_erased_memories {
            tick.brain.erase(*id);
        }
    }

    fn debug_string(&self) -> String {
        let running: Vec<String> = self
            .behaviors
            .iter()
            .filter(|behavior| behavior.status() == Status::Running)
            .map(|behavior| behavior.debug_string())
            .collect();
        format!("{}: [{}]", self.name, running.join(", "))
    }
}

#[must_use]
pub fn run_one(weighted_behaviors: Vec<(Box<dyn BehaviorControl>, i32)>) -> GateBehavior {
    run_one_with_conditions(Vec::new(), weighted_behaviors)
}

#[must_use]
pub fn run_one_with_conditions(
    entry_condition: Vec<(MemoryModuleId, MemoryStatus)>,
    weighted_behaviors: Vec<(Box<dyn BehaviorControl>, i32)>,
) -> GateBehavior {
    GateBehavior::new(
        "RunOne",
        entry_condition,
        Vec::new(),
        OrderPolicy::Shuffled,
        RunningPolicy::RunOne,
        weighted_behaviors,
    )
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use super::super::super::memory::types;
    use super::super::OneShot;
    use super::*;

    fn named(name: &'static str) -> (Box<dyn BehaviorControl>, i32) {
        (
            Box::new(OneShot::new(name, Vec::new(), |_| true)) as Box<dyn BehaviorControl>,
            1,
        )
    }

    fn names(gate: &GateBehavior) -> Vec<String> {
        gate.behaviors
            .iter()
            .map(|behavior| behavior.debug_string())
            .collect()
    }

    #[test]
    fn required_memories_are_the_entry_condition_plus_the_children() {
        let child = OneShot::new(
            "Child",
            vec![(types::HUNTED_RECENTLY.id(), MemoryStatus::ValueAbsent)],
            |_| true,
        );
        let gate = GateBehavior::new(
            "Gate",
            vec![(types::ADMIRING_ITEM.id(), MemoryStatus::ValuePresent)],
            Vec::new(),
            OrderPolicy::Ordered,
            RunningPolicy::TryAll,
            vec![(Box::new(child) as Box<dyn BehaviorControl>, 1)],
        );
        assert_eq!(
            gate.required_memories(),
            &[types::ADMIRING_ITEM.id(), types::HUNTED_RECENTLY.id()]
        );
        assert_eq!(gate.status(), Status::Stopped);
    }

    #[test]
    fn ordered_gates_keep_insertion_order() {
        let mut gate = GateBehavior::new(
            "Gate",
            Vec::new(),
            Vec::new(),
            OrderPolicy::Ordered,
            RunningPolicy::RunOne,
            vec![named("a"), named("b"), named("c")],
        );
        let mut rng = StdRng::seed_from_u64(3);
        for _ in 0..8 {
            gate.pick_order(&mut rng);
            assert_eq!(names(&gate), ["a", "b", "c"]);
        }
    }

    #[test]
    fn shuffled_gates_permute_without_losing_entries() {
        let mut gate = run_one(vec![named("a"), named("b"), named("c")]);
        let mut rng = StdRng::seed_from_u64(3);
        let mut permuted = false;
        for _ in 0..32 {
            gate.pick_order(&mut rng);
            let mut order = names(&gate);
            permuted |= order != ["a", "b", "c"];
            order.sort();
            assert_eq!(order, ["a", "b", "c"]);
        }
        assert!(permuted);
        assert_eq!(gate.debug_string(), "RunOne: []");
    }
}
