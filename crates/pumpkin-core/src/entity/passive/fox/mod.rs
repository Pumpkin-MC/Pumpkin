mod goals;

use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, AtomicU8, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::data_component_impl::{
    ConsumableImpl, ConsumeEffect, EquipmentSlot, UseRemainderImpl,
};
use pumpkin_data::entity::{EntityPose, EntityType, MobCategory};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::ai::behavior::fleeing::{self, Flee};
use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::goal::active_target::ActiveTargetGoal;
use crate::entity::ai::goal::avoid_entity::not_creative_or_spectator;
use crate::entity::ai::goal::climb_on_top_of_powder_snow::ClimbOnTopOfPowderSnowGoal;
use crate::entity::ai::goal::leap_at_target::LeapAtTargetGoal;
use crate::entity::ai::goal::stroll_through_village::StrollThroughVillageGoal;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::custom_sound::CustomSound;
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
    DefendTrustedTargetGoal, FaceplantGoal, FoxEatBerriesGoal, FoxGoal, FoxSearchForItemsGoal,
    FoxSeekShelterGoal, PerchAndSearchGoal, PounceGoal, SleepGoal, StalkPreyGoal, melee_attack,
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
    pub ticks_since_eaten: AtomicI32,
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
            ticks_since_eaten: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(fox);
        mob_arc.mob_entity.set_can_pick_up_loot(true);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            let not_defending: fn(&Self) -> bool = |fox| !fox.is_defending();

            goal_selector.add_goal(
                0,
                Box::new(FoxGoal::new(SwimGoal::default()).on_start(Self::clear_states)),
            );
            goal_selector.add_goal(0, Box::new(ClimbOnTopOfPowderSnowGoal));
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
            goal_selector.add_goal(
                6,
                Box::new(FoxGoal::new(FoxSeekShelterGoal::new(1.25)).on_start(Self::clear_states)),
            );
            goal_selector.add_goal(7, Box::new(melee_attack(1.2)));
            goal_selector.add_goal(7, Box::new(SleepGoal::default()));
            goal_selector.add_goal(
                8,
                Box::new(
                    FoxGoal::new(FollowParentGoal::new(1.25))
                        .gated(not_defending)
                        .on_start(Self::clear_states),
                ),
            );
            goal_selector.add_goal(
                9,
                Box::new(
                    FoxGoal::new(StrollThroughVillageGoal::new(1.0, 200))
                        .start_if(|fox| {
                            fox.can_move()
                                && !fox.is_defending()
                                && fox.mob_entity.get_target().is_none()
                        })
                        .on_start(Self::clear_states),
                ),
            );
            goal_selector.add_goal(10, Box::new(FoxEatBerriesGoal::new(1.2)));
            goal_selector.add_goal(10, Box::new(LeapAtTargetGoal::new(0.4)));
            goal_selector.add_goal(11, Box::new(WanderAroundGoal::water_avoiding(1.0)));
            goal_selector.add_goal(11, Box::new(FoxSearchForItemsGoal::default()));
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
            goal_selector.add_goal(13, Box::new(PerchAndSearchGoal::default()));

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

    #[must_use]
    pub fn can_move(&self) -> bool {
        !self.is_sleeping() && !self.is_sitting() && !self.is_faceplanted()
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
        if new_item.is_empty() {
            return false;
        }
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
        item_entity.set_thrower(entity.entity_uuid);
        world.spawn_entity(Arc::new(item_entity));
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
        self.mob_entity
            .living_entity
            .apply_suspicious_stew_effects(item);

        let mut next_item = item.clone();
        let count_before = next_item.item_count;
        next_item.item_count = next_item.item_count.saturating_sub(1);
        let world_arc = self.get_entity().world.load_full();
        let pos = self.get_entity().block_pos.load();
        let remaining = match item.get_data_component::<UseRemainderImpl>() {
            Some(use_remainder) => {
                use_remainder.convert_into_remainder(next_item, count_before, false, |extra| {
                    world_arc.drop_stack(&pos, extra);
                })
            }
            None => next_item,
        };
        self.set_mouth_item(remaining);
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
            self.mob_entity
                .set_guaranteed_drop(&EquipmentSlot::MAIN_HAND);
        }
    }

    fn tick_eating(&self, world: &World) {
        let ticks = self.ticks_since_eaten.fetch_add(1, Ordering::Relaxed) + 1;
        let mouth_item = self.get_mouth_item();
        if self.can_eat(&mouth_item) {
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
                    pumpkin_protocol::bedrock::server::ActorEventID::None,
                );
            }
        }
    }

    pub fn pick_up_item_stack(&self, candidate: &dyn EntityBase, world: &Arc<World>) -> bool {
        let Some(item_ent) = candidate.get_item_entity() else {
            return false;
        };
        if item_ent.get_pickup_delay() > 0 || !item_ent.get_entity().is_alive() {
            return false;
        }
        let stack = item_ent
            .get_item_stack()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if stack.is_empty() || !self.can_hold_item(&stack) {
            return false;
        }
        let living = &self.mob_entity.living_entity;
        if !living.pickup(item_ent.get_entity(), 1) {
            return false;
        }
        living.on_item_pickup(item_ent);
        let live_stack = item_ent
            .get_item_stack()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if live_stack.is_empty() {
            item_ent.get_entity().remove();
            return false;
        }
        if !self.can_hold_item(&live_stack) {
            return false;
        }
        let fox_pos = self.get_entity().pos.load();
        let prev_item = self.get_mouth_item();
        if !prev_item.is_empty() {
            self.spit_out_item(&prev_item);
        }
        if live_stack.item_count > 1 {
            let mut remainder = live_stack.clone();
            remainder.item_count = remainder.item_count.saturating_sub(1);
            let dropped_item = crate::entity::item::ItemEntity::new(
                Entity::new(world.clone(), fox_pos, &EntityType::ITEM),
                remainder,
            );
            world.spawn_entity(Arc::new(dropped_item));
        }
        let mut single = live_stack;
        single.item_count = 1;
        self.set_mouth_item(single);
        self.mob_entity
            .set_guaranteed_drop(&EquipmentSlot::MAIN_HAND);
        self.ticks_since_eaten.store(0, Ordering::Relaxed);
        item_ent.get_entity().remove();
        true
    }

    fn tick_touch_item_pickup(&self, world: &Arc<World>) {
        if !self.can_move()
            || !self.mob_entity.can_pick_up_loot()
            || !world.level_info.load().game_rules.mob_griefing
        {
            return;
        }
        let reach = self.get_pickup_reach();
        let area = self
            .get_entity()
            .bounding_box
            .load()
            .expand(reach.x, reach.y, reach.z);
        for candidate in world.get_entities_at_box(&area) {
            let Some(item_ent) = candidate.get_item_entity() else {
                continue;
            };
            if item_ent.get_pickup_delay() > 0 || !item_ent.get_entity().is_alive() {
                continue;
            }
            if self.pick_up_item_stack(candidate.as_ref(), world) {
                break;
            }
        }
    }
}

