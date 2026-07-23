use std::sync::{Arc, Weak};
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Warden — simplified Java-like behaviour until full vibration brain exists.
///
/// - Aggressive melee (no pause-when-idle so they keep closing)
/// - Revenge + nearest player (no LOS required — "hearing")
/// - Each tick: if a nearby player is moving, lock onto them (crude hearing)
pub struct WardenEntity {
    pub mob_entity: MobEntity,
}

impl WardenEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        // Long follow range for "hearing" (vanilla anger is more complex).
        mob_entity
            .living_entity
            .set_attribute_base(&pumpkin_data::attributes::Attributes::FOLLOW_RANGE, 32.0);

        let warden = Self { mob_entity };
        let mob_arc = Arc::new(warden);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // pause_when_mob_idle=false — keep chasing even if path briefly fails.
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.2, false)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(0.5)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 16.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(1, Box::new(RevengeGoal::new(false)));
            // check_visibility=false — lock players without line of sight (hearing).
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, false),
            );
        };

        mob_arc
    }

    /// Crude "vibration": if a player within 16 blocks is moving, set as target.
    async fn hear_nearby_players(&self) {
        let living = &self.mob_entity.living_entity;
        if !living.is_alive() {
            return;
        }
        // Already has a living target.
        {
            let t = self.mob_entity.target.lock().await;
            if let Some(cur) = t.as_ref()
                && cur.get_living_entity().is_some_and(|l| l.is_alive())
            {
                return;
            }
        }

        let world = living.entity.world.load();
        let pos = living.entity.pos.load();
        let mut best: Option<(f64, Arc<dyn EntityBase>)> = None;
        for player in world.get_nearby_players(pos, 16.0) {
            if player.is_spectator() || player.is_creative() {
                continue;
            }
            let ppos = player.position();
            // Moving if velocity is non-trivial or recently moved (age-based always "hear" if close).
            // Presence within range is enough for this simplified hearing model
            // (full vibration queue TBD).
            let d = pos.squared_distance_to_vec(&ppos);
            if best.as_ref().is_none_or(|(bd, _)| d < *bd) {
                best = Some((d, player as Arc<dyn EntityBase>));
            }
        }
        if let Some((_, p)) = best {
            *self.mob_entity.target.lock().await = Some(p);
        }
    }
}

impl NBTStorage for WardenEntity {}

impl Mob for WardenEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            // Run hearing every few ticks.
            let age = self.mob_entity.living_entity.entity.age.load(Relaxed);
            if age % 5 == 0 {
                self.hear_nearby_players().await;
            }
        })
    }
}
