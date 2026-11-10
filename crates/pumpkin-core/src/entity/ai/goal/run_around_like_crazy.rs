use pumpkin_data::entity::EntityStatus;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use rand::RngExt;

use super::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::ai::util::default_random_pos;
use crate::entity::mob::Mob;
use crate::entity::passive::abstract_horse::AbstractHorse;
use pumpkin_util::math::vector3::Vector3;

pub struct RunAroundLikeCrazyGoal {
    goal_control: Controls,
    speed_modifier: f64,
    pos_x: f64,
    pos_y: f64,
    pos_z: f64,
}

impl RunAroundLikeCrazyGoal {
    #[must_use]
    pub const fn new(speed_modifier: f64) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed_modifier,
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 0.0,
        }
    }
}

impl Goal for RunAroundLikeCrazyGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.is_tamed()
            || !mob.get_entity().has_passengers()
            || mob
                .as_abstract_horse()
                .is_some_and(AbstractHorse::is_mob_controlled)
        {
            return false;
        }

        let Some(pos) = default_random_pos::get_pos(mob, 5, 4) else {
            return false;
        };

        self.pos_x = pos.x;
        self.pos_y = pos.y;
        self.pos_z = pos.z;
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let is_idle = mob.is_navigator_idle();

        !mob.is_tamed() && !is_idle && mob.get_entity().has_passengers()
    }

    fn start(&mut self, mob: &dyn Mob) {
        let mob_pos = mob.get_entity().pos.load();
        let steered = mob.get_mob_entity().steered();
        let mut navigator = steered.navigator();
        navigator.set_progress(NavigatorGoal::new(
            mob_pos,
            Vector3::new(self.pos_x, self.pos_y, self.pos_z),
            self.speed_modifier,
        ));
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if mob.is_tamed() || mob.get_random().random_range(0..self.get_tick_count(50)) != 0 {
            return;
        }
        let Some(horse) = mob.as_abstract_horse() else {
            return;
        };
        let entity = mob.get_entity();
        let Some(passenger) = mob.get_mob_entity().first_passenger() else {
            return;
        };

        if passenger.get_player().is_some()
            && let Some(player) = entity
                .world
                .load()
                .get_player_by_id(passenger.get_entity().entity_id)
        {
            let temper = horse.get_temper();
            let max_temper = horse.get_max_temper();
            if max_temper > 0
                && mob.get_random().random_range(0..max_temper) < temper
                && mob.approve_taming(&player)
            {
                horse.tame_with_name(&player);
                return;
            }
            horse.modify_temper(5);
        }

        let passengers: Vec<i32> = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|passenger| passenger.get_entity().entity_id)
            .collect();
        for passenger in passengers {
            entity.remove_passenger(passenger);
        }
        horse.make_mad();
        entity.world.load().send_entity_status(
            entity,
            EntityStatus::TamingFailed,
            ActorEventID::TamingFailed,
        );
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
