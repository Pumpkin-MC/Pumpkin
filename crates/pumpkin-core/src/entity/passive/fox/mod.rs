mod goals;

use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, AtomicU8, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::var_int::VarInt;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::ai::behavior::fleeing::{self, Flee};
use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::goal::active_target::ActiveTargetGoal;
use crate::entity::ai::goal::avoid_entity::not_creative_or_spectator;
use crate::entity::ai::goal::leap_at_target::LeapAtTargetGoal;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::mob::spawn::SpawnGroupData;
use crate::entity::passive::tamable::is_untamed;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;
use goals::{
    DefendTrustedTargetGoal, FaceplantGoal, FoxGoal, PounceGoal, StalkPreyGoal, melee_attack,
};

pub const FOX_FOOD: &[&Item] = &[&Item::SWEET_BERRIES, &Item::GLOW_BERRIES];

const FOX_FLEES: &[Flee] = &[
    Flee::new(4, TargetMatch::Type(&EntityType::PLAYER), 16.0, 1.6, 1.4)
        .when(is_not_defending)
        .matching(is_untrusted_player),
    Flee::new(4, TargetMatch::Type(&EntityType::WOLF), 8.0, 1.6, 1.4)
        .when(is_not_defending)
        .matching(is_untamed_wolf),
    Flee::new(4, TargetMatch::Type(&EntityType::POLAR_BEAR), 8.0, 1.6, 1.4).when(is_not_defending),
];

/// Vanilla `landTargetGoal`, `turtleEggTargetGoal` and `fishTargetGoal`.
const fn land_prey(priority: u8) -> Hunt {
    Hunt::new(
        priority,
        TargetMatch::AnyOf(&[&EntityType::CHICKEN, &EntityType::RABBIT]),
        false,
    )
}

const fn fish_prey(priority: u8) -> Hunt {
    Hunt::new(priority, TargetMatch::FISH, false).every(20)
}

/// Red foxes hunt on land first, snow foxes fish first.
const RED_FOX_HUNTS: &[Hunt] = &[land_prey(4), hunting::baby_turtles(4, false), fish_prey(6)];
const SNOW_FOX_HUNTS: &[Hunt] = &[fish_prey(4), land_prey(6), hunting::baby_turtles(6, false)];

/// Vanilla `MAX_CROUCH_AMOUNT`: a full crouch, ready to pounce.
const MAX_CROUCH_AMOUNT: f32 = 5.0;

fn as_fox(mob: &dyn Mob) -> Option<&FoxEntity> {
    mob.cast_any().downcast_ref::<FoxEntity>()
}

fn is_not_defending(mob: &dyn Mob) -> bool {
    as_fox(mob).is_some_and(|fox| !fox.is_defending())
}

/// Sneaking players and the ones the fox trusts are left alone.
fn is_untrusted_player(mob: &dyn Mob, candidate: &dyn EntityBase) -> bool {
    let entity = candidate.get_entity();
    not_creative_or_spectator(mob, candidate)
        && !entity.is_sneaking()
        && as_fox(mob).is_some_and(|fox| !fox.trusts(entity.entity_uuid))
}

fn is_untamed_wolf(_mob: &dyn Mob, candidate: &dyn EntityBase) -> bool {
    candidate.get_mob().is_some_and(is_untamed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum FoxVariant {
    #[default]
    Red = 0,
    Snow = 1,
}

impl FoxVariant {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Snow,
            _ => Self::Red,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Snow => "snow",
        }
    }

    #[must_use]
    pub fn from_name(s: &str) -> Self {
        match s {
            "snow" | "minecraft:snow" => Self::Snow,
            _ => Self::Red,
        }
    }
}

/// Represents a Fox, a passive/neutral mob.
///
/// Wiki: <https://minecraft.wiki/w/Fox>
pub struct FoxEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub variant: AtomicI32,
    pub flags: AtomicU8,
    /// Vanilla's two trust slots.
    pub trusted: AtomicCell<[Option<Uuid>; 2]>,
    /// Server-side crouch progress, the pounce waits for a full crouch.
    crouch_amount: AtomicCell<f32>,
}

