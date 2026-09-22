use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicI64, AtomicU8, Ordering};

use crate::entity::{Entity, EntityBase, living::LivingEntity, player::Player};
use crossbeam::atomic::AtomicCell;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{
    damage::DamageType,
    data_component_impl::{EquipmentSlot, EquipmentType, EquippableImpl},
    entity::EntityStatus,
    item::Item,
    particle::Particle,
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::math::{euler_angle::EulerAngle, vector3::Vector3};

#[derive(Debug, Clone, Copy)]
pub struct PackedRotation {
    pub head: EulerAngle,
    pub body: EulerAngle,
    pub left_arm: EulerAngle,
    pub right_arm: EulerAngle,
    pub left_leg: EulerAngle,
    pub right_leg: EulerAngle,
}

impl Default for PackedRotation {
    fn default() -> Self {
        Self {
            head: EulerAngle::new(0.0, 0.0, 0.0),
            body: EulerAngle::new(0.0, 0.0, 0.0),
            left_arm: EulerAngle::new(-10.0, 0.0, -10.0),
            right_arm: EulerAngle::new(-15.0, 0.0, 10.0),
            left_leg: EulerAngle::new(-1.0, 0.0, -1.0),
            right_leg: EulerAngle::new(1.0, 0.0, 1.0),
        }
    }
}

impl From<PackedRotation> for NbtTag {
    fn from(val: PackedRotation) -> Self {
        let mut compound = NbtCompound::new();
        compound.put("Head", val.head);
        compound.put("Body", val.body);
        compound.put("LeftArm", val.left_arm);
        compound.put("RightArm", val.right_arm);
        compound.put("LeftLeg", val.left_leg);
        compound.put("RightLeg", val.right_leg);
        Self::Compound(compound)
    }
}

impl From<NbtTag> for PackedRotation {
    #[expect(clippy::unnecessary_fallible_conversions)]
    fn from(tag: NbtTag) -> Self {
        if let NbtTag::Compound(compound) = tag {
            fn get_rotation(
                compound: &NbtCompound,
                key: &'static str,
                default: EulerAngle,
            ) -> EulerAngle {
                compound
                    .get(key)
                    .and_then(|tag| tag.clone().try_into().ok())
                    .unwrap_or(default)
            }

            let default = Self::default();

            Self {
                head: get_rotation(&compound, "Head", default.head),
                body: get_rotation(&compound, "Body", default.body),
                left_arm: get_rotation(&compound, "LeftArm", default.left_arm),
                right_arm: get_rotation(&compound, "RightArm", default.right_arm),
                left_leg: get_rotation(&compound, "LeftLeg", default.left_leg),
                right_leg: get_rotation(&compound, "RightLeg", default.right_leg),
            }
        } else {
            Self::default()
        }
    }
}

pub struct ArmorStandEntity {
    living_entity: LivingEntity,

    armor_stand_flags: AtomicU8,
    last_hit_time: AtomicI64,
    disabled_slots: AtomicI32,

    rotation: AtomicCell<PackedRotation>,
}

impl ArmorStandEntity {
    pub fn new(entity: Entity) -> Self {
        let living_entity = LivingEntity::new(entity);
        let packed_rotation = PackedRotation::default();

        Self {
            living_entity,
            armor_stand_flags: AtomicU8::new(0),
            last_hit_time: AtomicI64::new(0),
            disabled_slots: AtomicI32::new(0),
            rotation: AtomicCell::new(packed_rotation),
        }
    }

    pub fn set_small(&self, small: bool) {
        self.set_bit_field(ArmorStandFlags::Small, small);
    }

    pub fn is_small(&self) -> bool {
        (self.armor_stand_flags.load(Ordering::Relaxed) & ArmorStandFlags::Small as u8) != 0
    }

    pub fn set_show_arms(&self, show_arms: bool) {
        self.set_bit_field(ArmorStandFlags::ShowArms, show_arms);
    }

    pub fn should_show_arms(&self) -> bool {
        (self.armor_stand_flags.load(Ordering::Relaxed) & ArmorStandFlags::ShowArms as u8) != 0
    }

    pub fn set_hide_base_plate(&self, hide_base_plate: bool) {
        self.set_bit_field(ArmorStandFlags::HideBasePlate, hide_base_plate);
    }

    pub fn should_show_base_plate(&self) -> bool {
        (self.armor_stand_flags.load(Ordering::Relaxed) & ArmorStandFlags::HideBasePlate as u8) == 0
    }

    pub fn set_marker(&self, marker: bool) {
        self.set_bit_field(ArmorStandFlags::Marker, marker);
    }

    pub fn is_marker(&self) -> bool {
        (self.armor_stand_flags.load(Ordering::Relaxed) & ArmorStandFlags::Marker as u8) != 0
    }

    fn set_bit_field(&self, bit_field: ArmorStandFlags, set: bool) {
        let current = self.armor_stand_flags.load(Ordering::Relaxed);
        let new_value = if set {
            current | bit_field as u8
        } else {
            current & !(bit_field as u8)
        };
        self.armor_stand_flags.store(new_value, Ordering::Relaxed);
    }

    pub fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        !matches!(slot, EquipmentSlot::Body(_) | EquipmentSlot::Saddle(_))
            && !self.is_slot_disabled(slot)
    }

    pub fn is_slot_disabled(&self, slot: &EquipmentSlot) -> bool {
        let disabled_slots = self.disabled_slots.load(Ordering::Relaxed);
        let slot_bit = Self::slot_bit(slot, 0);

        (disabled_slots & slot_bit) != 0
            || (slot.slot_type() == EquipmentType::Hand && !self.should_show_arms())
    }

    pub fn set_slot_disabled(&self, slot: &EquipmentSlot, disabled: bool) {
        let slot_bit = Self::slot_bit(slot, 0);
        let current = self.disabled_slots.load(Ordering::Relaxed);

        let new_val = if disabled {
            current | slot_bit
        } else {
            current & !slot_bit
        };

        self.disabled_slots.store(new_val, Ordering::Relaxed);
    }

    pub fn is_invisible(&self) -> bool {
        self.get_entity().invisible.load(Ordering::Relaxed)
    }

    pub fn pack_rotation(&self) -> PackedRotation {
        self.rotation.load()
    }

    pub fn unpack_rotation(&self, packed: &PackedRotation) {
        self.rotation.store(packed.to_owned());
    }

    fn drop_equipment(&self) {
        let entity = self.get_entity();
        let stacks = {
            let mut equipment = self
                .living_entity
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            take_non_empty_equipment(&mut equipment)
        };
        let world = entity.world.load();
        let block_pos = entity.block_pos.load();

        for stack in stacks {
            world.drop_stack(&block_pos, stack);
        }
    }

    fn break_and_drop_items(&self) {
        let entity = self.get_entity();
        //let name = entity.custom_name.unwrap_or(entity.get_name());

        //TODO: i am stupid! let armor_stand_item = ItemStack::new_with_component(1, &Item::ARMOR_STAND, vec![(DataComponent::CustomName, self.get_custom_name())]);
        let armor_stand_item = ItemStack::new(1, &Item::ARMOR_STAND);
        self.drop_equipment();
        entity
            .world
            .load()
            .drop_stack(&entity.block_pos.load(), armor_stand_item);

        Self::on_break(entity);
    }

    fn on_break(entity: &Entity) {
        let world = entity.world.load();
        world.play_sound(
            Sound::EntityArmorStandBreak,
            SoundCategory::Neutral,
            &entity.pos.load(),
        );
    }

    /// Spawns break particles at the armor stand's position.
    // TODO: use oak plank block particles like vanilla (requires block state data in particle system)
    fn spawn_break_particles(entity: &Entity) {
        let world = entity.world.load();
        let pos = entity.pos.load();
        let width = entity.width();
        let height = entity.height();

        // Spawn particles similar to vanilla: 10 particles with offset based on entity size
        world.spawn_particle(
            Vector3::new(pos.x, pos.y + f64::from(height) * 0.6666, pos.z),
            Vector3::new(width / 4.0, height / 4.0, width / 4.0),
            0.05,
            10,
            Particle::Poof,
        );
    }

    fn item_in_slot(&self, slot: &EquipmentSlot) -> ItemStack {
        self.living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(slot)
    }

    fn set_item_slot(&self, slot: &EquipmentSlot, stack: ItemStack) {
        self.living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .put(slot, stack.clone());
        self.living_entity
            .send_equipment_changes(&[(slot.clone(), stack)]);
    }

    fn get_clicked_slot(&self, position: Vector3<f64>) -> EquipmentSlot {
        let scale = if self.is_small() { 0.5 } else { 1.0 };
        clicked_slot(position.y / scale, self.is_small(), |slot| {
            !self.item_in_slot(slot).is_empty()
        })
        .clone()
    }

    fn swap_item(
        &self,
        player: &Arc<Player>,
        slot: &EquipmentSlot,
        item_stack: &mut ItemStack,
    ) -> bool {
        let stand_stack = self.item_in_slot(slot);
        let Some(new_stand_stack) = swap_item_stacks(
            &stand_stack,
            item_stack,
            player.is_creative(),
            self.is_slot_insertion_disabled(slot),
            self.is_slot_removal_disabled(slot),
        ) else {
            return false;
        };

        self.set_item_slot(slot, new_stand_stack);
        true
    }

    fn interact_at_position(
        &self,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        position: Option<Vector3<f64>>,
    ) -> bool {
        if self.is_marker() || item_stack.item.id == Item::NAME_TAG.id {
            return false;
        }

        let item_slot = item_equipment_slot(item_stack);
        let clicked_slot = position.map_or(EquipmentSlot::MAIN_HAND, |position| {
            self.get_clicked_slot(position)
        });
        let slot = select_interaction_slot(item_stack.is_empty(), &clicked_slot, &item_slot);

        if !self.can_use_slot(slot) {
            return false;
        }

        self.swap_item(player, slot, item_stack)
    }

    const fn slot_bit(slot: &EquipmentSlot, offset: i32) -> i32 {
        1 << (slot.get_slot_index() + offset)
    }

    fn is_slot_removal_disabled(&self, slot: &EquipmentSlot) -> bool {
        self.disabled_slots.load(Ordering::Relaxed) & Self::slot_bit(slot, 8) != 0
    }

    fn is_slot_insertion_disabled(&self, slot: &EquipmentSlot) -> bool {
        self.disabled_slots.load(Ordering::Relaxed) & Self::slot_bit(slot, 16) != 0
    }
}

