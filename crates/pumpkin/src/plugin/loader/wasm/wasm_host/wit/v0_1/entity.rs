use std::sync::Arc;
use wasmtime::component::Resource;

use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::goal::{Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::plugin::loader::wasm::wasm_host::{PluginInstance, WasmPlugin};
use crate::plugin::loader::wasm::wasm_host::{
    state::{EntityResource, PluginHostState},
    wit::v0_1::events::to_wasm_position,
    wit::v0_1::pumpkin::plugin::{
        attributes::{
            Attribute, AttributeModifier as WitAttributeModifier,
            ModifierOperation as WitModifierOperation,
        },
        common::{EntityPose, NbtTree as WitNbtTree, Position},
        damage_types::DamageType as WitDamageType,
        entity::Host,
        entity_types,
        item_stack::ItemStack as WitHostItemStack,
        text::TextComponent,
        uuid::Uuid,
        world::{
            BlockPos as WitBlockPos, BoundingBox as WitBoundingBox, Entity,
            EquipmentSlot as WitEquipmentSlot, HostEntity,
            RayTraceBlockResult as WitRayTraceBlockResult,
            RayTraceEntityResult as WitRayTraceEntityResult, RaycastResult as WitRaycastResult,
            World,
        },
    },
    wit::v0_1::uuid::UuidExt,
    wit::v0_1::world::to_wasm_block_direction,
};
use pumpkin_data::entity::EntityPose as InternalEntityPose;

impl Host for PluginHostState {}
impl entity_types::Host for PluginHostState {}

fn entity_from_resource(
    state: &PluginHostState,
    entity: &Resource<Entity>,
) -> wasmtime::Result<std::sync::Arc<dyn crate::entity::EntityBase>> {
    state
        .resource_table
        .get::<EntityResource>(&Resource::new_own(entity.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid entity resource handle"))
        .map(|resource| resource.provider.clone())
}

const fn map_entity_pose(pose: InternalEntityPose) -> EntityPose {
    match pose {
        InternalEntityPose::Standing => EntityPose::Standing,
        InternalEntityPose::FallFlying => EntityPose::FallFlying,
        InternalEntityPose::Sleeping => EntityPose::Sleeping,
        InternalEntityPose::Swimming => EntityPose::Swimming,
        InternalEntityPose::SpinAttack => EntityPose::SpinAttack,
        InternalEntityPose::Crouching => EntityPose::Crouching,
        InternalEntityPose::LongJumping => EntityPose::LongJumping,
        InternalEntityPose::Dying => EntityPose::Dying,
        InternalEntityPose::Croaking => EntityPose::Croaking,
        InternalEntityPose::UsingTongue => EntityPose::UsingTongue,
        InternalEntityPose::Sitting => EntityPose::Sitting,
        InternalEntityPose::Roaring => EntityPose::Roaring,
        InternalEntityPose::Sniffing => EntityPose::Sniffing,
        InternalEntityPose::Emerging => EntityPose::Emerging,
        InternalEntityPose::Digging => EntityPose::Digging,
        InternalEntityPose::Sliding => EntityPose::Sliding,
        InternalEntityPose::Shooting => EntityPose::Shooting,
        InternalEntityPose::Inhaling => EntityPose::Inhaling,
    }
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

impl HostEntity for PluginHostState {
    async fn get_id(&mut self, entity: Resource<Entity>) -> wasmtime::Result<u32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity.get_entity().entity_id as u32)
    }

    async fn get_uuid(&mut self, entity: Resource<Entity>) -> wasmtime::Result<Uuid> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(Uuid::to_wit(&entity.get_entity().entity_uuid))
    }

    async fn get_type(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<entity_types::EntityType> {
        let entity = entity_from_resource(self, &entity)?;
        let original_name = entity.get_entity().entity_type.resource_name;

        let mut names: Vec<String> = serde_json::from_str::<
            std::collections::BTreeMap<String, serde_json::Value>,
        >(&std::fs::read_to_string("assets/entities.json")?)?
        .keys()
        .cloned()
        .collect();
        names.sort();

        let index = names
            .iter()
            .position(|n| n == original_name)
            .ok_or_else(|| wasmtime::Error::msg(format!("Unknown entity type: {original_name}")))?;

        // SAFETY: The WIT enum is generated from the sorted keys of assets/entities.json.
        Ok(unsafe { std::mem::transmute::<u8, entity_types::EntityType>(index as u8) })
    }

    async fn get_position(&mut self, entity: Resource<Entity>) -> wasmtime::Result<Position> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(to_wasm_position(entity.get_entity().pos.load()))
    }

    async fn get_world(&mut self, entity: Resource<Entity>) -> wasmtime::Result<Resource<World>> {
        let entity = entity_from_resource(self, &entity)?;
        let world = entity.get_entity().world.load_full();
        self.add_world(world)
            .map_err(|_| wasmtime::Error::msg("failed to add world resource"))
    }

    async fn get_yaw(&mut self, entity: Resource<Entity>) -> wasmtime::Result<f32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity.get_entity().yaw.load())
    }

    async fn get_pitch(&mut self, entity: Resource<Entity>) -> wasmtime::Result<f32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity.get_entity().pitch.load())
    }

    async fn get_head_yaw(&mut self, entity: Resource<Entity>) -> wasmtime::Result<f32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity.get_entity().head_yaw.load())
    }

    async fn is_on_ground(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .on_ground
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn is_sneaking(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .sneaking
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn is_sprinting(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .sprinting
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn is_invisible(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .invisible
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn is_glowing(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .glowing
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn teleport(
        &mut self,
        entity: Resource<Entity>,
        pos: Position,
        world_ref: Resource<World>,
    ) -> wasmtime::Result<()> {
        let entity_base = entity_from_resource(self, &entity)?;
        let world = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::WorldResource>(
                &Resource::new_own(world_ref.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid world resource handle"))?;
        let world = world.provider.clone();
        entity_base
            .teleport(
                pumpkin_util::math::vector3::Vector3::new(pos.0, pos.1, pos.2),
                None,
                None,
                world,
            )
            .await;
        Ok(())
    }

    async fn set_velocity(
        &mut self,
        entity: Resource<Entity>,
        velocity: Position,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity
            .get_entity()
            .velocity
            .store(pumpkin_util::math::vector3::Vector3::new(
                velocity.0, velocity.1, velocity.2,
            ));
        Ok(())
    }

    async fn get_velocity(&mut self, entity: Resource<Entity>) -> wasmtime::Result<Position> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(to_wasm_position(entity.get_entity().velocity.load()))
    }

    async fn set_sneaking(
        &mut self,
        entity: Resource<Entity>,
        sneaking: bool,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().set_sneaking(sneaking).await;
        Ok(())
    }

    async fn set_sprinting(
        &mut self,
        entity: Resource<Entity>,
        sprinting: bool,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().set_sprinting(sprinting).await;
        Ok(())
    }

    async fn is_swimming(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .swimming
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn set_swimming(
        &mut self,
        entity: Resource<Entity>,
        swimming: bool,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().set_swimming(swimming).await;
        Ok(())
    }

    async fn set_invisible(
        &mut self,
        entity: Resource<Entity>,
        invisible: bool,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().set_invisible(invisible).await;
        Ok(())
    }

    async fn set_glowing(
        &mut self,
        entity: Resource<Entity>,
        glowing: bool,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().set_glowing(glowing).await;
        Ok(())
    }

    async fn is_fall_flying(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .fall_flying
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn set_fall_flying(
        &mut self,
        entity: Resource<Entity>,
        fall_flying: bool,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().set_fall_flying(fall_flying).await;
        Ok(())
    }

    async fn is_on_fire(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .fire_ticks
            .load(std::sync::atomic::Ordering::Relaxed)
            > 0)
    }

    async fn set_on_fire(
        &mut self,
        entity: Resource<Entity>,
        on_fire: bool,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().set_on_fire(on_fire).await;
        Ok(())
    }

    async fn get_pose(&mut self, entity: Resource<Entity>) -> wasmtime::Result<EntityPose> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(map_entity_pose(entity.get_entity().pose.load()))
    }

    async fn get_name(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let entity = entity_from_resource(self, &entity)?;
        let name = entity.get_name();
        self.add_text_component(name)
            .map_err(|_| wasmtime::Error::msg("failed to add text component resource"))
    }

    async fn set_custom_name(
        &mut self,
        entity: Resource<Entity>,
        name: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let entity_base = entity_from_resource(self, &entity)?;
        let text_res = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::TextComponentResource>(
                &Resource::new_own(name.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid text component resource handle"))?;
        let text = text_res.provider.clone();
        entity_base.get_entity().set_custom_name(text);
        Ok(())
    }

    async fn get_custom_name(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<TextComponent>>> {
        let entity = entity_from_resource(self, &entity)?;
        let name = entity.get_entity().custom_name.load();
        if let Some(name) = name.as_ref() {
            Ok(Some(self.add_text_component(name.clone()).map_err(
                |_| wasmtime::Error::msg("failed to add text component resource"),
            )?))
        } else {
            Ok(None)
        }
    }

    async fn set_custom_name_visible(
        &mut self,
        entity: Resource<Entity>,
        visible: bool,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().set_custom_name_visible(visible);
        Ok(())
    }

    async fn is_custom_name_visible(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .custom_name_visible
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn is_invulnerable(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .invulnerable
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn set_invulnerable(
        &mut self,
        entity: Resource<Entity>,
        invulnerable: bool,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity
            .get_entity()
            .invulnerable
            .store(invulnerable, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn get_fire_ticks(&mut self, entity: Resource<Entity>) -> wasmtime::Result<i32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .fire_ticks
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn set_fire_ticks(
        &mut self,
        entity: Resource<Entity>,
        ticks: i32,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity
            .get_entity()
            .fire_ticks
            .store(ticks, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn get_health(&mut self, entity: Resource<Entity>) -> wasmtime::Result<f32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_living_entity()
            .map_or(0.0, |living| living.health.load()))
    }

    async fn set_health(&mut self, entity: Resource<Entity>, health: f32) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(living) = entity.get_living_entity() {
            living.health.store(health);
        }
        Ok(())
    }

    async fn get_max_health(&mut self, entity: Resource<Entity>) -> wasmtime::Result<f32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_living_entity()
            .map_or(0.0, crate::entity::living::LivingEntity::get_max_health))
    }

    async fn damage(
        &mut self,
        entity: Resource<Entity>,
        amount: f32,
        damage_type: WitDamageType,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity
            .damage(&*entity, amount, from_wit_damage_type(damage_type))
            .await;
        Ok(())
    }

    async fn is_dead(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity.get_living_entity().map_or_else(
            || entity.get_entity().removal_reason.load().is_some(),
            |living| living.dead.load(std::sync::atomic::Ordering::Relaxed),
        ))
    }

    async fn get_absorption(&mut self, entity: Resource<Entity>) -> wasmtime::Result<f32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_living_entity()
            .map_or(0.0, |living| living.absorption.load()))
    }

    async fn set_absorption(
        &mut self,
        entity: Resource<Entity>,
        amount: f32,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(living) = entity.get_living_entity() {
            living.absorption.store(amount);
        }
        Ok(())
    }

    async fn get_attribute_value(
        &mut self,
        entity: Resource<Entity>,
        attr: Attribute,
    ) -> wasmtime::Result<f64> {
        let entity = entity_from_resource(self, &entity)?;
        let attribute = from_wit_attribute(attr);
        Ok(entity
            .get_living_entity()
            .map_or(attribute.default_value, |living| {
                living.get_attribute_value(attribute)
            }))
    }

    async fn get_attribute_base(
        &mut self,
        entity: Resource<Entity>,
        attr: Attribute,
    ) -> wasmtime::Result<f64> {
        let entity = entity_from_resource(self, &entity)?;
        let attribute = from_wit_attribute(attr);
        Ok(entity
            .get_living_entity()
            .map_or(attribute.default_value, |living| {
                living.get_attribute_base(attribute)
            }))
    }

    async fn set_attribute_base(
        &mut self,
        entity: Resource<Entity>,
        attr: Attribute,
        value: f64,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        let attribute = from_wit_attribute(attr);
        if let Some(living) = entity.get_living_entity() {
            living.set_attribute_base(attribute, value);
            crate::entity::attributes::send_attribute_updates_for_living(
                living,
                vec![attribute.clone()],
            )
            .await;
        }
        Ok(())
    }

    async fn add_attribute_modifier(
        &mut self,
        entity: Resource<Entity>,
        attr: Attribute,
        modifier: WitAttributeModifier,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
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
            )
            .await;
        }
        Ok(())
    }

    async fn remove_attribute_modifier(
        &mut self,
        entity: Resource<Entity>,
        attr: Attribute,
        id: String,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        let attribute = from_wit_attribute(attr);
        if let Some(living) = entity.get_living_entity() {
            living.update_attribute(attribute, |inst| inst.remove_modifier(&id));
            crate::entity::attributes::send_attribute_updates_for_living(
                living,
                vec![attribute.clone()],
            )
            .await;
        }
        Ok(())
    }

    async fn get_attribute_modifiers(
        &mut self,
        entity: Resource<Entity>,
        attr: Attribute,
    ) -> wasmtime::Result<Vec<WitAttributeModifier>> {
        let entity = entity_from_resource(self, &entity)?;
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
        entity: Resource<Entity>,
        attr: Attribute,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
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
            )
            .await;
        }
        Ok(())
    }

    async fn reset_all_attributes(&mut self, entity: Resource<Entity>) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(living) = entity.get_living_entity() {
            living.reset_effects_and_attributes().await;
        }
        Ok(())
    }

    async fn get_equipment(
        &mut self,
        entity: Resource<Entity>,
        slot: WitEquipmentSlot,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(living) = entity.get_living_entity() {
            let slot = from_wit_equipment_slot(slot);
            let equipment = living.entity_equipment.lock().await;
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
        entity: Resource<Entity>,
        slot: WitEquipmentSlot,
        stack: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(living) = entity.get_living_entity() {
            let slot = from_wit_equipment_slot(slot);
            let item_stack = if let Some(stack_res) = stack {
                self.get_item_stack(&stack_res)?.lock().await.clone()
            } else {
                pumpkin_data::item_stack::ItemStack::EMPTY.clone()
            };

            {
                let mut equipment = living.entity_equipment.lock().await;
                equipment.put(&slot, item_stack.clone());
            };

            living.send_equipment_changes(&[(slot, item_stack)]);
        }
        Ok(())
    }

    async fn clear_equipment(&mut self, entity: Resource<Entity>) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(living) = entity.get_living_entity() {
            let mut equipment = living.entity_equipment.lock().await;
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

    async fn get_age(&mut self, entity: Resource<Entity>) -> wasmtime::Result<i32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .age
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn set_age(&mut self, entity: Resource<Entity>, age: i32) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity
            .get_entity()
            .age
            .store(age, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn get_fall_distance(&mut self, entity: Resource<Entity>) -> wasmtime::Result<f32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_living_entity()
            .map_or(0.0, |living| living.fall_distance.load()))
    }

    async fn set_fall_distance(
        &mut self,
        entity: Resource<Entity>,
        distance: f32,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(living) = entity.get_living_entity() {
            living.fall_distance.store(distance);
        }
        Ok(())
    }

    async fn is_silent(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity.get_entity().is_silent())
    }

    async fn set_silent(&mut self, entity: Resource<Entity>, silent: bool) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().set_silent(silent);
        Ok(())
    }

    async fn has_gravity(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(!entity.get_entity().has_no_gravity())
    }

    async fn set_has_gravity(
        &mut self,
        entity: Resource<Entity>,
        gravity: bool,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().set_has_no_gravity(!gravity);
        Ok(())
    }

    async fn get_eye_height(&mut self, entity: Resource<Entity>) -> wasmtime::Result<f32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity.get_entity().entity_dimension.load().eye_height)
    }

    async fn get_eye_position(&mut self, entity: Resource<Entity>) -> wasmtime::Result<Position> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(to_wasm_position(entity.get_eye_pos()))
    }

    async fn get_nearby_entities(
        &mut self,
        entity: Resource<Entity>,
        x: f64,
        y: f64,
        z: f64,
    ) -> wasmtime::Result<Vec<Resource<Entity>>> {
        let entity = entity_from_resource(self, &entity)?;
        let pos = entity.get_entity().pos.load();
        let box_range = pumpkin_util::math::boundingbox::BoundingBox::new(
            Vector3::new(pos.x - x, pos.y - y, pos.z - z),
            Vector3::new(pos.x + x, pos.y + y, pos.z + z),
        );
        let world = entity.get_entity().world.load_full();
        let entities = world.get_entities_at_box(&box_range);

        let mut result = Vec::new();
        for e in entities {
            // Don't include the entity itself
            if e.get_entity().entity_id != entity.get_entity().entity_id {
                result.push(
                    self.add_entity(e)
                        .map_err(|_| wasmtime::Error::msg("failed to add entity resource"))?,
                );
            }
        }
        Ok(result)
    }

    async fn get_vehicle(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<Entity>>> {
        let entity = entity_from_resource(self, &entity)?;
        let vehicle = entity.get_entity().vehicle.lock().await;
        if let Some(v) = vehicle.as_ref() {
            Ok(Some(self.add_entity(Arc::clone(v)).map_err(|_| {
                wasmtime::Error::msg("failed to add entity resource")
            })?))
        } else {
            Ok(None)
        }
    }

    async fn set_vehicle(
        &mut self,
        entity: Resource<Entity>,
        vehicle: Option<Resource<Entity>>,
    ) -> wasmtime::Result<()> {
        let entity_base = entity_from_resource(self, &entity)?;

        // Remove from current vehicle if any
        let current_vehicle = entity_base.get_entity().vehicle.lock().await.clone();
        if let Some(v) = current_vehicle {
            v.get_entity()
                .remove_passenger(entity_base.get_entity().entity_id)
                .await;
        }

        if let Some(vehicle_res) = vehicle {
            let vehicle_base = entity_from_resource(self, &vehicle_res)?;
            vehicle_base
                .get_entity()
                .add_passenger(vehicle_base.clone(), entity_base)
                .await;
        }

        Ok(())
    }

    async fn get_passengers(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Vec<Resource<Entity>>> {
        let entity = entity_from_resource(self, &entity)?;
        let passengers = entity.get_entity().passengers.lock().await;
        let mut result = Vec::new();
        for p in passengers.iter() {
            result.push(
                self.add_entity(Arc::clone(p))
                    .map_err(|_| wasmtime::Error::msg("failed to add entity resource"))?,
            );
        }
        Ok(result)
    }

    async fn add_passenger(
        &mut self,
        entity: Resource<Entity>,
        passenger: Resource<Entity>,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        let passenger = entity_from_resource(self, &passenger)?;
        entity
            .get_entity()
            .add_passenger(Arc::clone(&entity), passenger)
            .await;
        Ok(())
    }

    async fn remove_passenger(
        &mut self,
        entity: Resource<Entity>,
        passenger: Resource<Entity>,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        let passenger = entity_from_resource(self, &passenger)?;
        entity
            .get_entity()
            .remove_passenger(passenger.get_entity().entity_id)
            .await;
        Ok(())
    }

    async fn eject_passengers(&mut self, entity: Resource<Entity>) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        let ids: Vec<i32> = entity
            .get_entity()
            .passengers
            .lock()
            .await
            .iter()
            .map(|p| p.get_entity().entity_id)
            .collect();
        for id in ids {
            entity.get_entity().remove_passenger(id).await;
        }
        Ok(())
    }

    async fn get_bounding_box(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<WitBoundingBox> {
        let entity = entity_from_resource(self, &entity)?;
        let bb = entity.get_entity().bounding_box.load();
        Ok(WitBoundingBox {
            min: to_wasm_position(bb.min),
            max: to_wasm_position(bb.max),
        })
    }

    async fn is_in_water(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .touching_water
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn is_in_lava(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .touching_lava
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn get_ticks_lived(&mut self, entity: Resource<Entity>) -> wasmtime::Result<i32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .age
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn set_ticks_lived(
        &mut self,
        entity: Resource<Entity>,
        ticks: i32,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity
            .get_entity()
            .age
            .store(ticks, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn get_width(&mut self, entity: Resource<Entity>) -> wasmtime::Result<f32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity.get_entity().entity_dimension.load().width)
    }

    async fn get_height(&mut self, entity: Resource<Entity>) -> wasmtime::Result<f32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity.get_entity().entity_dimension.load().height)
    }

    async fn set_rotation(
        &mut self,
        entity: Resource<Entity>,
        yaw: f32,
        pitch: f32,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().set_rotation(yaw, pitch);
        Ok(())
    }

    async fn has_visual_fire(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .has_visual_fire
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn set_visual_fire(
        &mut self,
        entity: Resource<Entity>,
        visual_fire: bool,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().set_on_fire(visual_fire).await;
        Ok(())
    }

    async fn get_portal_cooldown(&mut self, entity: Resource<Entity>) -> wasmtime::Result<u32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_entity()
            .portal_cooldown
            .load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn set_portal_cooldown(
        &mut self,
        entity: Resource<Entity>,
        cooldown: u32,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity
            .get_entity()
            .portal_cooldown
            .store(cooldown, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn get_remaining_air(&mut self, entity: Resource<Entity>) -> wasmtime::Result<i32> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity.get_player().map_or(0, |player| {
            player
                .breath_manager
                .air_supply
                .load(std::sync::atomic::Ordering::Relaxed)
        }))
    }

    async fn set_remaining_air(
        &mut self,
        entity: Resource<Entity>,
        air: i32,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(player) = entity.get_player() {
            player
                .breath_manager
                .air_supply
                .store(air, std::sync::atomic::Ordering::Relaxed);
            player.breath_manager.send_air_supply(player);
        }
        Ok(())
    }

    async fn get_max_air(&mut self, _entity: Resource<Entity>) -> wasmtime::Result<i32> {
        Ok(crate::entity::breath::MAX_AIR)
    }

    async fn send_system_message(
        &mut self,
        entity: Resource<Entity>,
        message: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(player) = entity.get_player() {
            let text_res = self
                .resource_table
                .get::<crate::plugin::loader::wasm::wasm_host::state::TextComponentResource>(
                    &Resource::new_own(message.rep()),
                )
                .map_err(|_| wasmtime::Error::msg("invalid text component resource handle"))?;
            player.send_system_message(&text_res.provider).await;
        }
        Ok(())
    }

    async fn remove(&mut self, entity: Resource<Entity>) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity.get_entity().remove().await;
        Ok(())
    }

    async fn add_ai_goal(
        &mut self,
        entity: Resource<Entity>,
        priority: u8,
        goal: crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(mob) = entity.get_mob() {
            let mob_entity = mob.get_mob_entity();
            match goal {
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal::Swim => {
                    mob_entity.add_goal(priority, crate::entity::ai::goal::swim::SwimGoal::default());
                }
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal::WanderAround(speed) => {
                    mob_entity.add_goal(priority, crate::entity::ai::goal::wander_around::WanderAroundGoal::new(f64::from(speed)));
                }
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal::MeleeAttack(speed) => {
                    mob_entity.add_goal(priority, crate::entity::ai::goal::melee_attack::MeleeAttackGoal::new(f64::from(speed), false));
                }
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal::LookAtPlayer(range) => {
                    mob_entity.add_goal(priority, crate::entity::ai::goal::look_at_entity::LookAtEntityGoal::new(
                        std::sync::Weak::<crate::entity::mob::zombie::zombie::ZombieEntity>::new() as std::sync::Weak<dyn crate::entity::mob::Mob>,
                        &pumpkin_data::entity::EntityType::PLAYER,
                        range,
                        0.02,
                        false,
                    ));
                }
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal::LookAround => {
                    mob_entity.add_goal(priority, crate::entity::ai::goal::look_around::RandomLookAroundGoal::default());
                }
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal::EscapeDanger(speed) => {
                    mob_entity.add_goal(priority, *crate::entity::ai::goal::escape_danger::EscapeDangerGoal::new(f64::from(speed)));
                }
                _ => {} // Remaining goals
            }
        }
        Ok(())
    }

    async fn add_custom_ai_goal(
        &mut self,
        entity: Resource<Entity>,
        priority: u8,
        goal_id: u32,
    ) -> wasmtime::Result<()> {
        let Some(plugin) = self.plugin.as_ref().and_then(std::sync::Weak::upgrade) else {
            return Err(wasmtime::Error::msg("Plugin not active"));
        };
        let entity = entity_from_resource(self, &entity)?;
        if let Some(mob) = entity.get_mob() {
            let mob_entity = mob.get_mob_entity();
            mob_entity.add_goal(priority, CustomWasmGoal { plugin, goal_id });
        }
        Ok(())
    }

    async fn clear_ai_goals(&mut self, entity: Resource<Entity>) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(mob) = entity.get_mob() {
            mob.get_mob_entity().clear_ai_goals(mob).await;
        }
        Ok(())
    }

    async fn set_ai_disabled(
        &mut self,
        entity: Resource<Entity>,
        disabled: bool,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(mob) = entity.get_mob() {
            mob.get_mob_entity().set_no_ai(disabled);
        }
        Ok(())
    }

    async fn is_ai_disabled(&mut self, entity: Resource<Entity>) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity
            .get_mob()
            .is_none_or(|mob| mob.get_mob_entity().is_no_ai()))
    }

    async fn set_target(
        &mut self,
        entity: Resource<Entity>,
        target: Option<Resource<Entity>>,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        let target_entity = if let Some(t) = target {
            Some(entity_from_resource(self, &t)?)
        } else {
            None
        };
        if let Some(mob) = entity.get_mob() {
            mob.get_mob_entity().set_target(target_entity).await;
        }
        Ok(())
    }

    async fn get_target(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<Entity>>> {
        let entity = entity_from_resource(self, &entity)?;
        if let Some(mob) = entity.get_mob()
            && let Some(target) = mob.get_mob_entity().get_target().await
        {
            return Ok(Some(self.add_entity(target)?));
        }
        Ok(None)
    }

    async fn raycast(
        &mut self,
        entity: Resource<Entity>,
        max_distance: f64,
        fluid_handling: bool,
    ) -> wasmtime::Result<Option<WitRaycastResult>> {
        let entity = entity_from_resource(self, &entity)?;
        let start = entity.get_eye_pos();
        let direction = entity.get_looking_vector();
        let end = start + direction * max_distance;
        let world = entity.get_entity().world.load_full();

        let hit = world.ray_trace_block(start, end, fluid_handling);

        Ok(hit.map(|(pos, face, _)| WitRaycastResult {
            pos: WitBlockPos {
                x: pos.0.x,
                y: pos.0.y,
                z: pos.0.z,
            },
            face: to_wasm_block_direction(face),
        }))
    }

    async fn ray_trace_block(
        &mut self,
        entity: Resource<Entity>,
        max_distance: f64,
        include_fluids: bool,
    ) -> wasmtime::Result<Option<WitRayTraceBlockResult>> {
        let entity = entity_from_resource(self, &entity)?;
        let start = entity.get_eye_pos();
        let direction = entity.get_looking_vector();
        let end = start + direction * max_distance;
        let world = entity.get_entity().world.load_full();

        let hit = world.ray_trace_block(start, end, include_fluids);

        Ok(hit.map(|(pos, face, hit_pos)| WitRayTraceBlockResult {
            pos: WitBlockPos {
                x: pos.0.x,
                y: pos.0.y,
                z: pos.0.z,
            },
            face: to_wasm_block_direction(face),
            hit_pos: to_wasm_position(hit_pos),
        }))
    }

    async fn ray_trace_entity(
        &mut self,
        entity: Resource<Entity>,
        max_distance: f64,
    ) -> wasmtime::Result<Option<WitRayTraceEntityResult>> {
        let entity_base = entity_from_resource(self, &entity)?;
        let start = entity_base.get_eye_pos();
        let direction = entity_base.get_looking_vector();
        let end = start + direction * max_distance;
        let world = entity_base.get_entity().world.load_full();
        let self_id = entity_base.get_entity().entity_id;

        let hits = world.ray_trace_entities(start, end);
        for (hit_entity, hit_pos, distance) in hits {
            if hit_entity.get_entity().entity_id != self_id {
                let entity_res = self
                    .add_entity(hit_entity)
                    .map_err(|_| wasmtime::Error::msg("failed to add entity resource"))?;
                return Ok(Some(WitRayTraceEntityResult {
                    entity: entity_res,
                    hit_pos: to_wasm_position(hit_pos),
                    distance,
                }));
            }
        }

        Ok(None)
    }

    async fn get_target_entity(
        &mut self,
        entity: Resource<Entity>,
        max_distance: f64,
    ) -> wasmtime::Result<Option<Resource<Entity>>> {
        let res = self.ray_trace_entity(entity, max_distance).await?;
        Ok(res.map(|r| r.entity))
    }

    async fn set_custom_data(
        &mut self,
        this: Resource<Entity>,
        namespace: String,
        key: String,
        value: WitNbtTree,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &this)?;
        let base_entity = entity.get_entity();
        let tag = super::common::from_wit_nbt_tree(&value).map_err(wasmtime::Error::msg)?;
        base_entity.set_custom_data(&namespace, &key, tag).await;
        Ok(())
    }

    async fn get_custom_data(
        &mut self,
        this: Resource<Entity>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<Option<WitNbtTree>> {
        let entity = entity_from_resource(self, &this)?;
        let base_entity = entity.get_entity();
        let tag = base_entity.get_custom_data(&namespace, &key).await;
        Ok(tag.map(super::common::to_wit_nbt_tree))
    }

    async fn remove_custom_data(
        &mut self,
        this: Resource<Entity>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &this)?;
        let base_entity = entity.get_entity();
        base_entity.remove_custom_data(&namespace, &key).await;
        Ok(())
    }

    async fn has_custom_data(
        &mut self,
        this: Resource<Entity>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<bool> {
        let entity = entity_from_resource(self, &this)?;
        let base_entity = entity.get_entity();
        Ok(base_entity.has_custom_data(&namespace, &key).await)
    }

    async fn drop(&mut self, rep: Resource<Entity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<EntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

pub struct CustomWasmGoal {
    pub plugin: Arc<WasmPlugin>,
    pub goal_id: u32,
}

fn current_mob_entity(mob: &dyn Mob) -> Option<Arc<dyn crate::entity::EntityBase>> {
    let entity = mob.get_entity();
    entity.world.load().get_entity_by_id(entity.entity_id)
}

impl Goal for CustomWasmGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let mut store = self.plugin.store.lock().await;
            if let Some(entity_arc) = current_mob_entity(mob) {
                match self.plugin.plugin_instance {
                    PluginInstance::V0_1(ref plugin) => {
                        let Some(server) = store.data_mut().server.clone() else {
                            return false;
                        };
                        let Ok(server_res) = store.data_mut().add_server(server) else {
                            return false;
                        };
                        let Ok(entity_res) = store.data_mut().add_entity(entity_arc) else {
                            let _ = store
                                .data_mut()
                                .resource_table
                                .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                    wasmtime::component::Resource::new_own(server_res.rep()),
                                );
                            return false;
                        };
                        let server_rep = server_res.rep();
                        let entity_rep = entity_res.rep();
                        let result = plugin
                            .call_handle_ai_goal_can_start(
                                &mut *store,
                                self.goal_id,
                                server_res,
                                entity_res,
                            )
                            .await
                            .unwrap_or(false);
                        let _ = store
                            .data_mut()
                            .resource_table
                            .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                wasmtime::component::Resource::new_own(server_rep),
                            );
                        let _ = store
                            .data_mut()
                            .resource_table
                            .delete::<crate::plugin::loader::wasm::wasm_host::state::EntityResource>(
                                wasmtime::component::Resource::new_own(entity_rep),
                            );
                        result
                    }
                }
            } else {
                false
            }
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let mut store = self.plugin.store.lock().await;
            if let Some(entity_arc) = current_mob_entity(mob) {
                match self.plugin.plugin_instance {
                    PluginInstance::V0_1(ref plugin) => {
                        let Some(server) = store.data_mut().server.clone() else {
                            return false;
                        };
                        let Ok(server_res) = store.data_mut().add_server(server) else {
                            return false;
                        };
                        let Ok(entity_res) = store.data_mut().add_entity(entity_arc) else {
                            let _ = store
                                .data_mut()
                                .resource_table
                                .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                    wasmtime::component::Resource::new_own(server_res.rep()),
                                );
                            return false;
                        };
                        let server_rep = server_res.rep();
                        let entity_rep = entity_res.rep();
                        let result = plugin
                            .call_handle_ai_goal_should_continue(
                                &mut *store,
                                self.goal_id,
                                server_res,
                                entity_res,
                            )
                            .await
                            .unwrap_or(false);
                        let _ = store
                            .data_mut()
                            .resource_table
                            .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                wasmtime::component::Resource::new_own(server_rep),
                            );
                        let _ = store
                            .data_mut()
                            .resource_table
                            .delete::<crate::plugin::loader::wasm::wasm_host::state::EntityResource>(
                                wasmtime::component::Resource::new_own(entity_rep),
                            );
                        result
                    }
                }
            } else {
                false
            }
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mut store = self.plugin.store.lock().await;
            if let Some(entity_arc) = current_mob_entity(mob) {
                match self.plugin.plugin_instance {
                    PluginInstance::V0_1(ref plugin) => {
                        let Some(server) = store.data_mut().server.clone() else {
                            return;
                        };
                        let Ok(server_res) = store.data_mut().add_server(server) else {
                            return;
                        };
                        let Ok(entity_res) = store.data_mut().add_entity(entity_arc) else {
                            let _ = store
                                .data_mut()
                                .resource_table
                                .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                    wasmtime::component::Resource::new_own(server_res.rep()),
                                );
                            return;
                        };
                        let server_rep = server_res.rep();
                        let entity_rep = entity_res.rep();
                        let _ = plugin
                            .call_handle_ai_goal_start(
                                &mut *store,
                                self.goal_id,
                                server_res,
                                entity_res,
                            )
                            .await;
                        let _ = store
                            .data_mut()
                            .resource_table
                            .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                wasmtime::component::Resource::new_own(server_rep),
                            );
                        let _ = store
                            .data_mut()
                            .resource_table
                            .delete::<crate::plugin::loader::wasm::wasm_host::state::EntityResource>(
                                wasmtime::component::Resource::new_own(entity_rep),
                            );
                    }
                }
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mut store = self.plugin.store.lock().await;
            if let Some(entity_arc) = current_mob_entity(mob) {
                match self.plugin.plugin_instance {
                    PluginInstance::V0_1(ref plugin) => {
                        let Some(server) = store.data_mut().server.clone() else {
                            return;
                        };
                        let Ok(server_res) = store.data_mut().add_server(server) else {
                            return;
                        };
                        let Ok(entity_res) = store.data_mut().add_entity(entity_arc) else {
                            let _ = store
                                .data_mut()
                                .resource_table
                                .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                    wasmtime::component::Resource::new_own(server_res.rep()),
                                );
                            return;
                        };
                        let server_rep = server_res.rep();
                        let entity_rep = entity_res.rep();
                        let _ = plugin
                            .call_handle_ai_goal_tick(
                                &mut *store,
                                self.goal_id,
                                server_res,
                                entity_res,
                            )
                            .await;
                        let _ = store
                            .data_mut()
                            .resource_table
                            .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                wasmtime::component::Resource::new_own(server_rep),
                            );
                        let _ = store
                            .data_mut()
                            .resource_table
                            .delete::<crate::plugin::loader::wasm::wasm_host::state::EntityResource>(
                                wasmtime::component::Resource::new_own(entity_rep),
                            );
                    }
                }
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mut store = self.plugin.store.lock().await;
            if let Some(entity_arc) = current_mob_entity(mob) {
                match self.plugin.plugin_instance {
                    PluginInstance::V0_1(ref plugin) => {
                        let Some(server) = store.data_mut().server.clone() else {
                            return;
                        };
                        let Ok(server_res) = store.data_mut().add_server(server) else {
                            return;
                        };
                        let Ok(entity_res) = store.data_mut().add_entity(entity_arc) else {
                            let _ = store
                                .data_mut()
                                .resource_table
                                .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                    wasmtime::component::Resource::new_own(server_res.rep()),
                                );
                            return;
                        };
                        let server_rep = server_res.rep();
                        let entity_rep = entity_res.rep();
                        let _ = plugin
                            .call_handle_ai_goal_stop(
                                &mut *store,
                                self.goal_id,
                                server_res,
                                entity_res,
                            )
                            .await;
                        let _ = store
                            .data_mut()
                            .resource_table
                            .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                wasmtime::component::Resource::new_own(server_rep),
                            );
                        let _ = store
                            .data_mut()
                            .resource_table
                            .delete::<crate::plugin::loader::wasm::wasm_host::state::EntityResource>(
                                wasmtime::component::Resource::new_own(entity_rep),
                            );
                    }
                }
            }
        })
    }
}
