use super::{AccessorExt, run_blocking};
use crate::pumpkin::plugin::world::HostLivingEntityWithStore;
use crate::{LockIgnorePoison, RwLockIgnorePoison};

use std::sync::Arc;
use wasmtime::component::{Accessor, HasSelf, Resource};

use crate::pumpkin::plugin::{
    attributes::{
        Attribute, AttributeModifier as WitAttributeModifier,
        ModifierOperation as WitModifierOperation,
    },
    damage_types::DamageType as WitDamageType,
    item_stack::ItemStack as WitHostItemStack,
    status_effect::{
        StatusEffectInstance as WitStatusEffectInstance, StatusEffectType as WitStatusEffectType,
    },
    text::TextComponent,
    world::{
        Entity, EquipmentSlot as WitEquipmentSlot, HostLivingEntity,
        LivingEntity as WitLivingEntity, Mob as WitMob,
    },
};
use pumpkin_wasm_host_common::state::PluginHostState;

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
) -> pumpkin_core::entity::attributes::ModifierOperation {
    match op {
        WitModifierOperation::Add => pumpkin_core::entity::attributes::ModifierOperation::Add,
        WitModifierOperation::MultiplyBase => {
            pumpkin_core::entity::attributes::ModifierOperation::MultiplyBase
        }
        WitModifierOperation::MultiplyTotal => {
            pumpkin_core::entity::attributes::ModifierOperation::MultiplyTotal
        }
    }
}

