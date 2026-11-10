use pumpkin_data::block_properties::WhiteBedLikeProperties as BedProperties;
use pumpkin_data::entity::{EntityStatus, EntityType, MobCategory};
use pumpkin_data::environment_attribute::Activity;
use pumpkin_data::poi_type::PoiType;
use pumpkin_data::tag::{Block as BlockTag, Taggable};
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::position::BlockPos;

use super::{VillagerEntity, VillagerProfession};
use crate::entity::EntityBase;
use crate::entity::ai::brain::{
    Brain, BrainProvider, BrainTick, Schedule,
    activity::ActivityData,
    behavior::{
        BehaviorControl, DoNothing, GateBehavior, LookAtTargetSink, MoveToTargetSink, OneShot,
        OrderPolicy, RunningPolicy, Swim, Timed, Trigger,
        crisis::{
            CelebrateVillagersSurvivedRaid, VillagerPanicTrigger, create_locate_hiding_place,
            create_move_to_sky_seeing_spot, create_reset_raid_status, create_set_hidden_state,
            create_set_raid_status, create_villager_calm_down,
            entity as set_walk_target_away_from_entity,
        },
        door::create as create_interact_with_door,
        go_to_wanted_item,
        interact::{
            GiveGiftToHero, LookAndFollowTradingPlayerSink, ShowTradesToPlayer, TradeWithVillager,
            VillagerMakeLove, interact_with, play_tag_with_other_kids, set_entity_look_target,
            set_look_and_interact,
        },
        job_site::{
            GoToPotentialJobSite, create_acquire_poi, create_assign_profession_from_job_site,
            create_poi_competitor_scan, create_reset_profession, create_validate_nearby_poi,
            create_yield_job_site,
        },
        one_shot::{sequence, trigger_one_shuffled},
        poi_walk::{
            go_to_closest_village, inside_brownian_walk, set_closest_home_as_walk_target,
            set_walk_target_from_block_memory, stroll_around_poi, stroll_to_poi,
            stroll_to_poi_list, village_bound_random_stroll,
        },
        rest::{
            JumpOnBed, SleepInBed, create_react_to_bell, create_ring_bell,
            create_socialize_at_bell, create_wake_up,
        },
        run_one, run_one_with_conditions, set_walk_target_from_look_target,
        update_activity_from_schedule,
        work::{HarvestFarmland, UseBonemeal, WorkAtPoi, create_work_at_composter},
    },
    memory::{MemoryStatus, PackedMemories, types},
    sensing::SensorType,
};
use crate::entity::mob::Mob;
use crate::world::World;
use crate::world::raid::Raid;

const SPEED_MODIFIER: f32 = 0.5;
const STROLL_SPEED_MODIFIER: f32 = 0.4;

pub(super) fn make_brain(villager: &VillagerEntity, packed: &PackedMemories) -> Brain {
    let mut brain = BrainProvider {
        memory_types: &[],
        sensor_types: &[
            SensorType::NearestLivingEntities,
            SensorType::NearestPlayers,
            SensorType::NearestItems,
            SensorType::NearestBed,
            SensorType::HurtBy,
            SensorType::VillagerHostiles,
            SensorType::VillagerBabies,
            SensorType::SecondaryPois,
            SensorType::GolemDetected,
        ],
        activities,
    }
    .make_brain(villager, packed);

    brain.set_schedule(Some(if villager.get_mob_entity().living_entity.is_baby() {
        Schedule::BabyVillager
    } else {
        Schedule::Villager
    }));
    let world = villager.get_entity().world.load();
    let game_time = world
        .level_time
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .world_age;
    let block_pos = villager.get_entity().block_pos.load();
    brain.update_activity_from_schedule(&world, game_time, &block_pos);
    brain
}

