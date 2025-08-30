use std::sync::{
    Arc,
    atomic::{AtomicI32, AtomicI64, AtomicU8, Ordering},
};

use crate::entity::{
    Entity, EntityBase, NBTStorage,
    living::{LivingEntity, LivingEntityTrait},
};
use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::{
    damage::DamageType,
    data_component_impl::{EquipmentSlot, EquipmentType},
    entity::EntityStatus,
    item::Item,
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{euler_angle::EulerAngle, vector3::Vector3};
use pumpkin_world::item::ItemStack;

#[derive(Debug, Clone)]
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

#[allow(dead_code)]
pub struct ArmorStandEntity {
    living_entity: LivingEntity,

    armor_stand_flags: AtomicU8,
    invisible: AtomicCell<bool>,
    last_hit_time: AtomicI64,
    disabled_slots: AtomicI32,

    head_rotation: AtomicCell<EulerAngle>,
    body_rotation: AtomicCell<EulerAngle>,
    left_arm_rotation: AtomicCell<EulerAngle>,
    right_arm_rotation: AtomicCell<EulerAngle>,
    left_leg_rotation: AtomicCell<EulerAngle>,
    right_leg_rotation: AtomicCell<EulerAngle>,
}

const SMALL_FLAG: u8 = 1;
const SHOW_ARMS_FLAG: u8 = 4;
const HIDE_BASE_PLATE_FLAG: u8 = 8;
const MARKER_FLAG: u8 = 16;

impl ArmorStandEntity {
    pub fn new(entity: Entity) -> Self {
        let living_entity = LivingEntity::new(entity);
        let packed_rotation = PackedRotation::default();

        Self {
            living_entity,
            armor_stand_flags: AtomicU8::new(0),
            invisible: AtomicCell::new(false),
            last_hit_time: AtomicI64::new(0),
            disabled_slots: AtomicI32::new(0),
            head_rotation: AtomicCell::new(packed_rotation.head),
            body_rotation: AtomicCell::new(packed_rotation.body),
            left_arm_rotation: AtomicCell::new(packed_rotation.left_arm),
            right_arm_rotation: AtomicCell::new(packed_rotation.right_arm),
            left_leg_rotation: AtomicCell::new(packed_rotation.left_leg),
            right_leg_rotation: AtomicCell::new(packed_rotation.right_leg),
        }
    }

    pub fn set_small(&self, small: bool) {
        self.set_bit_field(SMALL_FLAG, small);
    }

    pub fn is_small(&self) -> bool {
        (self.armor_stand_flags.load(Ordering::Relaxed) & SMALL_FLAG) != 0
    }

    pub fn set_show_arms(&self, show_arms: bool) {
        self.set_bit_field(SHOW_ARMS_FLAG, show_arms);
    }

    pub fn should_show_arms(&self) -> bool {
        (self.armor_stand_flags.load(Ordering::Relaxed) & SHOW_ARMS_FLAG) != 0
    }

    pub fn set_hide_base_plate(&self, hide_base_plate: bool) {
        self.set_bit_field(HIDE_BASE_PLATE_FLAG, hide_base_plate);
    }

    pub fn should_show_base_plate(&self) -> bool {
        (self.armor_stand_flags.load(Ordering::Relaxed) & HIDE_BASE_PLATE_FLAG) == 0
    }

    pub fn set_marker(&self, marker: bool) {
        self.set_bit_field(MARKER_FLAG, marker);
    }

    pub fn is_marker(&self) -> bool {
        (self.armor_stand_flags.load(Ordering::Relaxed) & MARKER_FLAG) != 0
    }

    fn set_bit_field(&self, bit_field: u8, set: bool) {
        let current = self.armor_stand_flags.load(Ordering::Relaxed);
        let new_value = if set {
            current | bit_field
        } else {
            current & !bit_field
        };
        self.armor_stand_flags.store(new_value, Ordering::Relaxed);
    }

    pub fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        !matches!(slot, EquipmentSlot::Body(_) | EquipmentSlot::Saddle(_))
            && !self.is_slot_disabled(slot)
    }

    pub fn is_slot_disabled(&self, slot: &EquipmentSlot) -> bool {
        let disabled_slots = self.disabled_slots.load(Ordering::Relaxed);
        let slot_bit = 1 << slot.get_offset_entity_slot_id(0);

        (disabled_slots & slot_bit) != 0
            || (slot.slot_type() == EquipmentType::Hand && !self.should_show_arms())
    }

    pub fn set_slot_disabled(&self, slot: &EquipmentSlot, disabled: bool) {
        let slot_bit = 1 << slot.get_offset_entity_slot_id(0);
        let current = self.disabled_slots.load(Ordering::Relaxed);

        let new_val = if disabled {
            current | slot_bit
        } else {
            current & !slot_bit
        };

        self.disabled_slots.store(new_val, Ordering::Relaxed);
    }

    pub fn set_head_rotation(&self, angle: EulerAngle) {
        self.head_rotation.store(angle);
    }

    pub fn get_head_rotation(&self) -> EulerAngle {
        self.head_rotation.load()
    }

    pub fn set_body_rotation(&self, angle: EulerAngle) {
        self.body_rotation.store(angle);
    }

    pub fn get_body_rotation(&self) -> EulerAngle {
        self.body_rotation.load()
    }

    pub fn set_left_arm_rotation(&self, angle: EulerAngle) {
        self.left_arm_rotation.store(angle);
    }

    pub fn get_left_arm_rotation(&self) -> EulerAngle {
        self.left_arm_rotation.load()
    }

    pub fn set_right_arm_rotation(&self, angle: EulerAngle) {
        self.right_arm_rotation.store(angle);
    }

    pub fn get_right_arm_rotation(&self) -> EulerAngle {
        self.right_arm_rotation.load()
    }

    pub fn set_left_leg_rotation(&self, angle: EulerAngle) {
        self.left_leg_rotation.store(angle);
    }

    pub fn get_left_leg_rotation(&self) -> EulerAngle {
        self.left_leg_rotation.load()
    }

    pub fn set_right_leg_rotation(&self, angle: EulerAngle) {
        self.right_leg_rotation.store(angle);
    }

    pub fn get_right_leg_rotation(&self) -> EulerAngle {
        self.right_leg_rotation.load()
    }

    pub fn pack_rotation(&self) -> PackedRotation {
        PackedRotation {
            head: self.get_head_rotation(),
            body: self.get_body_rotation(),
            left_arm: self.get_left_arm_rotation(),
            right_arm: self.get_right_arm_rotation(),
            left_leg: self.get_left_leg_rotation(),
            right_leg: self.get_right_leg_rotation(),
        }
    }

    pub fn unpack_rotation(&self, packed: &PackedRotation) {
        self.set_head_rotation(packed.head);
        self.set_body_rotation(packed.body);
        self.set_left_arm_rotation(packed.left_arm);
        self.set_right_arm_rotation(packed.right_arm);
        self.set_left_leg_rotation(packed.left_leg);
        self.set_right_leg_rotation(packed.right_leg);
    }

    async fn break_and_drop_items(&self) {
        let entity = self.get_entity();
        //let name = entity.custom_name.unwrap_or(entity.get_name());

        //TODO: i am stupid! let armor_stand_item = ItemStack::new_with_component(1, &Item::ARMOR_STAND, vec![(DataComponent::CustomName, self.get_custom_name())]);
        let armor_stand_item = ItemStack::new(1, &Item::ARMOR_STAND);
        entity
            .world
            .drop_stack(&entity.block_pos.load(), armor_stand_item)
            .await;

        self.on_break(entity).await;
    }

    async fn on_break(&self, entity: &Entity) {
        let world = &entity.world;
        world
            .play_sound(
                Sound::EntityArmorStandBreak,
                SoundCategory::Neutral,
                &entity.pos.load(),
            )
            .await;

        // TODO: Implement equipment slots and make them drop all of their stored items.
    }
}

