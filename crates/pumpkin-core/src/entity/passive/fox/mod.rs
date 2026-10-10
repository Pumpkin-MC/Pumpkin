use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::data_component_impl::{EquipmentSlot, FoodImpl};
use pumpkin_data::entity::{EntityType, MobCategory};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::ageable::AgeableMob;
use crate::entity::ai::goal::active_target::ActiveTargetGoal;
use crate::entity::ai::goal::avoid_entity::AvoidEntityGoal;
use crate::entity::ai::goal::breed::BreedGoal;
use crate::entity::ai::goal::climb_powder_snow::ClimbOnTopOfPowderSnowGoal;
use crate::entity::ai::goal::escape_danger::EscapeDangerGoal;
use crate::entity::ai::goal::flee_sun::FleeSunGoal;
use crate::entity::ai::goal::follow_parent::FollowParentGoal;
use crate::entity::ai::goal::goal_selector::GoalSelector;
use crate::entity::ai::goal::leap_at_target::LeapAtTargetGoal;
use crate::entity::ai::goal::look_at_entity::LookAtEntityGoal;
use crate::entity::ai::goal::melee_attack::MeleeAttackGoal;
use crate::entity::ai::goal::stroll_through_village::StrollThroughVillageGoal;
use crate::entity::ai::goal::swim::SwimGoal;
use crate::entity::ai::goal::water_avoiding_random_stroll::WaterAvoidingRandomStrollGoal;
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::custom_sound::CustomSound;
use crate::entity::living::LivingEntity;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::passive::animal::Animal;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::world::World;

pub mod goals;
pub use goals::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoxVariant {
    Red = 0,
    Snow = 1,
}

impl FoxVariant {
    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Snow,
            _ => Self::Red,
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "snow" => Self::Snow,
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
    pub fn select_for_biome(biome_name: &str) -> Self {
        let clean_name = biome_name.strip_prefix("minecraft:").unwrap_or(biome_name);
        if pumpkin_data::tag::WorldgenBiome::MINECRAFT_SPAWNS_SNOW_FOXES
            .0
            .contains(&clean_name)
            || clean_name.contains("snow")
            || clean_name.contains("frozen")
            || clean_name.contains("ice")
            || clean_name.contains("grove")
            || clean_name.contains("jagged_peaks")
        {
            Self::Snow
        } else {
            Self::Red
        }
    }
}

pub mod flags {
    pub const SITTING: u8 = 1;
    pub const CROUCHING: u8 = 4;
    pub const INTERESTED: u8 = 8;
    pub const POUNCING: u8 = 16;
    pub const SLEEPING: u8 = 32;
    pub const FACEPLANTED: u8 = 64;
    pub const DEFENDING: u8 = 128;
}

pub struct FoxEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: crate::entity::ageable::AgeableData,
    variant: AtomicU8,
    flags: AtomicU8,
    trusted_0: AtomicCell<Option<Uuid>>,
    trusted_1: AtomicCell<Option<Uuid>>,
    ticks_since_eaten: AtomicI32,
    crouch_amount: AtomicCell<f32>,
    interested_angle: AtomicCell<f32>,
    ambient_sound_time: AtomicI32,
}

impl FoxEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Arc<Self> {
        let world = entity.world.load();
        let biome = world.get_biome(&entity.block_pos.load());
        let variant = FoxVariant::select_for_biome(biome.registry_id);

        let mob_entity = MobEntity::new(entity);
        let mob_arc = Arc::new(Self {
            mob_entity,
            ageable_data: crate::entity::ageable::AgeableData::default(),
            variant: AtomicU8::new(variant.id() as u8),
            flags: AtomicU8::new(0),
            trusted_0: AtomicCell::new(None),
            trusted_1: AtomicCell::new(None),
            ticks_since_eaten: AtomicI32::new(0),
            crouch_amount: AtomicCell::new(0.0),
            interested_angle: AtomicCell::new(0.0),
            ambient_sound_time: AtomicI32::new(-80),
        });