const fn select_interaction_slot<'a>(
    item_stack_empty: bool,
    clicked_slot: &'a EquipmentSlot,
    item_slot: &'a EquipmentSlot,
) -> &'a EquipmentSlot {
    if item_stack_empty {
        clicked_slot
    } else {
        item_slot
    }
}

fn take_non_empty_equipment(
    equipment: &mut pumpkin_inventory::entity_equipment::EntityEquipment,
) -> Vec<ItemStack> {
    equipment
        .equipment
        .drain()
        .filter_map(|(_, stack)| (!stack.is_empty()).then_some(stack))
        .collect()
}

fn item_equipment_slot(item_stack: &ItemStack) -> EquipmentSlot {
    item_stack
        .get_data_component::<EquippableImpl>()
        .map_or(EquipmentSlot::MAIN_HAND, |equippable| {
            (*equippable.slot).clone()
        })
}

fn swap_item_stacks(
    stand_stack: &ItemStack,
    item_stack: &mut ItemStack,
    creative: bool,
    insertion_disabled: bool,
    removal_disabled: bool,
) -> Option<ItemStack> {
    let stand_empty = stand_stack.is_empty();
    if stand_empty {
        if insertion_disabled || item_stack.is_empty() {
            return None;
        }
    } else if removal_disabled {
        return None;
    }

    if creative && stand_empty {
        return Some(item_stack.copy_with_count(1));
    }

    if item_stack.item_count > 1 {
        if stand_empty {
            let new_stand_stack = item_stack.copy_with_count(1);
            item_stack.decrement(1);
            if item_stack.is_empty() {
                item_stack.clear();
            }
            return Some(new_stand_stack);
        }
        return None;
    }

    let new_stand_stack = item_stack.clone();
    *item_stack = stand_stack.clone();
    Some(new_stand_stack)
}

