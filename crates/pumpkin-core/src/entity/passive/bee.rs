use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{effect::StatusEffect, tracked_data};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::block::entities::{BlockEntity, beehive::BeehiveBlockEntity};
use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::brain::memory::codec::{block_pos_from_tag, block_pos_to_tag};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::custom_sound::CustomSound;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::control::{
        Control, LookControlTrait, flying_move_control::FlyingMoveControl,
        look_control::LookControl,
    },
    ai::goal::{
        active_target::TargetCondition, reset_universal_anger::ResetUniversalAngerGoal,
        revenge::RevengeGoal,
    },
    ai::pathfinder::{Navigator, node::PathType},
    mob::{
        Mob, MobEntity,
        neutral::{NeutralData, NeutralMob},
    },
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;

mod goals;

pub const FLAG_ROLL: u8 = 2;
pub const FLAG_HAS_STUNG: u8 = 4;
pub const FLAG_HAS_NECTAR: u8 = 8;

/// Vanilla `Bee.TICKS_PER_FLAP`, `Mth.ceil(1.4959966F)`.
const TICKS_PER_FLAP: i32 = 2;
const STING_DEATH_COUNTDOWN: i32 = 1200;
const MIN_ATTACK_DIST: f64 = 4.0;
const POISON_SECONDS_NORMAL: i32 = 10;
const POISON_SECONDS_HARD: i32 = 18;
const TOO_FAR_DISTANCE: i32 = 48;
const MIN_FIND_FLOWER_RETRY_COOLDOWN: i32 = 20;
const MAX_FIND_FLOWER_RETRY_COOLDOWN: i32 = 60;
const COOLDOWN_BEFORE_LOCATING_NEW_HIVE: i32 = 200;
const TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME: i32 = 3600;
const MAX_CROPS_GROWABLE: i32 = 10;

fn is_angry(mob: &dyn Mob) -> bool {
    mob.as_neutral().is_some_and(NeutralMob::is_angry)
}

/// Angry and still holding its stinger.
fn can_sting(mob: &dyn Mob) -> bool {
    is_angry(mob)
        && !mob
            .cast_any()
            .downcast_ref::<BeeEntity>()
            .is_some_and(BeeEntity::has_stung)
}

const BEE_HUNTS: &[Hunt] = &[Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true)
    .when(TargetCondition::AngryAt)
    .gated_by(can_sting)];

pub struct BeeEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub neutral_data: NeutralData,
    pub flags: AtomicU8,
    pub ticks_without_nectar: AtomicI32,
    pub cannot_enter_hive_ticks: AtomicI32,
    pub crops_grown_since_pollination: AtomicI32,
    pub time_since_sting: AtomicI32,
    underwater_ticks: AtomicI32,
    hive_pos: AtomicCell<Option<BlockPos>>,
    flower_pos: AtomicCell<Option<BlockPos>>,
    pollinating: AtomicBool,
    flower_cooldown: AtomicI32,
    hive_cooldown: AtomicI32,
}

