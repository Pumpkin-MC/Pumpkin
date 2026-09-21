use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::Block;
use pumpkin_data::Enchantment;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::block_properties::CampfireLikeProperties;
use pumpkin_data::data_component_impl::{BlocksAttacksImpl, EquipmentSlot};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::boundingbox::EntityDimensions;
use pumpkin_util::math::position::BlockPos;

use crate::entity::item::ItemEntity;
use crate::entity::living::LivingEntity;
use crate::entity::mob::equipment as mob_equipment;
use crate::entity::player::Player;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        active_target::ActiveTargetGoal, go_to_wanted_item::GoToWantedItemGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, open_door::OpenDoorGoal,
        ranged_crossbow_attack::RangedCrossbowAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity, crossbow_attack_mob::CrossbowAttackMob, equipment::RegionalDifficulty,
        piglin_ai::PiglinAi,
    },
};
use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PiglinActivity {
    Idle,
    AdmireItem,
    Fight,
    Celebrate,
}

pub struct PiglinEntity {
    pub mob_entity: MobEntity,
    pub immune_to_zombification: AtomicBool,
    pub time_in_overworld: AtomicI32,
    pub is_baby: AtomicBool,
    pub cannot_hunt: AtomicBool,
    pub is_charging_crossbow: AtomicBool,
    pub is_dancing: AtomicBool,
    pub inventory: Mutex<Vec<ItemStack>>,
    pub admire_timer: AtomicI32,
    pub admiring_disabled_timer: AtomicI32,
    pub eat_cooldown_timer: AtomicI32,
    pub celebration_timer: AtomicI32,
    pub hunt_cooldown_timer: AtomicI32,
    pub disable_walk_to_admire_timer: AtomicI32,
    pub time_trying_to_reach_item: AtomicI32,
    pub admiring_item: Mutex<Option<ItemStack>>,
    pub nearest_wanted_item: Mutex<Option<Arc<dyn EntityBase>>>,
    pub nearest_visible_player: Mutex<Option<Arc<Player>>>,
    pub near_repellent: AtomicBool,
    pub activity: AtomicCell<PiglinActivity>,
    pub ambient_sound_time: AtomicI32,
}

impl PiglinEntity {
    pub const CONVERSION_TIME: i32 = 300;
    pub const INVENTORY_SIZE: usize = 8;
    pub const XP_REWARD: u32 = 5;
    pub const AMBIENT_SOUND_INTERVAL: i32 = 80;