fn activities(mob: &dyn Mob) -> Vec<ActivityData> {
    let Some(villager) = mob.cast_any().downcast_ref::<VillagerEntity>() else {
        return Vec::new();
    };
    let profession = villager
        .villager_data
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .profession_enum();
    let is_baby = villager.get_mob_entity().living_entity.is_baby();

    let mut activities = Vec::new();
    if is_baby {
        activities.push(ActivityData::with_pairs(
            Activity::Play,
            get_play_package(SPEED_MODIFIER),
        ));
    } else {
        activities.push(ActivityData::with_pairs_and_conditions(
            Activity::Work,
            get_work_package(profession, SPEED_MODIFIER),
            vec![(types::JOB_SITE.id(), MemoryStatus::ValuePresent)],
        ));
    }
    activities.push(ActivityData::with_pairs(
        Activity::Core,
        get_core_package(profession, SPEED_MODIFIER),
    ));
    activities.push(ActivityData::with_pairs_and_conditions(
        Activity::Meet,
        get_meet_package(SPEED_MODIFIER),
        vec![(types::MEETING_POINT.id(), MemoryStatus::ValuePresent)],
    ));
    activities.push(ActivityData::with_pairs(
        Activity::Rest,
        get_rest_package(SPEED_MODIFIER),
    ));
    activities.push(ActivityData::with_pairs(
        Activity::Idle,
        get_idle_package(SPEED_MODIFIER),
    ));
    activities.push(ActivityData::with_pairs(
        Activity::Panic,
        get_panic_package(SPEED_MODIFIER),
    ));
    activities.push(ActivityData::with_pairs(
        Activity::PreRaid,
        get_pre_raid_package(SPEED_MODIFIER),
    ));
    activities.push(ActivityData::with_pairs(
        Activity::Raid,
        get_raid_package(SPEED_MODIFIER),
    ));
    activities.push(ActivityData::with_pairs(
        Activity::Hide,
        get_hide_package(SPEED_MODIFIER),
    ));
    activities
}

type Package = Vec<(i32, Box<dyn BehaviorControl>)>;

fn get_core_package(profession: VillagerProfession, speed_modifier: f32) -> Package {
    let held_job_site = profession.held_job_site();
    let acquirable_job_site = profession.acquirable_job_site();
    vec![
        (0, Box::new(Timed::new(Swim::new(0.8)))),
        (0, Box::new(create_interact_with_door())),
        (0, Box::new(Timed::new(LookAtTargetSink::new(45, 90)))),
        (0, Box::new(Timed::new(VillagerPanicTrigger::new()))),
        (0, Box::new(create_wake_up())),
        (0, Box::new(create_react_to_bell())),
        (0, Box::new(create_set_raid_status())),
        (
            0,
            Box::new(create_validate_nearby_poi(
                move |poi_type| held_job_site.contains(&poi_type),
                types::JOB_SITE,
            )),
        ),
        (
            0,
            Box::new(create_validate_nearby_poi(
                move |poi_type| acquirable_job_site.contains(&poi_type),
                types::POTENTIAL_JOB_SITE,
            )),
        ),
        (1, Box::new(Timed::new(MoveToTargetSink::default()))),
        (2, Box::new(create_poi_competitor_scan())),
        (
            3,
            Box::new(Timed::new(LookAndFollowTradingPlayerSink::new(
                speed_modifier,
            ))),
        ),
        (
            5,
            Box::new(go_to_wanted_item::create(speed_modifier, false, 4)),
        ),
        (
            6,
            Box::new(create_acquire_poi(
                move |poi_type| acquirable_job_site.contains(&poi_type),
                types::JOB_SITE,
                types::POTENTIAL_JOB_SITE,
                true,
                None,
                |_, _| true,
            )),
        ),
        (
            7,
            Box::new(Timed::new(GoToPotentialJobSite::new(speed_modifier))),
        ),
        (8, Box::new(create_yield_job_site(speed_modifier))),
        (
            10,
            Box::new(create_acquire_poi(
                |poi_type| poi_type == PoiType::Home,
                types::HOME,
                types::HOME,
                false,
                Some(poi_acquisition_event),
                validate_bed_poi,
            )),
        ),
        (
            10,
            Box::new(create_acquire_poi(
                |poi_type| poi_type == PoiType::Meeting,
                types::MEETING_POINT,
                types::MEETING_POINT,
                true,
                Some(poi_acquisition_event),
                |_, _| true,
            )),
        ),
        (10, Box::new(create_assign_profession_from_job_site())),
        (10, Box::new(create_reset_profession())),
    ]
}