fn clicked_slot(
    y: f64,
    small: bool,
    has_item: impl Fn(&EquipmentSlot) -> bool,
) -> &'static EquipmentSlot {
    let mut slot = &EquipmentSlot::MAIN_HAND;
    let feet_top = 0.1 + if small { 0.8 } else { 0.45 };
    let chest_bottom = 0.9 + if small { 0.3 } else { 0.0 };
    let chest_top = 0.9 + if small { 1.0 } else { 0.7 };
    let legs_top = 0.4 + if small { 1.0 } else { 0.8 };

    if y >= 0.1 && y < feet_top && has_item(&EquipmentSlot::FEET) {
        slot = &EquipmentSlot::FEET;
    } else if y >= chest_bottom && y < chest_top && has_item(&EquipmentSlot::CHEST) {
        slot = &EquipmentSlot::CHEST;
    } else if y >= 0.4 && y < legs_top && has_item(&EquipmentSlot::LEGS) {
        slot = &EquipmentSlot::LEGS;
    } else if y >= 1.6 && has_item(&EquipmentSlot::HEAD) {
        slot = &EquipmentSlot::HEAD;
    } else if !has_item(&EquipmentSlot::MAIN_HAND) && has_item(&EquipmentSlot::OFF_HAND) {
        slot = &EquipmentSlot::OFF_HAND;
    }

    slot
}

