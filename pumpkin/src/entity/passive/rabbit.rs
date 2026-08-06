use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering},
};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_data::{
    entity::{EntityType, MobCategory},
    item::Item,
};
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        active_target::ActiveTargetGoal, breed::BreedGoal,
        climb_on_top_of_powder_snow::ClimbOnTopOfPowderSnowGoal, escape_danger::EscapeDangerGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
        rabbit_avoid_entity::RabbitAvoidEntityGoal, raid_garden::RaidGardenGoal,
        revenge::RevengeGoal, swim::SwimGoal, tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    attributes::{Modifier, ModifierOperation},
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::text::TextComponent;

const TEMPT_ITEMS: &[&Item] = &[&Item::CARROT, &Item::GOLDEN_CARROT, &Item::DANDELION];

/// Vanilla `Rabbit.Variant`. Ids match `Variant.LEGACY_CODEC` (`RabbitType` NBT int).
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RabbitVariant {
    Brown = 0,
    White = 1,
    Black = 2,
    WhiteSplotched = 3,
    Gold = 4,
    Salt = 5,
    Evil = 99,
}

/// Sentinel meaning "no variant rolled yet"; not a valid vanilla id.
const VARIANT_UNSET: u8 = 0xFF;

impl From<u8> for RabbitVariant {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::White,
            2 => Self::Black,
            3 => Self::WhiteSplotched,
            4 => Self::Gold,
            5 => Self::Salt,
            99 => Self::Evil,
            _ => Self::Brown,
        }
    }
}

/// Vanilla `Rabbit.getRandomRabbitVariant`.
///
/// `biome` is `None` when the position's biome could not be resolved. Both gates are positive
/// membership tests over fixed tags, so an unresolved biome is in neither and this falls to the
/// roll-based Brown/Salt/Black branch - the branch vanilla uses for every biome carrying neither
/// tag, not an invented biome. A default is unavoidable rather than preferred here:
/// `init_data_tracker` runs exactly once at spawn and is never retried, so declining to pick
/// would persist the `VARIANT_UNSET` sentinel as the rabbit's tracked variant.
fn get_random_rabbit_variant(biome: Option<&'static pumpkin_data::biome::Biome>) -> RabbitVariant {
    let roll = rand::random_range(0..100);
    let has = |t: &'static tag::Tag| biome.is_some_and(|b| b.has_tag(t));

    if has(&tag::WorldgenBiome::MINECRAFT_SPAWNS_WHITE_RABBITS) {
        if roll < 80 {
            RabbitVariant::White
        } else {
            RabbitVariant::WhiteSplotched
        }
    } else if has(&tag::WorldgenBiome::MINECRAFT_SPAWNS_GOLD_RABBITS) {
        RabbitVariant::Gold
    } else if roll < 50 {
        RabbitVariant::Brown
    } else if roll < 90 {
        RabbitVariant::Salt
    } else {
        RabbitVariant::Black
    }
}

/// Vanilla `Rabbit.setLandingDelay` (lines 242-248): a rabbit that just landed waits 10 ticks
/// before hopping again, but only 1 tick once its speed modifier reaches 2.2 - the flee/panic
/// speed (`FLEE_SPEED_MOD`, line 72), which is what makes a fleeing rabbit hop near-continuously.
#[must_use]
const fn landing_delay_for_speed(speed: f64) -> i32 {
    if speed < 2.2 { 10 } else { 1 }
}

/// Vanilla `customServerAiStep` lines 182-187: `moreCarrotTicks -= random.nextInt(3)`, floored
/// at 0, and only while already positive.
#[must_use]
const fn decay_more_carrot_ticks(current: i32, roll: i32) -> i32 {
    if current <= 0 {
        return current;
    }
    let next = current - roll;
    if next < 0 { 0 } else { next }
}

pub struct RabbitEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    variant: AtomicU8,
    /// Vanilla `Rabbit.jumpDelayTicks`. Ticks remaining before the rabbit may start another
    /// hop; refreshed by `set_landing_delay` on every landing.
    jump_delay_ticks: AtomicI32,
    /// Vanilla `Rabbit.wasOnGround`.
    was_on_ground: AtomicBool,
    /// Vanilla `Rabbit.moreCarrotTicks`, persisted as the `MoreCarrotTicks` NBT int
    /// (`addAdditionalSaveData` line 275 / `readAdditionalSaveData` line 282).
    more_carrot_ticks: AtomicI32,
    /// Guards the one-time goal/target-selector registration in `set_variant(Evil)`. Without
    /// this, an evil kit gets `set_variant` called twice (once explicitly by
    /// `create_offspring`, once again by `mob_init_data_tracker`'s NBT-restore branch after
    /// spawning) and would otherwise register duplicate `MeleeAttackGoal`/`RevengeGoal`/
    /// `ActiveTargetGoal`s.
    evil_goals_registered: std::sync::atomic::AtomicBool,
}

