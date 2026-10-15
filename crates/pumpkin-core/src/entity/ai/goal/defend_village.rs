use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::boundingbox::BoundingBox;

use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;

/// Targets players that have a negative reputation with nearby villagers (<= -100).
/// Mirrors vanilla Minecraft's `DefendVillageTargetGoal`.
pub struct DefendVillageTargetGoal {
    track_target_goal: TrackTargetGoal,
    potential_target: Option<Arc<dyn EntityBase>>,
    target_predicate: TargetPredicate,
}

impl Default for DefendVillageTargetGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl DefendVillageTargetGoal {
    #[must_use]
    pub fn new() -> Self {
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = 64.0;
        Self {
            track_target_goal: TrackTargetGoal::with_default(false),
            potential_target: None,
            target_predicate,
        }
    }
}

impl Goal for DefendVillageTargetGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let world = entity.world.load();

        // Vanilla: AABB grow = this.golem.getBoundingBox().inflate(10.0, 8.0, 10.0);
        let search_box =
            BoundingBox::new(pos, pos.add_raw(1.0, 1.0, 1.0)).expand(10.0, 8.0, 10.0);
        let entities = world.get_entities_at_box(&search_box);

        let mut villagers = Vec::new();
        let mut players = Vec::new();

        for e in entities {
            let ent = e.get_entity();
            if ent.entity_type == &EntityType::VILLAGER {
                villagers.push(e);
            } else if ent.entity_type == &EntityType::PLAYER
                && let Some(player) = e.get_player()
                && !player.is_creative()
                && !player.is_spectator()
            {
                players.push(e);
            }
        }

        if villagers.is_empty() || players.is_empty() {
            return false;
        }

        self.potential_target = None;
        for villager_entity in &villagers {
            let Some(mob_ref) = villager_entity.get_mob() else {
                continue;
            };
            let Some(villager) = mob_ref.as_villager() else {
                continue;
            };
            for player_base in &players {
                if let Some(player) = player_base.get_player() {
                    let rep = villager.get_player_reputation(&player.gameprofile.id);
                    if rep <= -100 {
                        self.potential_target = Some(player_base.clone());
                        break;
                    }
                }
            }
            if self.potential_target.is_some() {
                break;
            }
        }

        let Some(target) = self.potential_target.as_ref() else {
            return false;
        };

        self.track_target_goal
            .can_track(mob, Some(target.as_ref()), &self.target_predicate)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.track_target_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.set_mob_target(self.potential_target.clone());
        self.track_target_goal
            .set_target_mob(self.potential_target.clone());
        self.track_target_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.potential_target = None;
        self.track_target_goal.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}
