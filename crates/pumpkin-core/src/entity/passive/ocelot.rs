use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_data::entity::{EntityPose, EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        active_target::{ActiveTargetGoal, DEFAULT_RANDOM_INTERVAL},
        avoid_entity::AvoidEntityGoal,
        breed::BreedGoal,
        leap_at_target::LeapAtTargetGoal,
        look_at_entity::LookAtEntityGoal,
        ocelot_attack::OcelotAttackGoal,
        swim::SwimGoal,
        tempt::TemptGoal,
        water_avoiding_random_stroll::WaterAvoidingRandomStrollGoal,
    },
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    passive::turtle::baby_on_land_selector,
    player::Player,
};

const TEMPT_ITEMS: &[&Item] = &[&Item::COD, &Item::SALMON];

/// Vanilla `Ocelot.WALK_SPEED_MOD`: breeding, strolling and the avoid goal's walk speed; the
/// avoid goal's sprint speed is `Ocelot.SPRINT_SPEED_MOD`.
const WALK_SPEED_MOD: f64 = 0.8;
const SPRINT_SPEED_MOD: f64 = 1.33;
const AVOID_DISTANCE: f64 = 16.0;
/// Vanilla `Ocelot.CROUCH_SPEED_MOD`: the tempt goal speed and the crouch key in
/// `customServerAiStep`.
const CROUCH_SPEED_MOD: f64 = 0.6;

/// Represents an Ocelot, a shy passive mob found in jungles.
///
/// Wiki: <https://minecraft.wiki/w/Ocelot>
pub struct OcelotEntity {
    pub mob_entity: MobEntity,
    pub is_trusting: AtomicBool,
    /// Set right after construction; the avoid goal needs a `Weak` to read the trust flag back
    /// out of the mob it is attached to.
    self_weak: Mutex<Option<Weak<Self>>>,
}

impl OcelotEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let ocelot = Self {
            mob_entity,
            is_trusting: AtomicBool::new(false),
            self_weak: Mutex::new(None),
        };
        let mob_arc = Arc::new(ocelot);
        *mob_arc
            .self_weak
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::downgrade(&mob_arc));
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        // Captured by the tempt goal, which re-reads the trust state from the ocelot on every
        // startle check (`OcelotTemptGoal#canScare`).
        let tempt_weak = mob_arc.weak_ref();

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            // Goal 1: FloatGoal (SwimGoal). Vanilla stops here: unlike the cat, an ocelot has
            // no panic goal at all.
            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            // Goal 3: OcelotTemptGoal. Once the ocelot is trusting, vanilla drops the startle
            // check (`OcelotTemptGoal#canScare`), so the fish can be followed steadily.
            goal_selector.add_goal(
                3,
                Box::new(
                    TemptGoal::new(CROUCH_SPEED_MOD, TEMPT_ITEMS, true).with_scare_condition(
                        move |_mob| {
                            tempt_weak
                                .upgrade()
                                .is_some_and(|ocelot| !ocelot.is_trusting())
                        },
                    ),
                ),
            );
            // Goal 7: LeapAtTargetGoal
            goal_selector.add_goal(7, Box::new(LeapAtTargetGoal::new(0.3)));
            // Goal 8: OcelotAttackGoal, shared with the cat
            goal_selector.add_goal(8, Box::new(OcelotAttackGoal::new()));
            // Goal 9: BreedGoal
            goal_selector.add_goal(9, BreedGoal::new(WALK_SPEED_MOD));
            // Goal 10: WaterAvoidingRandomStrollGoal
            goal_selector.add_goal(
                10,
                Box::new(WaterAvoidingRandomStrollGoal::with_probability(
                    WALK_SPEED_MOD,
                    1.000_000_1E-5,
                )),
            );
            // Goal 11: LookAtPlayerGoal. Vanilla stops here: no RandomLookAroundGoal.
            goal_selector.add_goal(
                11,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );

            // Goal 4: OcelotAvoidEntityGoal is added by `reassess_trusting_goals` below.
        };

        // Goal 4: OcelotAvoidEntityGoal (only while untrusted)
        mob_arc.reassess_trusting_goals();

        {
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            // Target Goal 1: NearestAttackableTargetGoal for Chicken and Turtle
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::CHICKEN, false),
            );
            target_selector.add_goal(
                1,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::TURTLE,
                    DEFAULT_RANDOM_INTERVAL,
                    false,
                    false,
                    Some(baby_on_land_selector),
                )),
            );
        };

        mob_arc
    }

    fn weak_ref(&self) -> Weak<Self> {
        self.self_weak
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .unwrap_or_default()
    }

    /// Vanilla `Ocelot#reassessTrustingGoals`: the avoid-players goal exists only while the
    /// ocelot is untrusting. The goal additionally re-checks the flag on every start, which is
    /// vanilla's second guard in `Ocelot$OcelotAvoidEntityGoal#canUse`.
    pub fn reassess_trusting_goals(&self) {
        let weak = self.weak_ref();
        let mob: &dyn Mob = self;
        let mut goal_selector = self
            .mob_entity
            .goals_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        goal_selector.remove_goal::<AvoidEntityGoal>(mob);
        if !self.is_trusting() {
            goal_selector.add_goal(
                4,
                Box::new(AvoidEntityGoal::with_gate(
                    &EntityType::PLAYER,
                    AVOID_DISTANCE,
                    WALK_SPEED_MOD,
                    SPRINT_SPEED_MOD,
                    move |_mob| weak.upgrade().is_some_and(|ocelot| !ocelot.is_trusting()),
                )),
            );
        }
    }

    pub fn is_trusting(&self) -> bool {
        self.is_trusting.load(Ordering::Relaxed)
    }

    pub fn set_trusting(&self, trusting: bool) {
        self.is_trusting.store(trusting, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::ocelot::TRUSTING, trusting);
        self.reassess_trusting_goals();
    }
}