    pub const ADULT_DIMENSIONS: EntityDimensions = EntityDimensions {
        width: 0.6,
        height: 1.95,
        eye_height: 1.79,
    };
    pub const BABY_DIMENSIONS: EntityDimensions = EntityDimensions {
        width: 0.49,
        height: 0.98,
        eye_height: 0.78,
    };

    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let piglin = Self {
            mob_entity,
            immune_to_zombification: AtomicBool::new(false),
            time_in_overworld: AtomicI32::new(0),
            is_baby: AtomicBool::new(false),
            cannot_hunt: AtomicBool::new(false),
            is_charging_crossbow: AtomicBool::new(false),
            is_dancing: AtomicBool::new(false),
            inventory: Mutex::new(Vec::new()),
            admire_timer: AtomicI32::new(0),
            admiring_disabled_timer: AtomicI32::new(0),
            eat_cooldown_timer: AtomicI32::new(0),
            celebration_timer: AtomicI32::new(0),
            hunt_cooldown_timer: AtomicI32::new(0),
            disable_walk_to_admire_timer: AtomicI32::new(0),
            time_trying_to_reach_item: AtomicI32::new(-1),
            admiring_item: Mutex::new(None),
            nearest_wanted_item: Mutex::new(None),
            nearest_visible_player: Mutex::new(None),
            near_repellent: AtomicBool::new(false),
            activity: AtomicCell::new(PiglinActivity::Idle),
            ambient_sound_time: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(piglin);
        mob_arc.mob_entity.set_can_pick_up_loot(true);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(OpenDoorGoal::new(true)));
            goal_selector.add_goal(2, Box::new(GoToWantedItemGoal::new(mob_arc.clone(), 1.0)));
            goal_selector.add_goal(3, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(4, Box::new(RangedCrossbowAttackGoal::new(1.0, 8.0)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));

            target_selector.add_goal(
                2,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::PLAYER,
                    10,
                    true,
                    false,
                    Some(|target: &LivingEntity, _world: &World| {
                        !PiglinAi::is_wearing_safe_armor(target)
                    }),
                )),
            );

            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.mob_entity,
                    &EntityType::WITHER_SKELETON,
                    true,
                ),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::WITHER, true),
            );

            let piglin_clone = mob_arc.clone();
            target_selector.add_goal(
                4,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::HOGLIN,
                    10,
                    true,
                    false,
                    Some(move |_target: &LivingEntity, _world: &World| {
                        piglin_clone.is_adult() && piglin_clone.can_hunt()
                    }),
                )),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn is_immune_to_zombification(&self) -> bool {
        self.immune_to_zombification.load(Ordering::Relaxed)
    }

    pub fn set_immune_to_zombification(&self, immune: bool) {
        self.immune_to_zombification
            .store(immune, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(tracked_data::piglin::DATA_IMMUNE_TO_ZOMBIFICATION, immune);
    }

    #[must_use]
    pub fn is_converting(&self, world: &World) -> bool {
        !self.is_immune_to_zombification()
            && !self.mob_entity.is_no_ai()
            && world.dimension.piglins_zombify
    }

    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.is_baby.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn is_adult(&self) -> bool {
        !self.is_baby()
    }

    pub fn set_baby(&self, baby: bool) {
        self.is_baby.store(baby, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.set_synced_data(tracked_data::piglin::DATA_BABY_ID, baby);
        if baby {
            entity.entity_dimension.store(Self::BABY_DIMENSIONS);
        } else {
            entity.entity_dimension.store(Self::ADULT_DIMENSIONS);
        }
    }

    #[must_use]
    pub fn is_charging_crossbow(&self) -> bool {
        self.is_charging_crossbow.load(Ordering::Relaxed)
    }

    pub fn set_charging_crossbow(&self, is_charging: bool) {
        self.is_charging_crossbow
            .store(is_charging, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(tracked_data::piglin::DATA_IS_CHARGING_CROSSBOW, is_charging);
    }

    #[must_use]
    pub fn is_dancing(&self) -> bool {
        self.is_dancing.load(Ordering::Relaxed)
    }

    pub fn set_dancing(&self, is_dancing: bool) {
        self.is_dancing.store(is_dancing, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(tracked_data::piglin::DATA_IS_DANCING, is_dancing);
    }

    #[must_use]
    pub fn can_hunt(&self) -> bool {
        !self.cannot_hunt.load(Ordering::Relaxed)
            && self.hunt_cooldown_timer.load(Ordering::Relaxed) <= 0
    }

    pub fn set_cannot_hunt(&self, cannot_hunt: bool) {
        self.cannot_hunt.store(cannot_hunt, Ordering::Relaxed);
    }

    #[must_use]
    pub fn is_admiring(&self) -> bool {
        self.admire_timer.load(Ordering::Relaxed) > 0
    }

    #[must_use]
    pub fn is_admiring_disabled(&self) -> bool {
        self.admiring_disabled_timer.load(Ordering::Relaxed) > 0
    }

    #[must_use]
    pub fn has_eaten_recently(&self) -> bool {
        self.eat_cooldown_timer.load(Ordering::Relaxed) > 0
    }

    const fn expiring_memories(&self) -> [(&'static str, &AtomicI32); 3] {
        [
            ("minecraft:admiring_item", &self.admire_timer),
            ("minecraft:admiring_disabled", &self.admiring_disabled_timer),
            ("minecraft:hunted_recently", &self.hunt_cooldown_timer),
        ]
    }

    fn write_brain_memories(&self, nbt: &mut NbtCompound) {
        let mut memories = NbtCompound::new();
        for (key, timer) in self.expiring_memories() {
            let ttl = timer.load(Ordering::Relaxed);
            if ttl > 0 {
                let mut memory = NbtCompound::new();
                memory.put_bool("value", true);
                memory.put_long("ttl", i64::from(ttl));
                memories.put(key, NbtTag::Compound(memory));
            }
        }
        if !memories.child_tags.is_empty() {
            let mut brain = NbtCompound::new();
            brain.put("memories", NbtTag::Compound(memories));
            nbt.put("Brain", NbtTag::Compound(brain));
        }
    }

    fn read_brain_memories(&self, nbt: &NbtCompound) {
        let Some(memories) = nbt
            .get_compound("Brain")
            .and_then(|brain| brain.get_compound("memories"))
        else {
            return;
        };
        for (key, timer) in self.expiring_memories() {
            if let Some(ttl) = memories
                .get_compound(key)
                .and_then(|memory| memory.get_long("ttl"))
            {
                timer.store(ttl.clamp(0, i64::from(i32::MAX)) as i32, Ordering::Relaxed);
            }
        }
    }

    #[must_use]
    pub fn is_walk_to_admire_disabled(&self) -> bool {
        self.disable_walk_to_admire_timer.load(Ordering::Relaxed) > 0
    }

    #[must_use]
    pub fn has_attack_target(&self) -> bool {
        self.mob_entity
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    #[must_use]
    pub fn is_near_repellent(&self) -> bool {
        self.near_repellent.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn off_hand_item(&self) -> ItemStack {
        self.mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::OFF_HAND)
    }

    #[must_use]
    pub fn is_holding_item_in_off_hand(&self) -> bool {
        !self.off_hand_item().is_empty()
    }

    #[must_use]
    pub fn can_add_to_inventory(&self, item: &ItemStack) -> bool {
        let inv = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inv.len() < Self::INVENTORY_SIZE
            || inv.iter().any(|slot| {
                slot.are_items_and_components_equal(item)
                    && slot.item_count < slot.get_max_stack_size()
            })
    }

    pub fn add_to_inventory(&self, item: ItemStack) -> Option<ItemStack> {
        if item.is_empty() {
            return None;
        }
        let mut inv = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut remaining = item;
        for slot in inv.iter_mut() {
            if !slot.are_items_and_components_equal(&remaining) {
                continue;
            }
            let space = slot.get_max_stack_size().saturating_sub(slot.item_count);
            let moved = remaining.item_count.min(space);
            slot.increment(moved);
            remaining.decrement(moved);
            if remaining.is_empty() {
                return None;
            }
        }
        if inv.len() < Self::INVENTORY_SIZE {
            inv.push(remaining);
            return None;
        }
        Some(remaining)
    }

    #[must_use]
    pub fn can_replace_current_item_for(&self, new_item: &ItemStack) -> bool {
        let slot = mob_equipment::get_equipment_slot_for_item(new_item);
        let current = self
            .mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&slot);
        Mob::can_replace_current_item(self, new_item, &current, &slot)
    }

    fn put_in_inventory(&self, item: ItemStack) {
        let remainder = self
            .add_to_inventory(item)
            .unwrap_or_else(|| ItemStack::EMPTY.clone());
        // Vanilla swings the off hand here even when nothing was left over.
        PiglinAi::throw_items_toward_random_pos(self, vec![remainder]);
    }

    fn hold_in_off_hand(&self, item: ItemStack) {
        if self.is_holding_item_in_off_hand() {
            self.mob_entity.spawn_at_location(self.off_hand_item());
        }
        let keep_loaded = !PiglinAi::is_barter_currency(&item);
        *self
            .admiring_item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(item.clone());
        self.mob_entity
            .set_item_slot_and_drop_when_killed(&EquipmentSlot::OFF_HAND, item);
        if keep_loaded {
            self.mob_entity
                .persistence_required
                .store(true, Ordering::Relaxed);
        }
    }

    fn hold_in_main_hand(&self, item: ItemStack) {
        self.mob_entity
            .set_item_slot_and_drop_when_killed(&EquipmentSlot::MAIN_HAND, item);
        self.mob_entity
            .persistence_required
            .store(true, Ordering::Relaxed);
    }

    fn main_hand_item(&self) -> ItemStack {
        self.mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND)
    }

    fn admire_gold_item(&self) {
        self.admire_timer
            .store(PiglinAi::ADMIRE_DURATION, Ordering::Relaxed);
    }

    fn stop_walking(&self) {
        self.mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    pub fn start_admiring(&self, item: ItemStack) {
        self.hold_in_off_hand(item);
        self.admire_gold_item();
        self.stop_walking();
    }

    #[must_use]
    pub fn nearest_wanted_item(&self) -> Option<Arc<dyn EntityBase>> {
        let item = self
            .nearest_wanted_item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        item.filter(|item| item.get_entity().is_alive())
    }

    #[must_use]
    pub fn nearest_visible_player(&self) -> Option<Arc<Player>> {
        let player = self
            .nearest_visible_player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        player.filter(|player| player.get_entity().is_alive())
    }

    fn wants_item_entity(&self, item: &ItemEntity) -> bool {
        let stack = item
            .get_item_stack()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        !stack.is_empty() && PiglinAi::wants_to_pickup(self, &stack)
    }

    fn tick_sensors(&self) {
        self.refresh_nearest_wanted_item();
        self.refresh_nearest_visible_player();
        self.refresh_nearest_repellent();
    }

    fn refresh_nearest_wanted_item(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        let eye_pos = entity.get_eye_pos();
        let search_box = entity.bounding_box.load().expand(
            PiglinAi::ITEM_SCAN_RANGE,
            PiglinAi::ITEM_SCAN_RANGE_Y,
            PiglinAi::ITEM_SCAN_RANGE,
        );
        let max_distance_sq = PiglinAi::ITEM_SCAN_RANGE * PiglinAi::ITEM_SCAN_RANGE;

        let nearest = if self.mob_entity.can_pick_up_loot()
            && world.level_info.load().game_rules.mob_griefing
        {
            let mut candidates: Vec<(f64, Arc<dyn EntityBase>)> = world
                .entities
                .load()
                .iter()
                .filter_map(|candidate| {
                    let item = candidate.get_item_entity()?;
                    let item_entity = item.get_entity();
                    let distance_sq = item_entity.pos.load().squared_distance_to_vec(&pos);
                    (item_entity.is_alive()
                        && item_entity.bounding_box.load().intersects(&search_box)
                        && distance_sq < max_distance_sq
                        && self.wants_item_entity(item))
                    .then(|| (distance_sq, candidate.clone()))
                })
                .collect();
            candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
            candidates
                .into_iter()
                .map(|(_, candidate)| candidate)
                .find(|candidate| {
                    world
                        .raycast(
                            eye_pos,
                            candidate.get_entity().get_eye_pos(),
                            |block_pos, w| w.get_block_state(block_pos).is_solid(),
                        )
                        .is_none()
                })
        } else {
            None
        };

        *self
            .nearest_wanted_item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = nearest;
    }

    fn refresh_nearest_visible_player(&self) {
        let living = &self.mob_entity.living_entity;
        let pos = living.entity.pos.load();
        let follow_range = living.get_attribute_value(&Attributes::FOLLOW_RANGE);

        let mut players = living
            .entity
            .world
            .load()
            .get_nearby_players(pos, follow_range);
        players.retain(|player| {
            !player.is_spectator()
                && player.get_entity().pos.load().squared_distance_to_vec(&pos)
                    < follow_range * follow_range
        });
        players.sort_by(|a, b| {
            a.get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&pos)
                .total_cmp(&b.get_entity().pos.load().squared_distance_to_vec(&pos))
        });
        let nearest = players
            .into_iter()
            .find(|player| PiglinAi::is_entity_targetable(self, player));

        *self
            .nearest_visible_player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = nearest;
    }

    fn refresh_nearest_repellent(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let center = entity.block_pos.load();
        let horizontal = PiglinAi::REPELLENT_DETECTION_RANGE_HORIZONTAL;
        let vertical = PiglinAi::REPELLENT_DETECTION_RANGE_VERTICAL;

        let mut found = false;
        'scan: for dy in -vertical..=vertical {
            for dx in -horizontal..=horizontal {
                for dz in -horizontal..=horizontal {
                    let pos = BlockPos::new(center.0.x + dx, center.0.y + dy, center.0.z + dz);
                    let (block, state) = world.get_block_and_state(&pos);
                    if !block.has_tag(&tag::Block::MINECRAFT_PIGLIN_REPELLENTS) {
                        continue;
                    }
                    if block.id == Block::SOUL_CAMPFIRE.id
                        && !CampfireLikeProperties::from_state_id(state.id).lit
                    {
                        continue;
                    }
                    found = true;
                    break 'scan;
                }
            }
        }
        self.near_repellent.store(found, Ordering::Relaxed);
    }

    fn current_activity(&self) -> PiglinActivity {
        if self.is_admiring() {
            PiglinActivity::AdmireItem
        } else if self.has_attack_target() {
            PiglinActivity::Fight
        } else if self.celebration_timer.load(Ordering::Relaxed) > 0 {
            PiglinActivity::Celebrate
        } else {
            PiglinActivity::Idle
        }
    }

    fn update_activity(&self) {
        let old_activity = self.activity.load();
        let new_activity = self.current_activity();
        if old_activity != new_activity {
            self.activity.store(new_activity);
            self.make_sound(PiglinAi::get_sound_for_activity(self, new_activity));
        }
    }

    fn make_sound(&self, sound: Sound) {
        let entity = &self.mob_entity.living_entity.entity;
        let base_pitch = if self.is_baby() { 1.5 } else { 1.0 };
        let pitch = (rand::random::<f32>() - rand::random::<f32>()).mul_add(0.2, base_pitch);
        entity.world.load().play_sound_fine(
            sound,
            SoundCategory::Hostile,
            &entity.pos.load(),
            1.0,
            pitch,
        );
    }

    fn tick_ambient_sound(&self) {
        let living = &self.mob_entity.living_entity;
        let alive = living.entity.is_alive() && living.health.load() > 0.0;
        if alive
            && rand::random_range(0..1000) < self.ambient_sound_time.fetch_add(1, Ordering::Relaxed)
        {
            self.reset_ambient_sound_time();
            self.make_sound(PiglinAi::get_sound_for_activity(self, self.activity.load()));
        }
    }

    fn reset_ambient_sound_time(&self) {
        self.ambient_sound_time
            .store(-Self::AMBIENT_SOUND_INTERVAL, Ordering::Relaxed);
    }

    fn stop_holding_item_if_no_longer_admiring(&self) {
        if self.is_admiring() {
            return;
        }
        let off_hand = self.off_hand_item();
        if off_hand.is_empty() || off_hand.get_data_component::<BlocksAttacksImpl>().is_some() {
            return;
        }
        self.stop_holding_off_hand_item(true);
    }

    fn start_admiring_if_seen(&self) {
        if self.is_admiring() || self.is_admiring_disabled() || self.is_walk_to_admire_disabled() {
            return;
        }
        let Some(item) = self.nearest_wanted_item() else {
            return;
        };
        let Some(item) = item.get_item_entity() else {
            return;
        };
        let is_loved = {
            let stack = item
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            PiglinAi::is_loved_item(&stack)
        };
        if is_loved {
            self.admire_gold_item();
        }
    }

    fn is_nearest_wanted_item_within(&self, distance: f64) -> bool {
        let pos = self.mob_entity.living_entity.entity.pos.load();
        self.nearest_wanted_item().is_some_and(|item| {
            item.get_entity().pos.load().squared_distance_to_vec(&pos) < distance * distance
        })
    }

    pub fn stop_admiring_if_item_too_far_away(&self) {
        if !self.is_admiring() || self.is_holding_item_in_off_hand() {
            return;
        }
        if self.is_nearest_wanted_item_within(PiglinAi::MAX_DISTANCE_TO_WALK_TO_ITEM) {
            return;
        }
        self.admire_timer.store(0, Ordering::Relaxed);
    }

    pub fn stop_admiring_if_tired_of_trying_to_reach_item(&self) {
        if !self.is_admiring()
            || self.nearest_wanted_item().is_none()
            || self.is_holding_item_in_off_hand()
        {
            return;
        }
        // -1 stands in for the brain memory being absent.
        let time = self.time_trying_to_reach_item.load(Ordering::Relaxed);
        if time < 0 {
            self.time_trying_to_reach_item.store(0, Ordering::Relaxed);
        } else if time > PiglinAi::MAX_TIME_TRYING_TO_REACH_ITEM {
            self.admire_timer.store(0, Ordering::Relaxed);
            self.time_trying_to_reach_item.store(-1, Ordering::Relaxed);
            self.disable_walk_to_admire_timer
                .store(PiglinAi::DISABLE_WALK_TO_ADMIRE_DURATION, Ordering::Relaxed);
        } else {
            self.time_trying_to_reach_item
                .store(time + 1, Ordering::Relaxed);
        }
    }

    fn pick_up_nearby_items(&self) {
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        if !self.mob_entity.can_pick_up_loot()
            || !entity.is_alive()
            || living.health.load() <= 0.0
            || living.dead.load(Ordering::Relaxed)
        {
            return;
        }
        let world = entity.world.load();
        if !world.level_info.load().game_rules.mob_griefing {
            return;
        }

        let reach = entity.bounding_box.load().expand(1.0, 0.0, 1.0);
        for candidate in world.get_entities_at_box(&reach) {
            let Some(item) = candidate.get_item_entity() else {
                continue;
            };
            if item.get_entity().is_alive()
                && item.get_pickup_delay() == 0
                && self.wants_item_entity(item)
            {
                self.pick_up_item(item);
            }
        }
    }

    fn pick_up_item(&self, item: &ItemEntity) {
        self.stop_walking();
        let count = {
            let stack = item
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            // Nuggets are taken whole, everything else one at a time.
            if stack.item.id == Item::GOLD_NUGGET.id {
                stack.item_count
            } else {
                stack.item_count.min(1)
            }
        };
        if count == 0
            || !self
                .mob_entity
                .living_entity
                .pickup(item.get_entity(), u32::from(count))
        {
            return;
        }

        let taken = item
            .get_item_stack()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .split(count);
        let emptied = item
            .get_item_stack()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty();
        if emptied {
            item.get_entity().remove();
        } else {
            item.init_data_tracker();
        }

        if PiglinAi::is_loved_item(&taken) {
            self.time_trying_to_reach_item.store(-1, Ordering::Relaxed);
            self.hold_in_off_hand(taken);
            self.admire_gold_item();
        } else if PiglinAi::is_food(&taken) && !self.has_eaten_recently() {
            self.eat_cooldown_timer
                .store(PiglinAi::EAT_COOLDOWN, Ordering::Relaxed);
        } else if mob_equipment::equip_item_if_possible(self, taken.clone()).is_empty() {
            self.put_in_inventory(taken);
        }
    }

    pub fn stop_holding_off_hand_item(&self, bartering_enabled: bool) {
        let item = self
            .mob_entity
            .set_item_slot(&EquipmentSlot::OFF_HAND, ItemStack::EMPTY.clone());
        self.admiring_item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();

        if self.is_adult() {
            let is_barter = PiglinAi::is_barter_currency(&item);
            if bartering_enabled && is_barter {
                let outcomes = PiglinAi::get_barter_response_items();
                let entity = &self.mob_entity.living_entity.entity;

                let mut event =
                    crate::plugin::api::events::entity::piglin_barter::PiglinBarterEvent::new(
                        entity.entity_id,
                        item,
                        outcomes,
                    );
                if let Some(server) = entity.world.load().server.upgrade() {
                    server.plugin_manager.fire_blocking(&server, &mut event);
                }

                if !event.cancelled {
                    PiglinAi::throw_items(self, event.outcome);
                }
            } else if !is_barter
                && mob_equipment::equip_item_if_possible(self, item.clone()).is_empty()
            {
                self.put_in_inventory(item);
            }
        } else if mob_equipment::equip_item_if_possible(self, item.clone()).is_empty() {
            let main_hand = self.main_hand_item();
            if PiglinAi::is_loved_item(&main_hand) {
                self.put_in_inventory(main_hand);
            } else {
                PiglinAi::throw_items(self, vec![main_hand]);
            }
            self.hold_in_main_hand(item);
        }
    }

    pub fn was_hurt_by(&self, attacker: &dyn EntityBase) {
        let attacker_entity = attacker.get_entity();
        if attacker.get_living_entity().is_none()
            || attacker_entity.entity_type.id == EntityType::PIGLIN.id
        {
            return;
        }
        if self.is_holding_item_in_off_hand() {
            self.stop_holding_off_hand_item(false);
        }
        self.set_dancing(false);
        self.celebration_timer.store(0, Ordering::Relaxed);
        self.admire_timer.store(0, Ordering::Relaxed);
        if attacker_entity.entity_type.id == EntityType::PLAYER.id {
            self.admiring_disabled_timer
                .store(PiglinAi::ADMIRING_DISABLED_DURATION, Ordering::Relaxed);
        }
    }

    pub fn cancel_admiring(&self) {
        if self.is_admiring() && self.is_holding_item_in_off_hand() {
            self.mob_entity.spawn_at_location(self.off_hand_item());
            self.mob_entity
                .set_item_slot(&EquipmentSlot::OFF_HAND, ItemStack::EMPTY.clone());
            self.admiring_item
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
        }
    }

    pub fn drop_inventory(&self) {
        let items = self
            .inventory
            .try_lock()
            .map_or_else(|_| Vec::new(), |mut inv| std::mem::take(&mut *inv));
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        for item in items {
            if !item.is_empty() {
                let item_entity =
                    ItemEntity::new(Entity::new(world.clone(), pos, &EntityType::ITEM), item);
                world.spawn_entity(Arc::new(item_entity));
            }
        }
    }

    #[must_use]
    pub fn check_piglin_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        let below = BlockPos::new(pos.0.x, pos.0.y - 1, pos.0.z);
        let state = world.get_block_state(&below);
        state.id != Block::NETHER_WART_BLOCK.default_state.id
    }

    fn convert_to_zombified(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();

        if world.level_info.load().difficulty != pumpkin_util::Difficulty::Peaceful {
            world.play_sound(
                Sound::EntityPiglinConvertedToZombified,
                SoundCategory::Hostile,
                &pos,
            );
        }

        self.drop_inventory();

        let zombified = crate::entity::r#type::from_type(
            &EntityType::ZOMBIFIED_PIGLIN,
            pos,
            &world,
            uuid::Uuid::new_v4(),
        );

        let zombified_base = zombified.get_entity();
        zombified_base.set_rotation(entity.yaw.load(), entity.pitch.load());
        zombified_base.head_yaw.store(entity.head_yaw.load());
        zombified_base.velocity.store(entity.velocity.load());

        if let Some(living) = zombified.get_living_entity() {
            living.set_health(self.mob_entity.living_entity.health.load());
        }

        if let Some(custom_name) = &**entity.custom_name.load() {
            zombified_base.set_custom_name(custom_name.clone());
        }

        {
            let src_equip = self
                .mob_entity
                .living_entity
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(living) = zombified.get_living_entity() {
                let mut dst_equip = living
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for (slot, item) in &src_equip.equipment {
                    dst_equip.put(slot, item.clone());
                }
            }
        }

        world.spawn_entity(zombified);
        entity.remove();
    }
}

