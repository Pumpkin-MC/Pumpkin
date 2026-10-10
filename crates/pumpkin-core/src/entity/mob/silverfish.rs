use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::BlockDirection;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::block::blocks::infested::InfestedBlock;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        Controls, Goal, active_target::ActiveTargetGoal,
        climb_on_top_of_powder_snow::ClimbOnTopOfPowderSnowGoal, melee_attack::MeleeAttackGoal,
        revenge::RevengeGoal, swim::SwimGoal, to_goal_ticks, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const Y_OFFSETS: [i32; 11] = [0, 1, -1, 2, -2, 3, -3, 4, -4, 5, -5];
const XZ_OFFSETS: [i32; 21] = [
    0, 1, -1, 2, -2, 3, -3, 4, -4, 5, -5, 6, -6, 7, -7, 8, -8, 9, -9, 10, -10,
];

pub struct SilverfishEntity {
    pub entity: Arc<MobEntity>,
    wake_up_friends_ticks: AtomicI32,
}

impl SilverfishEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = Arc::new(MobEntity::new(entity));
        let silverfish = Self {
            entity,
            wake_up_friends_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(silverfish);
        let mob_weak = Arc::downgrade(&mob_arc);

        {
            let mut goal_selector = mob_arc
                .entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut target_selector = mob_arc
                .entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(ClimbOnTopOfPowderSnowGoal));
            goal_selector.add_goal(
                3,
                Box::new(SilverfishWakeUpFriendsGoal {
                    silverfish: mob_weak,
                }),
            );
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(5, Box::new(SilverfishMergeWithStoneGoal::new()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true).alerting_others()));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

struct SilverfishMergeWithStoneGoal {
    stroll: WanderAroundGoal,
    selected_direction: Option<BlockDirection>,
    do_merge: bool,
}

impl SilverfishMergeWithStoneGoal {
    const fn new() -> Self {
        Self {
            stroll: WanderAroundGoal::with_interval(1.0, 10),
            selected_direction: None,
            do_merge: false,
        }
    }

    fn merge_pos(mob: &dyn Mob, direction: BlockDirection) -> BlockPos {
        let pos = mob.get_entity().pos.load();
        BlockPos::containing(pos.x, pos.y + 0.5, pos.z).offset(direction.to_offset())
    }
}

impl Goal for SilverfishMergeWithStoneGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.get_mob_entity().get_target().is_some() || !mob.is_navigator_idle() {
            return false;
        }

        let world = mob.get_entity().world.load();
        let mut random = mob.get_random();
        if world.level_info.load().game_rules.mob_griefing
            && random.random_range(0..to_goal_ticks(10)) == 0
        {
            let directions = BlockDirection::all();
            let direction = directions[random.random_range(0..directions.len())];
            let pos = Self::merge_pos(mob, direction);
            if InfestedBlock::is_compatible_host_block(world.get_block_state(&pos)) {
                self.selected_direction = Some(direction);
                self.do_merge = true;
                return true;
            }
        }

        self.do_merge = false;
        self.stroll.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !self.do_merge && self.stroll.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if !self.do_merge {
            self.stroll.start(mob);
            return;
        }

        let Some(direction) = self.selected_direction else {
            return;
        };
        let world = mob.get_entity().world.load();
        let pos = Self::merge_pos(mob, direction);
        let state = world.get_block_state(&pos);
        if let Some(infested) = InfestedBlock::infested_state_by_host(state) {
            let host_state = state.id;
            let written =
                world.set_block_state_if(&pos, infested, BlockFlags::NOTIFY_ALL, |current| {
                    current == host_state
                });
            if written.is_some() {
                world.send_entity_status(mob.get_entity(), EntityStatus::Poof, None);
                mob.get_entity().remove();
            }
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.stroll.stop(mob);
        self.selected_direction = None;
        self.do_merge = false;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.stroll.tick(mob);
    }

    fn controls(&self) -> Controls {
        self.stroll.controls()
    }
}

struct SilverfishWakeUpFriendsGoal {
    silverfish: Weak<SilverfishEntity>,
}

impl Goal for SilverfishWakeUpFriendsGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.silverfish
            .upgrade()
            .is_some_and(|silverfish| silverfish.wake_up_friends_ticks.load(Ordering::Relaxed) > 0)
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(silverfish) = self.silverfish.upgrade() else {
            return;
        };

        let remaining = silverfish.wake_up_friends_ticks.load(Ordering::Relaxed) - 1;
        silverfish
            .wake_up_friends_ticks
            .store(remaining, Ordering::Relaxed);
        if remaining > 0 {
            return;
        }

        let world = mob.get_entity().world.load();
        let base_pos = mob.get_entity().block_pos.load();
        let mob_griefing = world.level_info.load().game_rules.mob_griefing;

        for y_off in Y_OFFSETS {
            for x_off in XZ_OFFSETS {
                for z_off in XZ_OFFSETS {
                    let pos = base_pos.offset(Vector3::new(x_off, y_off, z_off));
                    let Some(host_state) =
                        InfestedBlock::host_state_by_infested(world.get_block_state(&pos))
                    else {
                        continue;
                    };

                    if mob_griefing {
                        if world
                            .break_block(&pos, None, BlockFlags::NOTIFY_ALL)
                            .is_some()
                        {
                            InfestedBlock::spawn_infestation(&world, &pos);
                        }
                    } else {
                        world.set_block_state(&pos, host_state, BlockFlags::NOTIFY_ALL);
                    }

                    if mob.get_random().random_bool(0.5) {
                        return;
                    }
                }
            }
        }
    }
}

impl Mob for SilverfishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let yaw = entity.yaw.load();
        entity.body_yaw.store(yaw);
        entity.head_yaw.store(yaw);
    }

    fn pre_damage(&self, damage_type: DamageType, source: Option<&dyn EntityBase>) -> bool {
        if !self.get_entity().is_invulnerable_to(&damage_type, source)
            && self.wake_up_friends_ticks.load(Ordering::Relaxed) == 0
            && (source.is_some()
                || damage_type.has_tag(&tag::DamageType::MINECRAFT_ALWAYS_TRIGGERS_SILVERFISH))
        {
            self.wake_up_friends_ticks
                .store(to_goal_ticks(20), Ordering::Relaxed);
        }

        true
    }
}