impl BeeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        *mob_entity
            .move_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(FlyingMoveControl::new(20, true));
        *mob_entity
            .look_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(BeeLookControl::default());
        let navigator = mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *navigator = Navigator::flying();
        navigator.set_can_open_doors(false);
        // FloatGoal enables floating after the navigation is created.
        navigator.set_can_float(true);
        navigator.set_required_path_length(TOO_FAR_DISTANCE as f32);
        navigator.set_pathfinding_malus(PathType::DamageFire, -1.0);
        navigator.set_pathfinding_malus(PathType::Water, -1.0);
        navigator.set_pathfinding_malus(PathType::WaterBorder, 16.0);
        navigator.set_pathfinding_malus(PathType::Cocoa, -1.0);
        navigator.set_pathfinding_malus(PathType::Fence, -1.0);
        let flower_cooldown =
            rand::random_range(MIN_FIND_FLOWER_RETRY_COOLDOWN..=MAX_FIND_FLOWER_RETRY_COOLDOWN);
        let bee = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            neutral_data: NeutralData::default(),
            flags: AtomicU8::new(0),
            ticks_without_nectar: AtomicI32::new(0),
            cannot_enter_hive_ticks: AtomicI32::new(0),
            crops_grown_since_pollination: AtomicI32::new(0),
            time_since_sting: AtomicI32::new(0),
            underwater_ticks: AtomicI32::new(0),
            hive_pos: AtomicCell::new(None),
            flower_pos: AtomicCell::new(None),
            pollinating: AtomicBool::new(false),
            flower_cooldown: AtomicI32::new(flower_cooldown),
            hive_cooldown: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(bee);
        goals::register(&mob_arc);

        {
            let target_selector = &mob_arc.mob_entity.target_selector;

            // Vanilla BeeHurtByOtherGoal.
            target_selector.add_goal(
                1,
                Box::new(
                    RevengeGoal::new(true)
                        .alerting_others()
                        .alerting_in_sight()
                        .continuing_while(is_angry),
                ),
            );
            hunting::apply(target_selector, &mob_arc.mob_entity, BEE_HUNTS);
            target_selector.add_goal(3, ResetUniversalAngerGoal::new(true));
        };

        mob_arc
    }

    #[must_use]
    pub fn has_flag(&self, flag: u8) -> bool {
        (self.flags.load(Ordering::Relaxed) & flag) != 0
    }

    pub fn set_flag(&self, flag: u8, val: bool) {
        let current = self.flags.load(Ordering::Relaxed);
        let new_flags = if val { current | flag } else { current & !flag };
        self.flags.store(new_flags, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::bee::DATA_FLAGS_ID,
            new_flags as i8,
        );
    }

    #[must_use]
    pub fn has_nectar(&self) -> bool {
        self.has_flag(FLAG_HAS_NECTAR)
    }

    pub fn set_has_nectar(&self, val: bool) {
        if val {
            self.ticks_without_nectar.store(0, Ordering::Relaxed);
        }
        self.set_flag(FLAG_HAS_NECTAR, val);
    }

    #[must_use]
    pub fn has_stung(&self) -> bool {
        self.has_flag(FLAG_HAS_STUNG)
    }

    pub fn set_has_stung(&self, val: bool) {
        self.set_flag(FLAG_HAS_STUNG, val);
    }

    #[must_use]
    pub fn is_rolling(&self) -> bool {
        self.has_flag(FLAG_ROLL)
    }

    pub fn set_rolling(&self, val: bool) {
        self.set_flag(FLAG_ROLL, val);
    }

    fn drop_flower(&self) {
        self.flower_pos.store(None);
        self.flower_cooldown.store(
            self.get_random()
                .random_range(MIN_FIND_FLOWER_RETRY_COOLDOWN..=MAX_FIND_FLOWER_RETRY_COOLDOWN),
            Ordering::Relaxed,
        );
    }

    pub fn saved_flower_pos(&self) -> Option<BlockPos> {
        self.flower_pos.load()
    }

    pub fn set_saved_flower_pos(&self, pos: BlockPos) {
        self.flower_pos.store(Some(pos));
    }

    pub fn set_hive_pos(&self, pos: BlockPos) {
        self.hive_pos.store(Some(pos));
    }

    pub fn drop_off_nectar(&self) {
        self.set_has_nectar(false);
        self.crops_grown_since_pollination
            .store(0, Ordering::Relaxed);
    }

    fn closer_than(&self, pos: BlockPos, distance: i32) -> bool {
        pos.to_f64()
            .squared_distance_to_vec(&self.get_entity().block_pos.load().to_f64())
            < f64::from(distance * distance)
    }

    fn hive(&self) -> Option<Arc<dyn BlockEntity>> {
        let pos = self.hive_pos.load()?;
        self.closer_than(pos, TOO_FAR_DISTANCE).then_some(())?;
        self.get_entity()
            .world
            .load()
            .get_block_entity(&pos)
            .filter(|entity| entity.as_any().is::<BeehiveBlockEntity>())
    }

    fn drop_hive(&self) {
        self.hive_pos.store(None);
        self.hive_cooldown
            .store(COOLDOWN_BEFORE_LOCATING_NEW_HIVE, Ordering::Relaxed);
    }

    fn wants_to_enter_hive(&self) -> bool {
        if self.cannot_enter_hive_ticks.load(Ordering::Relaxed) > 0
            || self.pollinating.load(Ordering::Relaxed)
            || self.has_stung()
            || self.mob_entity.get_target().is_some()
        {
            return false;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        (self.has_nectar()
            || self.ticks_without_nectar.load(Ordering::Relaxed)
                > TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME
            || world.bees_stay_in_hive(&entity.block_pos.load()))
            && self.hive().is_none_or(|hive| {
                hive.as_any()
                    .downcast_ref::<BeehiveBlockEntity>()
                    .is_none_or(|hive| !hive.is_fire_nearby(&world))
            })
    }
}