#[async_trait]
impl NBTStorage for ArmorStandEntity {
    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        let disabled_slots = self.disabled_slots.load(Ordering::Relaxed);

        // TODO: ADD `ArmorStandEntity`.is_invisible()
        //nbt.put_bool("Invisible", self.is_invisible());
        nbt.put_bool("Small", self.is_small());
        nbt.put_bool("ShowArms", self.should_show_arms());
        nbt.put_int("DisabledSlots", disabled_slots);
        nbt.put_bool("NoBasePlate", !self.should_show_base_plate());
        if self.is_marker() {
            nbt.put_bool("Marker", true);
        }

        // TODO: Implement pose saving
        //nbt.put("Pose", )
    }

    async fn read_nbt_non_mut(&self, nbt: &NbtCompound) {
        let mut flags = 0u8;

        if let Some(small) = nbt.get_bool("Small")
            && small
        {
            flags |= 1;
        }

        if let Some(show_arms) = nbt.get_bool("ShowArms")
            && show_arms
        {
            flags |= 4;
        }

        if let Some(no_base_plate) = nbt.get_bool("NoBasePlate") {
            if !no_base_plate {
                flags |= 8;
            }
        } else {
            flags |= 8;
        }

        if let Some(marker) = nbt.get_bool("Marker")
            && marker
        {
            flags |= 16;
        }

        self.armor_stand_flags.store(flags, Ordering::Relaxed);

        if let Some(disabled_slots) = nbt.get_int("DisabledSlots") {
            self.disabled_slots.store(disabled_slots, Ordering::Relaxed);
        }
    }
}