impl Mob for PiglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn populate_default_equipment_slots(
        &self,
        _world: &Arc<World>,
        _difficulty: &RegionalDifficulty,
    ) {
        if !self.is_baby.load(Ordering::Relaxed) {
            let living = &self.mob_entity.living_entity;
            let mut equipment = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            let weapon = if rand::random::<f32>() < 0.5 {
                &Item::CROSSBOW
            } else if rand::random_range(0..10) == 0 {
                &Item::GOLDEN_SPEAR
            } else {
                &Item::GOLDEN_SWORD
            };
            equipment.put(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, weapon));

            if rand::random::<f32>() < 0.1 {
                equipment.put(
                    &EquipmentSlot::HEAD,
                    ItemStack::new(1, &Item::GOLDEN_HELMET),
                );
            }
            if rand::random::<f32>() < 0.1 {
                equipment.put(
                    &EquipmentSlot::CHEST,
                    ItemStack::new(1, &Item::GOLDEN_CHESTPLATE),
                );
            }
            if rand::random::<f32>() < 0.1 {
                equipment.put(
                    &EquipmentSlot::LEGS,
                    ItemStack::new(1, &Item::GOLDEN_LEGGINGS),
                );
            }
            if rand::random::<f32>() < 0.1 {
                equipment.put(&EquipmentSlot::FEET, ItemStack::new(1, &Item::GOLDEN_BOOTS));
            }
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        if self.is_immune_to_zombification() {
            entity.set_synced_data(tracked_data::piglin::DATA_IMMUNE_TO_ZOMBIFICATION, true);
        }
        if self.is_baby() {
            entity.set_synced_data(tracked_data::piglin::DATA_BABY_ID, true);
        }
        if self.is_charging_crossbow() {
            entity.set_synced_data(tracked_data::piglin::DATA_IS_CHARGING_CROSSBOW, true);
        }
        if self.is_dancing() {
            entity.set_synced_data(tracked_data::piglin::DATA_IS_DANCING, true);
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_brain_memories(nbt);
        if self.is_immune_to_zombification() {
            nbt.put_bool("IsImmuneToZombification", true);
        }
        let time_in_overworld = self.time_in_overworld.load(Ordering::Relaxed);
        if time_in_overworld > 0 {
            nbt.put_int("TimeInOverworld", time_in_overworld);
        }
        if self.is_baby() {
            nbt.put_bool("IsBaby", true);
        }
        if !self.can_hunt() {
            nbt.put_bool("CannotHunt", true);
        }
        if self.is_charging_crossbow() {
            nbt.put_bool("IsChargingCrossbow", true);
        }
        if self.is_dancing() {
            nbt.put_bool("IsDancing", true);
        }

        let inv = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !inv.is_empty() {
            let mut items_tag = Vec::new();
            for item in inv.iter() {
                if !item.is_empty() {
                    let mut item_nbt = NbtCompound::new();
                    item.write_item_stack(&mut item_nbt);
                    items_tag.push(NbtTag::Compound(item_nbt));
                }
            }
            if !items_tag.is_empty() {
                nbt.put_list("Inventory", items_tag);
            }
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_brain_memories(nbt);
        if let Some(immune) = nbt.get_bool("IsImmuneToZombification") {
            self.set_immune_to_zombification(immune);
        }
        if let Some(time) = nbt.get_int("TimeInOverworld") {
            self.time_in_overworld.store(time, Ordering::Relaxed);
        }
        if let Some(baby) = nbt.get_bool("IsBaby") {
            self.set_baby(baby);
        }
        if let Some(cannot_hunt) = nbt.get_bool("CannotHunt") {
            self.set_cannot_hunt(cannot_hunt);
        }
        if let Some(charging) = nbt.get_bool("IsChargingCrossbow") {
            self.set_charging_crossbow(charging);
        }
        if let Some(dancing) = nbt.get_bool("IsDancing") {
            self.set_dancing(dancing);
        }
        if let Some(inv_list) = nbt.get_list("Inventory") {
            let mut inv = self
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inv.clear();
            for tag in inv_list {
                if let Some(compound) = tag.extract_compound()
                    && let Some(stack) = ItemStack::read_item_stack(compound)
                {
                    inv.push(stack);
                }
            }
        }
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if PiglinAi::can_admire(self, item_stack) {
            let taken = item_stack.split_unless_creative(player.gamemode.load(), 1);
            self.start_admiring(taken);
            return true;
        }
        self.mob_entity.mob_interact(player, item_stack)
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }
        self.tick_ambient_sound();

        let world = entity.world.load();
        if self.is_converting(&world) {
            let time = self.time_in_overworld.fetch_add(1, Ordering::Relaxed) + 1;
            if time > Self::CONVERSION_TIME {
                self.convert_to_zombified();
            }
        } else {
            self.time_in_overworld.store(0, Ordering::Relaxed);
        }

        if self.admiring_disabled_timer.load(Ordering::Relaxed) > 0 {
            self.admiring_disabled_timer.fetch_sub(1, Ordering::Relaxed);
        }
        if self.eat_cooldown_timer.load(Ordering::Relaxed) > 0 {
            self.eat_cooldown_timer.fetch_sub(1, Ordering::Relaxed);
        }
        if self.hunt_cooldown_timer.load(Ordering::Relaxed) > 0 {
            self.hunt_cooldown_timer.fetch_sub(1, Ordering::Relaxed);
        }
        if self.disable_walk_to_admire_timer.load(Ordering::Relaxed) > 0 {
            self.disable_walk_to_admire_timer
                .fetch_sub(1, Ordering::Relaxed);
        }
        if self.celebration_timer.load(Ordering::Relaxed) > 0 {
            let remaining = self.celebration_timer.fetch_sub(1, Ordering::Relaxed) - 1;
            if remaining <= 0 {
                self.set_dancing(false);
            }
        }
        if self.admire_timer.load(Ordering::Relaxed) > 0 {
            self.admire_timer.fetch_sub(1, Ordering::Relaxed);
        }

        let tick = world.get_world_age() + i64::from(entity.entity_id);
        if tick.rem_euclid(PiglinAi::SENSOR_SCAN_INTERVAL) == 0 {
            self.tick_sensors();
        }
        self.stop_holding_item_if_no_longer_admiring();
        self.start_admiring_if_seen();
    }

    fn post_tick(&self) {
        self.update_activity();
        self.pick_up_nearby_items();
    }

    fn on_damage(
        &self,
        _damage_type: pumpkin_data::damage::DamageType,
        source: Option<&dyn EntityBase>,
    ) {
        self.reset_ambient_sound_time();
        if self.mob_entity.living_entity.dead.load(Ordering::Relaxed) {
            self.drop_inventory();
        } else if let Some(attacker) = source {
            self.was_hurt_by(attacker);
        }
    }

    fn get_preferred_weapon_type(&self) -> Option<&'static tag::Tag> {
        (!self.is_baby()).then_some(&tag::Item::MINECRAFT_PIGLIN_PREFERRED_WEAPONS)
    }

    fn can_replace_current_item(
        &self,
        new_item: &ItemStack,
        current_item: &ItemStack,
        slot: &EquipmentSlot,
    ) -> bool {
        if current_item.get_enchantment_level(&Enchantment::BINDING_CURSE) > 0 {
            return false;
        }
        let preferred = self.get_preferred_weapon_type();
        let new_wanted = PiglinAi::is_loved_item(new_item)
            || preferred.is_some_and(|weapons| new_item.item.has_tag(weapons));
        let current_wanted = PiglinAi::is_loved_item(current_item)
            || preferred.is_some_and(|weapons| current_item.item.has_tag(weapons));
        new_wanted && !current_wanted
            || (new_wanted || !current_wanted)
                && mob_equipment::can_replace_current_item(
                    &self.mob_entity,
                    preferred,
                    new_item,
                    current_item,
                    slot,
                )
    }

    fn as_crossbow_attack_mob(&self) -> Option<&dyn CrossbowAttackMob> {
        Some(self)
    }

    fn get_base_experience_reward(&self) -> u32 {
        Self::XP_REWARD
    }
}

impl CrossbowAttackMob for PiglinEntity {
    fn set_charging_crossbow(&self, is_charging: bool) {
        self.set_charging_crossbow(is_charging);
    }

    fn is_charging_crossbow(&self) -> bool {
        self.is_charging_crossbow()
    }
}
