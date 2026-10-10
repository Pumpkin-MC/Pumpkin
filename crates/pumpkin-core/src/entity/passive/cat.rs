use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

use pumpkin_data::cat_sound_variant::CatSoundVariant;
use pumpkin_data::cat_variant::CatVariant;
use pumpkin_data::entity::{EntityPose, EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::codec::var_int::VarInt;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::custom_sound::CustomSound;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        active_target::{ActiveTargetGoal, DEFAULT_RANDOM_INTERVAL},
        avoid_entity::AvoidEntityGoal,
        breed::BreedGoal,
        escape_danger::EscapeDangerGoal,
        follow_owner::FollowOwnerGoal,
        leap_at_target::LeapAtTargetGoal,
        look_at_entity::LookAtEntityGoal,
        ocelot_attack::OcelotAttackGoal,
        sit_when_ordered_to::SitWhenOrderedToGoal,
        swim::SwimGoal,
        tempt::TemptGoal,
        water_avoiding_random_stroll::WaterAvoidingRandomStrollGoal,
    },
    living::LivingEntity,
    mob::{Mob, MobEntity},
    passive::{
        animal::Animal,
        tamable::{TamableAnimal, TamableData},
        turtle::baby_on_land_selector,
    },
    player::Player,
};
use crate::world::World;

const TEMPT_ITEMS: &[&Item] = &[&Item::COD, &Item::SALMON];

/// Vanilla `Cat.WALK_SPEED_MOD`: breed, stroll and the avoid goal's walk speed; the avoid
/// goal's sprint speed is `Cat.SPRINT_SPEED_MOD`.
const WALK_SPEED_MOD: f64 = 0.8;
const SPRINT_SPEED_MOD: f64 = 1.33;
const AVOID_DISTANCE: f64 = 16.0;
/// Vanilla `Cat.TEMPT_SPEED_MOD`; `customServerAiStep` keys the crouch off it.
const TEMPT_SPEED_MOD: f64 = 0.6;

/// `Cat#registerGoals`'s `NonTameRandomTargetGoal` gate: the cat only hunts while untamed.
/// The turtle row adds vanilla's baby-on-land selector, shared from the turtle module.
fn cat_prey_predicate(
    cat: &Weak<CatEntity>,
    baby_only: bool,
) -> impl Fn(&LivingEntity, &World) -> bool + Send + Sync + 'static {
    let cat = cat.clone();
    move |target, world| {
        if cat.upgrade().is_none_or(|cat| cat.is_tame()) {
            return false;
        }
        !baby_only || baby_on_land_selector(target, world)
    }
}

fn get_dye_color_from_item(item: &Item) -> Option<u8> {
    let key = item.registry_key;
    if key.contains("white") {
        Some(0)
    } else if key.contains("orange") {
        Some(1)
    } else if key.contains("magenta") {
        Some(2)
    } else if key.contains("light_blue") {
        Some(3)
    } else if key.contains("yellow") {
        Some(4)
    } else if key.contains("lime") {
        Some(5)
    } else if key.contains("pink") {
        Some(6)
    } else if key.contains("light_gray") {
        Some(8)
    } else if key.contains("gray") {
        Some(7)
    } else if key.contains("cyan") {
        Some(9)
    } else if key.contains("purple") {
        Some(10)
    } else if key.contains("blue") {
        Some(11)
    } else if key.contains("brown") {
        Some(12)
    } else if key.contains("green") {
        Some(13)
    } else if key.contains("red") {
        Some(14)
    } else if key.contains("black") {
        Some(15)
    } else {
        None
    }
}

pub struct CatEntity {
    pub mob_entity: MobEntity,
    pub variant: AtomicU8,
    pub sound_variant: AtomicU8,
    pub collar_color: AtomicU8,
    pub tamable_data: TamableData,
    pub is_lying: AtomicBool,
    pub relax_state_one: AtomicBool,
    /// Set right after construction; the avoid goal and the prey goals need a `Weak` to read the
    /// tame state back out of the mob they are attached to.
    self_weak: Mutex<Option<Weak<Self>>>,
}

