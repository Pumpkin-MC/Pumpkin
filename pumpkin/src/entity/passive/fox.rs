use std::sync::{
    Arc, Weak,
    atomic::{AtomicU8, Ordering::Relaxed},
};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{
    entity::EntityType, item::Item, meta_data_type::MetaDataType, tracked_data::TrackedData,
};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        active_target::ActiveTargetGoal, breed::BreedGoal,
        climb_on_top_of_powder_snow::ClimbOnTopOfPowderSnowGoal, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, fox_faceplant::FoxFaceplantGoal,
        fox_melee_attack::FoxMeleeAttackGoal, fox_pounce::FoxPounceGoal, fox_sleep::FoxSleepGoal,
        fox_stalk_prey::StalkPreyGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};
use pumpkin_nbt::compound::NbtCompound;

const TEMPT_ITEMS: &[&Item] = &[&Item::SWEET_BERRIES, &Item::GLOW_BERRIES];

const FLAG_SITTING: u8 = 1;
const FLAG_CROUCHING: u8 = 4;
const FLAG_INTERESTED: u8 = 8;
const FLAG_POUNCING: u8 = 16;
const FLAG_SLEEPING: u8 = 32;
const FLAG_FACEPLANTED: u8 = 64;
const FLAG_DEFENDING: u8 = 128;

/// `Fox.crouchAmount` reaches `MAX_CROUCH_AMOUNT` (5.0) after 25 ticks at +0.2/tick; tracked
/// here as a tick counter instead of a float, since nothing server-side needs the
/// partial-tick-interpolated value vanilla exposes to the renderer.
const FULLY_CROUCHED_TICKS: u8 = 25;

/// Sentinel for "variant not yet rolled". There's no finalize-spawn hook in this codebase (the
/// same gap Turtle's `home_pos` and Salmon's variant roll already work around), so the
/// biome-based variant roll happens in `mob_init_data_tracker`, which needs to tell "not yet
/// rolled" apart from "rolled to Red (0)".
const VARIANT_UNSET: u8 = 2;

/// `Fox.Variant`: biome-based, rolled once at spawn (`byBiome`) unless restored from NBT.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum FoxVariant {
    Red = 0,
    Snow = 1,
}

impl FoxVariant {
    #[must_use]
    pub const fn from_id(id: u8) -> Self {
        match id {
            1 => Self::Snow,
            _ => Self::Red,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Snow => "snow",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "snow" => Self::Snow,
            _ => Self::Red,
        }
    }
}

/// Represents a Fox, a passive nocturnal mob.
///
/// PR-1 scope: synced flag state (`isSitting`/`isCrouching`/`isInterested`/`isPouncing`/
/// `isSleeping`/`isFaceplanted`/`isDefending`), biome-based variant, sleeping, and the
/// crouch/stalk/pounce state machine.
///
/// PR-2 scope so far: `FaceplantGoal` (clearing `isFaceplanted` back to `false` -- the pounce
/// goal could already set it on a snow landing, but nothing cleared it back until this landed),
/// and the melee attack goal (gated off while sitting/sleeping/crouching/faceplanted, matching
/// vanilla's `FoxMeleeAttackGoal.canUse` -- the `FOX_BITE` sound on a successful bite is left as
/// a documented simplification, see `fox_melee_attack.rs`).
///
/// Still deferred to this same follow-up PR: item-in-mouth mechanics (hold/steal/spit/eat), the
/// three `AvoidEntityGoal` registrations, trust list + `DefendTrustedTargetGoal`, berries-eating,
/// village-stroll, and per-variant target-goal ordering (all foxes currently hunt
/// chickens/rabbits with the same priority, rather than red
/// foxes preferring land prey and snow foxes preferring fish).
///
/// Wiki: <https://minecraft.wiki/w/Fox>
pub struct FoxEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    flags: AtomicU8,
    variant: AtomicU8,
    crouch_ticks: AtomicU8,
}