impl AgeableMob for BeeEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for BeeEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_BEE_FOOD)
    }
}

impl NeutralMob for BeeEntity {
    fn updates_anger_in_ai_step(&self) -> bool {
        true
    }

    fn get_neutral_data(&self) -> &NeutralData {
        &self.neutral_data
    }

    fn set_anger_end_time(&self, end_time: i64) {
        self.neutral_data
            .anger_end_time
            .store(end_time, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::bee::DATA_ANGER_END_TIME, end_time);
    }

    /// Bees calm down once the target is gone, unlike the other neutral mobs.
    fn stays_angry_with_target(&self) -> bool {
        false
    }
}

impl CustomSound for BeeEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBeeDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBeeHurt)
    }

    fn sound_volume(&self) -> f32 {
        0.4
    }
}

#[derive(Default)]
struct BeeLookControl(LookControl);

impl Control for BeeLookControl {}

impl LookControlTrait for BeeLookControl {
    fn tick(&mut self, mob: &dyn Mob) {
        if !is_angry(mob) {
            self.0.tick(mob);
        }
    }

    fn look_at_with_range(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        max_yaw_change: f32,
        max_pitch_change: f32,
    ) {
        self.0
            .look_at_with_range(x, y, z, max_yaw_change, max_pitch_change);
    }
}

impl Mob for BeeEntity {
    fn mob_is_flapping(&self) -> bool {
        let entity = self.get_entity();
        !entity.on_ground.load(Ordering::Relaxed)
            && entity.tick_count.load(Ordering::Relaxed) % TICKS_PER_FLAP == 0
    }

    /// Vanilla `Bee` steps silently.
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {}

    fn omnidirectional_air_mover(&self) -> bool {
        true
    }