impl Animal for OcelotEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        let item = item_stack.get_item();
        item.has_tag(&tag::Item::MINECRAFT_OCELOT_FOOD)
            || item == &Item::COD
            || item == &Item::SALMON
    }
}

impl Mob for OcelotEntity {
    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("Trusting", self.is_trusting.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(trusting) = nbt.get_bool("Trusting") {
            self.is_trusting.store(trusting, Ordering::Relaxed);
            // Vanilla `Ocelot#setTrusting` re-runs the goal reassessment, and
            // `readAdditionalSaveData` goes through it.
            self.reassess_trusting_goals();
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

    /// Vanilla `Ocelot.customServerAiStep`: pose and sprint follow the navigation's speed,
    /// identical to the cat's copy.
    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        let (idle, speed) = {
            let navigator = self
                .mob_entity
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (navigator.is_idle(), navigator.get_speed())
        };

        let entity = self.get_entity();
        // Vanilla compares the move-control modifier directly; ours is one of the fixed
        // goal speeds, so exact equality is intended. The crouch is cosmetic for an ocelot:
        // vanilla `LivingEntity.getDimensions` only swaps boxes for `Sleeping`, so the pose
        // must not resize the hitbox to the player-sized pose table.
        if !idle && speed == SPRINT_SPEED_MOD {
            entity.set_pose_keep_dimensions(EntityPose::Standing);
            self.mob_entity.living_entity.set_sprinting(true);
        } else if !idle && speed == CROUCH_SPEED_MOD {
            entity.set_pose_keep_dimensions(EntityPose::Crouching);
            self.mob_entity.living_entity.set_sprinting(false);
        } else {
            entity.set_pose_keep_dimensions(EntityPose::Standing);
            self.mob_entity.living_entity.set_sprinting(false);
        }
    }

    /// Vanilla `Ocelot.mobInteract`: trust only rolls while the ocelot is following the fish
    /// (`temptGoal.isRunning()`), which is why feeding one that is not interested does nothing.
    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let is_food = self.is_food(item_stack);
        let dist_sqr = self
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&player.get_entity().pos.load());
        let tempted = self
            .mob_entity
            .goals_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_running::<TemptGoal>();

        if tempted && !self.is_trusting() && is_food && dist_sqr < 9.0 {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);

            let mut rng = rand::rng();
            if rng.random_range(0..3) == 0 {
                self.set_trusting(true);
                self.get_entity().world.load().send_entity_status(
                    self.get_entity(),
                    EntityStatus::TrustingSucceeded,
                    Some(ActorEventID::TamingSucceeded),
                );
            } else {
                self.get_entity().world.load().send_entity_status(
                    self.get_entity(),
                    EntityStatus::TrustingFailed,
                    Some(ActorEventID::TamingFailed),
                );
            }

            return true;
        }

        self.animal_interact(
            player,
            item_stack,
            pumpkin_data::sound::Sound::EntityOcelotAmbient,
        )
    }
}
