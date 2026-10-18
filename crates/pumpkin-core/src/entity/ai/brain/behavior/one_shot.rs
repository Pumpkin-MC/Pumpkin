use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus};
use super::gate::{OrderPolicy, RunningPolicy};
use super::shuffling_list::ShufflingList;
use super::{BehaviorControl, Status, required_memories_of};

pub type Trigger = Box<dyn FnMut(&mut BrainTick<'_>) -> bool + Send + Sync>;

pub struct OneShot {
    name: &'static str,
    conditions: Box<[(MemoryModuleId, MemoryStatus)]>,
    required: Box<[MemoryModuleId]>,
    trigger: Trigger,
    status: Status,
}

impl OneShot {
    pub fn new(
        name: &'static str,
        conditions: Vec<(MemoryModuleId, MemoryStatus)>,
        trigger: impl FnMut(&mut BrainTick<'_>) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self::with_required(name, conditions, Vec::new(), trigger)
    }

    pub fn with_required(
        name: &'static str,
        conditions: Vec<(MemoryModuleId, MemoryStatus)>,
        extra_required: Vec<MemoryModuleId>,
        trigger: impl FnMut(&mut BrainTick<'_>) -> bool + Send + Sync + 'static,
    ) -> Self {
        let mut required = required_memories_of(&conditions).into_vec();
        for id in extra_required {
            if !required.contains(&id) {
                required.push(id);
            }
        }
        Self {
            name,
            conditions: conditions.into_boxed_slice(),
            required: required.into_boxed_slice(),
            trigger: Box::new(trigger),
            status: Status::Stopped,
        }
    }
}

impl OneShot {
    fn fire(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.brain.check_all(&self.conditions) && (self.trigger)(tick)
    }
}

impl BehaviorControl for OneShot {
    fn status(&self) -> Status {
        self.status
    }

    fn required_memories(&self) -> &[MemoryModuleId] {
        &self.required
    }

    fn try_start(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if !self.fire(tick) {
            return false;
        }
        self.status = Status::Running;
        true
    }

    fn tick_or_stop(&mut self, tick: &mut BrainTick<'_>) {
        self.do_stop(tick);
    }

    fn do_stop(&mut self, _tick: &mut BrainTick<'_>) {
        self.status = Status::Stopped;
    }

    fn debug_string(&self) -> String {
        self.name.to_owned()
    }
}

#[must_use]
pub fn trigger_if(
    name: &'static str,
    conditions: Vec<(MemoryModuleId, MemoryStatus)>,
    predicate: impl Fn(&BrainTick<'_>) -> bool + Send + Sync + 'static,
) -> OneShot {
    OneShot::new(name, conditions, move |tick| predicate(tick))
}

#[must_use]
pub fn sequence(
    name: &'static str,
    mut first: impl FnMut(&mut BrainTick<'_>) -> bool + Send + Sync + 'static,
    mut second: OneShot,
) -> OneShot {
    let required = second.required.to_vec();
    OneShot::with_required(name, Vec::new(), required, move |tick| {
        first(tick) && second.fire(tick)
    })
}

#[must_use]
pub fn trigger_gate(
    name: &'static str,
    weighted_triggers: Vec<(Trigger, i32)>,
    order: OrderPolicy,
    running: RunningPolicy,
) -> OneShot {
    let mut triggers = ShufflingList::new();
    for (trigger, weight) in weighted_triggers {
        triggers.add(trigger, weight);
    }
    OneShot::new(name, Vec::new(), move |tick| {
        if order == OrderPolicy::Shuffled {
            let mut rng = tick.mob.get_random();
            triggers.shuffle(&mut rng);
        }
        for trigger in triggers.iter_mut() {
            if trigger(tick) && running == RunningPolicy::RunOne {
                break;
            }
        }
        true
    })
}

#[must_use]
pub fn trigger_one_shuffled(name: &'static str, weighted_triggers: Vec<(Trigger, i32)>) -> OneShot {
    trigger_gate(
        name,
        weighted_triggers,
        OrderPolicy::Shuffled,
        RunningPolicy::RunOne,
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::Brain;
    use super::super::super::memory::types;
    use super::*;

    fn admire_conditions() -> Vec<(MemoryModuleId, MemoryStatus)> {
        vec![
            (types::ADMIRING_ITEM.id(), MemoryStatus::ValuePresent),
            (types::ADMIRING_DISABLED.id(), MemoryStatus::ValueAbsent),
        ]
    }

    #[test]
    fn required_memories_are_the_declared_conditions() {
        let behavior = OneShot::new("Admire", admire_conditions(), |_| true);
        assert_eq!(
            behavior.required_memories(),
            &[types::ADMIRING_ITEM.id(), types::ADMIRING_DISABLED.id()]
        );
        assert_eq!(behavior.status(), Status::Stopped);
        assert_eq!(behavior.debug_string(), "Admire");
    }

    #[test]
    fn extra_required_memories_are_merged_without_duplicates() {
        let behavior = OneShot::with_required(
            "Admire",
            admire_conditions(),
            vec![types::ADMIRING_ITEM.id(), types::HUNTED_RECENTLY.id()],
            |_| true,
        );
        assert_eq!(
            behavior.required_memories(),
            &[
                types::ADMIRING_ITEM.id(),
                types::ADMIRING_DISABLED.id(),
                types::HUNTED_RECENTLY.id()
            ]
        );
    }

    #[test]
    fn check_all_requires_every_condition() {
        let conditions = admire_conditions();
        let mut brain = Brain::default();
        assert!(!brain.check_all(&conditions));

        brain.register_memory(types::ADMIRING_ITEM.id());
        brain.register_memory(types::ADMIRING_DISABLED.id());
        assert!(!brain.check_all(&conditions));

        brain.set(types::ADMIRING_ITEM, true);
        assert!(brain.check_all(&conditions));

        brain.set(types::ADMIRING_DISABLED, true);
        assert!(!brain.check_all(&conditions));
    }
}
