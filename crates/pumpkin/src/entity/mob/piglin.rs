use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::Block;
use pumpkin_data::Enchantment;
use pumpkin_data::data_component_impl::EquipmentSlot;
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
use crate::entity::mob::equipment as mob_equipment;
use crate::entity::mob::{abstract_piglin, piglin_ai};
use crate::entity::player::Player;
use crate::entity::{
    Entity, EntityBase,
    mob::{Mob, MobEntity, crossbow_attack_mob::CrossbowAttackMob, equipment::RegionalDifficulty},
};
use crate::world::World;

pub struct PiglinEntity {
    pub mob_entity: MobEntity,
    pub immune_to_zombification: AtomicBool,
    pub time_in_overworld: AtomicI32,
    pub is_baby: AtomicBool,
    pub cannot_hunt: AtomicBool,
    pub is_charging_crossbow: AtomicBool,
    pub is_dancing: AtomicBool,
    pub inventory: Mutex<Vec<ItemStack>>,
    pub ambient_sound_time: AtomicI32,
}

impl PiglinEntity {
    pub const CONVERSION_TIME: i32 = 300;
    pub const INVENTORY_SIZE: usize = 8;
    pub const XP_REWARD: u32 = 5;
    pub const AMBIENT_SOUND_INTERVAL: i32 = 80;
    /// `CrossbowItem.getDefaultProjectileRange`.
    const CROSSBOW_PROJECTILE_RANGE: i32 = 8;

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
            ambient_sound_time: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(piglin);
        mob_arc.mob_entity.set_can_pick_up_loot(true);
        mob_arc.mob_entity.init_brain(mob_arc.as_ref());
        // Vanilla does this in finalizeSpawn; a loaded mob rebuilds its brain after
        mob_arc.mob_entity.with_brain(mob_arc.as_ref(), |tick| {
            let mut rng = tick.mob.get_random();
            piglin_ai::init_memories(tick.brain, &mut rng);
        });
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
    }

    pub fn set_cannot_hunt(&self, cannot_hunt: bool) {
        self.cannot_hunt.store(cannot_hunt, Ordering::Relaxed);
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
    pub fn hold_in_off_hand(&self, item: ItemStack) {
        if self.is_holding_item_in_off_hand() {
            self.mob_entity.spawn_at_location(self.off_hand_item());
        }
        let keep_loaded = !piglin_ai::is_barter_currency(&item);
        self.mob_entity
            .set_item_slot_and_drop_when_killed(&EquipmentSlot::OFF_HAND, item);
        if keep_loaded {
            self.mob_entity
                .persistence_required
                .store(true, Ordering::Relaxed);
        }
    }

    pub fn hold_in_main_hand(&self, item: ItemStack) {
        self.mob_entity
            .set_item_slot_and_drop_when_killed(&EquipmentSlot::MAIN_HAND, item);
        self.mob_entity
            .persistence_required
            .store(true, Ordering::Relaxed);
    }

    pub fn main_hand_item(&self) -> ItemStack {
        self.mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND)
    }
    #[must_use]
    fn wants_item_entity(&self, item: &ItemEntity) -> bool {
        let stack = item
            .get_item_stack()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        !stack.is_empty()
            && self.mob_entity.with_brain(self, |tick| {
                <Self as Mob>::wants_to_pick_up(self, tick.brain, &stack)
            })
    }
    pub fn make_sound(&self, sound: Sound) {
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
            let sound = self.mob_entity.with_brain(self, |tick| {
                let converting = self.is_converting(tick.world);
                piglin_ai::get_sound_for_current_activity(&tick.visibility(), converting)
            });
            if let Some(sound) = sound {
                self.make_sound(sound);
            }
        }
    }

    fn reset_ambient_sound_time(&self) {
        self.ambient_sound_time
            .store(-Self::AMBIENT_SOUND_INTERVAL, Ordering::Relaxed);
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

        self.mob_entity.with_brain(self, |tick| {
            piglin_ai::pick_up_item(tick, self, taken);
        });
    }
    pub fn drop_inventory(&self) {
        let items = std::mem::take(
            &mut *self
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
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

impl abstract_piglin::AbstractPiglin for PiglinEntity {
    fn is_adult(&self) -> bool {
        Self::is_adult(self)
    }

    fn can_hunt(&self) -> bool {
        Self::can_hunt(self)
    }

    fn is_converting(&self, world: &World) -> bool {
        Self::is_converting(self, world)
    }

    fn is_immune_to_zombification(&self) -> bool {
        Self::is_immune_to_zombification(self)
    }

    fn arm_pose(&self) -> abstract_piglin::PiglinArmPose {
        use abstract_piglin::PiglinArmPose;

        if self.is_dancing() {
            return PiglinArmPose::Dancing;
        }
        if piglin_ai::is_loved_item(&self.off_hand_item()) {
            return PiglinArmPose::AdmiringItem;
        }
        let holding_melee_weapon = self
            .main_hand_item()
            .get_data_component::<pumpkin_data::data_component_impl::ToolImpl>()
            .is_some();
        if self.mob_entity.is_attacking() && holding_melee_weapon {
            return PiglinArmPose::AttackingWithMeleeWeapon;
        }
        if self.is_charging_crossbow() {
            return PiglinArmPose::CrossbowCharge;
        }
        let main_hand = self.main_hand_item();
        let crossbow_charged = main_hand
            .get_data_component::<pumpkin_data::data_component_impl::ChargedProjectilesImpl>()
            .is_some_and(|charged| !charged.projectiles.is_empty());
        if main_hand.item.id == Item::CROSSBOW.id && crossbow_charged {
            return PiglinArmPose::CrossbowHold;
        }
        PiglinArmPose::Default
    }

    fn play_converted_sound(&self) {
        self.make_sound(Sound::EntityPiglinConvertedToZombified);
    }

    fn finish_conversion(&self) {
        self.mob_entity
            .with_brain(self, |tick| piglin_ai::cancel_admiring(tick, self));
        self.convert_to_zombified();
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
        let can_admire = self.mob_entity.with_brain(self, |tick| {
            piglin_ai::can_admire(tick.brain, self, item_stack)
        });
        if can_admire {
            let taken = item_stack.split_unless_creative(player.gamemode.load(), 1);
            self.mob_entity.with_brain(self, |tick| {
                piglin_ai::start_admiring(tick, self, taken);
            });
            return true;
        }
        self.mob_entity.mob_interact(player, item_stack)
    }

    fn make_brain(
        &self,
        packed: &crate::entity::ai::brain::memory::PackedMemories,
    ) -> crate::entity::ai::brain::Brain {
        piglin_ai::PIGLIN_PROVIDER.make_brain(self, packed)
    }

    fn after_brain_tick(&self, tick: &mut crate::entity::ai::brain::BrainTick<'_>) {
        piglin_ai::update_activity(tick);
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !crate::entity::ai::brain::behavior::utils::is_alive(self) {
            // No AI while dying, but still drain the inbox
            self.mob_entity.apply_brain_inbox(self);
            return;
        }
        self.mob_entity.tick_brain(self);
        let world = entity.world.load();
        abstract_piglin::tick_conversion(self, &world, &self.time_in_overworld);
    }

    fn wants_to_pick_up(&self, brain: &crate::entity::ai::brain::Brain, stack: &ItemStack) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity
            .world
            .load()
            .level_info
            .load()
            .game_rules
            .mob_griefing
            || !self.mob_entity.can_pick_up_loot()
        {
            return false;
        }
        piglin_ai::wants_to_pickup(brain, self, stack)
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        if !crate::entity::ai::brain::behavior::utils::is_alive(self) {
            return;
        }
        self.tick_ambient_sound();
    }

    fn post_tick(&self) {
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
        }
        let Some(attacker) = source else {
            return;
        };
        let world = self.mob_entity.living_entity.entity.world.load_full();
        let Some(attacker) = world
            .get_entity_by_id(attacker.get_entity().entity_id)
            .filter(|attacker| attacker.get_living_entity().is_some())
        else {
            return;
        };
        // Queued: taking the victim's brain here deadlocks two mobs fighting
        self.mob_entity.post_to_brain(Box::new(move |tick| {
            let Some(piglin) = tick.mob.cast_any().downcast_ref::<Self>() else {
                return;
            };
            piglin_ai::was_hurt_by(tick, piglin, &attacker);
        }));
    }

    fn get_preferred_weapon_type(&self) -> Option<&'static tag::Tag> {
        (!self.is_baby()).then_some(&tag::Item::MINECRAFT_PIGLIN_PREFERRED_WEAPONS)
    }

    fn can_use_non_melee_weapon(&self, stack: &ItemStack) -> bool {
        stack.item.id == Item::CROSSBOW.id
    }

    fn non_melee_weapon_range(&self) -> Option<i32> {
        (self.main_hand_item().item.id == Item::CROSSBOW.id)
            .then_some(Self::CROSSBOW_PROJECTILE_RANGE)
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
        let new_wanted = piglin_ai::is_loved_item(new_item)
            || preferred.is_some_and(|weapons| new_item.item.has_tag(weapons));
        let current_wanted = piglin_ai::is_loved_item(current_item)
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