pub fn is_alertable(fox: &FoxEntity, world: &World) -> bool {
    let pos = fox.get_entity().pos.load();

    let nearby_players = world.get_nearby_players(pos, 12.0);
    for player in nearby_players {
        if !player.living_entity.is_alive() {
            continue;
        }
        let p_pos = player.get_entity().pos.load();
        let dy = p_pos.y - pos.y;
        if dy.abs() > 6.0 {
            continue;
        }
        let gm = player.gamemode.load();
        if gm == GameMode::Creative || gm == GameMode::Spectator {
            continue;
        }
        if fox.trusts(player.gameprofile.id)
            || player.living_entity.entity.is_sneaking()
            || player.is_sleeping()
        {
            continue;
        }
        return true;
    }

    let aabb = pumpkin_util::math::boundingbox::BoundingBox::new(
        Vector3::new(pos.x - 12.0, pos.y - 6.0, pos.z - 12.0),
        Vector3::new(pos.x + 12.0, pos.y + 6.0, pos.z + 12.0),
    );
    let nearby = world.get_entities_at_box(&aabb);
    for candidate in nearby {
        if candidate.get_entity().entity_uuid == fox.get_entity().entity_uuid {
            continue;
        }
        if !candidate.is_alive() || candidate.get_living_entity().is_none() {
            continue;
        }
        let c_entity = candidate.get_entity();
        let c_pos = c_entity.pos.load();
        let dy = c_pos.y - pos.y;

        if dy.abs() > 6.0 {
            continue;
        }

        if pos.squared_distance_to_vec(&c_pos) > 144.0 {
            continue;
        }

        if *c_entity.entity_type == EntityType::FOX {
            continue;
        }

        if c_entity.entity_type.category == &MobCategory::MONSTER
            || *c_entity.entity_type == EntityType::CHICKEN
            || *c_entity.entity_type == EntityType::RABBIT
        {
            return true;
        }

        if let Some(mob) = candidate.get_mob()
            && let Some(tamable) = mob.as_tamable()
        {
            if !tamable.is_tame() {
                return true;
            }
            continue;
        }

        if fox.trusts(c_entity.entity_uuid) {
            continue;
        }

        let is_sleeping = c_entity.pose.load() == EntityPose::Sleeping;
        let is_discrete = c_entity.is_sneaking();
        if !is_sleeping && !is_discrete {
            return true;
        }
    }

    false
}