impl FoxEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let fox = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            variant: AtomicI32::new(FoxVariant::Red.id()),
            flags: AtomicU8::new(0),
            trusted: AtomicCell::new([None; 2]),
            crouch_amount: AtomicCell::new(0.0),
        };
        let mob_arc = Arc::new(fox);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            let not_defending: fn(&Self) -> bool = |fox| !fox.is_defending();

            // TODO: ClimbOnTopOfPowderSnowGoal(0), SeekShelterGoal(6), SleepGoal(7),
            // StrollThroughVillageGoal(9), EatBerriesGoal(10), SearchForItemsGoal(11),
            // PerchAndSearchGoal(13).
            goal_selector.add_goal(
                0,
                Box::new(FoxGoal::new(SwimGoal::default()).on_start(Self::clear_states)),
            );
            goal_selector.add_goal(1, Box::new(FaceplantGoal::default()));
            goal_selector.add_goal(
                2,
                Box::new(FoxGoal::new(*EscapeDangerGoal::new(2.2)).start_if(not_defending)),
            );
            goal_selector.add_goal(
                3,
                Box::new(FoxGoal::new(*BreedGoal::new(1.0)).on_start(Self::clear_states)),
            );
            fleeing::apply(goal_selector, FOX_FLEES);
            goal_selector.add_goal(5, Box::new(StalkPreyGoal));
            goal_selector.add_goal(6, Box::new(PounceGoal));
            goal_selector.add_goal(7, Box::new(melee_attack(1.2)));
            goal_selector.add_goal(
                8,
                Box::new(
                    FoxGoal::new(FollowParentGoal::new(1.25))
                        .gated(not_defending)
                        .on_start(Self::clear_states),
                ),
            );
            goal_selector.add_goal(10, Box::new(LeapAtTargetGoal::new(0.4)));
            goal_selector.add_goal(11, Box::new(WanderAroundGoal::water_avoiding(1.0)));
            goal_selector.add_goal(
                12,
                Box::new(
                    FoxGoal::new(*LookAtEntityGoal::with_default(
                        mob_weak,
                        &EntityType::PLAYER,
                        24.0,
                    ))
                    .gated(|fox| !fox.is_faceplanted() && !fox.is_interested()),
                ),
            );

            mob_arc
                .mob_entity
                .target_selector
                .add_goal(3, Box::new(DefendTrustedTargetGoal::new()));
        };

        mob_arc
    }

    #[must_use]
    pub fn get_variant(&self) -> FoxVariant {
        FoxVariant::from_id(self.variant.load(Ordering::Relaxed))
    }

    pub fn set_variant(&self, variant: FoxVariant) {
        self.variant.store(variant.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::fox::TYPE_ID,
            VarInt(variant.id()),
        );
    }

    fn get_flag(&self, flag: u8) -> bool {
        (self.flags.load(Ordering::Relaxed) & flag) != 0
    }

    fn set_flag(&self, flag: u8, value: bool) {
        let current = self.flags.load(Ordering::Relaxed);
        let new_flags = if value {
            current | flag
        } else {
            current & !flag
        };
        self.flags.store(new_flags, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::fox::FLAGS_ID, new_flags);
    }

    #[must_use]
    pub fn is_sitting(&self) -> bool {
        self.get_flag(1)
    }

    pub fn set_sitting(&self, val: bool) {
        self.set_flag(1, val);
    }

    #[must_use]
    pub fn is_crouching(&self) -> bool {
        self.get_flag(4)
    }

    pub fn set_crouching(&self, val: bool) {
        self.set_flag(4, val);
    }

    #[must_use]
    pub fn is_interested(&self) -> bool {
        self.get_flag(8)
    }

    pub fn set_interested(&self, val: bool) {
        self.set_flag(8, val);
    }

    #[must_use]
    pub fn is_pouncing(&self) -> bool {
        self.get_flag(16)
    }

    pub fn set_pouncing(&self, val: bool) {
        self.set_flag(16, val);
    }

    #[must_use]
    pub fn is_sleeping(&self) -> bool {
        self.get_flag(32)
    }

    pub fn set_sleeping(&self, val: bool) {
        self.set_flag(32, val);
    }

    #[must_use]
    pub fn is_faceplanted(&self) -> bool {
        self.get_flag(64)
    }

    pub fn set_faceplanted(&self, val: bool) {
        self.set_flag(64, val);
    }

    #[must_use]
    pub fn is_defending(&self) -> bool {
        self.get_flag(128)
    }

    pub fn set_defending(&self, val: bool) {
        self.set_flag(128, val);
    }

    /// With both slots taken the second is replaced.
    pub fn add_trusted(&self, uuid: Uuid) {
        let trusted = match self.trusted.load() {
            [Some(first), _] => [Some(first), Some(uuid)],
            [None, _] => [Some(uuid), None],
        };
        self.trusted.store(trusted);
    }

    #[must_use]
    pub fn trusts(&self, uuid: Uuid) -> bool {
        self.trusted.load().contains(&Some(uuid))
    }

    #[must_use]
    pub fn is_fully_crouched(&self) -> bool {
        self.crouch_amount.load() == MAX_CROUCH_AMOUNT
    }

    /// Vanilla `setTargetGoals`, run on spawn and load: the hunts in the variant's order.
    fn set_target_goals(&self) {
        let targets = &self.mob_entity.target_selector;
        targets.remove_goal::<ActiveTargetGoal>(self);
        let hunts = match self.get_variant() {
            FoxVariant::Red => RED_FOX_HUNTS,
            FoxVariant::Snow => SNOW_FOX_HUNTS,
        };
        hunting::apply(targets, &self.mob_entity, hunts);
    }

    /// Vanilla `Fox.tick` after the AI step.
    fn tick_states(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        if !self.mob_entity.is_no_ai() {
            let in_water = entity.touching_water.load(Ordering::Relaxed);
            if self.is_sleeping()
                && (in_water || self.mob_entity.get_target().is_some() || is_thundering(&world))
            {
                self.set_sleeping(false);
            }
            if self.is_sitting() && (in_water || self.is_sleeping()) {
                self.set_sitting(false);
            }
            if self.is_faceplanted() && rand::random::<f32>() < 0.2 {
                let pos = entity.block_pos.load();
                let state = world.get_block_state_id(&pos);
                world.sync_world_event(
                    WorldEvent::ParticlesAndSoundDestroyBlock,
                    pos,
                    i32::from(state.as_u16()),
                );
            }
        }
        let crouch = if self.is_crouching() {
            (self.crouch_amount.load() + 0.2).min(MAX_CROUCH_AMOUNT)
        } else {
            0.0
        };
        self.crouch_amount.store(crouch);
    }

    pub fn clear_states(&self) {
        self.set_interested(false);
        self.set_crouching(false);
        self.set_sitting(false);
        self.set_sleeping(false);
        self.set_defending(false);
        self.set_faceplanted(false);
    }
}

