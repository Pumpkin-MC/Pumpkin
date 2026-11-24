use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::Activity;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};

use super::{CopperGolemEntity, CopperGolemState};
use crate::block::blocks::chests;
use crate::entity::EntityBase;
use crate::entity::ai::brain::{
    Brain, BrainProvider, BrainState,
    activity::ActivityData,
    behavior::{
        AnimalPanic, CountDownCooldownTicks, DoNothing, LookAtTargetSink, MoveToTargetSink, Timed,
        door, random_stroll, run_one_with_conditions, set_entity_look_target_sometimes,
        transport_items_between_containers::{
            ContainerInteractionState, TARGET_INTERACTION_TIME, TransportItemTarget,
            TransportItemsBetweenContainers,
        },
    },
    memory::{MemoryStatus, PackedMemories, types},
    sensing::SensorType,
};
use crate::entity::mob::Mob;

const SPEED_MULTIPLIER_WHEN_PANICKING: f32 = 1.5;
const SPEED_MULTIPLIER_WHEN_IDLING: f32 = 1.0;
const TRANSPORT_ITEM_HORIZONTAL_SEARCH_RADIUS: i32 = 32;
const TRANSPORT_ITEM_VERTICAL_SEARCH_RADIUS: i32 = 8;
const TICK_TO_START_ON_REACHED_INTERACTION: i32 = 1;
const TICK_TO_PLAY_ON_REACHED_SOUND: i32 = 9;

pub(super) fn make_brain(golem: &CopperGolemEntity, packed: &PackedMemories) -> Brain {
    BrainProvider {
        memory_types: &[],
        sensor_types: &[SensorType::NearestLivingEntities, SensorType::HurtBy],
        activities,
    }
    .make_brain(golem, packed)
}

pub(super) fn update_activity(brain: &mut BrainState) {
    brain.set_active_activity_to_first_valid(&[Activity::Idle]);
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![init_core_activity(), init_idle_activity()]
}

fn init_core_activity() -> ActivityData {
    ActivityData::with_priority_start(
        Activity::Core,
        0,
        vec![
            Box::new(Timed::new(AnimalPanic::new(
                SPEED_MULTIPLIER_WHEN_PANICKING,
            ))),
            Box::new(Timed::new(LookAtTargetSink::new(45, 90))),
            Box::new(Timed::new(MoveToTargetSink::default())),
            Box::new(door::create()),
            Box::new(CountDownCooldownTicks::new(types::GAZE_COOLDOWN_TICKS)),
            Box::new(CountDownCooldownTicks::new(
                types::TRANSPORT_ITEMS_COOLDOWN_TICKS,
            )),
        ],
    )
}

fn init_idle_activity() -> ActivityData {
    ActivityData::with_pairs(
        Activity::Idle,
        vec![
            (
                0,
                Box::new(TransportItemsBetweenContainers::new(
                    SPEED_MULTIPLIER_WHEN_IDLING,
                    is_source_block,
                    is_destination_block,
                    TRANSPORT_ITEM_HORIZONTAL_SEARCH_RADIUS,
                    TRANSPORT_ITEM_VERTICAL_SEARCH_RADIUS,
                    on_target_interaction,
                    on_travelling,
                    should_queue_for_target,
                )),
            ),
            (
                1,
                Box::new(set_entity_look_target_sometimes::create_matching(
                    Some(&EntityType::PLAYER),
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
                        (
                            Box::new(random_stroll::stroll_with_distance(
                                SPEED_MULTIPLIER_WHEN_IDLING,
                                2,
                                2,
                            )),
                            1,
                        ),
                        (Box::new(DoNothing::new(30, 60)), 1),
                    ],
                )),
            ),
        ],
    )
}

fn is_source_block(block: &'static Block) -> bool {
    block.has_tag(&tag::Block::MINECRAFT_COPPER_CHESTS)
}

fn is_destination_block(block: &'static Block) -> bool {
    block == &Block::CHEST || block == &Block::TRAPPED_CHEST
}

fn on_target_interaction(
    mob: &dyn Mob,
    interaction: ContainerInteractionState,
    target: &TransportItemTarget,
    ticks_since_reaching_target: i32,
) {
    let Some(golem) = mob.cast_any().downcast_ref::<CopperGolemEntity>() else {
        return;
    };
    let (state, sound) = match interaction {
        ContainerInteractionState::PickupItem => (
            CopperGolemState::GettingItem,
            Sound::EntityCopperGolemNoItemGet,
        ),
        ContainerInteractionState::PickupNoItem => (
            CopperGolemState::GettingNoItem,
            Sound::EntityCopperGolemNoItemNoGet,
        ),
        ContainerInteractionState::PlaceItem => (
            CopperGolemState::DroppingItem,
            Sound::EntityCopperGolemItemDrop,
        ),
        ContainerInteractionState::PlaceNoItem => (
            CopperGolemState::DroppingNoItem,
            Sound::EntityCopperGolemItemNoDrop,
        ),
    };

    if ticks_since_reaching_target == TICK_TO_START_ON_REACHED_INTERACTION {
        if let Some(container) = target.container(&golem.get_entity().world.load()) {
            golem.open_chest(target.pos, container);
        }
        golem.set_state(state);
    }

    if ticks_since_reaching_target == TICK_TO_PLAY_ON_REACHED_SOUND {
        golem.play_entity_sound(sound, 1.0, 1.0);
    }

    if ticks_since_reaching_target == TARGET_INTERACTION_TIME {
        golem.close_opened_chest();
    }
}

fn on_travelling(mob: &dyn Mob) {
    if let Some(golem) = mob.cast_any().downcast_ref::<CopperGolemEntity>() {
        golem.close_opened_chest();
        golem.set_state(CopperGolemState::Idle);
    }
}

fn should_queue_for_target(target: &TransportItemTarget) -> bool {
    chests::is_chest_open(target.block_entity.as_ref())
}