impl AgeableMob for FoxEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for FoxEntity {
    fn play_eating_sound(&self) {
        self.play_entity_sound(Sound::EntityFoxEat, 1.0, 1.0);
    }

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
    fn can_dispenser_equip_into_slot(
        &self,
        slot: &pumpkin_data::data_component_impl::EquipmentSlot,
    ) -> bool {
        *slot == pumpkin_data::data_component_impl::EquipmentSlot::MAIN_HAND
            && self.mob_entity.can_pick_up_loot()
    }

    // Needs fox eating first.
    fn wants_to_pick_up(&self, _stack: &ItemStack) -> bool {
        false
    }

    fn drop_inventory(&self) {
        let stack = self.get_mouth_item();
        if !stack.is_empty() {
            self.get_entity().spawn_at_location(stack);
            self.set_mouth_item(ItemStack::EMPTY.clone());
        }
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: crate::entity::mob::spawn::EntitySpawnReason,
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
        self.populate_default_equipment();
        self.set_target_goals();
        self.mob_entity.finalize_spawn_base();
        Some(SpawnGroupData::Fox {
            variant,
            size: size + 1,
        })
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        // Vanilla `aiStep`: a lost target ends the stalk.
        if !self.mob_entity.is_no_ai() && self.mob_entity.living_entity.is_alive() {
            if (self.is_crouching() || self.is_interested())
                && self
                    .mob_entity
                    .get_target()
                    .is_none_or(|target| !target.is_alive())
            {
                self.set_crouching(false);
                self.set_interested(false);
            }
            let entity = self.get_entity();
            let world = entity.world.load();
            self.tick_eating(&world);
            let world_full = entity.world.load_full();
            self.tick_touch_item_pickup(&world_full);
        }
    }

    fn post_tick(&self) {
        self.tick_states();
        if self.is_defending() && rand::random::<f32>() < 0.05 {
            self.get_entity().play_sound(Sound::EntityFoxAggro);
        }
    }

    fn has_global_tick_reach(&self) -> bool {
        let mouth_item = self.get_mouth_item();
        if let Some(consumable) = mouth_item.get_data_component::<ConsumableImpl>() {
            return consumable
                .effects
                .iter()
                .any(|effect| matches!(effect, ConsumeEffect::TeleportRandomly(_)));
        }
        false
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        if self.is_sleeping() {
            Some(Sound::EntityFoxSleep)
        } else {
            let entity = self.get_entity();
            let world = entity.world.load();
            if !world.is_bright_outside() && rand::rng().random_range(0..10) == 0 {
                let pos = entity.pos.load();
                let player_nearby = world
                    .get_nearest_player(pos, 16.0, |p| {
                        let gm = p.gamemode.load();
                        gm != GameMode::Spectator
                    })
                    .is_some();
                if !player_nearby {
                    return Some(Sound::EntityFoxScreech);
                }
            }
            Some(Sound::EntityFoxAmbient)
        }
    }

    fn play_ambient_sound(&self) {
        let Some(ambient) = self.get_ambient_sound() else {
            return;
        };
        if ambient == Sound::EntityFoxScreech {
            let living = &self.mob_entity.living_entity;
            living.make_sound(ambient, SoundCategory::Neutral, 2.0, living.get_pitch());
        } else {
            self.make_sound(ambient);
        }
    }

    fn on_offspring_spawned_from_egg(&self, player: &Player, parent: &dyn EntityBase) {
        self.add_trusted(player.gameprofile.id);
        if let Some(parent_fox) = parent.cast_any().downcast_ref::<Self>() {
            self.set_variant(parent_fox.get_variant());
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
        self.animal_interact(player, item_stack)
    }
}

impl CustomSound for FoxEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityFoxDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityFoxHurt)
    }

    fn sound_category(&self) -> SoundCategory {
        SoundCategory::Neutral
    }
}
