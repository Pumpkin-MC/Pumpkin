use std::sync::Arc;
use wasmtime::component::Resource;

use pumpkin_util::math::vector3::Vector3;

use crate::plugin::loader::wasm::wasm_host::{
    state::{EntityResource, PluginHostState},
    wit::v0_1::events::to_wasm_position,
    wit::v0_1::pumpkin::plugin::{
        common::{EntityPose, Position},
        entity::Host,
        entity_types,
        text::TextComponent,
        uuid::Uuid,
        world::{
            BlockPos as WitBlockPos, BoundingBox as WitBoundingBox, Entity, HostEntity,
            RaycastResult as WitRaycastResult, World,
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

        // Safety: The WIT enum is generated from the sorted keys of assets/entities.json.
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
        entity_base
            .get_entity()
            .custom_name
            .store(std::sync::Arc::new(Some(text)));
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
        entity
            .get_entity()
            .custom_name_visible
            .store(visible, std::sync::atomic::Ordering::Relaxed);
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

    async fn damage(&mut self, entity: Resource<Entity>, amount: f32) -> wasmtime::Result<()> {
        let entity = entity_from_resource(self, &entity)?;
        entity
            .damage(&*entity, amount, pumpkin_data::damage::DamageType::GENERIC)
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

    async fn raycast(
        &mut self,
        entity: Resource<Entity>,
        max_distance: f64,
        _fluid_handling: bool,
    ) -> wasmtime::Result<Option<WitRaycastResult>> {
        let entity = entity_from_resource(self, &entity)?;
        let start = entity.get_eye_pos();
        let direction = entity.get_looking_vector();
        let end = start + direction * max_distance;
        let world = entity.get_entity().world.load_full();

        let hit = world
            .raycast(
                start,
                end,
                |pos: &pumpkin_util::math::position::BlockPos, w: &Arc<crate::world::World>| {
                    let pos = *pos;
                    let world = w.clone();
                    async move {
                        let block = world.get_block_state(&pos);
                        !block.is_air()
                    }
                },
            )
            .await;

        Ok(hit.map(|(pos, face)| WitRaycastResult {
            pos: WitBlockPos {
                x: pos.0.x,
                y: pos.0.y,
                z: pos.0.z,
            },
            face: to_wasm_block_direction(face),
        }))
    }

    async fn drop(&mut self, rep: Resource<Entity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<EntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