impl RabbitEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let this = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            variant: AtomicU8::new(VARIANT_UNSET),
            jump_delay_ticks: AtomicI32::new(0),
            was_on_ground: AtomicBool::new(true),
            more_carrot_ticks: AtomicI32::new(0),
            evil_goals_registered: std::sync::atomic::AtomicBool::new(false),
        };
        let mob_arc = Arc::new(this);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let rabbit_weak = Arc::downgrade(&mob_arc);
        let raid_weak = rabbit_weak.clone();

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, ClimbOnTopOfPowderSnowGoal::new());
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.2));
            goal_selector.add_goal(2, BreedGoal::new(0.8));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.0, TEMPT_ITEMS, false)));
            goal_selector.add_goal(
                4,
                RabbitAvoidEntityGoal::new(&EntityType::PLAYER, 8.0, 2.2, 2.2, rabbit_weak.clone()),
            );
            goal_selector.add_goal(
                4,
                RabbitAvoidEntityGoal::new(&EntityType::WOLF, 10.0, 2.2, 2.2, rabbit_weak.clone()),
            );
            goal_selector.add_goal(
                4,
                RabbitAvoidEntityGoal::new_for_category(
                    &MobCategory::MONSTER,
                    4.0,
                    2.2,
                    2.2,
                    rabbit_weak,
                ),
            );
            goal_selector.add_goal(5, RaidGardenGoal::new(0.7, raid_weak));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                11,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );
        };

        mob_arc
    }

    /// Vanilla `Rabbit.setJumping` (line 157): sets the physics jump flag and, on the rising
    /// edge, plays `entity.rabbit.jump`.
    fn set_jumping(&self, jumping: bool) {
        self.mob_entity
            .living_entity
            .jumping
            .store(jumping, Ordering::SeqCst);
        if jumping {
            self.mob_entity
                .living_entity
                .entity
                .play_sound(Sound::EntityRabbitJump);
        }
    }

    /// Vanilla `Rabbit.startJumping` (line 164). The `jumpDuration`/`jumpTicks` pair it also
    /// sets drives only the client-side hop animation (`getJumpCompletion`, line 147) and the
    /// `aiStep` teardown at line 258; neither is ported, so this sets the flag alone.
    fn start_jumping(&self) {
        self.set_jumping(true);
    }

    /// Vanilla `Rabbit.setLandingDelay` (line 242): 10 ticks normally, 1 tick when the active
    /// speed modifier is at least 2.2 (i.e. while fleeing or panicking).
    fn set_landing_delay(&self) {
        let speed = self
            .mob_entity
            .navigator
            .lock()
            .unwrap()
            .speed()
            .unwrap_or(0.0);
        self.jump_delay_ticks
            .store(landing_delay_for_speed(speed), Ordering::Relaxed);
    }

    /// Vanilla `Rabbit.wantsMoreFood` (line 397). Read by `RaidGardenGoal`.
    #[must_use]
    pub fn wants_more_food(&self) -> bool {
        self.more_carrot_ticks.load(Ordering::Relaxed) <= 0
    }

    /// Vanilla `RaidGardenGoal.tick` line 575: `this.rabbit.moreCarrotTicks = 40`
    /// (`MORE_CARROTS_DELAY`, line 80).
    pub fn set_more_carrot_delay(&self) {
        self.more_carrot_ticks.store(40, Ordering::Relaxed);
    }

    #[must_use]
    pub fn get_variant(&self) -> RabbitVariant {
        RabbitVariant::from(self.variant.load(Ordering::Relaxed))
    }

    /// Vanilla `Rabbit.setVariant`. Un-setting `EVIL` is not implemented (vanilla itself only
    /// removes the transient attack-damage modifier on un-set and never actually un-sets a
    /// rabbit's variant in practice -- it's set once at spawn/NBT-read and never reverted).
    ///
    /// Deferred: no entity-metadata sync for the variant byte. `TrackedData::RABBIT_TYPE` is
    /// `255` (absent) for the protocol versions this server currently targets -- vanilla itself
    /// moved rabbit variant to a `DataComponents.RABBIT_VARIANT` data component in newer
    /// versions, which Pumpkin's client metadata sync does not yet cover for this mob. State is
    /// still tracked server-side (goals/attributes/NBT/genetics all work); only the client-side
    /// visual variant is unaddressed.
    pub fn set_variant(&self, variant: RabbitVariant) {
        self.variant.store(variant as u8, Ordering::Relaxed);

        if variant == RabbitVariant::Evil {
            self.mob_entity
                .living_entity
                .update_attribute(&Attributes::ARMOR, |inst| {
                    inst.base_value = 8.0;
                });
            self.mob_entity
                .living_entity
                .update_attribute(&Attributes::ATTACK_DAMAGE, |inst| {
                    inst.add_or_replace_modifier(Modifier {
                        id: "evil".to_string(),
                        amount: 5.0,
                        operation: ModifierOperation::Add,
                    });
                });

            if !self.evil_goals_registered.swap(true, Ordering::Relaxed) {
                self.mob_entity
                    .goals_selector
                    .lock()
                    .unwrap()
                    .add_goal(4, Box::new(MeleeAttackGoal::new(1.4, true)));

                let mut target_selector = self.mob_entity.target_selector.lock().unwrap();
                target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
                target_selector.add_goal(
                    2,
                    ActiveTargetGoal::with_default(&self.mob_entity, &EntityType::PLAYER, true),
                );
                target_selector.add_goal(
                    2,
                    ActiveTargetGoal::with_default(&self.mob_entity, &EntityType::WOLF, true),
                );
                drop(target_selector);
            }

            let entity = &self.mob_entity.living_entity.entity;
            if (**entity.custom_name.load()).is_none() {
                entity.set_custom_name(TextComponent::translate(
                    "entity.minecraft.killer_bunny",
                    [],
                ));
            }
        }
    }
}

