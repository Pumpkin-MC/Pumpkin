use std::sync::Arc;

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use rand::RngExt;

use super::{Controls, Goal, GoalFuture};
use crate::entity::{EntityBase, mob::Mob, player::Player};

/// `Dolphin.DolphinSwimWithPlayerGoal`: finds the nearest swimming player within 10 blocks
/// (ignoring line of sight) and applies `Dolphin's Grace` to them on start and roughly every
/// 6th tick while still nearby.
const RANGE: f64 = 10.0;
const GRACE_DURATION_TICKS: i32 = 100;

pub struct DolphinSwimWithPlayerGoal {
    speed: f64,
    target: Option<Arc<Player>>,
}

impl DolphinSwimWithPlayerGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            speed,
            target: None,
        })
    }

    fn find_swimming_player(mob: &dyn Mob) -> Option<Arc<Player>> {
        let entity = mob.get_entity();
        let pos = entity.pos.load();
        let world = entity.world.load();

        world.get_nearby_players(pos, RANGE).into_iter().find(|p| {
            p.get_entity()
                .touching_water
                .load(std::sync::atomic::Ordering::Relaxed)
        })
    }

    async fn grant_grace(player: &Player) {
        player
            .living_entity
            .add_effect(Effect {
                effect_type: &StatusEffect::DOLPHINS_GRACE,
                duration: GRACE_DURATION_TICKS,
                amplifier: 0,
                ambient: true,
                show_particles: true,
                show_icon: true,
                blend: false,
            })
            .await;
    }
}

impl Goal for DolphinSwimWithPlayerGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.target = Self::find_swimming_player(mob);
            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(target) = &self.target else {
                return false;
            };
            if !target.get_entity().is_alive() {
                return false;
            }
            let pos = mob.get_entity().pos.load();
            pos.squared_distance_to_vec(&target.get_entity().pos.load()) <= RANGE * RANGE
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = &self.target {
                Self::grant_grace(target).await;
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(target) = self.target.clone() else {
                return;
            };

            let target_entity_base: Arc<dyn EntityBase> = target.clone();
            {
                let mut look_control = mob.get_mob_entity().look_control.lock().unwrap();
                look_control.look_at_entity(mob, &target_entity_base);
            };

            let my_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            {
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(crate::entity::ai::pathfinder::NavigatorGoal::new(
                    my_pos, target_pos, self.speed,
                ));
            };

            // Vanilla: `random.nextInt(6) == 0` while continuing.
            if mob.get_random().random_range(0..6) == 0 {
                Self::grant_grace(&target).await;
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
