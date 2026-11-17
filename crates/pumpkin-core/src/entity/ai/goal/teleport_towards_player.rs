use std::sync::{Arc, Weak};

use super::track_target::TrackTargetGoal;
use super::{Controls, Goal, to_goal_ticks};
use crate::entity::EntityBase;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::mob::enderman::EndermanEntity;
use crate::entity::mob::neutral::NeutralMob;
use crate::entity::player::Player;
use crate::world::World;
use pumpkin_data::attributes::Attributes;

const STARE_CLOSE_DISTANCE_SQ: f64 = 16.0;
const TELEPORT_FAR_DISTANCE_SQ: f64 = 256.0;

pub struct TeleportTowardsPlayerGoal {
    enderman: Weak<EndermanEntity>,
    track_target_goal: TrackTargetGoal,
    target_player: Option<Weak<dyn EntityBase>>,
    committed_target: Option<Weak<dyn EntityBase>>,
    target_predicate: TargetPredicate,
    continue_predicate: TargetPredicate,
    warmup: i32,
    unseen_ticks: i32,
}

impl TeleportTowardsPlayerGoal {
    pub fn new(enderman: &Arc<EndermanEntity>) -> Self {
        let track_target_goal = TrackTargetGoal::with_default(false);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = enderman
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);
        Self {
            enderman: Arc::downgrade(enderman),
            track_target_goal,
            target_player: None,
            committed_target: None,
            target_predicate,
            continue_predicate: TargetPredicate::create_attackable().ignore_visibility(),
            warmup: 0,
            unseen_ticks: 0,
        }
    }

    /// Stared at or held against by the grudge, and not riding this enderman.
    fn is_anger_inducing(enderman: &EndermanEntity, player: &Player, world: &World) -> bool {
        (enderman.is_player_staring(player) || enderman.is_angry_at(player, world))
            && !Self::has_indirect_passenger(enderman, player)
    }

    fn has_indirect_passenger(enderman: &EndermanEntity, target: &dyn EntityBase) -> bool {
        let enderman_id = enderman.mob_entity.living_entity.entity.entity_id;
        let mut vehicle = target.get_entity().get_vehicle();
        while let Some(current) = vehicle {
            if current.get_entity().entity_id == enderman_id {
                return true;
            }
            vehicle = current.get_entity().get_vehicle();
        }
        false
    }

    fn find_staring_player(&self, enderman: &EndermanEntity) -> Option<Arc<Player>> {
        let entity = &enderman.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();

        // A nearer player that is not anger-inducing must not hide a farther one that is.
        // The conditions carry the follow range.
        let enderman_base: &dyn EntityBase = enderman;
        world.get_nearest_player(pos, f64::INFINITY, |player| {
            self.target_predicate
                .test(&world, Some(enderman_base), player.as_ref())
                && Self::is_anger_inducing(enderman, player, &world)
        })
    }
}

impl Goal for TeleportTowardsPlayerGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(enderman) = self.enderman.upgrade() else {
            return false;
        };
        let Some(player) = self.find_staring_player(&enderman) else {
            return false;
        };
        let player: Arc<dyn EntityBase> = player;
        self.target_player = Some(Arc::downgrade(&player));
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(enderman) = self.enderman.upgrade() else {
            return false;
        };
        let world = enderman.mob_entity.living_entity.entity.world.load();
        if let Some(pending) = &self.target_player {
            let Some(pending) = pending.upgrade() else {
                return false;
            };
            let Some(player) = pending.get_player() else {
                return false;
            };
            if !Self::is_anger_inducing(&enderman, player, &world) {
                return false;
            }
            mob.get_mob_entity()
                .look_at_entity(pending.as_ref(), 10.0, 10.0);
            return true;
        }
        if let Some(target) = self.committed_target.as_ref().and_then(Weak::upgrade) {
            if Self::has_indirect_passenger(&enderman, target.as_ref()) {
                return false;
            }
            let enderman_base: &dyn EntityBase = enderman.as_ref();
            if self
                .continue_predicate
                .test(&world, Some(enderman_base), target.as_ref())
            {
                return true;
            }
        }
        self.track_target_goal.should_continue(mob)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.warmup = to_goal_ticks(5);
        self.unseen_ticks = 0;
        if let Some(enderman) = self.enderman.upgrade() {
            enderman.set_provoked(true);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(enderman) = self.enderman.upgrade() else {
            return;
        };
        if mob.get_mob_entity().get_target().is_none() {
            self.committed_target = None;
        }

        if self.target_player.is_some() {
            self.warmup -= 1;
            if self.warmup <= 0 {
                let target = self.target_player.take();
                self.committed_target.clone_from(&target);
                enderman.set_target(target.as_ref().and_then(Weak::upgrade));
                self.track_target_goal.start(mob);
            }
            return;
        }

        let entity = &mob.get_mob_entity().living_entity.entity;
        let Some(target) = self.committed_target.as_ref().and_then(Weak::upgrade) else {
            return;
        };
        if entity.has_vehicle() {
            return;
        }
        let dist_sq = entity
            .pos
            .load()
            .squared_distance_to_vec(&target.get_entity().pos.load());

        if let Some(player) = target.get_player()
            && enderman.is_player_staring(player)
        {
            if dist_sq < STARE_CLOSE_DISTANCE_SQ {
                enderman.teleport_randomly();
            }
            self.unseen_ticks = 0;
        } else if dist_sq > TELEPORT_FAR_DISTANCE_SQ {
            let waited = self.unseen_ticks;
            self.unseen_ticks += 1;
            if waited >= to_goal_ticks(30) && enderman.teleport_towards(target.as_ref()) {
                self.unseen_ticks = 0;
            }
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target_player = None;
        self.committed_target = None;
        if let Some(enderman) = self.enderman.upgrade() {
            enderman.set_target(None);
        }
    }

    fn controls(&self) -> Controls {
        Controls::TARGET
    }
}