    fn should_check_fall_damage(&self) -> bool {
        false
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_walk_target_value(&self, pos: &BlockPos) -> f32 {
        if self.get_entity().world.load().get_block_state(pos).is_air() {
            10.0
        } else {
            0.0
        }
    }

    fn is_stable_destination(&self, world: &World, pos: &BlockPos) -> bool {
        !world.get_block_state(&pos.down()).is_air()
    }

    fn look_resets_pitch(&self) -> bool {
        !self.pollinating.load(Ordering::Relaxed)
    }

    fn ticks_navigation(&self) -> bool {
        !self.pollinating.load(Ordering::Relaxed)
    }

    fn conservative_block_read_radius(&self) -> f64 {
        f64::from(TOO_FAR_DISTANCE + 2)
    }

    fn fluid_jump_velocity(&self) -> f64 {
        0.01
    }

    fn mob_hurt_server(
        &self,
        _amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> Option<bool> {
        if self.get_entity().is_invulnerable_to(&damage_type, cause) {
            return Some(false);
        }
        self.pollinating.store(false, Ordering::Relaxed);
        None
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn as_neutral(&self) -> Option<&dyn NeutralMob> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(pos) = self.hive_pos.load() {
            nbt.put("hive_pos", block_pos_to_tag(pos));
        }
        if let Some(pos) = self.flower_pos.load() {
            nbt.put("flower_pos", block_pos_to_tag(pos));
        }
        nbt.put_bool("HasNectar", self.has_nectar());
        nbt.put_bool("HasStung", self.has_stung());
        nbt.put_int(
            "TicksSincePollination",
            self.ticks_without_nectar.load(Ordering::Relaxed),
        );
        nbt.put_int(
            "CannotEnterHiveTicks",
            self.cannot_enter_hive_ticks.load(Ordering::Relaxed),
        );
        nbt.put_int(
            "CropsGrownSincePollination",
            self.crops_grown_since_pollination.load(Ordering::Relaxed),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_has_nectar(nbt.get_bool("HasNectar").unwrap_or_default());
        self.set_has_stung(nbt.get_bool("HasStung").unwrap_or_default());
        self.ticks_without_nectar.store(
            nbt.get_int("TicksSincePollination").unwrap_or_default(),
            Ordering::Relaxed,
        );
        self.cannot_enter_hive_ticks.store(
            nbt.get_int("CannotEnterHiveTicks").unwrap_or_default(),
            Ordering::Relaxed,
        );
        self.crops_grown_since_pollination.store(
            nbt.get_int("CropsGrownSincePollination")
                .unwrap_or_default(),
            Ordering::Relaxed,
        );
        self.hive_pos
            .store(nbt.get("hive_pos").and_then(block_pos_from_tag));
        self.flower_pos
            .store(nbt.get("flower_pos").and_then(block_pos_from_tag));
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn custom_server_ai_step(&self, caller: &dyn EntityBase) {
        if !self.get_entity().is_in_water() {
            self.underwater_ticks.store(0, Ordering::Relaxed);
        } else if self.underwater_ticks.fetch_add(1, Ordering::Relaxed) >= 20 {
            caller.damage(caller, 1.0, DamageType::DROWN);
        }
        if self.has_stung() {
            let time = self.time_since_sting.fetch_add(1, Ordering::Relaxed) + 1;
            if time % 5 == 0
                && self
                    .get_random()
                    .random_range(0..(STING_DEATH_COUNTDOWN - time).clamp(1, STING_DEATH_COUNTDOWN))
                    == 0
            {
                caller.damage(
                    caller,
                    self.mob_entity.living_entity.health.load(),
                    DamageType::GENERIC,
                );
            }
        }
        if !self.has_nectar() {
            self.ticks_without_nectar.fetch_add(1, Ordering::Relaxed);
        }
        self.update_persistent_anger();
    }

    fn mob_ai_step(&self) {
        if self.cannot_enter_hive_ticks.load(Ordering::Relaxed) > 0 {
            self.cannot_enter_hive_ticks.fetch_sub(1, Ordering::Relaxed);
        }
        if self.flower_cooldown.load(Ordering::Relaxed) > 0 {
            self.flower_cooldown.fetch_sub(1, Ordering::Relaxed);
        }
        if self.hive_cooldown.load(Ordering::Relaxed) > 0 {
            self.hive_cooldown.fetch_sub(1, Ordering::Relaxed);
        }
        if self.get_entity().tick_count.load(Ordering::Relaxed) % 20 == 0 && self.hive().is_none() {
            self.hive_pos.store(None);
        }
        self.set_rolling(
            can_sting(self)
                && self.mob_entity.get_target().is_some_and(|target| {
                    target
                        .get_entity()
                        .pos
                        .load()
                        .squared_distance_to_vec(&self.get_entity().pos.load())
                        < MIN_ATTACK_DIST
                }),
        );
    }

    fn do_hurt_target(&self, target: &dyn EntityBase) -> bool {
        let living = &self.mob_entity.living_entity;
        let damage = living.get_attribute_value(&Attributes::ATTACK_DAMAGE) as i32;
        if !target.damage_with_context(
            target,
            damage as f32,
            DamageType::STING,
            None,
            Some(self),
            Some(self),
        ) {
            return false;
        }
        crate::enchantment::EnchantmentHelper::on_post_attack(
            self,
            target.get_entity(),
            &living.held_item(self),
        );
        if let Some(target) = target.get_living_entity() {
            target.set_stinger_count(target.get_stinger_count() + 1);
            let seconds = match self.get_entity().world.load().level_info.load().difficulty {
                Difficulty::Normal => POISON_SECONDS_NORMAL,
                Difficulty::Hard => POISON_SECONDS_HARD,
                _ => 0,
            };
            if seconds > 0 {
                target.add_effect(Effect {
                    effect_type: &StatusEffect::POISON,
                    duration: seconds * 20,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                });
            }
        }
        self.set_has_stung(true);
        self.stop_being_angry();
        self.play_entity_sound(Sound::EntityBeeSting, 1.0, 1.0);
        true
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::bee::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::bee::DATA_FLAGS_ID,
            self.flags.load(Ordering::Relaxed) as i8,
        );
        entity.set_synced_data(
            tracked_data::bee::DATA_ANGER_END_TIME,
            self.get_anger_end_time(),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        use crate::block::blocks::plant::{
            eyeblossom::EyeblossomBlock, wither_rose::WitherRoseBlock,
        };
        let effect = match item_stack.item.id {
            id if id == Item::OPEN_EYEBLOSSOM.id => Some(EyeblossomBlock::bee_interaction_effect()),
            id if id == Item::WITHER_ROSE.id => Some(WitherRoseBlock::bee_interaction_effect()),
            _ => None,
        };
        if self.is_food(item_stack)
            && let Some(effect) = effect
        {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            self.mob_entity.living_entity.add_effect(effect);
            return true;
        }
        self.animal_interact(player, item_stack)
    }
}