fn get_work_package(profession: VillagerProfession, speed_modifier: f32) -> Package {
    let work_at_poi = if profession == VillagerProfession::Farmer {
        Box::new(Timed::new(create_work_at_composter()))
    } else {
        Box::new(Timed::new(WorkAtPoi::new()))
    };
    let harvest_weight = if profession == VillagerProfession::Farmer {
        2
    } else {
        5
    };
    let bonemeal_weight = if profession == VillagerProfession::Farmer {
        4
    } else {
        7
    };
    vec![
        get_minimal_look_behavior(),
        (
            5,
            Box::new(run_one(vec![
                (work_at_poi, 7),
                (
                    Box::new(stroll_around_poi::create(
                        types::JOB_SITE,
                        STROLL_SPEED_MODIFIER,
                        4,
                    )),
                    2,
                ),
                (
                    Box::new(stroll_to_poi::create(
                        types::JOB_SITE,
                        STROLL_SPEED_MODIFIER,
                        1,
                        10,
                    )),
                    5,
                ),
                (
                    Box::new(stroll_to_poi_list::create(
                        types::SECONDARY_JOB_SITE,
                        speed_modifier,
                        1,
                        6,
                        types::JOB_SITE,
                    )),
                    5,
                ),
                (Box::new(Timed::new(HarvestFarmland::new())), harvest_weight),
                (Box::new(Timed::new(UseBonemeal::new())), bonemeal_weight),
            ])),
        ),
        (10, Box::new(Timed::new(ShowTradesToPlayer::new(400, 1600)))),
        (
            10,
            Box::new(set_look_and_interact::create(&EntityType::PLAYER, 4)),
        ),
        (
            2,
            Box::new(set_walk_target_from_block_memory::create(
                types::JOB_SITE,
                speed_modifier,
                9,
                100,
                1200,
            )),
        ),
        (3, Box::new(Timed::new(GiveGiftToHero::new(100)))),
        (99, Box::new(update_activity_from_schedule::create())),
    ]
}

fn get_play_package(speed_modifier: f32) -> Package {
    vec![
        (0, Box::new(Timed::new(MoveToTargetSink::new(80, 120)))),
        get_full_look_behavior(),
        (5, Box::new(play_tag_with_other_kids::create())),
        (
            5,
            Box::new(run_one_with_conditions(
                vec![(
                    types::VISIBLE_VILLAGER_BABIES.id(),
                    MemoryStatus::ValueAbsent,
                )],
                vec![
                    (
                        Box::new(interact_with::of(
                            &EntityType::VILLAGER,
                            8,
                            types::INTERACTION_TARGET,
                            speed_modifier,
                            2,
                        )),
                        2,
                    ),
                    (
                        Box::new(interact_with::of(
                            &EntityType::CAT,
                            8,
                            types::INTERACTION_TARGET,
                            speed_modifier,
                            2,
                        )),
                        1,
                    ),
                    (
                        Box::new(village_bound_random_stroll::create(speed_modifier)),
                        1,
                    ),
                    (
                        Box::new(set_walk_target_from_look_target::create(speed_modifier, 2)),
                        1,
                    ),
                    (Box::new(Timed::new(JumpOnBed::new(speed_modifier))), 2),
                    (Box::new(DoNothing::new(20, 40)), 2),
                ],
            )),
        ),
        (99, Box::new(update_activity_from_schedule::create())),
    ]
}