impl CatEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let cat = Self {
            mob_entity,
            variant: AtomicU8::new(CatVariant::Black.id()),
            sound_variant: AtomicU8::new(0), // Default to classic
            collar_color: AtomicU8::new(14), // Default to red
            tamable_data: TamableData::default(),
            is_lying: AtomicBool::new(false),
            relax_state_one: AtomicBool::new(false),
            self_weak: Mutex::new(None),
        };
        let mob_arc = Arc::new(cat);
        *mob_arc
            .self_weak
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::downgrade(&mob_arc));
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        // Captured by the tempt goal, which re-reads the tame state from the cat on every start
        // attempt (`CatTemptGoal#canUse`).
        let tempt_weak = mob_arc.weak_ref();

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            // Goal 1: SwimGoal (FloatGoal)
            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            // Goal 1: TamableAnimalPanicGoal (EscapeDangerGoal)
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.5));
            // Goal 2: SitWhenOrderedToGoal
            goal_selector.add_goal(2, Box::new(SitWhenOrderedToGoal::new()));
            // Goal 4: CatTemptGoal. Vanilla gates the goal on `!cat.isTame()`
            // (`CatTemptGoal#canUse`), so a tamed cat ignores the fish entirely.
            goal_selector.add_goal(
                4,
                Box::new(
                    TemptGoal::new(TEMPT_SPEED_MOD, TEMPT_ITEMS, true).with_can_use_gate(
                        move |_mob| tempt_weak.upgrade().is_some_and(|cat| !cat.is_tame()),
                    ),
                ),
            );
            // Goal 6: FollowOwnerGoal
            goal_selector.add_goal(6, FollowOwnerGoal::new(1.0, 10.0, 5.0));
            // Goal 8: LeapAtTargetGoal
            goal_selector.add_goal(8, Box::new(LeapAtTargetGoal::new(0.3)));
            // Goal 9: OcelotAttackGoal, shared with the ocelot
            goal_selector.add_goal(9, Box::new(OcelotAttackGoal::new()));
            // Goal 10: BreedGoal
            goal_selector.add_goal(10, BreedGoal::new(WALK_SPEED_MOD));
            // Goal 11: WaterAvoidingRandomStrollGoal
            goal_selector.add_goal(
                11,
                Box::new(WaterAvoidingRandomStrollGoal::with_probability(
                    WALK_SPEED_MOD,
                    1.000_000_1E-5,
                )),
            );
            // Goal 12: LookAtPlayerGoal. Vanilla stops at this one — no RandomLookAroundGoal, so
            // a cat that is not looking at a player simply keeps its head where it is.
            goal_selector.add_goal(
                12,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );

            // Goal 4: CatAvoidEntityGoal is added last, by `reassess_tame_goals` below, so it
            // registers after the tempt goal: both sit at priority 4 and the first registered
            // goal wins the MOVE control, which matches the vanilla insertion order.
        };

        // Goal 4: CatAvoidEntityGoal (only while untamed)
        mob_arc.reassess_tame_goals();

        {
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let weak = mob_arc.weak_ref();

            // Target Goal 1: NonTameRandomTargetGoal for Rabbit and Turtle
            target_selector.add_goal(
                1,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::RABBIT,
                    DEFAULT_RANDOM_INTERVAL,
                    false,
                    false,
                    Some(cat_prey_predicate(&weak, false)),
                )),
            );
            target_selector.add_goal(
                1,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::TURTLE,
                    DEFAULT_RANDOM_INTERVAL,
                    false,
                    false,
                    Some(cat_prey_predicate(&weak, true)),
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

    /// Vanilla `Cat#reassessTameGoals`: the avoid-players goal exists only while the cat is
    /// untamed, so every tame-state change has to add it back or drop it. The goal additionally
    /// re-checks the tame state on every start, which is vanilla's second guard in
    /// `Cat$CatAvoidEntityGoal#canUse`.
    pub fn reassess_tame_goals(&self) {
        let weak = self.weak_ref();
        let mob: &dyn Mob = self;
        let mut goal_selector = self
            .mob_entity
            .goals_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        goal_selector.remove_goal::<AvoidEntityGoal>(mob);
        if !self.is_tame() {
            goal_selector.add_goal(
                4,
                Box::new(AvoidEntityGoal::with_gate(
                    &EntityType::PLAYER,
                    AVOID_DISTANCE,
                    WALK_SPEED_MOD,
                    SPRINT_SPEED_MOD,
                    move |_mob| weak.upgrade().is_some_and(|cat| !cat.is_tame()),
                )),
            );
        }
    }

    pub fn get_tame_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.is_in_sitting_pose() {
            flags |= 0x01;
        }
        if self.is_tame() {
            flags |= 0x04;
        }
        flags
    }

    pub fn is_sitting(&self) -> bool {
        self.is_in_sitting_pose()
    }

    pub fn is_lying(&self) -> bool {
        self.is_lying.load(Ordering::Relaxed)
    }

    pub fn is_relax_state_one(&self) -> bool {
        self.relax_state_one.load(Ordering::Relaxed)
    }

    pub fn get_collar_color(&self) -> u8 {
        self.collar_color.load(Ordering::Relaxed)
    }

    pub fn set_collar_color(&self, color: u8) {
        self.collar_color.store(color, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::CAT_COLLAR_COLOR,
            VarInt(color as i32),
        );
    }

    pub fn set_sitting(&self, sitting: bool) {
        self.set_in_sitting_pose(sitting);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::TAMEABLE_FLAGS,
            self.get_tame_flags(),
        );
    }

    pub fn set_tame(&self, tame: bool, owner: Option<Uuid>) {
        self.tamable_data.is_tame.store(tame, Ordering::Relaxed);
        self.set_owner(owner);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::TAMEABLE_FLAGS,
            self.get_tame_flags(),
        );
        entity.set_synced_data(pumpkin_data::tracked_data::cat::OWNER_UUID, owner);
        self.reassess_tame_goals();
    }

    pub fn set_variant(&self, variant: u8) {
        self.variant.store(variant, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::CAT_VARIANT,
            VarInt(variant as i32),
        );
    }

    pub fn set_lying(&self, lying: bool) {
        self.is_lying.store(lying, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::cat::IS_LYING, lying);
    }

    pub fn set_relax_state_one(&self, relax: bool) {
        self.relax_state_one.store(relax, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::cat::RELAX_STATE_ONE, relax);
    }

    pub fn set_sound_variant(&self, sound_variant: CatSoundVariant) {
        self.sound_variant
            .store(sound_variant as u8, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::SOUND_VARIANT,
            VarInt(sound_variant as u8 as i32),
        );
    }

    pub fn play_eating_sound(&self) {
        let mob_entity = self.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        let sound_variant = CatSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
            .unwrap_or_default();
        let world = entity.world.load();
        world.play_sound(
            sound_variant.eat_sound(is_baby),
            pumpkin_data::sound::SoundCategory::Neutral,
            &entity.pos.load(),
        );
    }
}

