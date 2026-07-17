use pumpkin_util::math::position::BlockPos;

use super::{Controls, Goal, GoalFuture};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};

pub struct WorkAtJobSiteGoal {
    speed: f64,
    target: Option<BlockPos>,
}

impl WorkAtJobSiteGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            speed,
            target: None,
        }
    }

    async fn should_move_to_job_site(mob: &dyn Mob) -> bool {
        if mob.is_job_site_pending().await {
            return true;
        }
        let world = mob.get_mob_entity().living_entity.entity.world.load();
        (2_000..9_000).contains(&world.level_time.lock().await.query_daytime())
    }
}

impl Goal for WorkAtJobSiteGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(target) = mob.get_job_site() else {
                return false;
            };
            if !Self::should_move_to_job_site(mob).await {
                return false;
            }
            let position = mob.get_mob_entity().living_entity.entity.pos.load();
            if target.to_centered_f64().squared_distance_to_vec(&position) < 1.73f64.powi(2) {
                return false;
            }
            self.target = Some(target);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(target) = self.target else {
                return false;
            };
            if mob.get_job_site() != Some(target) || !Self::should_move_to_job_site(mob).await {
                return false;
            }
            let entity = &mob.get_mob_entity().living_entity.entity;
            target
                .to_centered_f64()
                .squared_distance_to_vec(&entity.pos.load())
                >= 1.73f64.powi(2)
                && !mob.get_mob_entity().navigator.lock().unwrap().is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target {
                let entity = &mob.get_mob_entity().living_entity.entity;
                mob.get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap()
                    .set_progress(NavigatorGoal::new(
                        entity.pos.load(),
                        target.to_centered_f64(),
                        self.speed,
                    ));
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target
                && target
                    .to_centered_f64()
                    .squared_distance_to_vec(&mob.get_mob_entity().living_entity.entity.pos.load())
                    >= 2.0f64.powi(2)
                && mob.is_job_site_pending().await
            {
                mob.release_pending_job_site(target).await;
            }
            self.target = None;
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