fn get_rest_package(speed_modifier: f32) -> Package {
    vec![
        (
            2,
            Box::new(set_walk_target_from_block_memory::create(
                types::HOME,
                speed_modifier,
                1,
                150,
                1200,
            )),
        ),
        (
            3,
            Box::new(create_validate_nearby_poi(
                |poi_type| poi_type == PoiType::Home,
                types::HOME,
            )),
        ),
        (3, Box::new(Timed::new(SleepInBed::new()))),
        (
            5,
            Box::new(run_one_with_conditions(
                vec![(types::HOME.id(), MemoryStatus::ValueAbsent)],
                vec![
                    (
                        Box::new(set_closest_home_as_walk_target::create(speed_modifier)),
                        1,
                    ),
                    (Box::new(inside_brownian_walk::create(speed_modifier)), 4),
                    (
                        Box::new(go_to_closest_village::create(speed_modifier, 4)),
                        2,
                    ),
                    (Box::new(DoNothing::new(20, 40)), 2),
                ],
            )),
        ),
        get_minimal_look_behavior(),
        (99, Box::new(update_activity_from_schedule::create())),
    ]
}

fn get_meet_package(speed_modifier: f32) -> Package {
    vec![
        (
            2,
            Box::new(trigger_one_shuffled(
                "TriggerOneShuffled",
                vec![
                    (
                        trigger_from(stroll_around_poi::create(
                            types::MEETING_POINT,
                            STROLL_SPEED_MODIFIER,
                            40,
                        )),
                        2,
                    ),
                    (trigger_from(create_socialize_at_bell()), 2),
                ],
            )),
        ),
        (10, Box::new(Timed::new(ShowTradesToPlayer::new(400, 1600)))),
        (
            10,
            Box::new(set_look_and_interact::create(&EntityType::PLAYER, 4)),
        ),
        (
            2,
            Box::new(set_walk_target_from_block_memory::create(
                types::MEETING_POINT,
                speed_modifier,
                6,
                100,
                200,
            )),
        ),
        (3, Box::new(Timed::new(GiveGiftToHero::new(100)))),
        (
            3,
            Box::new(create_validate_nearby_poi(
                |poi_type| poi_type == PoiType::Meeting,
                types::MEETING_POINT,
            )),
        ),
        (
            3,
            Box::new(GateBehavior::new(
                "GateBehavior",
                Vec::new(),
                vec![types::INTERACTION_TARGET.id()],
                OrderPolicy::Ordered,
                RunningPolicy::RunOne,
                vec![(Box::new(Timed::new(TradeWithVillager::new())), 1)],
            )),
        ),
        get_full_look_behavior(),
        (99, Box::new(update_activity_from_schedule::create())),
    ]
}

fn get_idle_package(speed_modifier: f32) -> Package {
    vec![
        (
            2,
            Box::new(run_one(vec![
                (
                    Box::new(interact_with::of(
                        &EntityType::VILLAGER,
                        8,
                        types::INTERACTION_TARGET,
                        speed_modifier,
                        2,
                    )),
                    2,
                ),
                (
                    Box::new(interact_with::of_filtered(
                        &EntityType::VILLAGER,
                        8,
                        can_breed_mob,
                        can_breed_entity,
                        types::BREED_TARGET,
                        speed_modifier,
                        2,
                    )),
                    1,
                ),
                (
                    Box::new(interact_with::of(
                        &EntityType::CAT,
                        8,
                        types::INTERACTION_TARGET,
                        speed_modifier,
                        2,
                    )),
                    1,
                ),
                (
                    Box::new(village_bound_random_stroll::create(speed_modifier)),
                    1,
                ),
                (
                    Box::new(set_walk_target_from_look_target::create(speed_modifier, 2)),
                    1,
                ),
                (Box::new(Timed::new(JumpOnBed::new(speed_modifier))), 1),
                (Box::new(DoNothing::new(30, 60)), 1),
            ])),
        ),
        (3, Box::new(Timed::new(GiveGiftToHero::new(100)))),
        (
            3,
            Box::new(set_look_and_interact::create(&EntityType::PLAYER, 4)),
        ),
        (3, Box::new(Timed::new(ShowTradesToPlayer::new(400, 1600)))),
        (
            3,
            Box::new(GateBehavior::new(
                "GateBehavior",
                Vec::new(),
                vec![types::INTERACTION_TARGET.id()],
                OrderPolicy::Ordered,
                RunningPolicy::RunOne,
                vec![(Box::new(Timed::new(TradeWithVillager::new())), 1)],
            )),
        ),
        (
            3,
            Box::new(GateBehavior::new(
                "GateBehavior",
                Vec::new(),
                vec![types::BREED_TARGET.id()],
                OrderPolicy::Ordered,
                RunningPolicy::RunOne,
                vec![(Box::new(Timed::new(VillagerMakeLove::new())), 1)],
            )),
        ),
        get_full_look_behavior(),
        (99, Box::new(update_activity_from_schedule::create())),
    ]
}