        let fox_weak = Arc::downgrade(&mob_arc);
        let dyn_mob_weak: Weak<dyn Mob> = {
            let dyn_mob: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&dyn_mob)
        };

        mob_arc.init_goals(fox_weak, dyn_mob_weak);
        mob_arc.set_target_goals();
        mob_arc.mob_entity.set_can_pick_up_loot(true);
        mob_arc.populate_default_equipment();

        mob_arc
    }

    fn init_navigator(&self) {
        let mut nav = self
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        nav.set_can_float(true);
        nav.set_required_path_length(32.0);
        nav.set_pathfinding_malus(PathType::WaterBorder, 0.0);
        nav.set_pathfinding_malus(PathType::DangerOther, 0.0);
        nav.set_pathfinding_malus(PathType::DamageOther, 0.0);
    }

    fn add_avoid_goals(goal_selector: &mut GoalSelector) {
        goal_selector.add_goal(
            4,
            Box::new(AvoidEntityGoal::predicated(
                &EntityType::PLAYER,
                16.0,
                1.6,
                1.4,
                |target, mob| {
                    let Some(fox) = mob.cast_any().downcast_ref::<Self>() else {
                        return false;
                    };
                    if fox.is_defending() {
                        return false;
                    }
                    let Some(player) = target.get_player() else {
                        return false;
                    };
                    let gm = player.gamemode.load();
                    gm != GameMode::Creative
                        && gm != GameMode::Spectator
                        && !player.living_entity.entity.sneaking.load(Ordering::Relaxed)
                        && !player.is_sleeping()
                        && !fox.trusts(&player.gameprofile.id)
                },
            )),
        );
        goal_selector.add_goal(
            4,
            Box::new(AvoidEntityGoal::predicated(
                &EntityType::WOLF,
                8.0,
                1.6,
                1.4,
                |target, mob| {
                    let Some(fox) = mob.cast_any().downcast_ref::<Self>() else {
                        return false;
                    };
                    if fox.is_defending() {
                        return false;
                    }
                    if let Some(target_mob) = target.get_mob()
                        && let Some(tamable) = target_mob.as_tamable()
                        && tamable.is_tame()
                    {
                        return false;
                    }
                    true
                },
            )),
        );
        goal_selector.add_goal(
            4,
            Box::new(AvoidEntityGoal::predicated(
                &EntityType::POLAR_BEAR,
                8.0,
                1.6,
                1.4,
                |_target, mob| {
                    let Some(fox) = mob.cast_any().downcast_ref::<Self>() else {
                        return false;
                    };
                    !fox.is_defending()
                },
            )),
        );
    }

    fn init_goals(&self, fox_weak: Weak<Self>, dyn_mob_weak: Weak<dyn Mob>) {
        self.init_navigator();

        let mut goal_selector = self
            .mob_entity
            .goals_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        goal_selector.add_goal(0, Box::new(SwimGoal::default()));
        goal_selector.add_goal(0, Box::new(ClimbOnTopOfPowderSnowGoal::new()));
        goal_selector.add_goal(1, Box::new(FaceplantGoal::new(fox_weak.clone())));
        goal_selector.add_goal(
            2,
            Box::new(EscapeDangerGoal::new(2.2).gated_by(|mob| {
                mob.cast_any()
                    .downcast_ref::<Self>()
                    .is_none_or(|fox| !fox.is_defending())
            })),
        );
        goal_selector.add_goal(3, BreedGoal::new(1.0));
        Self::add_avoid_goals(&mut goal_selector);
        goal_selector.add_goal(5, Box::new(StalkPreyGoal::new(fox_weak.clone())));
        goal_selector.add_goal(6, Box::new(FoxPounceGoal::new(fox_weak.clone())));
        goal_selector.add_goal(6, Box::new(FoxSeekShelterGoal::new(fox_weak.clone(), 1.25)));
        goal_selector.add_goal(
            7,
            Box::new(MeleeAttackGoal::new(1.2, true).gated_by(|mob| {
                mob.cast_any().downcast_ref::<Self>().is_none_or(|fox| {
                    !fox.is_sitting()
                        && !fox.is_sleeping()
                        && !fox.is_crouching()
                        && !fox.is_faceplanted()
                })
            })),
        );
        goal_selector.add_goal(7, Box::new(SleepGoal::new(fox_weak.clone())));
        goal_selector.add_goal(8, Box::new(FollowParentGoal::new(1.25)));
        goal_selector.add_goal(
            9,
            Box::new(StrollThroughVillageGoal::with_search_radius(1.0, 200, 32)),
        );
        goal_selector.add_goal(10, Box::new(FoxEatBerriesGoal::new(fox_weak.clone(), 1.2)));
        goal_selector.add_goal(10, Box::new(LeapAtTargetGoal::new(0.4)));
        goal_selector.add_goal(11, Box::new(WaterAvoidingRandomStrollGoal::new(1.0)));
        goal_selector.add_goal(11, Box::new(FoxSearchForItemsGoal::new(fox_weak.clone())));
        goal_selector.add_goal(
            12,
            LookAtEntityGoal::with_default(dyn_mob_weak, &EntityType::PLAYER, 24.0),
        );
        goal_selector.add_goal(13, Box::new(PerchAndSearchGoal::new(fox_weak.clone())));
        drop(goal_selector);

        let mut target_selector = self
            .mob_entity
            .target_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        target_selector.add_goal(3, Box::new(DefendTrustedTargetGoal::new(fox_weak)));
    }

    fn set_target_goals(&self) {
        let mut target_selector = self
            .mob_entity
            .target_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        target_selector.remove_goals::<ActiveTargetGoal>();

        let (land_priority, fish_priority) = match self.get_variant() {
            FoxVariant::Red => (4, 6),
            FoxVariant::Snow => (6, 4),
        };

        target_selector.add_goal(
            land_priority,
            ActiveTargetGoal::predicated(
                &self.mob_entity,
                10,
                false,
                |target: &LivingEntity, _world: &World| {
                    target.entity.entity_type == &EntityType::CHICKEN
                        || target.entity.entity_type == &EntityType::RABBIT
                },
            ),
        );
        target_selector.add_goal(
            land_priority,
            ActiveTargetGoal::predicated(
                &self.mob_entity,
                10,
                false,
                |target: &LivingEntity, _world: &World| {
                    target.entity.entity_type == &EntityType::TURTLE
                        && target.entity.age.load(Ordering::Relaxed) < 0
                        && !target.entity.is_submerged_in_water()
                },
            ),
        );
        target_selector.add_goal(
            fish_priority,
            ActiveTargetGoal::predicated(
                &self.mob_entity,
                20,
                false,
                |target: &LivingEntity, _world: &World| {
                    target.entity.entity_type == &EntityType::COD
                        || target.entity.entity_type == &EntityType::SALMON
                },
            ),
        );
    }

    fn get_flag(&self, flag: u8) -> bool {
        (self.flags.load(Ordering::Relaxed) & flag) != 0
    }

    fn set_flag(&self, flag: u8, val: bool) {
        let old = self.flags.load(Ordering::Relaxed);
        let new_val = if val { old | flag } else { old & !flag };
        self.flags.store(new_val, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(pumpkin_data::tracked_data::fox::DATA_FLAGS_ID, new_val);
    }

    #[must_use]
    pub fn get_variant(&self) -> FoxVariant {
        FoxVariant::from_id(i32::from(self.variant.load(Ordering::Relaxed)))
    }

    pub fn set_variant(&self, variant: FoxVariant) {
        self.variant.store(variant.id() as u8, Ordering::Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_TYPE_ID,
            VarInt(variant.id()),
        );
        self.set_target_goals();
    }

    pub fn is_sitting(&self) -> bool {
        self.get_flag(flags::SITTING)
    }

    pub fn set_sitting(&self, val: bool) {
        self.set_flag(flags::SITTING, val);
    }

    pub fn is_crouching(&self) -> bool {
        self.get_flag(flags::CROUCHING)
    }

    pub fn set_crouching(&self, val: bool) {
        self.set_flag(flags::CROUCHING, val);
        if !val {
            self.crouch_amount.store(0.0);
        }
    }

    pub fn is_fully_crouched(&self) -> bool {
        self.crouch_amount.load() >= 5.0
    }

    pub fn is_interested(&self) -> bool {
        self.get_flag(flags::INTERESTED)
    }

    pub fn set_interested(&self, val: bool) {
        self.set_flag(flags::INTERESTED, val);
    }

    pub fn is_pouncing(&self) -> bool {
        self.get_flag(flags::POUNCING)
    }

    pub fn set_pouncing(&self, val: bool) {
        self.set_flag(flags::POUNCING, val);
    }

    pub fn is_sleeping(&self) -> bool {
        self.get_flag(flags::SLEEPING)
    }

    pub fn set_sleeping(&self, val: bool) {
        self.set_flag(flags::SLEEPING, val);
    }

    pub fn is_faceplanted(&self) -> bool {
        self.get_flag(flags::FACEPLANTED)
    }

    pub fn set_faceplanted(&self, val: bool) {
        self.set_flag(flags::FACEPLANTED, val);
    }

    pub fn is_defending(&self) -> bool {
        self.get_flag(flags::DEFENDING)
    }

    pub fn set_defending(&self, val: bool) {
        self.set_flag(flags::DEFENDING, val);
    }

    pub fn clear_states(&self) {
        self.set_interested(false);
        self.set_crouching(false);
        self.set_sitting(false);
        self.set_sleeping(false);
        self.set_defending(false);
        self.set_faceplanted(false);
    }

    #[must_use]
    pub fn can_move(&self) -> bool {
        !self.is_sleeping() && !self.is_sitting() && !self.is_faceplanted()
    }

    pub fn wake_up(&self) {
        self.set_sleeping(false);
    }

    pub fn trusts(&self, uuid: &Uuid) -> bool {
        self.trusted_0.load().as_ref() == Some(uuid) || self.trusted_1.load().as_ref() == Some(uuid)
    }

    pub fn add_trusted(&self, uuid: Uuid) {
        if self.trusted_0.load().is_none() {
            self.trusted_0.store(Some(uuid));
            self.get_entity().set_synced_data(
                pumpkin_data::tracked_data::fox::DATA_TRUSTED_ID_0,
                Some(uuid),
            );
        } else if self.trusted_1.load().is_none() && self.trusted_0.load() != Some(uuid) {
            self.trusted_1.store(Some(uuid));
            self.get_entity().set_synced_data(
                pumpkin_data::tracked_data::fox::DATA_TRUSTED_ID_1,
                Some(uuid),
            );
        }
    }

    pub fn clear_trusted(&self) {
        self.trusted_0.store(None);
        self.trusted_1.store(None);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_TRUSTED_ID_0,
            None::<Uuid>,
        );
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_TRUSTED_ID_1,
            None::<Uuid>,
        );
    }

    #[must_use]
    pub fn get_mouth_item(&self) -> ItemStack {
        let equipment = self
            .mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        equipment
            .equipment
            .get(&EquipmentSlot::MAIN_HAND)
            .cloned()
            .unwrap_or_else(|| ItemStack::EMPTY.clone())
    }

    pub fn set_mouth_item(&self, item: ItemStack) {
        let mut equipment = self
            .mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        equipment
            .equipment
            .insert(EquipmentSlot::MAIN_HAND, item.clone());
        drop(equipment);
        self.mob_entity
            .set_guaranteed_drop(&EquipmentSlot::MAIN_HAND);
        self.mob_entity
            .living_entity
            .send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, item)]);
    }

    fn can_eat(&self, item: &ItemStack) -> bool {
        !item.is_empty()
            && item.has_data_component(pumpkin_data::data_component::DataComponent::Food)
            && self.mob_entity.get_target().is_none()
            && self.get_entity().on_ground.load(Ordering::Relaxed)
            && !self.is_sleeping()
    }

    pub fn can_hold_item(&self, new_item: &ItemStack) -> bool {
        let held = self.get_mouth_item();
        if held.is_empty() {
            return true;
        }
        let is_new_food =
            new_item.has_data_component(pumpkin_data::data_component::DataComponent::Food);
        let is_held_food =
            held.has_data_component(pumpkin_data::data_component::DataComponent::Food);
        self.ticks_since_eaten.load(Ordering::Relaxed) > 0 && is_new_food && !is_held_food
    }

    pub fn spit_out_item(&self, item: &ItemStack) {
        if item.is_empty() {
            return;
        }
        let entity = self.get_entity();
        let world = entity.world.load_full();
        let pos = entity.pos.load();
        world.play_sound(Sound::EntityFoxSpit, SoundCategory::Neutral, &pos);
        let look = entity.rotation();
        let spit_pos = Vector3::new(
            pos.x + f64::from(look.x),
            pos.y + 1.0,
            pos.z + f64::from(look.z),
        );
        let item_entity = crate::entity::item::ItemEntity::new(
            Entity::new(world.clone(), spit_pos, &EntityType::ITEM),
            item.clone(),
        );
        item_entity.set_pickup_delay(40);
        world.spawn_entity(Arc::new(item_entity));
    }

    fn play_ambient_sound(&self, world: &World) {
        let pos = self.get_entity().pos.load();
        if self.is_sleeping() {
            world.play_sound(Sound::EntityFoxSleep, SoundCategory::Neutral, &pos);
        } else {
            if !world.is_bright_outside() && rand::rng().random_range(0..10) == 0 {
                let player_nearby = world
                    .get_nearest_player(pos, 16.0, |p| {
                        let gm = p.gamemode.load();
                        gm != GameMode::Spectator
                    })
                    .is_some();
                if !player_nearby {
                    world.play_sound_fine(
                        Sound::EntityFoxScreech,
                        SoundCategory::Neutral,
                        &pos,
                        2.0,
                        1.0,
                    );
                    return;
                }
            }
            world.play_sound(Sound::EntityFoxAmbient, SoundCategory::Neutral, &pos);
        }
    }

    fn eat_food_in_mouth(&self, world: &World, item: &ItemStack) {
        world.play_sound(
            Sound::EntityFoxEat,
            SoundCategory::Neutral,
            &self.get_entity().pos.load(),
        );
        self.mob_entity
            .living_entity
            .apply_consumable_effects(self, item);

        if let Some(food) = item.get_data_component::<FoodImpl>() {
            self.mob_entity.living_entity.heal(food.nutrition as f32);
        }

        let mut next_item = item.clone();
        next_item.item_count = next_item.item_count.saturating_sub(1);
        if next_item.item_count == 0 {
            let remainder_item = match item.get_item().id {
                id if id == Item::MUSHROOM_STEW.id
                    || id == Item::BEETROOT_SOUP.id
                    || id == Item::RABBIT_STEW.id
                    || id == Item::SUSPICIOUS_STEW.id =>
                {
                    Some(&Item::BOWL)
                }
                id if id == Item::HONEY_BOTTLE.id => Some(&Item::GLASS_BOTTLE),
                _ => None,
            };
            if let Some(remainder) = remainder_item {
                self.set_mouth_item(ItemStack::new(1, remainder));
            } else {
                self.set_mouth_item(ItemStack::EMPTY.clone());
            }
        } else {
            self.set_mouth_item(next_item);
        }
    }

    pub fn populate_default_equipment(&self) {
        if rand::random::<f32>() < 0.2 {
            let roll = rand::random::<f32>();
            let item = if roll < 0.05 {
                &Item::EMERALD
            } else if roll < 0.2 {
                &Item::EGG
            } else if roll < 0.4 {
                if rand::random::<bool>() {
                    &Item::RABBIT_FOOT
                } else {
                    &Item::RABBIT_HIDE
                }
            } else if roll < 0.6 {
                &Item::WHEAT
            } else if roll < 0.8 {
                &Item::LEATHER
            } else {
                &Item::FEATHER
            };
            self.set_mouth_item(ItemStack::new(1, item));
        }
    }

    fn tick_eating(&self, world: &World) {
        let mouth_item = self.get_mouth_item();
        if self.can_eat(&mouth_item) {
            let ticks = self.ticks_since_eaten.fetch_add(1, Ordering::Relaxed) + 1;
            if ticks > 600 {
                self.eat_food_in_mouth(world, &mouth_item);
                self.ticks_since_eaten.store(0, Ordering::Relaxed);
            } else if ticks > 560 && rand::rng().random_range(0..10) == 0 {
                world.play_sound(
                    Sound::EntityFoxEat,
                    SoundCategory::Neutral,
                    &self.get_entity().pos.load(),
                );
                world.send_entity_status(
                    self.get_entity(),
                    pumpkin_data::entity::EntityStatus::FoxEat,
                    None,
                );
            }
        } else {
            self.ticks_since_eaten.store(0, Ordering::Relaxed);
        }
    }

    fn tick_touch_item_pickup(&self, world: &Arc<World>) {
        if !self.can_move() {
            return;
        }
        let fox_pos = self.get_entity().pos.load();
        let entities = world.entities.load();
        for candidate in entities.iter() {
            let Some(item_ent) = candidate.get_item_entity() else {
                continue;
            };
            if item_ent.get_pickup_delay() > 0 || !item_ent.get_entity().is_alive() {
                continue;
            }
            let item_pos = item_ent.get_entity().pos.load();
            if fox_pos.squared_distance_to_vec(&item_pos) <= 2.25 {
                let stack = item_ent
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                if !stack.is_empty() && self.can_hold_item(&stack) {
                    let prev_item = self.get_mouth_item();
                    if !prev_item.is_empty() {
                        self.spit_out_item(&prev_item);
                    }
                    if stack.item_count > 1 {
                        let mut remainder = stack.clone();
                        remainder.item_count = remainder.item_count.saturating_sub(1);
                        let dropped_item = crate::entity::item::ItemEntity::new(
                            Entity::new(world.clone(), fox_pos, &EntityType::ITEM),
                            remainder,
                        );
                        world.spawn_entity(Arc::new(dropped_item));
                    }
                    let mut single = stack;
                    single.item_count = 1;
                    self.set_mouth_item(single);
                    self.ticks_since_eaten.store(0, Ordering::Relaxed);
                    item_ent.get_entity().remove();
                    break;
                }
            }
        }
    }

    fn tick_crouch_and_interest(&self) {
        let target = self.mob_entity.get_target();
        if target.as_ref().is_none_or(|t| !t.is_alive()) {
            self.set_crouching(false);
            self.set_interested(false);
        }

        if self.is_crouching() {
            let amount = self.crouch_amount.load();
            let next = (amount + 0.2).min(5.0);
            self.crouch_amount.store(next);
        } else {
            self.crouch_amount.store(0.0);
        }

        let curr_angle = self.interested_angle.load();
        let target_angle = if self.is_interested() { 1.0 } else { 0.0 };
        self.interested_angle
            .store(curr_angle + (target_angle - curr_angle) * 0.4);
    }

    fn fox_tick(&self) {
        let entity = self.get_entity();
        if !entity.is_alive() {
            return;
        }

        let world = entity.world.load_full();
        let in_water = entity.is_in_water();
        let target = self.mob_entity.get_target();

        if in_water || target.is_some() || world.is_thundering() {
            self.wake_up();
        }

        if in_water || self.is_sleeping() {
            self.set_sitting(false);
        }

        if self.is_sleeping() || self.is_crouching() || self.is_sitting() {
            let living = &self.mob_entity.living_entity;
            living.jumping.store(false, Ordering::SeqCst);
            living.movement_input.store(Vector3::new(0.0, 0.0, 0.0));
            living.set_speed(0.0);
        }

        self.tick_eating(&world);

        if entity.is_alive() {
            self.tick_touch_item_pickup(&world);

            let sound_time = self.ambient_sound_time.fetch_add(1, Ordering::Relaxed);
            if sound_time > 0 && rand::rng().random_range(0..1000) < sound_time {
                self.ambient_sound_time.store(-80, Ordering::Relaxed);
                self.play_ambient_sound(&world);
            }
        }

        self.tick_crouch_and_interest();

        if self.is_faceplanted() && rand::random::<f32>() < 0.2 {
            let pos = entity.block_pos.load();
            let state_id = world.get_block_state_id(&pos);
            world.sync_world_event(
                WorldEvent::ParticlesAndSoundDestroyBlock,
                pos,
                state_id.as_u16() as i32,
            );
        }

        if self.is_defending() && rand::random::<f32>() < 0.05 {
            world.play_sound(
                Sound::EntityFoxAggro,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
        }
    }
}