#[must_use]
pub const fn to_wit_modifier_op(
    op: pumpkin_core::entity::attributes::ModifierOperation,
) -> WitModifierOperation {
    match op {
        pumpkin_core::entity::attributes::ModifierOperation::Add => WitModifierOperation::Add,
        pumpkin_core::entity::attributes::ModifierOperation::MultiplyBase => {
            WitModifierOperation::MultiplyBase
        }
        pumpkin_core::entity::attributes::ModifierOperation::MultiplyTotal => {
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
    fn as_entity(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<Resource<Entity>> {
        self.add(self.get(&this)?.clone())
    }

    fn as_mob(
        &mut self,
        this: Resource<WitLivingEntity>,
    ) -> wasmtime::Result<Option<Resource<WitMob>>> {
        let entity = self.get(&this)?;
        if entity.get_mob().is_some() {
            Ok(Some(self.add(entity.clone())?))
        } else {
            Ok(None)
        }
    }

    fn is_mob(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<bool> {
        let entity = self.get(&this)?;
        Ok(entity.get_mob().is_some())
    }

    fn get_health(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<f32> {
        let entity = self.get(&this)?;
        Ok(entity
            .get_living_entity()
            .map_or(0.0, |living| living.health.load()))
    }

    fn set_health(&mut self, this: Resource<WitLivingEntity>, health: f32) -> wasmtime::Result<()> {
        let entity = self.get(&this)?;
        if let Some(living) = entity.get_living_entity() {
            living.health.store(health);
        }
        Ok(())
    }

    fn get_max_health(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<f32> {
        let entity = self.get(&this)?;
        Ok(entity.get_living_entity().map_or(
            0.0,
            pumpkin_core::entity::living::LivingEntity::get_max_health,
        ))
    }

    fn set_max_health(
        &mut self,
        this: Resource<WitLivingEntity>,
        max_health: f32,
    ) -> wasmtime::Result<()> {
        let entity = self.get(&this)?;
        if let Some(living) = entity.get_living_entity() {
            living.set_max_health(max_health);
        }
        Ok(())
    }

    fn is_dead(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<bool> {
        let entity = self.get(&this)?;
        Ok(entity.get_living_entity().map_or_else(
            || entity.get_entity().removal_reason.load().is_some(),
            |living| living.dead.load(std::sync::atomic::Ordering::Relaxed),
        ))
    }

    fn get_no_damage_ticks(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<i32> {
        let entity = self.get(&this)?;
        Ok(entity.get_living_entity().map_or(0, |living| {
            living
                .hurt_cooldown
                .load(std::sync::atomic::Ordering::Relaxed)
        }))
    }

    fn set_no_damage_ticks(
        &mut self,
        this: Resource<WitLivingEntity>,
        ticks: i32,
    ) -> wasmtime::Result<()> {
        let entity = self.get(&this)?;
        if let Some(living) = entity.get_living_entity() {
            living
                .hurt_cooldown
                .store(ticks, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }

    fn get_last_damage(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<f32> {
        let entity = self.get(&this)?;
        Ok(entity
            .get_living_entity()
            .map_or(0.0, |living| living.last_damage_taken.load()))
    }

    fn set_last_damage(
        &mut self,
        this: Resource<WitLivingEntity>,
        damage: f32,
    ) -> wasmtime::Result<()> {
        let entity = self.get(&this)?;
        if let Some(living) = entity.get_living_entity() {
            living.last_damage_taken.store(damage);
        }
        Ok(())
    }

    fn get_absorption(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<f32> {
        let entity = self.get(&this)?;
        Ok(entity
            .get_living_entity()
            .map_or(0.0, |living| living.absorption.load()))
    }

    fn set_absorption(
        &mut self,
        this: Resource<WitLivingEntity>,
        amount: f32,
    ) -> wasmtime::Result<()> {
        let entity = self.get(&this)?;
        if let Some(living) = entity.get_living_entity() {
            living.absorption.store(amount);
        }
        Ok(())
    }

    fn get_attribute_value(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
    ) -> wasmtime::Result<f64> {
        let entity = self.get(&this)?;
        let attribute = from_wit_attribute(attr);
        Ok(entity
            .get_living_entity()
            .map_or(attribute.default_value, |living| {
                living.get_attribute_value(attribute)
            }))
    }

    fn get_attribute_base(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
    ) -> wasmtime::Result<f64> {
        let entity = self.get(&this)?;
        let attribute = from_wit_attribute(attr);
        Ok(entity
            .get_living_entity()
            .map_or(attribute.default_value, |living| {
                living.get_attribute_base(attribute)
            }))
    }

    fn set_attribute_base(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
        value: f64,
    ) -> wasmtime::Result<()> {
        let entity = self.get(&this)?;
        let attribute = from_wit_attribute(attr);
        if let Some(living) = entity.get_living_entity() {
            living.set_attribute_base(attribute, value);
            pumpkin_core::entity::attributes::send_attribute_updates_for_living(
                living,
                vec![attribute.clone()],
            );
        }
        Ok(())
    }

    fn add_attribute_modifier(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
        modifier: WitAttributeModifier,
    ) -> wasmtime::Result<()> {
        let entity = self.get(&this)?;
        let attribute = from_wit_attribute(attr);
        if let Some(living) = entity.get_living_entity() {
            let internal_mod = pumpkin_core::entity::attributes::Modifier {
                id: modifier.id,
                amount: modifier.amount,
                operation: from_wit_modifier_op(modifier.operation),
            };
            living.update_attribute(attribute, |inst| inst.add_or_replace_modifier(internal_mod));
            pumpkin_core::entity::attributes::send_attribute_updates_for_living(
                living,
                vec![attribute.clone()],
            );
        }
        Ok(())
    }

    fn remove_attribute_modifier(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
        id: String,
    ) -> wasmtime::Result<()> {
        let entity = self.get(&this)?;
        let attribute = from_wit_attribute(attr);
        if let Some(living) = entity.get_living_entity() {
            living.update_attribute(attribute, |inst| inst.remove_modifier(&id));
            pumpkin_core::entity::attributes::send_attribute_updates_for_living(
                living,
                vec![attribute.clone()],
            );
        }
        Ok(())
    }

    fn get_attribute_modifiers(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
    ) -> wasmtime::Result<Vec<WitAttributeModifier>> {
        let entity = self.get(&this)?;
        let attribute = from_wit_attribute(attr);
        if let Some(living) = entity.get_living_entity() {
            let map = living.attributes.read_ignore_poison();
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

    fn reset_attribute(
        &mut self,
        this: Resource<WitLivingEntity>,
        attr: Attribute,
    ) -> wasmtime::Result<()> {
        let entity = self.get(&this)?;
        let attribute = from_wit_attribute(attr);
        if let Some(living) = entity.get_living_entity() {
            {
                let mut map = living.attributes.write_ignore_poison();
                map.remove(&attribute.id);
            };
            pumpkin_core::entity::attributes::send_attribute_updates_for_living(
                living,
                vec![attribute.clone()],
            );
        }
        Ok(())
    }

    fn reset_all_attributes(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<()> {
        let entity = self.get(&this)?;
        if let Some(living) = entity.get_living_entity() {
            living.reset_effects_and_attributes();
        }
        Ok(())
    }

    fn get_equipment(
        &mut self,
        this: Resource<WitLivingEntity>,
        slot: WitEquipmentSlot,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let entity = self.get(&this)?.clone();
        if let Some(living) = entity.get_living_entity() {
            let slot = from_wit_equipment_slot(slot);
            let equipment = living.entity_equipment.lock_ignore_poison();
            let stack = equipment.get(&slot);
            if !stack.is_empty() {
                return Ok(Some(self.add(Arc::new(tokio::sync::Mutex::new(stack)))?));
            }
        }
        Ok(None)
    }

    fn clear_equipment(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<()> {
        let entity = self.get(&this)?;
        if let Some(living) = entity.get_living_entity() {
            let mut equipment = living.entity_equipment.lock_ignore_poison();
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

    fn get_age(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<i32> {
        let entity = self.get(&this)?;
        Ok(entity.get_living_entity().map_or(0, |living| {
            living.entity.age.load(std::sync::atomic::Ordering::Relaxed)
        }))
    }

    fn set_age(&mut self, this: Resource<WitLivingEntity>, age: i32) -> wasmtime::Result<()> {
        let entity = self.get(&this)?;
        if let Some(living) = entity.get_living_entity() {
            living
                .entity
                .age
                .store(age, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }

    fn send_system_message(
        &mut self,
        this: Resource<WitLivingEntity>,
        message: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let text_res = self.take(message)?;
        if let Some(player) = self.get(&this)?.get_player() {
            player.send_system_message(&text_res);
        }
        Ok(())
    }

    fn remove_effect(
        &mut self,
        this: Resource<WitLivingEntity>,
        effect: WitStatusEffectType,
    ) -> wasmtime::Result<bool> {
        let entity = self.get(&this)?;
        let effect_type = super::status_effect::from_wasm_status_effect_type(effect);
        let Some(status_effect) =
            pumpkin_data::effect::StatusEffect::from_name(effect_type.to_name())
        else {
            return Ok(false);
        };
        Ok(entity
            .get_living_entity()
            .is_some_and(|living| living.remove_effect(status_effect)))
    }

    fn has_effect(
        &mut self,
        this: Resource<WitLivingEntity>,
        effect: WitStatusEffectType,
    ) -> wasmtime::Result<bool> {
        let entity = self.get(&this)?;
        let effect_type = super::status_effect::from_wasm_status_effect_type(effect);
        let Some(status_effect) =
            pumpkin_data::effect::StatusEffect::from_name(effect_type.to_name())
        else {
            return Ok(false);
        };
        Ok(entity
            .get_living_entity()
            .is_some_and(|living| living.has_effect(status_effect)))
    }

    fn get_effect(
        &mut self,
        this: Resource<WitLivingEntity>,
        effect: WitStatusEffectType,
    ) -> wasmtime::Result<Option<WitStatusEffectInstance>> {
        let entity = self.get(&this)?;
        let effect_type = super::status_effect::from_wasm_status_effect_type(effect);
        let Some(status_effect) =
            pumpkin_data::effect::StatusEffect::from_name(effect_type.to_name())
        else {
            return Ok(None);
        };
        if let Some(living) = entity.get_living_entity()
            && let Some(eff) = living.get_effect(status_effect)
        {
            return Ok(super::status_effect::to_wasm_status_effect_instance(&eff));
        }
        Ok(None)
    }

    fn get_active_effects(
        &mut self,
        this: Resource<WitLivingEntity>,
    ) -> wasmtime::Result<Vec<WitStatusEffectInstance>> {
        let entity = self.get(&this)?;
        let Some(living) = entity.get_living_entity() else {
            return Ok(Vec::new());
        };
        let effects = living.active_effects.lock_ignore_poison();
        let mut list = Vec::with_capacity(effects.len());
        for eff in effects.values() {
            if let Some(instance) = super::status_effect::to_wasm_status_effect_instance(eff) {
                list.push(instance);
            }
        }
        Ok(list)
    }

    fn is_climbing(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<bool> {
        let entity = self.get(&this)?;
        Ok(entity
            .get_living_entity()
            .is_some_and(|living| living.climbing.load(std::sync::atomic::Ordering::Relaxed)))
    }

    fn is_gliding(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<bool> {
        let entity = self.get(&this)?;
        Ok(entity
            .get_entity()
            .fall_flying
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    fn is_riptiding(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<bool> {
        let entity = self.get(&this)?;
        Ok(entity
            .get_living_entity()
            .is_some_and(pumpkin_core::entity::living::LivingEntity::is_auto_spin_attack))
    }

    fn is_sleeping(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<bool> {
        let entity = self.get(&this)?;
        Ok(entity.get_entity().pose.load() == pumpkin_data::entity::EntityPose::Sleeping)
    }

    fn is_blocking(&mut self, this: Resource<WitLivingEntity>) -> wasmtime::Result<bool> {
        let entity = self.get(&this)?;
        Ok(entity
            .get_living_entity()
            .is_some_and(pumpkin_core::entity::living::LivingEntity::is_blocking))
    }

    fn drop(&mut self, rep: Resource<WitLivingEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostLivingEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn set_equipment(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<WitLivingEntity>,
        slot: WitEquipmentSlot,
        stack: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let item_stack = if let Some(stack_res) = stack {
            accessor.take_res(stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };

        let entity = accessor.get_res(&this)?;
        if let Some(living) = entity.get_living_entity() {
            let slot = from_wit_equipment_slot(slot);
            {
                let mut equipment = living.entity_equipment.lock_ignore_poison();
                equipment.put(&slot, item_stack.clone());
            };

            living.send_equipment_changes(&[(slot, item_stack)]);
        }
        Ok(())
    }

    async fn heal(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<WitLivingEntity>,
        amount: f32,
    ) -> wasmtime::Result<()> {
        let entity = accessor.get_res(&this)?;
        run_blocking(accessor, move || {
            if let Some(player) = entity.get_player() {
                player.heal(amount);
            } else if let Some(living) = entity.get_living_entity() {
                living.heal(amount);
            }
        })
        .await
    }

    async fn damage(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<WitLivingEntity>,
        amount: f32,
        damage_type: WitDamageType,
    ) -> wasmtime::Result<()> {
        let entity = accessor.get_res(&this)?;
        let damage_type = from_wit_damage_type(damage_type);
        run_blocking(accessor, move || {
            entity.damage(&*entity, amount, damage_type);
        })
        .await
    }

    async fn add_effect(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<WitLivingEntity>,
        effect: WitStatusEffectInstance,
    ) -> wasmtime::Result<()> {
        let entity = accessor.get_res(&this)?;
        let effect_type = super::status_effect::from_wasm_status_effect_type(effect.effect_type);
        let Some(status_effect) =
            pumpkin_data::effect::StatusEffect::from_name(effect_type.to_name())
        else {
            return Ok(());
        };
        let effect = pumpkin_data::potion::Effect {
            effect_type: status_effect,
            duration: effect.duration as i32,
            amplifier: effect.amplifier,
            ambient: effect.ambient,
            show_particles: effect.show_particles,
            show_icon: effect.show_icon,
            blend: false,
        };
        if let Some(living) = entity.get_living_entity() {
            living.add_effect(effect);
        }
        Ok(())
    }

    async fn clear_effects(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<WitLivingEntity>,
    ) -> wasmtime::Result<()> {
        let entity = accessor.get_res(&this)?;
        if let Some(living) = entity.get_living_entity() {
            living.reset_effects_and_attributes();
        }
        Ok(())
    }

    async fn swing_hand(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<WitLivingEntity>,
        hand: crate::pumpkin::plugin::common::Hand,
    ) -> wasmtime::Result<()> {
        let entity = accessor.get_res(&this)?;
        run_blocking(accessor, move || {
            if let Some(player) = entity.get_player() {
                let internal_hand = match hand {
                    crate::pumpkin::plugin::common::Hand::Left => pumpkin_util::Hand::Left,
                    crate::pumpkin::plugin::common::Hand::Right => pumpkin_util::Hand::Right,
                };
                player.swing_hand(internal_hand, true);
            } else if let Some(living) = entity.get_living_entity() {
                match hand {
                    crate::pumpkin::plugin::common::Hand::Left => living.swing_off_hand(),
                    crate::pumpkin::plugin::common::Hand::Right => living.swing_hand(),
                }
            }
        })
        .await
    }
}
