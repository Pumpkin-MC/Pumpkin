use std::sync::Arc;

use uuid::Uuid;

use pumpkin_data::entity::EntityStatus;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use rand::RngExt;

use crate::entity::ai::target_match::{SearchArea, TargetMatch};
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::player::advancement::trigger::AdvancementTrigger;
use crate::entity::{EntityBase, ai::pathfinder::NavigatorGoal, mob::Mob, r#type::from_type};

use super::{Controls, Goal, GoalReach};

const PARTNER_RANGE: f64 = 8.0;

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
        let world = entity.world.load();
        // Vanilla `PARTNER_TARGETING`: 8 blocks, seen or not, an invisible partner counts less.
        let conditions = TargetPredicate::create_non_attackable()
            .ignore_visibility()
            .set_base_max_distance(PARTNER_RANGE);
        let area = SearchArea::from_feet(entity, PARTNER_RANGE, PARTNER_RANGE);

        TargetMatch::Type(entity.entity_type).find_nearest(&world, &area, |candidate| {
            candidate.is_in_love()
                && candidate.is_breeding_ready()
                && !candidate.is_panicking()
                && conditions.test(&world, Some(mob), candidate)
        })
    }

    fn breed(mob: &dyn Mob, mate: &dyn EntityBase) {
        let mob_entity = mob.get_mob_entity();
        let entity = mob.get_entity();
        let world = entity.world.load();
        let world_full = entity.world.load_full();
        let parent_pos = entity.pos.load();

        let baby = mob.as_animal().map_or_else(
            || from_type(entity.entity_type, parent_pos, &world, Uuid::new_v4()),
            |animal| animal.get_breed_offspring(mate),
        );
        let experience = mob.get_random().random_range(1..8);
        let Some(experience) = mob.approve_breeding(
            mate.get_entity().entity_id,
            baby.get_entity().entity_id,
            experience,
        ) else {
            // Cancelled: end love mode with the usual cooldown, a plugin resets it if wanted.
            mob_entity.reset_love_ticks();
            mob_entity
                .breeding_cooldown
                .store(6000, std::sync::atomic::Ordering::Relaxed);
            mate.reset_love();
            mate.set_breeding_cooldown(6000);
            return;
        };

        // Both parents' online breeders, vanilla's love causes.
        let online = |uuid: &Uuid| world.get_player_by_uuid(*uuid).is_some();
        let first = mob_entity.breeder.load().filter(online);
        let second = mate
            .get_mob()
            .and_then(|mate| mate.get_mob_entity().breeder.load())
            .filter(|uuid| online(uuid) && first != Some(*uuid));
        if let Some(baby_mob) = baby.get_mob() {
            for breeder in first.into_iter().chain(second) {
                baby_mob.on_bred_by(breeder);
            }
        }

        // Vanilla `getLoveCause`: this parent's breeder, else the mate's.
        let love_cause = first.or(second);
        if let Some(player) = love_cause.and_then(|uuid| world.get_player_by_uuid(uuid)) {
            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::AnimalsBred as i32,
                1,
            );
            player.trigger_advancement(AdvancementTrigger::BredAnimal {
                parent_type: format!("minecraft:{}", entity.entity_type.resource_name),
            });
        }

        mob_entity.reset_love_ticks();
        mob_entity
            .breeding_cooldown
            .store(6000, std::sync::atomic::Ordering::Relaxed);

        mate.reset_love();
        mate.set_breeding_cooldown(6000);

        baby.get_entity().set_age(-24000);
        world_full.spawn_entity(baby);

        world_full.send_entity_status(
            entity,
            EntityStatus::InLoveHearts,
            ActorEventID::InLoveHearts,
        );
        if world.level_info.load().game_rules.mob_drops
            && let Ok(experience) = u32::try_from(experience)
            && experience > 0
        {
            ExperienceOrbEntity::spawn(&world_full, parent_pos, experience);
        }
    }
}

impl Goal for BreedGoal {
    fn tick_reach(&self, mob: &dyn Mob, running: bool) -> Option<GoalReach> {
        let mob_entity = mob.get_mob_entity();
        let world = mob.get_entity().world.load();
        let mut links: Vec<i32> = self
            .mate
            .iter()
            .map(|mate| mate.get_entity().entity_id)
            .collect();
        if running {
            // Both parents' breeders get trust, stats and the advancement.
            let mate_breeder = self
                .mate
                .as_ref()
                .and_then(|mate| mate.get_mob())
                .and_then(|mate| mate.get_mob_entity().breeder.load());
            for breeder in [mob_entity.breeder.load(), mate_breeder]
                .into_iter()
                .flatten()
            {
                if let Some(player) = world.get_player_by_uuid(breeder) {
                    links.push(player.entity_id());
                }
            }
        }
        let searching = !running && mob_entity.is_breeding_ready() && mob_entity.is_in_love();
        Some(GoalReach {
            entities: if searching { PARTNER_RANGE } else { 0.0 },
            speed: if running || searching {
                self.speed
            } else {
                0.0
            },
            links,
            ..GoalReach::default()
        })
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        if !mob_entity.is_breeding_ready() || !mob_entity.is_in_love() {
            return false;
        }

        self.mate = Self::find_mate(mob);
        self.mate.is_some()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(mate) = &self.mate else {
            return false;
        };

        if !mate.get_entity().is_alive() || mate.is_panicking() {
            return false;
        }

        mate.is_in_love() && self.timer < 60
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.timer = 0;
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.mate = None;
        self.timer = 0;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(mate) = &self.mate else {
            return;
        };

        let mob_entity = mob.get_mob_entity();
        let mate_pos = mate.get_entity().pos.load();

        {
            let mut look_control = mob_entity
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            look_control.look_at_entity(mob, mate);
        };

        let my_pos = mob.get_entity().pos.load();
        let dist_sq = my_pos.squared_distance_to_vec(&mate_pos);

        {
            let steered = mob_entity.steered();
            let mut navigator = steered.navigator();
            navigator.set_progress(NavigatorGoal::new(my_pos, mate_pos, self.speed));
        };

        self.timer += 1;

        if self.timer >= self.get_tick_count(60) && dist_sq < 9.0 {
            Self::breed(mob, mate.as_ref());
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
