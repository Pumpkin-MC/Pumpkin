use std::sync::{
    Arc, Mutex, Weak,
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
use rand::{RngExt, rng};
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal, breed::BreedGoal,
        climb_on_top_of_powder_snow::ClimbOnTopOfPowderSnowGoal, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, fox_defend_trusted::DefendTrustedTargetGoal,
        fox_faceplant::FoxFaceplantGoal, fox_melee_attack::FoxMeleeAttackGoal,
        fox_pounce::FoxPounceGoal, fox_sleep::FoxSleepGoal, fox_stalk_prey::StalkPreyGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;
use pumpkin_nbt::compound::NbtCompound;

const TEMPT_ITEMS: &[&Item] = &[&Item::SWEET_BERRIES, &Item::GLOW_BERRIES];

/// `Fox.addTrustedEntity`'s slot-fill rule, split out as a free function (mirroring
/// `fox_util::sample_offsets`) so it's testable without a live `FoxEntity`: fills slot 0 if
/// empty, else slot 1 if empty, else leaves both slots as they were (a third trust is dropped,
/// not swapped in).
#[must_use]
const fn fill_trust_slot(mut slots: [Option<Uuid>; 2], uuid: Uuid) -> [Option<Uuid>; 2] {
    if slots[0].is_none() {
        slots[0] = Some(uuid);
    } else if slots[1].is_none() {
        slots[1] = Some(uuid);
    }
    slots
}

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
/// PR-2 scope: `FaceplantGoal` (clearing `isFaceplanted` back to `false` -- the pounce goal could
/// already set it on a snow landing, but nothing cleared it back until this landed); the melee
/// attack goal (gated off while sitting/sleeping/crouching/faceplanted, matching vanilla's
/// `FoxMeleeAttackGoal.canUse` -- the `FOX_BITE` sound on a successful bite is left as a
/// documented simplification, see `fox_melee_attack.rs`); the three `AvoidEntityGoal`
/// registrations (Player/Wolf/`PolarBear` -- vanilla `Fox.java`, not Player/Wolf/Monster as an
/// earlier design pass had it); the two-slot trust list plus `DefendTrustedTargetGoal` and trust
/// inheritance on breeding; and per-variant target-goal ordering (red foxes prefer land prey and
/// turtle eggs over fish; snow foxes prefer fish first -- `register_target_goals`).
///
/// Still deferred: item-in-mouth mechanics (hold/steal/spit/eat) -- this needs a generic mob
/// item-pickup pipeline that doesn't exist yet anywhere in this codebase (confirmed against
/// `PillagerEntity`'s own identical gap, `pumpkin/src/entity/mob/pillager.rs`), plus a
/// food-component query surface on `ItemStack` this pass didn't confirm exists; berries-eating
/// (needs that same deferred mouth-item slot, a `MoveToBlockGoal` analog, and
/// `SweetBerryBushBlock` age-state mutation from mob AI, none wired together yet); and
/// village-stroll (`pumpkin/src/world/village_poi.rs` has low-level POI classification/distance
/// helpers, but no `ServerLevel.isVillage`-equivalent village-boundary query built on top of them
/// yet, which `StrollThroughVillageGoal`'s `canUse` needs).
///
/// Wiki: <https://minecraft.wiki/w/Fox>
pub struct FoxEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    flags: AtomicU8,
    variant: AtomicU8,
    crouch_ticks: AtomicU8,
    /// `Fox.DATA_TRUSTED_ID_0`/`_1`: up to two entities (usually the players that bred this fox
    /// or its ancestor) this fox won't flee from or attack via `DefendTrustedTargetGoal`.
    trusted: Mutex<[Option<Uuid>; 2]>,
    /// Guards the one-time per-variant target-selector registration in `mob_init_data_tracker`
    /// (mirrors `RabbitEntity::evil_goals_registered`'s reasoning: nothing else calls
    /// `mob_init_data_tracker` more than once in practice, but the guard makes that an invariant
    /// instead of an assumption).
    target_goals_registered: std::sync::atomic::AtomicBool,
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
            trusted: Mutex::new([None, None]),
            target_goals_registered: std::sync::atomic::AtomicBool::new(false),
        };
        let mob_arc = Arc::new(this);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let fox_weak = Arc::downgrade(&mob_arc);

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
            // Vanilla registers the three `AvoidEntityGoal`s at priority 4 too (above stalk/
            // pounce/sleep at 5, below tempt/follow-parent) -- kept at the same relative slot.
            goal_selector.add_goal(
                4,
                Box::new(
                    AvoidEntityGoal::new(&EntityType::PLAYER, 16.0, 1.6, 1.4).with_predicate({
                        let fox_weak = fox_weak.clone();
                        Arc::new(move |candidate: &dyn EntityBase| {
                            let Some(fox) = fox_weak.upgrade() else {
                                return false;
                            };
                            if fox.is_defending() {
                                return false;
                            }
                            if fox.trusts(candidate.get_entity().entity_uuid) {
                                return false;
                            }
                            let Some(player) = candidate.get_player() else {
                                return false;
                            };
                            !candidate.get_entity().is_sneaking()
                                && !player.is_creative()
                                && !player.is_spectator()
                        })
                    }),
                ),
            );
            goal_selector.add_goal(
                4,
                Box::new(
                    AvoidEntityGoal::new(&EntityType::WOLF, 8.0, 1.6, 1.4).with_predicate({
                        let fox_weak = fox_weak.clone();
                        Arc::new(move |candidate: &dyn EntityBase| {
                            let Some(fox) = fox_weak.upgrade() else {
                                return false;
                            };
                            !fox.is_defending()
                                && candidate
                                    .get_mob()
                                    .is_some_and(|m| !m.get_mob_entity().is_tamed())
                        })
                    }),
                ),
            );
            goal_selector.add_goal(
                4,
                Box::new(
                    AvoidEntityGoal::new(&EntityType::POLAR_BEAR, 8.0, 1.6, 1.4).with_predicate(
                        Arc::new(move |_candidate: &dyn EntityBase| {
                            fox_weak.upgrade().is_some_and(|fox| !fox.is_defending())
                        }),
                    ),
                ),
            );
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
            target_selector.add_goal(3, DefendTrustedTargetGoal::new());
            // The land/turtle-egg/fish target goals are registered from `mob_init_data_tracker`
            // once the biome variant is known, since their relative ordering depends on it (see
            // `register_target_goals`).
        };

        mob_arc
    }

    /// `Fox.setTargetGoals`: red foxes prefer land prey (chicken/rabbit, vanilla's
    /// `landTargetGoal`) and turtle eggs over fish; snow foxes prefer fish first.
    ///
    /// Vanilla's `landTargetGoal` targets `Animal.class` filtered to chicken/rabbit via a single
    /// `NearestAttackableTargetGoal`; `turtleEggTargetGoal` targets `Turtle` filtered to
    /// `BABY_ON_LAND_SELECTOR`; `fishTargetGoal` targets `AbstractFish` filtered to
    /// `AbstractSchoolingFish` (cod/salmon, not the solitary pufferfish/tropical fish). This
    /// codebase's `ActiveTargetGoal` only matches a single `EntityType`, so each vanilla goal is
    /// registered as 1-2 separate `ActiveTargetGoal`s instead, mirroring how the land-target
    /// split already worked before this method existed.
    fn register_target_goals(&self, variant: FoxVariant) {
        let mut target_selector = self.mob_entity.target_selector.lock().unwrap();

        let (land_prio, fish_prio) = match variant {
            FoxVariant::Red => (4, 6),
            FoxVariant::Snow => (6, 4),
        };

        target_selector.add_goal(
            land_prio,
            ActiveTargetGoal::with_default(&self.mob_entity, &EntityType::CHICKEN, false),
        );
        target_selector.add_goal(
            land_prio,
            ActiveTargetGoal::with_default(&self.mob_entity, &EntityType::RABBIT, false),
        );
        target_selector.add_goal(
            land_prio,
            Box::new(ActiveTargetGoal::new(
                &self.mob_entity,
                &EntityType::TURTLE,
                10,
                false,
                false,
                Some(
                    |target: Arc<crate::entity::living::LivingEntity>, _world| async move {
                        target.entity.age.load(Relaxed) < 0 && !target.is_in_water()
                    },
                ),
            )),
        );

        target_selector.add_goal(
            fish_prio,
            ActiveTargetGoal::with_default(&self.mob_entity, &EntityType::COD, false),
        );
        target_selector.add_goal(
            fish_prio,
            ActiveTargetGoal::with_default(&self.mob_entity, &EntityType::SALMON, false),
        );
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

    /// `Fox.trusts`.
    #[must_use]
    pub fn trusts(&self, uuid: Uuid) -> bool {
        self.trusted.lock().unwrap().contains(&Some(uuid))
    }

    /// `Fox.addTrustedEntity`: fills the first empty of the two trust slots, overwriting neither
    /// if both are already taken (matches vanilla -- a third trust never displaces the first
    /// two).
    pub fn add_trusted_entity(&self, uuid: Uuid) {
        let mut trusted = self.trusted.lock().unwrap();
        *trusted = fill_trust_slot(*trusted, uuid);
    }

    /// `Fox.clearTrusted`.
    pub fn clear_trusted(&self) {
        *self.trusted.lock().unwrap() = [None, None];
    }

    /// `Fox.getTrustedEntities`, flattened to just the UUIDs (callers resolve them against a
    /// `World` themselves, e.g. `DefendTrustedTargetGoal`).
    pub fn trusted_uuids(&self) -> impl Iterator<Item = Uuid> + use<> {
        let trusted = *self.trusted.lock().unwrap();
        trusted.into_iter().flatten()
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
            // `Fox.addAdditionalSaveData`'s `Trusted` is a list of `EntityReference`s; simplified
            // here to up to two flat optional UUID keys since this fox's trust list is already
            // capped at two slots.
            let trusted = *self.trusted.lock().unwrap();
            if let Some(uuid) = trusted[0] {
                nbt.put_uuid("TrustedUuid0", uuid);
            }
            if let Some(uuid) = trusted[1] {
                nbt.put_uuid("TrustedUuid1", uuid);
            }
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
            self.clear_trusted();
            if let Some(uuid) = nbt.get_uuid("TrustedUuid0") {
                self.add_trusted_entity(uuid);
            }
            if let Some(uuid) = nbt.get_uuid("TrustedUuid1") {
                self.add_trusted_entity(uuid);
            }
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
            // `Fox.setTargetGoals`, called once the variant is known either way (freshly rolled
            // above, or already restored from NBT before this ran).
            if !self.target_goals_registered.swap(true, Relaxed) {
                self.register_target_goals(self.variant());
            }
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
                // `Fox.setTarget`: clearing the target also clears `isDefending` -- without this,
                // one `DefendTrustedTargetGoal` trigger would permanently suppress all three
                // `AvoidEntityGoal`s (they all gate on `!isDefending()`).
                if self.is_defending() {
                    self.set_defending(false);
                }
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

    /// `Fox.getBreedOffspring` + `FoxBreedGoal.breed`'s trust-inheritance: the kit's variant is a
    /// coin flip between the two parents', and it trusts whichever parent(s) have a recorded
    /// love-cause player (the player(s) that fed them to start breeding).
    fn create_offspring<'a>(
        &'a self,
        mate: &'a dyn EntityBase,
        world: &'a Arc<World>,
    ) -> EntityBaseFuture<'a, Option<Arc<dyn EntityBase>>> {
        Box::pin(async move {
            let entity = self.get_entity();
            let baby = crate::entity::r#type::from_type(
                entity.entity_type,
                entity.pos.load(),
                world,
                Uuid::new_v4(),
            );

            if let Some(kit) = baby.cast_any().downcast_ref::<Self>() {
                let mate_variant = mate.cast_any().downcast_ref::<Self>().map(Self::variant);
                let variant = if rng().random_bool(0.5) {
                    self.variant()
                } else {
                    mate_variant.unwrap_or_else(|| self.variant())
                };
                kit.set_variant(variant);

                let self_love_cause = self.mob_entity.breeder.load();
                let mate_love_cause = mate
                    .get_mob()
                    .and_then(|m| m.get_mob_entity().breeder.load());

                if let Some(uuid) = self_love_cause {
                    kit.add_trusted_entity(uuid);
                }
                if let Some(uuid) = mate_love_cause
                    && Some(uuid) != self_love_cause
                {
                    kit.add_trusted_entity(uuid);
                }
            }

            Some(baby)
        })
    }
}

#[cfg(test)]
mod trust_slot_tests {
    use super::fill_trust_slot;
    use uuid::Uuid;

    #[test]
    fn fills_first_empty_slot() {
        let a = Uuid::new_v4();
        assert_eq!(fill_trust_slot([None, None], a), [Some(a), None]);
    }

    #[test]
    fn fills_second_slot_once_first_is_taken() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        assert_eq!(fill_trust_slot([Some(a), None], b), [Some(a), Some(b)]);
    }

    #[test]
    fn third_trust_is_dropped_once_both_slots_are_full() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        assert_eq!(fill_trust_slot([Some(a), Some(b)], c), [Some(a), Some(b)]);
    }
}
