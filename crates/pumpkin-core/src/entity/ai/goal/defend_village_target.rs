use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use super::track_target::TrackTargetGoal;
use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::passive::villager::VillagerEntity;
use crate::plugin::api::events::entity::entity_target::TargetReason;

const ATTACK_TARGETING_RANGE: f64 = 64.0;
const SCAN_RADIUS_XZ: f64 = 10.0;
const SCAN_RADIUS_Y: f64 = 8.0;
const REPUTATION_TO_DEFEND: i32 = -100;

pub struct DefendVillageTargetGoal {
    track_target_goal: TrackTargetGoal,
    potential_target: Option<Weak<dyn EntityBase>>,
    attack_targeting: TargetPredicate,
}

impl DefendVillageTargetGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            track_target_goal: TrackTargetGoal::new(false, true),
            potential_target: None,
            attack_targeting: TargetPredicate::create_attackable()
                .set_base_max_distance(ATTACK_TARGETING_RANGE),
        })
    }
}

impl Goal for DefendVillageTargetGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load_full();
        let area = entity
            .bounding_box
            .load()
            .expand(SCAN_RADIUS_XZ, SCAN_RADIUS_Y, SCAN_RADIUS_XZ);
        let villagers = world.get_entities_at_box(&area);
        let players = world.get_players_at_box(&area);

        for villager in villagers
            .iter()
            .filter(|villager| villager.get_entity().entity_type == &EntityType::VILLAGER)
            .filter(|villager| {
                self.attack_targeting
                    .test(&world, Some(mob), villager.as_ref())
            })
            .filter_map(|villager| {
                villager
                    .get_mob()
                    .and_then(|villager| villager.cast_any().downcast_ref::<VillagerEntity>())
            })
        {
            for player in players.iter().filter(|player| {
                self.attack_targeting
                    .test(&world, Some(mob), player.as_ref())
            }) {
                if villager.get_player_reputation(player) <= REPUTATION_TO_DEFEND {
                    let player_base: Arc<dyn EntityBase> = player.clone();
                    self.potential_target = Some(Arc::downgrade(&player_base));
                }
            }
        }

        self.potential_target
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some_and(|target| {
                target
                    .get_player()
                    .is_none_or(|player| !player.is_spectator() && !player.is_creative())
            })
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.track_target_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.track_target_goal.start_targeting(
            mob,
            self.potential_target.as_ref().and_then(Weak::upgrade),
            TargetReason::Unknown,
        );
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