impl AgeableMob for RabbitEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for RabbitEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl NBTStorage for RabbitEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_ageable_nbt(nbt);
            self.write_animal_nbt(nbt);
            nbt.put_int("RabbitType", self.variant.load(Ordering::Relaxed) as i32);
            // Vanilla `addAdditionalSaveData` line 275.
            nbt.put_int(
                "MoreCarrotTicks",
                self.more_carrot_ticks.load(Ordering::Relaxed),
            );
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_ageable_nbt(nbt);
            self.read_animal_nbt(nbt);
            if let Some(variant) = nbt.get_int("RabbitType") {
                self.variant.store(variant as u8, Ordering::Relaxed);
            }
            // Vanilla `readAdditionalSaveData` line 282.
            self.more_carrot_ticks.store(
                nbt.get_int("MoreCarrotTicks").unwrap_or(0),
                Ordering::Relaxed,
            );
        })
    }
}

impl Mob for RabbitEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Port of vanilla `Rabbit.customServerAiStep` (lines 177-223).
    ///
    /// This lives here, not in a goal, because vanilla's hop logic is not a goal: it runs
    /// unconditionally every server AI step, outside the goal selector. The previous
    /// `RabbitHopGoal` modelled it as a `Controls::JUMP` goal whose `can_start` was
    /// unconditionally `true`, which permanently held the JUMP control and had to be given an
    /// artificial priority to avoid locking out `ClimbOnTopOfPowderSnowGoal`. Moving it here
    /// removes that invented contention entirely.
    ///
    /// Not ported: vanilla's `facePoint` yaw snap (lines 214/198) and `getJumpPower`
    /// (line 110); see the deferred list in the branch commit message.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let living = &self.mob_entity.living_entity;
            let entity = &living.entity;

            if self.jump_delay_ticks.load(Ordering::Relaxed) > 0 {
                self.jump_delay_ticks.fetch_sub(1, Ordering::Relaxed);
            }

            // Vanilla lines 182-187: decay by `random.nextInt(3)` every tick, floored at 0.
            let carrots = self.more_carrot_ticks.load(Ordering::Relaxed);
            if carrots > 0 {
                self.more_carrot_ticks.store(
                    decay_more_carrot_ticks(carrots, rand::random_range(0..3)),
                    Ordering::Relaxed,
                );
            }

            let on_ground = entity.on_ground.load(Ordering::SeqCst);
            if on_ground {
                if !self.was_on_ground.load(Ordering::Relaxed) {
                    // Vanilla `checkLandingDelay` (line 250).
                    self.set_jumping(false);
                    self.set_landing_delay();
                }

                let mut evil_lunged = false;

                // Vanilla lines 195-203: the killer bunny lunges at a nearby target the moment
                // its landing delay expires, bypassing the usual "needs a navigation
                // destination" gate below.
                if self.get_variant() == RabbitVariant::Evil
                    && self.jump_delay_ticks.load(Ordering::Relaxed) == 0
                {
                    let target = self.mob_entity.target.lock().await.clone();
                    if let Some(target) = target {
                        let dist_sq = target
                            .get_entity()
                            .pos
                            .load()
                            .squared_distance_to_vec(&entity.pos.load());
                        if dist_sq < 16.0 {
                            self.start_jumping();
                            evil_lunged = true;
                        }
                    }
                }

                // Vanilla lines 206-216. `moveControl.hasWanted()` is "the move control has a
                // destination it is still travelling to"; the closest equivalent here is a
                // non-idle navigator.
                if !evil_lunged
                    && !living.jumping.load(Ordering::SeqCst)
                    && self.jump_delay_ticks.load(Ordering::Relaxed) == 0
                    && !self.mob_entity.navigator.lock().unwrap().is_idle()
                {
                    self.start_jumping();
                }
            }

            // Vanilla line 222.
            self.was_on_ground.store(on_ground, Ordering::Relaxed);
        })
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;

            if self.variant.load(Ordering::Relaxed) == VARIANT_UNSET {
                let world = entity.world.load();
                let pos = entity.block_pos.load();
                let variant = get_random_rabbit_variant(world.get_biome(&pos));
                self.set_variant(variant);
            } else {
                self.set_variant(self.get_variant());
            }

            // This override replaces (rather than chains to) `Mob::mob_init_data_tracker`'s
            // default body, which sends `BABY_ID` for age < 0 -- replicate that here so bred
            // kits (spawned at age -24000) still render baby-sized.
            if entity.age.load(Ordering::Relaxed) < 0 {
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        self.animal_interact(player, item_stack, Sound::EntityRabbitAmbient)
    }

    /// Vanilla `Rabbit.getBreedOffspring`: 95% chance (`random.nextInt(20) != 0`) to inherit a
    /// parent's variant (50/50 which parent), else roll a fresh biome-based variant.
    fn create_offspring<'a>(
        &'a self,
        mate: &'a dyn EntityBase,
        world: &'a Arc<crate::world::World>,
    ) -> EntityBaseFuture<'a, Option<Arc<dyn EntityBase>>> {
        Box::pin(async move {
            let entity = self.get_entity();
            let baby = crate::entity::r#type::from_type(
                entity.entity_type,
                entity.pos.load(),
                world,
                uuid::Uuid::new_v4(),
            );

            if let Some(kit) = baby.cast_any().downcast_ref::<Self>() {
                let variant = if rand::random_range(0..20) != 0 {
                    let mate_rabbit = mate.cast_any().downcast_ref::<Self>();
                    if rand::random_bool(0.5) {
                        self.get_variant()
                    } else {
                        mate_rabbit.map_or_else(|| self.get_variant(), Self::get_variant)
                    }
                } else {
                    get_random_rabbit_variant(world.get_biome(&entity.block_pos.load()))
                };
                kit.set_variant(variant);
            }

            Some(baby)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vanilla `Rabbit.setLandingDelay` (lines 242-248). The pre-existing `RabbitHopGoal` this
    /// replaces hardcoded 10 unconditionally and its doc comment claimed the fast case was 3;
    /// vanilla's fast case is 1, and the branch is on `< 2.2`, not `<= 2.2`.
    #[test]
    fn landing_delay_is_ten_when_slow_and_one_at_flee_speed() {
        assert_eq!(landing_delay_for_speed(0.0), 10);
        assert_eq!(landing_delay_for_speed(0.6), 10);
        assert_eq!(landing_delay_for_speed(1.0), 10);
        // 2.2 is `FLEE_SPEED_MOD`; vanilla's `else` branch owns the boundary itself.
        assert_eq!(landing_delay_for_speed(2.2), 1);
        assert_eq!(landing_delay_for_speed(3.0), 1);
    }

    /// Vanilla `customServerAiStep` lines 182-187.
    #[test]
    fn more_carrot_ticks_decay_floors_at_zero_and_ignores_non_positive() {
        assert_eq!(decay_more_carrot_ticks(40, 2), 38);
        assert_eq!(decay_more_carrot_ticks(40, 0), 40);
        assert_eq!(decay_more_carrot_ticks(1, 2), 0);
        // Never decays below zero, and a zero/negative counter is left untouched rather than
        // being driven further negative every tick.
        assert_eq!(decay_more_carrot_ticks(0, 2), 0);
    }

    /// Vanilla `MORE_CARROTS_DELAY` (line 80) is 40, and `wantsMoreFood` (line 397) is
    /// `moreCarrotTicks <= 0`, so a rabbit that just ate needs at least 20 ticks (40 decayed by
    /// the maximum roll of 2 each tick) before it will raid again.
    #[test]
    fn full_carrot_delay_takes_at_least_twenty_ticks_to_expire() {
        let mut ticks = 40;
        let mut elapsed = 0;
        while ticks > 0 {
            ticks = decay_more_carrot_ticks(ticks, 2);
            elapsed += 1;
        }
        assert_eq!(elapsed, 20);
    }
}
