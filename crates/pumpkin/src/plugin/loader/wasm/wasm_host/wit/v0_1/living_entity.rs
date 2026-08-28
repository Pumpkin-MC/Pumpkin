use std::sync::Arc;
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{LivingEntityResource, PluginHostState},
    wit::v0_1::pumpkin::plugin::{
        attributes::{
            Attribute, AttributeModifier as WitAttributeModifier,
            ModifierOperation as WitModifierOperation,
        },
        damage_types::DamageType as WitDamageType,
        item_stack::ItemStack as WitHostItemStack,
        text::TextComponent,
        world::{
            Entity, EquipmentSlot as WitEquipmentSlot, HostLivingEntity,
            LivingEntity as WitLivingEntity, Mob as WitMob,
        },
    },
};

pub fn living_entity_from_resource(
    state: &PluginHostState,
    entity: &Resource<WitLivingEntity>,
) -> wasmtime::Result<std::sync::Arc<dyn crate::entity::EntityBase>> {
    state
        .resource_table
        .get::<LivingEntityResource>(&Resource::new_own(entity.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid living entity resource handle"))
        .map(|resource| resource.provider.clone())
}

#[must_use]
pub const fn from_wit_attribute(attr: Attribute) -> &'static pumpkin_data::attributes::Attributes {
    use pumpkin_data::attributes::Attributes;
    match attr {
        Attribute::AirDragModifier => &Attributes::AIR_DRAG_MODIFIER,
        Attribute::Armor => &Attributes::ARMOR,
        Attribute::ArmorToughness => &Attributes::ARMOR_TOUGHNESS,
        Attribute::AttackDamage => &Attributes::ATTACK_DAMAGE,
        Attribute::AttackKnockback => &Attributes::ATTACK_KNOCKBACK,
        Attribute::AttackSpeed => &Attributes::ATTACK_SPEED,
        Attribute::BelowNameDistance => &Attributes::BELOW_NAME_DISTANCE,
        Attribute::BlockBreakSpeed => &Attributes::BLOCK_BREAK_SPEED,
        Attribute::BlockInteractionRange => &Attributes::BLOCK_INTERACTION_RANGE,
        Attribute::Bounciness => &Attributes::BOUNCINESS,
        Attribute::BurningTime => &Attributes::BURNING_TIME,
        Attribute::CameraDistance => &Attributes::CAMERA_DISTANCE,
        Attribute::ExplosionKnockbackResistance => &Attributes::EXPLOSION_KNOCKBACK_RESISTANCE,
        Attribute::EntityInteractionRange => &Attributes::ENTITY_INTERACTION_RANGE,
        Attribute::FallDamageMultiplier => &Attributes::FALL_DAMAGE_MULTIPLIER,
        Attribute::FlyingSpeed => &Attributes::FLYING_SPEED,
        Attribute::FollowRange => &Attributes::FOLLOW_RANGE,
        Attribute::FrictionModifier => &Attributes::FRICTION_MODIFIER,
        Attribute::Gravity => &Attributes::GRAVITY,
        Attribute::JumpStrength => &Attributes::JUMP_STRENGTH,
        Attribute::KnockbackResistance => &Attributes::KNOCKBACK_RESISTANCE,
        Attribute::Luck => &Attributes::LUCK,
        Attribute::MaxAbsorption => &Attributes::MAX_ABSORPTION,
        Attribute::MaxHealth => &Attributes::MAX_HEALTH,
        Attribute::MiningEfficiency => &Attributes::MINING_EFFICIENCY,
        Attribute::MovementEfficiency => &Attributes::MOVEMENT_EFFICIENCY,
        Attribute::MovementSpeed => &Attributes::MOVEMENT_SPEED,
        Attribute::NameTagDistance => &Attributes::NAME_TAG_DISTANCE,
        Attribute::OxygenBonus => &Attributes::OXYGEN_BONUS,
        Attribute::SafeFallDistance => &Attributes::SAFE_FALL_DISTANCE,
        Attribute::Scale => &Attributes::SCALE,
        Attribute::SneakingSpeed => &Attributes::SNEAKING_SPEED,
        Attribute::SpawnReinforcements => &Attributes::SPAWN_REINFORCEMENTS,
        Attribute::StepHeight => &Attributes::STEP_HEIGHT,
        Attribute::SubmergedMiningSpeed => &Attributes::SUBMERGED_MINING_SPEED,
        Attribute::SweepingDamageRatio => &Attributes::SWEEPING_DAMAGE_RATIO,
        Attribute::TemptRange => &Attributes::TEMPT_RANGE,
        Attribute::WaterMovementEfficiency => &Attributes::WATER_MOVEMENT_EFFICIENCY,
        Attribute::WaypointTransmitRange => &Attributes::WAYPOINT_TRANSMIT_RANGE,
        Attribute::WaypointReceiveRange => &Attributes::WAYPOINT_RECEIVE_RANGE,
    }
}

#[must_use]
pub const fn from_wit_modifier_op(
    op: WitModifierOperation,
) -> crate::entity::attributes::ModifierOperation {
    match op {
        WitModifierOperation::Add => crate::entity::attributes::ModifierOperation::Add,
        WitModifierOperation::MultiplyBase => {
            crate::entity::attributes::ModifierOperation::MultiplyBase
        }
        WitModifierOperation::MultiplyTotal => {
            crate::entity::attributes::ModifierOperation::MultiplyTotal
        }
    }
}

#[must_use]
pub const fn to_wit_modifier_op(
    op: crate::entity::attributes::ModifierOperation,
) -> WitModifierOperation {
    match op {
        crate::entity::attributes::ModifierOperation::Add => WitModifierOperation::Add,
        crate::entity::attributes::ModifierOperation::MultiplyBase => {
            WitModifierOperation::MultiplyBase
        }
        crate::entity::attributes::ModifierOperation::MultiplyTotal => {
            WitModifierOperation::MultiplyTotal
        }
    }
}

#[must_use]
pub const fn from_wit_equipment_slot(
    slot: WitEquipmentSlot,
) -> pumpkin_data::data_component_impl::EquipmentSlot {
    use pumpkin_data::data_component_impl::EquipmentSlot;
    match slot {
        WitEquipmentSlot::MainHand => EquipmentSlot::MAIN_HAND,
        WitEquipmentSlot::OffHand => EquipmentSlot::OFF_HAND,
        WitEquipmentSlot::Feet => EquipmentSlot::FEET,
        WitEquipmentSlot::Legs => EquipmentSlot::LEGS,
        WitEquipmentSlot::Chest => EquipmentSlot::CHEST,
        WitEquipmentSlot::Head => EquipmentSlot::HEAD,
        WitEquipmentSlot::Body => EquipmentSlot::BODY,
        WitEquipmentSlot::Saddle => EquipmentSlot::SADDLE,
    }
}

#[must_use]
pub const fn to_wit_damage_type(damage_type: &pumpkin_data::damage::DamageType) -> WitDamageType {
    // SAFETY: WIT enum is generated in the same order as the internal enum / id
    unsafe { std::mem::transmute(damage_type.id) }
}

#[must_use]
pub fn from_wit_damage_type(wit: WitDamageType) -> pumpkin_data::damage::DamageType {
    pumpkin_data::damage::DamageType::from_id(wit as u8)
        .unwrap_or(pumpkin_data::damage::DamageType::GENERIC)
}

impl HostLivingEntity for PluginHostState {
    async fn as_entity(
        &mut self,
        this: Resource<WitLivingEntity>,
    ) -> wasmtime::Result<Resource<Entity>> {
        let entity = living_entity_from_resource(self, &this)?;
        self.add_entity(entity)
    }

    async fn as_mob(
        &mut self,
        this: Resource<WitLivingEntity>,
    ) -> wasmtime::Result<Option<Resource<WitMob>>> {
        let entity = living_entity_from_resource(self, &this)?;
        if entity.get_mob().is_some() {
            Ok(Some(self.add_mob(entity)?))
        } else {
            Ok(None)
        }
    }

    async fn is_mob(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<bool> {
        let entity = living_entity_from_resource(self, &this)?;
        Ok(entity.get_mob().is_some())
    }

    async fn get_health(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<f32> {
        let entity = living_entity_from_resource(self, &this)?;
        Ok(entity
            .get_living_entity()
            .map_or(0.0, |living| living.health.load()))
    }

    async fn set_health(
        &mut self,
        this: Resource<WitLivingEntity>,
        health: f32,
    ) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        if let Some(living) = entity.get_living_entity() {
            living.health.store(health);
        }
        Ok(())
    }

    async fn get_max_health(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<f32> {
        let entity = living_entity_from_resource(self, &this)?;
        Ok(entity
            .get_living_entity()
            .map_or(0.0, crate::entity::living::LivingEntity::get_max_health))
    }

    async fn set_max_health(
        &mut self,
        this: Resource<WitLivingEntity>,
        max_health: f32,
    ) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        if let Some(living) = entity.get_living_entity() {
            living.set_max_health(max_health);
        }
        Ok(())
    }

    async fn damage(
        &mut self,
        this: Resource<WitLivingEntity>,
        amount: f32,
        damage_type: WitDamageType,
    ) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        entity.damage(&*entity, amount, from_wit_damage_type(damage_type));
        Ok(())
    }

    async fn is_dead(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<bool> {
        let entity = living_entity_from_resource(self, &this)?;
        Ok(entity.get_living_entity().map_or_else(
            || entity.get_entity().removal_reason.load().is_some(),
            |living| living.dead.load(std::sync::atomic::Ordering::Relaxed),
        ))
    }

    async fn get_absorption(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<f32> {
        let entity = living_entity_from_resource(self, &this)?;
        Ok(entity
            .get_living_entity()
            .map_or(0.0, |living| living.absorption.load()))
    }

    async fn set_absorption(
        &mut self,
        this: Resource<WitLivingEntity>,
        amount: f32,
    ) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        if let Some(living) = entity.get_living_entity() {
            living.absorption.store(amount);
        }
        Ok(())
    }

    async fn get_attribute_value(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
    ) -> wasmtime::Result<f64> {
        let entity = living_entity_from_resource(self, &this)?;
        let attribute = from_wit_attribute(attr);
        Ok(entity
            .get_living_entity()
            .map_or(attribute.default_value, |living| {
                living.get_attribute_value(attribute)
            }))
    }

    async fn get_attribute_base(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
    ) -> wasmtime::Result<f64> {
        let entity = living_entity_from_resource(self, &this)?;
        let attribute = from_wit_attribute(attr);
        Ok(entity
            .get_living_entity()
            .map_or(attribute.default_value, |living| {
                living.get_attribute_base(attribute)
            }))
    }

    async fn set_attribute_base(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
        value: f64,
    ) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        let attribute = from_wit_attribute(attr);
        if let Some(living) = entity.get_living_entity() {
            living.set_attribute_base(attribute, value);
            crate::entity::attributes::send_attribute_updates_for_living(
                living,
                vec![attribute.clone()],
            );
        }
        Ok(())
    }

    async fn add_attribute_modifier(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
        modifier: WitAttributeModifier,
    ) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        let attribute = from_wit_attribute(attr);
        if let Some(living) = entity.get_living_entity() {
            let internal_mod = crate::entity::attributes::Modifier {
                id: modifier.id,
                amount: modifier.amount,
                operation: from_wit_modifier_op(modifier.operation),
            };
            living.update_attribute(attribute, |inst| inst.add_or_replace_modifier(internal_mod));
            crate::entity::attributes::send_attribute_updates_for_living(
                living,
                vec![attribute.clone()],
            );
        }
        Ok(())
    }

    async fn remove_attribute_modifier(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
        id: String,
    ) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        let attribute = from_wit_attribute(attr);
        if let Some(living) = entity.get_living_entity() {
            living.update_attribute(attribute, |inst| inst.remove_modifier(&id));
            crate::entity::attributes::send_attribute_updates_for_living(
                living,
                vec![attribute.clone()],
            );
        }
        Ok(())
    }

    async fn get_attribute_modifiers(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
    ) -> wasmtime::Result<Vec<WitAttributeModifier>> {
        let entity = living_entity_from_resource(self, &this)?;
        let attribute = from_wit_attribute(attr);
        if let Some(living) = entity.get_living_entity() {
            let map = living
                .attributes
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(inst) = map.get(&attribute.id) {
                return Ok(inst
                    .modifiers
                    .iter()
                    .map(|m| WitAttributeModifier {
                        id: m.id.clone(),
                        amount: m.amount,
                        operation: to_wit_modifier_op(m.operation),
                    })
                    .collect());
            }
        }
        Ok(Vec::new())
    }

    async fn reset_attribute(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
    ) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        let attribute = from_wit_attribute(attr);
        if let Some(living) = entity.get_living_entity() {
            {
                let mut map = living
                    .attributes
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                map.remove(&attribute.id);
            };
            crate::entity::attributes::send_attribute_updates_for_living(
                living,
                vec![attribute.clone()],
            );
        }
        Ok(())
    }

    async fn reset_all_attributes(
        &mut self,
        this: Resource<WitLivingEntity>,
    ) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        if let Some(living) = entity.get_living_entity() {
            living.reset_effects_and_attributes();
        }
        Ok(())
    }

    async fn get_equipment(
        &mut self,
        this: Resource<WitLivingEntity>,
        slot: WitEquipmentSlot,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let entity = living_entity_from_resource(self, &this)?;
        if let Some(living) = entity.get_living_entity() {
            let slot = from_wit_equipment_slot(slot);
            let equipment = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let stack = equipment.get(&slot);
            if !stack.is_empty() {
                return Ok(Some(
                    self.add_item_stack(Arc::new(tokio::sync::Mutex::new(stack)))?,
                ));
            }
        }
        Ok(None)
    }

    async fn set_equipment(
        &mut self,
        this: Resource<WitLivingEntity>,
        slot: WitEquipmentSlot,
        stack: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        if let Some(living) = entity.get_living_entity() {
            let slot = from_wit_equipment_slot(slot);
            let item_stack = if let Some(stack_res) = stack {
                self.get_item_stack(&stack_res)?.lock().await.clone()
            } else {
                pumpkin_data::item_stack::ItemStack::EMPTY.clone()
            };

            {
                let mut equipment = living
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                equipment.put(&slot, item_stack.clone());
            };

            living.send_equipment_changes(&[(slot, item_stack)]);
        }
        Ok(())
    }

    async fn clear_equipment(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        if let Some(living) = entity.get_living_entity() {
            let mut equipment = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let slots_to_clear: Vec<(
                pumpkin_data::data_component_impl::EquipmentSlot,
                pumpkin_data::item_stack::ItemStack,
            )> = equipment
                .equipment
                .drain()
                .map(|(slot, _)| (slot, pumpkin_data::item_stack::ItemStack::EMPTY.clone()))
                .collect();
            drop(equipment);

            living.send_equipment_changes(&slots_to_clear);
        }
        Ok(())
    }

    async fn get_age(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<i32> {
        let entity = living_entity_from_resource(self, &this)?;
        Ok(entity.get_living_entity().map_or(0, |living| {
            living.entity.age.load(std::sync::atomic::Ordering::Relaxed)
        }))
    }

    async fn set_age(&mut self, this: Resource<WitLivingEntity>, age: i32) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        if let Some(living) = entity.get_living_entity() {
            living
                .entity
                .age
                .store(age, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }

    async fn send_system_message(
        &mut self,
        this: Resource<WitLivingEntity>,
        message: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let entity = living_entity_from_resource(self, &this)?;
        if let Some(player) = entity.get_player() {
            let text_res = self
                .resource_table
                .get::<crate::plugin::loader::wasm::wasm_host::state::TextComponentResource>(
                    &Resource::new_own(message.rep()),
                )
                .map_err(|_| wasmtime::Error::msg("invalid text component resource handle"))?;
            player.send_system_message(&text_res.provider);
        }
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<WitLivingEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<LivingEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