impl AgeableMob for FoxEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }
}

impl Animal for FoxEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        let item = item_stack.get_item();
        item.has_tag(&tag::Item::MINECRAFT_FOX_FOOD)
    }
}

impl CustomSound for FoxEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityFoxDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityFoxHurt)
    }
}

impl Mob for FoxEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_max_look_yaw_change(&self) -> f32 {
        10.0
    }

    fn get_max_head_rotation(&self) -> f32 {
        75.0
    }

    fn mob_set_variant_name(&self, name: &str) {
        self.set_variant(FoxVariant::from_name(name));
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.ageable_ai_step();
    }

    fn post_tick(&self) {
        self.fox_tick();
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        if entity.age.load(Ordering::Relaxed) < 0 {
            entity.set_synced_data(pumpkin_data::tracked_data::fox::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_TYPE_ID,
            VarInt(self.get_variant().id()),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_FLAGS_ID,
            self.flags.load(Ordering::Relaxed),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_TRUSTED_ID_0,
            self.trusted_0.load(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_TRUSTED_ID_1,
            self.trusted_1.load(),
        );
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("Type", self.get_variant().name().to_string());
        nbt.put_bool("Sleeping", self.is_sleeping());
        nbt.put_bool("Sitting", self.is_sitting());
        nbt.put_bool("Crouching", self.is_crouching());

        let mut trusted_list = Vec::new();
        if let Some(uuid) = self.trusted_0.load() {
            let u = uuid.as_u128();
            trusted_list.push(pumpkin_nbt::tag::NbtTag::IntArray(vec![
                (u >> 96) as i32,
                (u >> 64) as i32,
                (u >> 32) as i32,
                u as i32,
            ]));
        }
        if let Some(uuid) = self.trusted_1.load() {
            let u = uuid.as_u128();
            trusted_list.push(pumpkin_nbt::tag::NbtTag::IntArray(vec![
                (u >> 96) as i32,
                (u >> 64) as i32,
                (u >> 32) as i32,
                u as i32,
            ]));
        }
        if !trusted_list.is_empty() {
            nbt.put_list("Trusted", trusted_list);
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(variant_name) = nbt.get_string("Type") {
            self.set_variant(FoxVariant::from_name(variant_name));
        }
        self.set_sleeping(nbt.get_bool("Sleeping").unwrap_or(false));
        self.set_sitting(nbt.get_bool("Sitting").unwrap_or(false));
        self.set_crouching(nbt.get_bool("Crouching").unwrap_or(false));

        self.clear_trusted();
        if let Some(list) = nbt.get_list("Trusted") {
            for entry in list {
                match entry {
                    pumpkin_nbt::tag::NbtTag::IntArray(arr) if arr.len() == 4 => {
                        let value = ((arr[0] as u128) << 96)
                            | (((arr[1] as u32) as u128) << 64)
                            | (((arr[2] as u32) as u128) << 32)
                            | ((arr[3] as u32) as u128);
                        self.add_trusted(Uuid::from_u128(value));
                    }
                    pumpkin_nbt::tag::NbtTag::String(s) => {
                        if let Ok(uuid) = Uuid::parse_str(s) {
                            self.add_trusted(uuid);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack, Sound::EntityFoxAmbient)
    }
}

pub fn is_path_clear(fox_pos: Vector3<f64>, target_pos: Vector3<f64>, world: &World) -> bool {
    let zdiff = target_pos.z - fox_pos.z;
    let xdiff = target_pos.x - fox_pos.x;

    for i in 0..6 {
        let factor = f64::from(i) / 6.0;
        let z = zdiff * factor;
        let x = xdiff * factor;

        for j in 1..4 {
            let check_pos = BlockPos::new(
                (fox_pos.x + x).floor() as i32,
                (fox_pos.y + f64::from(j)).floor() as i32,
                (fox_pos.z + z).floor() as i32,
            );
            let state = world.get_block_state(&check_pos);
            if !state.is_air()
                && !state.replaceable()
                && !state.is_liquid()
                && !state.collision_shapes.is_empty()
            {
                return false;
            }
        }
    }
    true
}

pub fn is_alertable(fox: &FoxEntity, world: &World) -> bool {
    let pos = fox.get_entity().pos.load();
    let entities = world.entities.load();
    for candidate in entities.iter() {
        if candidate.get_entity().entity_uuid == fox.get_entity().entity_uuid {
            continue;
        }
        if !candidate.is_alive() {
            continue;
        }
        let c_entity = candidate.get_entity();
        let c_pos = c_entity.pos.load();
        let dx = c_pos.x - pos.x;
        let dy = c_pos.y - pos.y;
        let dz = c_pos.z - pos.z;

        // Java: boundingBox.inflate(12.0D, 6.0D, 12.0D)
        if (dx * dx + dz * dz) > 144.0 || dy.abs() > 6.0 {
            continue;
        }

        // Foxes do not alert each other
        if *c_entity.entity_type == EntityType::FOX {
            continue;
        }

        // Monsters, chickens, and rabbits always alert
        if c_entity.entity_type.category == &MobCategory::MONSTER
            || *c_entity.entity_type == EntityType::CHICKEN
            || *c_entity.entity_type == EntityType::RABBIT
        {
            return true;
        }

        // Polar bears alert
        if *c_entity.entity_type == EntityType::POLAR_BEAR {
            return true;
        }

        // Tamable animals: tame animals do not alert, untamed do
        if let Some(mob) = candidate.get_mob()
            && let Some(tamable) = mob.as_tamable()
        {
            if !tamable.is_tame() {
                return true;
            }
            continue;
        }

        // Players: check spectator/creative, trusted, sneaking, sleeping
        if *c_entity.entity_type == EntityType::PLAYER
            && let Some(player) = candidate.get_player()
        {
            let gm = player.gamemode.load();
            if gm == GameMode::Creative || gm == GameMode::Spectator {
                continue;
            }
            if fox.trusts(&player.gameprofile.id)
                || player.living_entity.entity.sneaking.load(Ordering::Relaxed)
                || player.is_sleeping()
            {
                continue;
            }
            return true;
        }

        // Other living entities (cows, pigs, sheep, etc.):
        // Java Fox.java:1618: return Fox.this.trusts(target) ? false : !target.isSleeping() && !target.isDiscrete();
        if let Some(living) = candidate.get_living_entity() {
            if fox.trusts(&c_entity.entity_uuid)
                || c_entity.sneaking.load(Ordering::Relaxed)
                || living.is_sleeping()
            {
                continue;
            }
            return true;
        }
    }
    false
}
