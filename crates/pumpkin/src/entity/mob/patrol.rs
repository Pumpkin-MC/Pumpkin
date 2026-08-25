use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::goal::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::mob::raider::create_ominous_banner;

pub struct PatrolData {
    pub patrol_target: AtomicCell<Option<BlockPos>>,
    pub patrol_leader: AtomicBool,
    pub patrolling: AtomicBool,
}

impl Default for PatrolData {
    fn default() -> Self {
        Self {
            patrol_target: AtomicCell::new(None),
            patrol_leader: AtomicBool::new(false),
            patrolling: AtomicBool::new(false),
        }
    }
}

pub trait PatrollingMonster: Mob {
    fn get_patrol_data(&self) -> &PatrolData;

    fn can_be_leader(&self) -> bool {
        true
    }

    fn is_patrol_leader(&self) -> bool {
        self.get_patrol_data().patrol_leader.load(Ordering::Relaxed)
    }

    fn set_patrol_leader(&self, is_leader: bool) {
        self.get_patrol_data()
            .patrol_leader
            .store(is_leader, Ordering::Relaxed);
        self.set_patrolling(true);
    }

    fn is_patrolling(&self) -> bool {
        self.get_patrol_data().patrolling.load(Ordering::Relaxed)
    }

    fn set_patrolling(&self, patrolling: bool) {
        self.get_patrol_data()
            .patrolling
            .store(patrolling, Ordering::Relaxed);
    }

    fn get_patrol_target(&self) -> Option<BlockPos> {
        self.get_patrol_data().patrol_target.load()
    }

    fn set_patrol_target(&self, target: BlockPos) {
        self.get_patrol_data().patrol_target.store(Some(target));
        self.set_patrolling(true);
    }

    fn has_patrol_target(&self) -> bool {
        self.get_patrol_data().patrol_target.load().is_some()
    }

    fn can_join_patrol(&self) -> bool {
        true
    }

    fn find_patrol_target(&self) {
        let block_pos = self.get_mob_entity().living_entity.entity.block_pos.load();
        let dx: i32 = rand::random::<i32>().rem_euclid(1000) - 500;
        let dz: i32 = rand::random::<i32>().rem_euclid(1000) - 500;
        let target = BlockPos(Vector3::new(
            block_pos.0.x + dx,
            block_pos.0.y,
            block_pos.0.z + dz,
        ));
        self.set_patrol_target(target);
    }

    fn finalize_patrol_spawn(&self, is_patrol_spawn: bool) {
        if !is_patrol_spawn && self.can_be_leader() {
            let r: f32 = rand::random();
            if r < 0.06 {
                self.set_patrol_leader(true);
            }
        }

        if self.is_patrol_leader() {
            let banner = create_ominous_banner();
            let living = &self.get_mob_entity().living_entity;
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let mut equipment = living.entity_equipment.lock().await;
                    equipment.put(&EquipmentSlot::HEAD, banner.clone());
                    drop(equipment);
                    living.send_equipment_changes(&[(EquipmentSlot::HEAD, banner)]);
                });
            });
        }

        if is_patrol_spawn {
            self.set_patrolling(true);
        }
    }

    fn write_patrol_nbt(&self, nbt: &mut NbtCompound) {
        let data = self.get_patrol_data();
        if let Some(target) = data.patrol_target.load() {
            nbt.put_int("PatrolTargetX", target.0.x);
            nbt.put_int("PatrolTargetY", target.0.y);
            nbt.put_int("PatrolTargetZ", target.0.z);
        }
        nbt.put_bool("PatrolLeader", data.patrol_leader.load(Ordering::Relaxed));
        nbt.put_bool("Patrolling", data.patrolling.load(Ordering::Relaxed));
    }

    fn read_patrol_nbt(&self, nbt: &NbtCompound) {
        let data = self.get_patrol_data();
        if let (Some(x), Some(y), Some(z)) = (
            nbt.get_int("PatrolTargetX"),
            nbt.get_int("PatrolTargetY"),
            nbt.get_int("PatrolTargetZ"),
        ) {
            data.patrol_target
                .store(Some(BlockPos(Vector3::new(x, y, z))));
        }
        if let Some(leader) = nbt.get_bool("PatrolLeader") {
            data.patrol_leader.store(leader, Ordering::Relaxed);
        }
        if let Some(patrolling) = nbt.get_bool("Patrolling") {
            data.patrolling.store(patrolling, Ordering::Relaxed);
        }
    }
}

pub struct LongDistancePatrolGoal {
    speed_modifier: f64,
    leader_speed_modifier: f64,
    cooldown_until: i64,
}

impl LongDistancePatrolGoal {
    #[must_use]
    pub const fn new(speed_modifier: f64, leader_speed_modifier: f64) -> Self {
        Self {
            speed_modifier,
            leader_speed_modifier,
            cooldown_until: -1,
        }
    }
}

impl Goal for LongDistancePatrolGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(patrol) = mob.as_patrolling_monster() else {
                return false;
            };
            let world = mob.get_entity().world.load();
            let game_time = world.level_time.lock().await.query_daytime();
            let is_on_cooldown = game_time < self.cooldown_until;

            let target = mob.get_mob_entity().target.lock().await.clone();
            patrol.is_patrolling()
                && target.is_none()
                && patrol.has_patrol_target()
                && !is_on_cooldown
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(patrol) = mob.as_patrolling_monster() else {
                return false;
            };
            let target = mob.get_mob_entity().target.lock().await.clone();
            patrol.is_patrolling() && target.is_none() && patrol.has_patrol_target()
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(patrol) = mob.as_patrolling_monster() else {
                return;
            };
            let is_leader = patrol.is_patrol_leader();
            let entity = mob.get_entity();
            let pos = entity.pos.load();

            let Some(patrol_target) = patrol.get_patrol_target() else {
                return;
            };

            let dist_sq = pos.squared_distance_to_vec(&patrol_target.to_f64());
            if is_leader && dist_sq < 100.0 {
                patrol.find_patrol_target();
            } else {
                let speed = if is_leader {
                    self.leader_speed_modifier
                } else {
                    self.speed_modifier
                };
                let mut nav = mob
                    .get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                nav.set_progress(NavigatorGoal {
                    current_progress: pos,
                    destination: patrol_target.to_f64(),
                    speed,
                });
            }
        })
    }
}
