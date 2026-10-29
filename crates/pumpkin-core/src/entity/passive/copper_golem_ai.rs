use pumpkin_data::{entity::EntityType, environment_attribute::Activity};

use crate::entity::{
    ai::brain::{
        Brain, BrainProvider,
        activity::ActivityData,
        behavior::{
            AnimalPanic, CountDownCooldownTicks, DoNothing, LookAtTargetSink, MoveToTargetSink,
            Timed, interact_with_door, random_stroll, run_one_with_conditions,
            set_entity_look_target_sometimes,
            transport_items_between_containers::TransportItemsBetweenContainers,
        },
        memory::{MemoryStatus, PackedMemories, types},
        sensing::SensorType,
    },
    mob::Mob,
};

const BRAIN_PROVIDER: BrainProvider = BrainProvider {
    memory_types: &[],
    sensor_types: &[SensorType::NearestLivingEntities, SensorType::HurtBy],
    activities: get_activities,
};

pub(super) fn make_brain(mob: &dyn Mob, packed: &PackedMemories) -> Brain {
    BRAIN_PROVIDER.make_brain(mob, packed)
}

fn get_activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![
                Box::new(Timed::new(AnimalPanic::new(1.5))),
                Box::new(Timed::new(LookAtTargetSink::new(45, 90))),
                Box::new(Timed::new(MoveToTargetSink::default())),
                Box::new(interact_with_door::create()),
                Box::new(CountDownCooldownTicks::new(types::GAZE_COOLDOWN_TICKS)),
                Box::new(CountDownCooldownTicks::new(
                    types::TRANSPORT_ITEMS_COOLDOWN_TICKS,
                )),
            ],
        ),
        ActivityData::with_pairs(
            Activity::Idle,
            vec![
                (
                    0,
                    Box::new(Timed::new(TransportItemsBetweenContainers::default())),
                ),
                (
                    1,
                    Box::new(set_entity_look_target_sometimes::create_for_type(
                        &EntityType::PLAYER,
                        6.0,
                        40,
                        80,
                    )),
                ),
                (
                    2,
                    Box::new(run_one_with_conditions(
                        vec![
                            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
                            (
                                types::TRANSPORT_ITEMS_COOLDOWN_TICKS.id(),
                                MemoryStatus::ValuePresent,
                            ),
                        ],
                        vec![
                            (Box::new(random_stroll::stroll(1.0, 2, 2)), 1),
                            (Box::new(DoNothing::new(30, 60)), 1),
                        ],
                    )),
                ),
            ],
        ),
    ]
}
