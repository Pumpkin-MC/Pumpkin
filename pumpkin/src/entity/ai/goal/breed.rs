use std::sync::Arc;

use uuid::Uuid;

use crate::entity::{EntityBase, ai::pathfinder::NavigatorGoal, mob::Mob, r#type::from_type};

use super::{Controls, Goal, GoalFuture};

pub struct BreedGoal {
    speed: f64,
    mate: Option<Arc<dyn EntityBase>>,
    timer: i32,
}

impl BreedGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            speed,
            mate: None,
            timer: 0,
        })
    }

    fn find_mate(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let mob_entity = mob.get_mob_entity();
        if !mob_entity.is_in_love() {
            return None;
        }

        let entity = mob.get_entity();
        let pos = entity.pos.load();
        let world = entity.world.load();
        let my_type = entity.entity_type;
        let my_uuid = entity.entity_uuid;

        let nearby = world.get_nearby_entities(pos, 8.0);
        let mut closest: Option<(f64, Arc<dyn EntityBase>)> = None;

        for candidate in nearby.values() {
            let c_entity = candidate.get_entity();
            if c_entity.entity_uuid == my_uuid {
                continue;
            }
            if c_entity.entity_type != my_type {
                continue;
            }
            if !candidate.is_in_love() || !candidate.is_breeding_ready() || candidate.is_panicking()
            {
                continue;
            }

            let dist = pos.squared_distance_to_vec(&c_entity.pos.load());
            match &closest {
                Some((best_dist, _)) if dist >= *best_dist => {}
                _ => closest = Some((dist, candidate.clone())),
            }
        }

        closest.map(|(_, e)| e)
    }

    /// Which of the two partners is the one that spawns the child.
    ///
    /// Both partners run their own [`BreedGoal`], find each other as a mate, and
    /// count their timers up in lockstep, so both reach the point of breeding on
    /// the same tick and each would spawn a child of its own. Having one side
    /// re-read the other's in-love state is not enough to stop that: the world
    /// ticks entities concurrently on a `JoinSet`, so both sides can observe the
    /// not-yet-bred state before either of them writes.
    ///
    /// Deciding it from the entity ids instead needs no shared state and cannot
    /// race, because both sides compute the same answer from the same two ids.
    #[must_use]
    pub const fn spawns_the_child(entity_id: i32, mate_entity_id: i32) -> bool {
        entity_id < mate_entity_id
    }

    async fn breed(mob: &dyn Mob, mate: &dyn EntityBase) {
        let mob_entity = mob.get_mob_entity();

        // `tick` keeps running on the ticks in between goal re-evaluations, so
        // this can be reached again after the pair has already bred. Breeding
        // consumes the in-love state and puts both parents on cooldown, so once
        // that is gone there is nothing left to breed.
        if !mob_entity.is_in_love() || !mob_entity.is_breeding_ready() {
            return;
        }

        let entity = mob.get_entity();
        let world = entity.world.load();

        if let Some(player) = mob_entity
            .breeder
            .load()
            .and_then(|uuid| world.get_player_by_uuid(uuid))
        {
            player
                .increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::AnimalsBred as i32,
                    1,
                )
                .await;

            player
                .trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::BredAnimal {
                        parent_type: format!("minecraft:{}", entity.entity_type.resource_name),
                    },
                )
                .await;
        }

        mob_entity.reset_love_ticks();
        mob_entity
            .breeding_cooldown
            .store(6000, std::sync::atomic::Ordering::Relaxed);

        mate.reset_love();
        mate.set_breeding_cooldown(6000);

        let parent_pos = entity.pos.load();
        let baby = from_type(entity.entity_type, parent_pos, &world, Uuid::new_v4());
        baby.get_entity().set_age(-24000);
        world.spawn_entity(baby).await;
    }
}

impl Goal for BreedGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let mob_entity = mob.get_mob_entity();
            if !mob_entity.is_breeding_ready() || !mob_entity.is_in_love() {
                return false;
            }

            self.mate = Self::find_mate(mob);
            self.mate.is_some()
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let Some(mate) = &self.mate else {
                return false;
            };

            if !mate.get_entity().is_alive() || mate.is_panicking() {
                return false;
            }

            mate.is_in_love() && self.timer < 60
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.timer = 0;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.mate = None;
            self.timer = 0;
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let Some(mate) = &self.mate else {
                return;
            };

            let mob_entity = mob.get_mob_entity();
            let mate_pos = mate.get_entity().pos.load();

            {
                let mut look_control = mob_entity.look_control.lock().unwrap();
                look_control.look_at_entity(mob, mate);
            };

            let my_pos = mob.get_entity().pos.load();
            let dist_sq = my_pos.squared_distance_to_vec(&mate_pos);

            {
                let mut navigator = mob_entity.navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(my_pos, mate_pos, self.speed));
            };

            self.timer += 1;

            if self.timer >= 60
                && dist_sq < 9.0
                && Self::spawns_the_child(mob.get_entity().entity_id, mate.get_entity().entity_id)
            {
                Self::breed(mob, mate.as_ref()).await;
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

#[cfg(test)]
mod tests {
    use super::BreedGoal;

    #[test]
    fn exactly_one_partner_spawns_the_child() {
        // The property that matters: for a pair, asking from both sides must
        // yield exactly one "yes". Two yeses is the reported bug (two babies);
        // two noes would mean breeding silently never produces anything.
        let pairs = [(1, 2), (2, 1), (7, 400), (400, 7), (-5, 3), (3, -5), (0, 1)];
        for (a, b) in pairs {
            assert_ne!(
                BreedGoal::spawns_the_child(a, b),
                BreedGoal::spawns_the_child(b, a),
                "ids {a} and {b} did not agree on a single parent"
            );
        }
    }

    #[test]
    fn the_lower_entity_id_is_the_one_that_spawns() {
        assert!(BreedGoal::spawns_the_child(1, 2));
        assert!(!BreedGoal::spawns_the_child(2, 1));
    }

    #[test]
    fn a_mob_is_never_its_own_partner() {
        // `find_mate` skips candidates sharing our uuid, so equal ids should not
        // reach this. If one ever did, declining is the safe direction: it costs
        // a missed child rather than spawning one from a single parent.
        assert!(!BreedGoal::spawns_the_child(42, 42));
    }

    #[test]
    fn negative_entity_ids_still_pick_one_side() {
        // Entity ids are i32 and Bedrock paths can hand out negatives, so the
        // comparison must not depend on them being positive.
        assert!(BreedGoal::spawns_the_child(i32::MIN, i32::MAX));
        assert!(!BreedGoal::spawns_the_child(i32::MAX, i32::MIN));
    }
}