impl EntityBase for ArmorStandEntity {
    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        let disabled_slots = self.disabled_slots.load(Ordering::Relaxed);
        // ...

        nbt.put_bool("Invisible", self.is_invisible());
        nbt.put_bool("Small", self.is_small());
        nbt.put_bool("ShowArms", self.should_show_arms());
        nbt.put_int("DisabledSlots", disabled_slots);
        nbt.put_bool("NoBasePlate", !self.should_show_base_plate());
        if self.is_marker() {
            nbt.put_bool("Marker", true);
        }

        nbt.put("Pose", self.pack_rotation());
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        let mut flags = 0u8;
        // ...

        if let Some(invisible) = nbt.get_bool("Invisible")
            && invisible
        {
            self.get_entity().set_invisible(invisible);
        }

        if let Some(small) = nbt.get_bool("Small")
            && small
        {
            flags |= ArmorStandFlags::Small as u8;
        }

        if let Some(show_arms) = nbt.get_bool("ShowArms")
            && show_arms
        {
            flags |= ArmorStandFlags::ShowArms as u8;
        }

        if let Some(disabled_slots) = nbt.get_int("DisabledSlots") {
            self.disabled_slots.store(disabled_slots, Ordering::Relaxed);
        }

        if let Some(no_base_plate) = nbt.get_bool("NoBasePlate") {
            if !no_base_plate {
                flags |= ArmorStandFlags::HideBasePlate as u8;
            }
        } else {
            flags |= ArmorStandFlags::HideBasePlate as u8;
        }

        if let Some(marker) = nbt.get_bool("Marker")
            && marker
        {
            flags |= ArmorStandFlags::Marker as u8;
        }

        self.armor_stand_flags.store(flags, Ordering::Relaxed);