impl AgeableMob for FoxEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for FoxEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_FOX_FOOD)
            || FOX_FOOD.iter().any(|i| i.id == item_stack.item.id)
    }

    /// Vanilla `Fox.getBreedOffspring`: the cub takes either parent's variant.
    /// No `finalizeSpawn`, so like vanilla its hunts wait for the next load.
    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = Self::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            &EntityType::FOX,
        ));
        let partner_variant = partner
            .cast_any()
            .downcast_ref::<Self>()
            .map_or_else(|| self.get_variant(), Self::get_variant);
        baby.set_variant(if self.get_random().random::<bool>() {
            self.get_variant()
        } else {
            partner_variant
        });
        baby
    }
}

/// Vanilla `Level.isThundering`.
fn is_thundering(world: &World) -> bool {
    let weather = world
        .weather
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    weather.thunder_level * weather.rain_level > 0.9
}

impl Mob for FoxEntity {
    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let (variant, size) = if let Some(SpawnGroupData::Fox { variant, size }) = group_data {
            (variant, size)
        } else {
            let biome = world.get_biome(&self.get_entity().block_pos.load());
            let bare = biome
                .registry_id
                .strip_prefix("minecraft:")
                .unwrap_or(biome.registry_id);
            let snowy = tag::WorldgenBiome::MINECRAFT_SPAWNS_SNOW_FOXES
                .0
                .contains(&bare);
            (
                if snowy {
                    FoxVariant::Snow
                } else {
                    FoxVariant::Red
                },
                0,
            )
        };
        self.set_variant(variant);
        if size >= 2 {
            self.get_entity().set_age(-24000);
        }
        self.set_target_goals();
        self.mob_entity.finalize_spawn_base();
        Some(SpawnGroupData::Fox {
            variant,
            size: size + 1,
        })
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        // Vanilla `aiStep`: a lost target ends the stalk.
        if !self.mob_entity.is_no_ai()
            && (self.is_crouching() || self.is_interested())
            && self
                .mob_entity
                .get_target()
                .is_none_or(|target| !target.get_entity().is_alive())
        {
            self.set_crouching(false);
            self.set_interested(false);
        }
    }

    fn post_tick(&self) {
        self.tick_states();
        if self.is_defending() && rand::random::<f32>() < 0.05 {
            self.get_entity().play_sound(Sound::EntityFoxAggro);
        }
    }

    fn apply_mob_target(&self, target: Option<Arc<dyn EntityBase>>) {
        if target.is_none() && self.is_defending() {
            self.set_defending(false);
        }
        self.mob_entity.set_target(target);
    }

    fn look_resets_pitch(&self) -> bool {
        !self.is_pouncing()
            && !self.is_crouching()
            && !self.is_interested()
            && !self.is_faceplanted()
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("Type", self.get_variant().as_str().to_string());
        nbt.put_bool("Sleeping", self.is_sleeping());
        nbt.put_bool("Sitting", self.is_sitting());
        nbt.put_bool("Crouching", self.is_crouching());
        let trusted = self.trusted.load().into_iter().flatten().map(NbtTag::uuid);
        nbt.put_list("Trusted", trusted.collect());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(variant_str) = nbt.get_string("Type") {
            self.set_variant(FoxVariant::from_name(variant_str));
        }
        if let Some(sleeping) = nbt.get_bool("Sleeping") {
            self.set_sleeping(sleeping);
        }
        if let Some(sitting) = nbt.get_bool("Sitting") {
            self.set_sitting(sitting);
        }
        if let Some(crouching) = nbt.get_bool("Crouching") {
            self.set_crouching(crouching);
        }
        self.trusted.store([None; 2]);
        for uuid in nbt
            .get_list("Trusted")
            .unwrap_or_default()
            .iter()
            .filter_map(NbtTag::extract_uuid)
        {
            self.add_trusted(uuid);
        }
        self.set_target_goals();
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_set_variant_name(&self, name: &str) {
        self.set_variant(FoxVariant::from_name(name));
    }

    fn on_bred_by(&self, player: Uuid) {
        self.add_trusted(player);
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::fox::BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::fox::TYPE_ID,
            VarInt(self.get_variant().id()),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::fox::FLAGS_ID,
            self.flags.load(Ordering::Relaxed),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack, Sound::EntityFoxAmbient)
    }
}