fn get_panic_package(speed_modifier: f32) -> Package {
    let runaway_speed = speed_modifier * 1.5;
    vec![
        (0, Box::new(create_villager_calm_down())),
        (
            1,
            Box::new(set_walk_target_away_from_entity(
                types::NEAREST_HOSTILE,
                runaway_speed,
                6,
                false,
            )),
        ),
        (
            1,
            Box::new(set_walk_target_away_from_entity(
                types::HURT_BY_ENTITY,
                runaway_speed,
                6,
                false,
            )),
        ),
        (
            3,
            Box::new(village_bound_random_stroll::create_with_distances(
                runaway_speed,
                2,
                2,
            )),
        ),
        get_minimal_look_behavior(),
    ]
}

fn get_pre_raid_package(speed_modifier: f32) -> Package {
    vec![
        (0, Box::new(create_ring_bell())),
        (
            0,
            Box::new(trigger_one_shuffled(
                "TriggerOneShuffled",
                vec![
                    (
                        trigger_from(set_walk_target_from_block_memory::create(
                            types::MEETING_POINT,
                            speed_modifier * 1.5,
                            2,
                            150,
                            200,
                        )),
                        6,
                    ),
                    (
                        trigger_from(village_bound_random_stroll::create(speed_modifier * 1.5)),
                        2,
                    ),
                ],
            )),
        ),
        get_minimal_look_behavior(),
        (99, Box::new(create_reset_raid_status())),
    ]
}

fn get_raid_package(speed_modifier: f32) -> Package {
    vec![
        (
            0,
            Box::new(sequence(
                "Sequence",
                |tick| raid_exists_and_not_victory(tick.world, tick),
                trigger_one_shuffled(
                    "TriggerOneShuffled",
                    vec![
                        (
                            trigger_from(create_move_to_sky_seeing_spot(speed_modifier)),
                            5,
                        ),
                        (
                            trigger_from(village_bound_random_stroll::create(speed_modifier * 1.1)),
                            2,
                        ),
                    ],
                ),
            )),
        ),
        (
            0,
            Box::new(Timed::new(CelebrateVillagersSurvivedRaid::new(600, 600))),
        ),
        (
            2,
            Box::new(sequence(
                "Sequence",
                |tick| raid_exists_and_active(tick.world, tick),
                create_locate_hiding_place(24, speed_modifier * 1.4, 1),
            )),
        ),
        get_minimal_look_behavior(),
        (99, Box::new(create_reset_raid_status())),
    ]
}

fn get_hide_package(speed_modifier: f32) -> Package {
    vec![
        (0, Box::new(create_set_hidden_state(15, 3))),
        (
            1,
            Box::new(create_locate_hiding_place(32, speed_modifier * 1.25, 2)),
        ),
        get_minimal_look_behavior(),
    ]
}