        if let Some(pose_tag) = nbt.get("Pose") {
            let packed: PackedRotation = pose_tag.clone().into();
            self.unpack_rotation(&packed);
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.living_entity)
    }

    fn kill(&self, _caller: &dyn EntityBase) {
        self.get_entity().remove();
        // TODO: emit GameEvent::ENTITY_DIE
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        _amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        let entity = self.get_entity();
        if entity.is_removed() {
            return false;
        }

        let world = entity.world.load();

        let mob_griefing_gamerule = {
            let game_rules = &world.level_info.load().game_rules;
            game_rules.mob_griefing
        };

        if !mob_griefing_gamerule && source.is_some_and(|source| source.get_player().is_none()) {
            return false;
        }

        let bypasses_invulnerability =
            damage_type == DamageType::OUT_OF_WORLD || damage_type == DamageType::GENERIC_KILL;

        if bypasses_invulnerability {
            entity.remove();
            return false;
        }

        if entity.is_invulnerable_to(&damage_type) || self.is_invisible() || self.is_marker() {
            return false;
        }

        let is_explosion = damage_type == DamageType::FIREWORKS
            || damage_type == DamageType::EXPLOSION
            || damage_type == DamageType::PLAYER_EXPLOSION
            || damage_type == DamageType::BAD_RESPAWN_POINT;

        if is_explosion {
            self.drop_equipment();
            Self::on_break(entity);
            entity.remove();
            return false;
        }

        // TODO: IGNITES_ARMOR_STANDS (in_fire, campfire) - set on fire
        // TODO: BURNS_ARMOR_STANDS (on_fire) - reduce health

        let can_break = damage_type == DamageType::PLAYER_EXPLOSION
            || damage_type == DamageType::PLAYER_ATTACK
            || damage_type == DamageType::SPEAR
            || damage_type == DamageType::MACE_SMASH;

        let always_kills = damage_type == DamageType::ARROW
            || damage_type == DamageType::TRIDENT
            || damage_type == DamageType::FIREBALL
            || damage_type == DamageType::WITHER_SKULL
            || damage_type == DamageType::WIND_CHARGE;

        if !can_break && !always_kills {
            return false;
        }

        let attacker = cause.or(source);
        if let Some(attacker) = attacker
            && let Some(player) = attacker.get_player()
        {
            if !player
                .abilities
                .try_lock()
                .is_ok_and(|a| a.allow_modify_world)
            {
                return false;
            } else if player.is_creative() {
                Self::spawn_break_particles(entity);
                entity.remove();
                return true;
            }
        }

        let time = world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .query_gametime();

        if time - self.last_hit_time.load(Ordering::Relaxed) > 5 && !always_kills {
            world.send_entity_status(entity, EntityStatus::ArmorstandWobble, None);
            world.play_sound(
                Sound::EntityArmorStandHit,
                SoundCategory::Neutral,
                &entity.block_pos.load().to_f64(),
            );
            self.last_hit_time.store(time, Ordering::Relaxed);
        } else {
            Self::spawn_break_particles(entity);
            world.play_sound(
                Sound::EntityArmorStandBreak,
                SoundCategory::Neutral,
                &entity.block_pos.load().to_f64(),
            );
            self.break_and_drop_items();
            entity.remove();
        }

        true
    }

    fn get_gravity(&self) -> f64 {
        0.08
    }

    fn interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.interact_at_position(player, item_stack, None)
    }

    fn interact_at(
        &self,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        position: Vector3<f64>,
    ) -> bool {
        self.interact_at_position(player, item_stack, Some(position))
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub enum ArmorStandFlags {
    /// Small armor stand Flag
    Small = 1,
    /// Show arms Flag
    ShowArms = 4,
    /// Hide base plate fLag
    HideBasePlate = 8,
    /// Marker Flag
    Marker = 16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clicked_slot_uses_vanilla_hit_area_and_existing_items() {
        let has_item = |slot: &EquipmentSlot| *slot == EquipmentSlot::FEET;
        assert!(*clicked_slot(0.2, false, has_item) == EquipmentSlot::FEET);

        let has_item = |slot: &EquipmentSlot| *slot == EquipmentSlot::CHEST;
        assert!(*clicked_slot(1.0, false, has_item) == EquipmentSlot::CHEST);

        let has_item = |slot: &EquipmentSlot| *slot == EquipmentSlot::LEGS;
        assert!(*clicked_slot(0.6, false, has_item) == EquipmentSlot::LEGS);

        let has_item = |slot: &EquipmentSlot| *slot == EquipmentSlot::HEAD;
        assert!(*clicked_slot(1.6, false, has_item) == EquipmentSlot::HEAD);
    }

    #[test]
    fn clicked_slot_falls_back_to_the_offhand_when_mainhand_is_empty() {
        assert!(
            *clicked_slot(0.0, false, |slot| *slot == EquipmentSlot::OFF_HAND)
                == EquipmentSlot::OFF_HAND
        );
    }

    #[test]
    fn disabled_slot_bits_use_equipment_slot_indices() {
        assert_eq!(ArmorStandEntity::slot_bit(&EquipmentSlot::MAIN_HAND, 0), 1);
        assert_eq!(ArmorStandEntity::slot_bit(&EquipmentSlot::FEET, 0), 2);
        assert_eq!(ArmorStandEntity::slot_bit(&EquipmentSlot::HEAD, 8), 1 << 12);
        assert_eq!(
            ArmorStandEntity::slot_bit(&EquipmentSlot::OFF_HAND, 16),
            1 << 21
        );
    }

    #[test]
    fn equippable_items_select_their_vanilla_slot() {
        let helmet = ItemStack::new(1, &Item::DIAMOND_HELMET);
        assert!(item_equipment_slot(&helmet) == EquipmentSlot::HEAD);
    }

    #[test]
    fn stack_swap_inserts_one_from_a_stack() {
        let mut held = ItemStack::new(3, &Item::DIAMOND_HELMET);
        let new_stand_stack = swap_item_stacks(ItemStack::EMPTY, &mut held, false, false, false)
            .expect("an empty stand accepts an item");

        assert_eq!(new_stand_stack.item.id, Item::DIAMOND_HELMET.id);
        assert_eq!(new_stand_stack.item_count, 1);
        assert_eq!(held.item_count, 2);
    }

    #[test]
    fn stack_swap_removes_or_swaps_single_items() {
        let stand_stack = ItemStack::new(1, &Item::DIAMOND_HELMET);
        let mut empty_hand = ItemStack::EMPTY.clone();
        let removed = swap_item_stacks(&stand_stack, &mut empty_hand, false, false, false)
            .expect("an occupied stand allows removal");

        assert!(removed.is_empty());
        assert_eq!(empty_hand.item.id, Item::DIAMOND_HELMET.id);

        let mut replacement = ItemStack::new(1, &Item::DIAMOND_CHESTPLATE);
        let swapped = swap_item_stacks(&stand_stack, &mut replacement, false, false, false)
            .expect("a single item swaps with the stand item");

        assert_eq!(swapped.item.id, Item::DIAMOND_CHESTPLATE.id);
        assert_eq!(replacement.item.id, Item::DIAMOND_HELMET.id);
    }

    #[test]
    fn stack_swap_honors_insertion_and_removal_locks() {
        let mut held = ItemStack::new(1, &Item::DIAMOND_HELMET);
        assert!(swap_item_stacks(ItemStack::EMPTY, &mut held, false, true, false).is_none());
        assert_eq!(held.item_count, 1);

        let stand_stack = ItemStack::new(1, &Item::DIAMOND_HELMET);
        let mut empty_hand = ItemStack::EMPTY.clone();
        assert!(swap_item_stacks(&stand_stack, &mut empty_hand, false, false, true).is_none());
        assert!(empty_hand.is_empty());
    }

    #[test]
    fn creative_stack_swap_keeps_the_held_stack() {
        let mut held = ItemStack::new(4, &Item::DIAMOND_HELMET);
        let new_stand_stack = swap_item_stacks(ItemStack::EMPTY, &mut held, true, false, false)
            .expect("creative players can insert into an empty stand slot");

        assert_eq!(new_stand_stack.item_count, 1);
        assert_eq!(held.item_count, 4);
    }

    #[test]
    fn taking_equipment_drains_non_empty_stacks() {
        let mut equipment = pumpkin_inventory::entity_equipment::EntityEquipment::new();
        equipment.put(
            &EquipmentSlot::HEAD,
            ItemStack::new(1, &Item::DIAMOND_HELMET),
        );
        equipment.put(&EquipmentSlot::FEET, ItemStack::EMPTY.clone());

        let stacks = take_non_empty_equipment(&mut equipment);

        assert_eq!(stacks.len(), 1);
        assert_eq!(stacks[0].item.id, Item::DIAMOND_HELMET.id);
        assert!(equipment.equipment.is_empty());
    }

    #[test]
    fn empty_hand_keeps_the_clicked_slot_for_disabled_slot_checks() {
        let clicked_slot = EquipmentSlot::HEAD;
        let item_slot = EquipmentSlot::MAIN_HAND;

        assert!(select_interaction_slot(true, &clicked_slot, &item_slot) == &clicked_slot);
        assert!(select_interaction_slot(false, &clicked_slot, &item_slot) == &item_slot);
    }
}
