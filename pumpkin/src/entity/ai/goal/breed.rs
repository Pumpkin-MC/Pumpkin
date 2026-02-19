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
            if !candidate.is_in_love() || !candidate.is_breeding_ready() {
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

    async fn breed(mob: &dyn Mob, mate: &dyn EntityBase) {
        let mob_entity = mob.get_mob_entity();
        let entity = mob.get_entity();
        let world = entity.world.load();

        mob_entity.reset_love_ticks();
        mob_entity
            .breeding_cooldown
            .store(6000, std::sync::atomic::Ordering::Relaxed);

        mate.reset_love();
        mate.set_breeding_cooldown(6000);

        let parent_pos = entity.pos.load();
        let baby = from_type(entity.entity_type, parent_pos, &world, Uuid::new_v4()).await;
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

            if !mate.get_entity().is_alive() {
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
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
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

            let mut look_control = mob_entity.look_control.lock().await;
            look_control.look_at_entity(mob, mate);
            drop(look_control);

            let my_pos = mob.get_entity().pos.load();
            let dist_sq = my_pos.squared_distance_to_vec(&mate_pos);

            if dist_sq > 9.0 || self.timer % 10 == 0 {
                let mut navigator = mob_entity.navigator.lock().await;
                navigator.set_progress(NavigatorGoal::new(my_pos, mate_pos, self.speed));
            }

            self.timer += 1;

            if self.timer >= 60 && dist_sq < 9.0 {
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