#[async_trait]
impl LivingEntityTrait for ArmorStandEntity {
    //async fn on_actually_hurt()
}

#[async_trait]
impl EntityBase for ArmorStandEntity {
    fn get_entity(&self) -> &Entity {
        &self.living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.living_entity)
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    // TODO: Is this wrong here?
    async fn damage_with_context(
        &self,
        _caller: Arc<dyn EntityBase>,
        _amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        let entity = self.get_entity();
        if entity.is_removed() {
            return false;
        }

        let world = &entity.world;
        if let Some(server) = world.server.upgrade() {
            let mob_griefing_gamerule = {
                let game_rules = &server.level_info.read().await.game_rules;
                game_rules.mob_griefing
            };

            if !mob_griefing_gamerule && source.is_some_and(|source| source.get_player().is_none())
            {
                return false;
            }
        }

        // TODO: <DamageSource>.isIn(DamageTypeTags::BYPASSES_INVULNERABILITY)

        if damage_type == DamageType::EXPLOSION {
            // TODO: Implement Dropping Items that are in the Equipment Slots & entity.kill()
            self.on_break(entity).await;
            entity.remove().await;
            return false;
        } // TODO: Implement <DamageSource>.isIn(DamageTypeTags::IGNITES_ARMOR_STANDS)

        // TODO: Implement <DamageSource>.isIn(DamageTypeTags::BURNS_ARMOR_STANDS)

        /* // TODO:
        bl1: bool = <DamageSource>.isIn(DamageTypeTags.CAN_BREAK_ARMOR_STAND);
        bl2: bool = <DamageSource>.isIn(DamageTypeTags.ALWAYS_KILLS_ARMOR_STANDS);

        if !bl1 && !bl2 {
            return false;
        }
        */

        let Some(source) = source else { return false };

        // TODO: source is not giving the real player or wrong stuff cause .is_creative() is false even tho the player is in creative.
        if let Some(player) = source.get_player() {
            if !player.abilities.lock().await.allow_modify_world {
                return false;
            } else if player.is_creative() {
                world
                    .play_sound(
                        Sound::EntityArmorStandBreak,
                        SoundCategory::Neutral,
                        &entity.block_pos.load().to_f64(),
                    )
                    .await;
                self.break_and_drop_items().await;
                entity.remove().await;
                return true;
            }
        }

        let time = world.level_time.lock().await.query_gametime();

        if time - self.last_hit_time.load(Ordering::Relaxed) > 5 {
            // && !bl2 {
            world
                .send_entity_status(entity, EntityStatus::HitArmorStand)
                .await;
            world
                .play_sound(
                    Sound::EntityArmorStandHit,
                    SoundCategory::Neutral,
                    &entity.block_pos.load().to_f64(),
                )
                .await;
            self.last_hit_time.store(time, Ordering::Relaxed);
        } else {
            world
                .play_sound(
                    Sound::EntityArmorStandBreak,
                    SoundCategory::Neutral,
                    &entity.block_pos.load().to_f64(),
                )
                .await;
            self.break_and_drop_items().await;
            entity.remove().await;
        }

        true
    }
}