impl FoxEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let this = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            flags: AtomicU8::new(0),
            variant: AtomicU8::new(VARIANT_UNSET),
            crouch_ticks: AtomicU8::new(0),
        };
        let mob_arc = Arc::new(this);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(0, ClimbOnTopOfPowderSnowGoal::new());
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.5));
            // Note: vanilla registers `FaceplantGoal` at the same relative slot (above panic/
            // breed, below float/climb) -- also priority 1 here since `EscapeDangerGoal` already
            // occupies it, matching vanilla's own numbering.
            goal_selector.add_goal(1, FoxFaceplantGoal::new());
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.2, TEMPT_ITEMS, false)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.1)));
            // Below `WanderAroundGoal`'s priority 6 so stalking/pouncing/sleeping/melee always
            // preempts idle wandering when their own gates are satisfied. Equal-priority goals
            // are never preempted by one another (`can_be_replaced_by` needs strictly-lower
            // priority) and ties break by insertion order, so melee is registered before sleep
            // here to match vanilla's own relative order (`FoxMeleeAttackGoal` before
            // `SleepGoal`, both priority 7 in `Fox.java`).
            goal_selector.add_goal(5, StalkPreyGoal::new());
            goal_selector.add_goal(5, FoxPounceGoal::new());
            goal_selector.add_goal(5, FoxMeleeAttackGoal::new());
            goal_selector.add_goal(5, FoxSleepGoal::new());
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                9,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            // `Fox.landTargetGoal`, simplified: vanilla targets `Animal.class` filtered to
            // chicken/rabbit via a single `NearestAttackableTargetGoal`; this codebase's
            // `ActiveTargetGoal` only matches a single `EntityType`, so it's registered twice
            // instead. Per-variant reordering against the (not yet ported) fish/turtle-egg
            // target goals is deferred, see the struct doc comment.
            target_selector.add_goal(
                4,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::CHICKEN, false),
            );
            target_selector.add_goal(
                4,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::RABBIT, false),
            );
        };

        mob_arc
    }

    fn flag(&self, mask: u8) -> bool {
        self.flags.load(Relaxed) & mask != 0
    }

    /// Note: `TrackedData::FOX_FLAGS` resolves to the "not present in this protocol version"
    /// sentinel (255, a documented no-op in `Metadata::write`) on the v26.x versions this
    /// build targets -- `pumpkin-data`'s codegen flattens `DATA_FLAGS_ID` across every vanilla
    /// class that reuses that literal field name (Fox, Bee, Spider, Vex, `IronGolem`,
    /// `TamableAnimal`, Blaze) into one global key per version, and loses Fox's entry for v26.x
    /// in the process. `BeeEntity` hits the identical gap with `BEE_FLAGS` today. Server-side
    /// state (`self.flags`) stays authoritative either way; this only means clients on v26.x
    /// won't see the crouch/sit/sleep pose until `pumpkin-data`'s tracked-data generation is
    /// fixed to disambiguate per-class collisions (out of scope here).
    fn set_flag(&self, mask: u8, value: bool) {
        let old = if value {
            self.flags.fetch_or(mask, Relaxed)
        } else {
            self.flags.fetch_and(!mask, Relaxed)
        };
        let byte = if value { old | mask } else { old & !mask };
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::FOX_FLAGS,
                MetaDataType::BYTE,
                byte as i8,
            )],
            None,
        );
    }

    #[must_use]
    pub fn is_sitting(&self) -> bool {
        self.flag(FLAG_SITTING)
    }

    pub fn set_sitting(&self, value: bool) {
        self.set_flag(FLAG_SITTING, value);
    }

    #[must_use]
    pub fn is_crouching(&self) -> bool {
        self.flag(FLAG_CROUCHING)
    }

    pub fn set_is_crouching(&self, value: bool) {
        self.set_flag(FLAG_CROUCHING, value);
    }

    #[must_use]
    pub fn is_interested(&self) -> bool {
        self.flag(FLAG_INTERESTED)
    }

    pub fn set_is_interested(&self, value: bool) {
        self.set_flag(FLAG_INTERESTED, value);
    }

    #[must_use]
    pub fn is_pouncing(&self) -> bool {
        self.flag(FLAG_POUNCING)
    }

    pub fn set_is_pouncing(&self, value: bool) {
        self.set_flag(FLAG_POUNCING, value);
    }

    #[must_use]
    pub fn is_sleeping(&self) -> bool {
        self.flag(FLAG_SLEEPING)
    }

    pub fn set_sleeping(&self, value: bool) {
        self.set_flag(FLAG_SLEEPING, value);
    }

    #[must_use]
    pub fn is_faceplanted(&self) -> bool {
        self.flag(FLAG_FACEPLANTED)
    }

    pub fn set_faceplanted(&self, value: bool) {
        self.set_flag(FLAG_FACEPLANTED, value);
    }

    #[must_use]
    pub fn is_defending(&self) -> bool {
        self.flag(FLAG_DEFENDING)
    }

    pub fn set_defending(&self, value: bool) {
        self.set_flag(FLAG_DEFENDING, value);
    }

    /// `Fox.clearStates`: zeroes every transient behavior flag except `POUNCING` (which
    /// vanilla also leaves untouched here).
    pub fn clear_states(&self) {
        self.set_is_interested(false);
        self.set_is_crouching(false);
        self.set_sitting(false);
        self.set_sleeping(false);
        self.set_defending(false);
        self.set_faceplanted(false);
    }

    pub fn wake_up(&self) {
        self.set_sleeping(false);
    }

    #[must_use]
    pub fn is_fully_crouched(&self) -> bool {
        self.crouch_ticks.load(Relaxed) >= FULLY_CROUCHED_TICKS
    }

    #[must_use]
    pub fn variant(&self) -> FoxVariant {
        FoxVariant::from_id(self.variant.load(Relaxed))
    }

    fn set_variant(&self, variant: FoxVariant) {
        self.variant.store(variant as u8, Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::TYPE_ID,
                MetaDataType::INT,
                VarInt(i32::from(variant as u8)),
            )],
            None,
        );
    }
}