impl CustomSound for CatEntity {
    fn death_sound(&self) -> Option<pumpkin_data::sound::Sound> {
        let is_baby = self.get_entity().age.load(Ordering::Relaxed) < 0;
        let sound_variant = CatSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
            .unwrap_or_default();
        Some(sound_variant.death_sound(is_baby))
    }

    fn hurt_sound(&self) -> Option<pumpkin_data::sound::Sound> {
        let is_baby = self.get_entity().age.load(Ordering::Relaxed) < 0;
        let sound_variant = CatSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
            .unwrap_or_default();
        Some(sound_variant.hurt_sound(is_baby))
    }
}

impl Animal for CatEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        let item = item_stack.get_item();
        item.has_tag(&tag::Item::MINECRAFT_CAT_FOOD) || item == &Item::COD || item == &Item::SALMON
    }
}

impl TamableAnimal for CatEntity {
    fn get_tamable_data(&self) -> &TamableData {
        &self.tamable_data
    }
}

impl Mob for CatEntity {
    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn as_tamable(&self) -> Option<&dyn TamableAnimal> {
        Some(self)
    }

    /// Vanilla `Cat.customServerAiStep`: the pose and sprint state follow the speed the
    /// navigation was last told to move at. The sprint flag is what the client keys the
    /// feet dust off, and the sprint attribute modifier is what makes the flee actually
    /// faster than the walk.
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
        // goal speeds, so exact equality is intended. The crouch is cosmetic for a cat:
        // vanilla `LivingEntity.getDimensions` only swaps boxes for `Sleeping`, so the pose
        // must not resize the hitbox to the player-sized pose table.
        if !idle && speed == SPRINT_SPEED_MOD {
            entity.set_pose_keep_dimensions(EntityPose::Standing);
            self.mob_entity.living_entity.set_sprinting(true);
        } else if !idle && speed == TEMPT_SPEED_MOD {
            entity.set_pose_keep_dimensions(EntityPose::Crouching);
            self.mob_entity.living_entity.set_sprinting(false);
        } else {
            entity.set_pose_keep_dimensions(EntityPose::Standing);
            self.mob_entity.living_entity.set_sprinting(false);
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        let variant_id = self.variant.load(Ordering::Relaxed);
        let variant_str = CatVariant::all()
            .get(variant_id as usize)
            .map_or("minecraft:tabby", CatVariant::asset_id);
        nbt.put_string("variant", variant_str.to_string());
        let sound_variant = CatSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
            .unwrap_or_default();
        nbt.put_string(
            "sound_variant",
            format!("minecraft:{}", sound_variant.to_name()),
        );
        nbt.put_byte(
            "CollarColor",
            self.collar_color.load(Ordering::Relaxed) as i8,
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(variant_str) = nbt.get_string("variant") {
            let variant =
                CatVariant::from_name(variant_str).map_or(CatVariant::Tabby.id(), |v| v.id());
            self.variant.store(variant, Ordering::Relaxed);
        }
        if let Some(sound_str) = nbt.get_string("sound_variant")
            && let Some(sound_variant) = CatSoundVariant::from_name(sound_str)
        {
            self.sound_variant
                .store(sound_variant as u8, Ordering::Relaxed);
        }
        if let Some(collar) = nbt.get_byte("CollarColor") {
            self.collar_color.store(collar as u8, Ordering::Relaxed);
        } else if let Some(collar_int) = nbt.get_int("CollarColor") {
            self.collar_color.store(collar_int as u8, Ordering::Relaxed);
        }
        // `TamableAnimal::read_tamable_nbt` has already applied `Owner`/`IsTame`, so a cat
        // loaded from disk needs its goals re-checked here — vanilla does the same through
        // `setTame`, which `readAdditionalSaveData` reaches via `TamableAnimal`.
        self.reassess_tame_goals();
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_set_variant_name(&self, name: &str) {
        let variant = CatVariant::from_name(name).map_or(CatVariant::Tabby.id(), |v| v.id());
        self.variant.store(variant, Ordering::Relaxed);
    }

    fn mob_set_sound_variant_name(&self, name: &str) {
        if let Some(v) = CatSoundVariant::from_name(name) {
            self.set_sound_variant(v);
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::cat::BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::TAMEABLE_FLAGS,
            self.get_tame_flags(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::OWNER_UUID,
            self.get_owner(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::CAT_VARIANT,
            VarInt(self.variant.load(Ordering::Relaxed) as i32),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::IS_LYING,
            self.is_lying.load(Ordering::Relaxed),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::RELAX_STATE_ONE,
            self.relax_state_one.load(Ordering::Relaxed),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::CAT_COLLAR_COLOR,
            VarInt(self.collar_color.load(Ordering::Relaxed) as i32),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cat::SOUND_VARIANT,
            VarInt(self.sound_variant.load(Ordering::Relaxed) as i32),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let item = item_stack.get_item();
        let is_food = self.is_food(item_stack);

        if self.is_tame() {
            if self.get_owner_uuid() == Some(player.gameprofile.id) {
                if item.has_tag(&tag::Item::MINECRAFT_CAT_COLLAR_DYES)
                    || item.has_tag(&tag::Item::C_DYES)
                {
                    if let Some(color) = get_dye_color_from_item(item)
                        && color != self.get_collar_color()
                    {
                        self.set_collar_color(color);
                        item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                        return true;
                    }
                } else if is_food
                    && self.mob_entity.living_entity.health.load()
                        < self.mob_entity.living_entity.get_max_health()
                {
                    item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                    self.mob_entity.living_entity.heal(2.0);
                    self.play_eating_sound();
                    return true;
                }

                let parent_interaction = self.mob_entity.mob_interact(player, item_stack);
                if !parent_interaction {
                    self.set_sitting(!self.is_sitting());
                    return true;
                }
                return parent_interaction;
            }
        } else if is_food {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            self.play_eating_sound();

            let mut rng = rand::rng();
            if rng.random_range(0..3) == 0 {
                self.set_tame(true, Some(player.gameprofile.id));
                self.set_sitting(true);
                self.get_entity().world.load().send_entity_status(
                    self.get_entity(),
                    EntityStatus::TamingSucceeded,
                    Some(ActorEventID::TamingSucceeded),
                );
            } else {
                self.get_entity().world.load().send_entity_status(
                    self.get_entity(),
                    EntityStatus::TamingFailed,
                    Some(ActorEventID::TamingFailed),
                );
            }

            return true;
        }

        self.mob_entity.mob_interact(player, item_stack)
    }
}
