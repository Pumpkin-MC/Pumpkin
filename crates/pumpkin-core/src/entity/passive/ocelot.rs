use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_data::Block;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ai::behavior::{
        feline::{self, TEMPT_SPEED},
        fleeing::{self, Flee},
        hunting::{self, Hunt},
    },
    ai::goal::{Controls, Goal, GoalReach, swim::SwimGoal, tempt::TemptGoal},
    ai::target_match::TargetMatch,
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;

const AMBIENT_SOUND_INTERVAL: i32 = 900;
/// How far away a player can be and still feed it.
const FEED_DISTANCE_SQ: f64 = 9.0;
const DESPAWN_MIN_AGE_TICKS: i32 = 2400;

const OCELOT_FLEES: &[Flee] = &[feline::flee_players(4, is_wary)];

const OCELOT_HUNTS: &[Hunt] = &[
    Hunt::new(1, TargetMatch::Type(&EntityType::CHICKEN), false),
    hunting::baby_turtles(1, false),
];

fn is_wary(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<OcelotEntity>()
        .is_some_and(|ocelot| !ocelot.is_trusting())
}

/// Represents an Ocelot, a shy passive mob found in jungles.
///
/// Wiki: <https://minecraft.wiki/w/Ocelot>
pub struct OcelotEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: crate::entity::ageable::AgeableData,
    pub is_trusting: AtomicBool,
}

impl OcelotEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let ocelot = Self {
            mob_entity,
            ageable_data: crate::entity::ageable::AgeableData::default(),
            is_trusting: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(ocelot);
        let ocelot_weak = Arc::downgrade(&mob_arc);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(3, Box::new(OcelotTemptGoal::new(ocelot_weak)));
            fleeing::apply(goal_selector, OCELOT_FLEES);
            feline::apply_tail(goal_selector, mob_weak, 7);
        };

        {
            let target_selector = &mob_arc.mob_entity.target_selector;
            hunting::apply(target_selector, &mob_arc.mob_entity, OCELOT_HUNTS);
        };

        mob_arc
    }

    pub fn is_trusting(&self) -> bool {
        self.is_trusting.load(Ordering::Relaxed)
    }

    pub fn set_trusting(&self, trusting: bool) {
        self.is_trusting.store(trusting, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::ocelot::TRUSTING, trusting);
    }
}

impl Animal for OcelotEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack
            .get_item()
            .has_tag(&tag::Item::MINECRAFT_OCELOT_FOOD)
    }
}

impl crate::entity::ageable::AgeableMob for OcelotEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }

    fn default_baby_spawn_chance(&self) -> f32 {
        crate::entity::ageable::OCELOT_BABY_SPAWN_CHANCE
    }
}

impl Mob for OcelotEntity {
    fn as_ageable(&self) -> Option<&dyn crate::entity::ageable::AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("Trusting", self.is_trusting.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(trusting) = nbt.get_bool("Trusting") {
            self.is_trusting.store(trusting, Ordering::Relaxed);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::ocelot::BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::ocelot::TRUSTING,
            self.is_trusting.load(Ordering::Relaxed),
        );
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.ambient_sound
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        AMBIENT_SOUND_INTERVAL
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        feline::update_pose_from_speed(self);
    }

    fn mob_is_stepping_carefully(&self) -> bool {
        feline::is_stepping_carefully(self)
    }

    fn check_spawn_obstruction(&self, world: &World) -> bool {
        let entity = self.get_entity();
        if !world.is_unobstructed(entity) || world.contains_any_liquid(entity.bounding_box.load()) {
            return false;
        }
        let pos = entity.block_pos.load();
        if pos.0.y < world.sea_level {
            return false;
        }
        let below = world.get_block(&pos.down());
        below == &Block::GRASS_BLOCK || below.has_tag(&tag::Block::MINECRAFT_LEAVES)
    }

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        !self.is_trusting()
            && self.get_entity().tick_count.load(Ordering::Relaxed) > DESPAWN_MIN_AGE_TICKS
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        // Only an ocelot that is following the food can be fed.
        if self
            .mob_entity
            .goals_selector
            .is_running::<OcelotTemptGoal>()
            && !self.is_trusting()
            && self.is_food(item_stack)
            && player
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&self.get_entity().pos.load())
                < FEED_DISTANCE_SQ
        {
            self.use_player_item(player, item_stack);

            if self.get_random().random_range(0..3) == 0 {
                self.set_trusting(true);
                self.get_entity().world.load().send_entity_status(
                    self.get_entity(),
                    EntityStatus::TrustingSucceeded,
                    ActorEventID::TamingSucceeded,
                );
            } else {
                self.get_entity().world.load().send_entity_status(
                    self.get_entity(),
                    EntityStatus::TrustingFailed,
                    ActorEventID::TamingFailed,
                );
            }

            return true;
        }

        self.animal_interact(player, item_stack)
    }
}

/// Tempted by fish, but only a wary ocelot is scared off by a player moving too fast.
struct OcelotTemptGoal {
    ocelot: Weak<OcelotEntity>,
    tempt_goal: TemptGoal,
}

impl OcelotTemptGoal {
    fn new(ocelot: Weak<OcelotEntity>) -> Self {
        Self {
            ocelot,
            tempt_goal: TemptGoal::with_tag(TEMPT_SPEED, &tag::Item::MINECRAFT_OCELOT_FOOD, true),
        }
    }
}

impl Goal for OcelotTemptGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.tempt_goal.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let trusting = self
            .ocelot
            .upgrade()
            .is_some_and(|ocelot| ocelot.is_trusting());
        self.tempt_goal.set_can_scare(!trusting);
        self.tempt_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.tempt_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.tempt_goal.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.tempt_goal.tick(mob);
    }

    fn tick_reach(&self, mob: &dyn Mob, running: bool) -> Option<GoalReach> {
        self.tempt_goal.tick_reach(mob, running)
    }

    fn controls(&self) -> Controls {
        self.tempt_goal.controls()
    }
}