impl AgeableMob for FoxEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for FoxEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl NBTStorage for FoxEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_ageable_nbt(nbt);
            self.write_animal_nbt(nbt);
            nbt.put_bool("Sleeping", self.is_sleeping());
            nbt.put_string("Type", self.variant().name().to_string());
            nbt.put_bool("Sitting", self.is_sitting());
            nbt.put_bool("Crouching", self.is_crouching());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_ageable_nbt(nbt);
            self.read_animal_nbt(nbt);
            self.set_sleeping(nbt.get_bool("Sleeping").unwrap_or(false));
            let variant = nbt
                .get_string("Type")
                .map_or(FoxVariant::Red, FoxVariant::from_name);
            self.set_variant(variant);
            self.set_sitting(nbt.get_bool("Sitting").unwrap_or(false));
            self.set_is_crouching(nbt.get_bool("Crouching").unwrap_or(false));
        })
    }
}

impl Mob for FoxEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            if self.variant.load(Relaxed) == VARIANT_UNSET {
                let entity = &self.mob_entity.living_entity.entity;
                let world = entity.world.load();
                let pos = entity.block_pos.load();
                let variant = if world
                    .get_biome(&pos)
                    .has_tag(&tag::WorldgenBiome::MINECRAFT_SPAWNS_SNOW_FOXES)
                {
                    FoxVariant::Snow
                } else {
                    FoxVariant::Red
                };
                self.set_variant(variant);
            } else {
                // NBT restore already rolled/loaded a valid variant; just resend it so the
                // client has the up-to-date tracked value.
                self.set_variant(self.variant());
            }
            self.mob_entity.living_entity.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::FOX_FLAGS,
                    MetaDataType::BYTE,
                    self.flags.load(Relaxed) as i8,
                )],
                None,
            );
        })
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load();

            let target = self.mob_entity.target.lock().await.clone();
            let target_alive = target.as_ref().is_some_and(|t| t.get_entity().is_alive());
            if !target_alive {
                self.set_is_crouching(false);
                self.set_is_interested(false);
            }

            let in_water = self.mob_entity.living_entity.is_in_water();
            if in_water || target.is_some() || world.is_thundering().await {
                self.wake_up();
            }
            if in_water || self.is_sleeping() {
                self.set_sitting(false);
            }

            if self.is_crouching() {
                let ticks = self.crouch_ticks.load(Relaxed);
                if ticks < FULLY_CROUCHED_TICKS {
                    self.crouch_ticks.store(ticks + 1, Relaxed);
                }
            } else {
                self.crouch_ticks.store(0, Relaxed);
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        self.animal_interact(player, item_stack, Sound::EntityFoxAmbient)
    }
}