fn get_full_look_behavior() -> (i32, Box<dyn BehaviorControl>) {
    (
        5,
        Box::new(run_one(vec![
            (
                Box::new(set_entity_look_target::for_type(&EntityType::CAT, 8.0)),
                8,
            ),
            (
                Box::new(set_entity_look_target::for_type(&EntityType::VILLAGER, 8.0)),
                2,
            ),
            (
                Box::new(set_entity_look_target::for_type(&EntityType::PLAYER, 8.0)),
                2,
            ),
            (
                Box::new(set_entity_look_target::for_category(
                    &MobCategory::CREATURE,
                    8.0,
                )),
                1,
            ),
            (
                Box::new(set_entity_look_target::for_category(
                    &MobCategory::WATER_CREATURE,
                    8.0,
                )),
                1,
            ),
            (
                Box::new(set_entity_look_target::for_category(
                    &MobCategory::AXOLOTLS,
                    8.0,
                )),
                1,
            ),
            (
                Box::new(set_entity_look_target::for_category(
                    &MobCategory::UNDERGROUND_WATER_CREATURE,
                    8.0,
                )),
                1,
            ),
            (
                Box::new(set_entity_look_target::for_category(
                    &MobCategory::WATER_AMBIENT,
                    8.0,
                )),
                1,
            ),
            (
                Box::new(set_entity_look_target::for_category(
                    &MobCategory::MONSTER,
                    8.0,
                )),
                1,
            ),
            (Box::new(DoNothing::new(30, 60)), 2),
        ])),
    )
}

fn get_minimal_look_behavior() -> (i32, Box<dyn BehaviorControl>) {
    (
        5,
        Box::new(run_one(vec![
            (
                Box::new(set_entity_look_target::for_type(&EntityType::VILLAGER, 8.0)),
                2,
            ),
            (
                Box::new(set_entity_look_target::for_type(&EntityType::PLAYER, 8.0)),
                2,
            ),
            (Box::new(DoNothing::new(30, 60)), 8),
        ])),
    )
}

fn trigger_from(mut behavior: OneShot) -> Trigger {
    Box::new(move |tick| BehaviorControl::try_start(&mut behavior, tick))
}

fn can_breed_mob(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<VillagerEntity>()
        .is_some_and(VillagerEntity::can_breed)
}

fn can_breed_entity(entity: &dyn EntityBase) -> bool {
    entity
        .cast_any()
        .downcast_ref::<VillagerEntity>()
        .is_some_and(VillagerEntity::can_breed)
}

fn validate_bed_poi(world: &World, block_pos: BlockPos) -> bool {
    let (block, state) = world.get_block_and_state(&block_pos);
    block.has_tag(&BlockTag::MINECRAFT_VILLAGERS_CAN_SLEEP_ON_BED)
        && !BedProperties::from_state_id(state.id).occupied
}

const fn poi_acquisition_event() -> (EntityStatus, ActorEventID) {
    (EntityStatus::VillagerHappy, ActorEventID::VillagerHappy)
}

fn raid_exists_and_active(world: &World, tick: &BrainTick<'_>) -> bool {
    let pos = tick.mob.get_entity().block_pos.load();
    world
        .raids
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_raid_at(&pos)
        .is_some_and(|raid| raid.is_active() && !raid.is_victory() && !raid.is_loss())
}

fn raid_exists_and_not_victory(world: &World, tick: &BrainTick<'_>) -> bool {
    let pos = tick.mob.get_entity().block_pos.load();
    world
        .raids
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_raid_at(&pos)
        .is_some_and(Raid::is_victory)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering::Relaxed;

    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::brain::memory::GlobalPos;
    use crate::entity::ai::test_support::world;

    fn villager_brain(world: &Arc<World>, baby: bool) -> Brain {
        let entity = Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::VILLAGER,
        );
        entity.age.store(if baby { -1 } else { 0 }, Relaxed);
        let villager = VillagerEntity::new(entity);
        make_brain(&villager, &PackedMemories::empty())
    }

    #[tokio::test]
    async fn work_and_play_depend_on_the_villagers_age() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);

        let mut adult = villager_brain(&world, false);
        adult.set(
            types::JOB_SITE,
            GlobalPos::new(&Dimension::OVERWORLD, BlockPos::new(0, 64, 0)),
        );
        assert!(adult.activity_requirements_are_met(Activity::Work));
        assert!(!adult.activity_requirements_are_met(Activity::Play));

        let baby = villager_brain(&world, true);
        assert!(baby.activity_requirements_are_met(Activity::Play));
        assert!(!baby.activity_requirements_are_met(Activity::Work));

        world.level.shutdown().await;
        Ok(())
    }
}
