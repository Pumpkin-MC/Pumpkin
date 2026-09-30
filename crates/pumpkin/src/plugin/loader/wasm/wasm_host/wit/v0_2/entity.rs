use super::AccessorExt;
use super::run_blocking;
use std::sync::Arc;
use wasmtime::component::Accessor;
use wasmtime::component::{HasSelf, Resource};

use pumpkin_util::math::vector3::Vector3;

use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_2::events::to_wasm_position,
    wit::v0_2::pumpkin::plugin::{
        common::{EntityPose, NbtTree as WitNbtTree, Position},
        entity::Host,
        entity_types,
        text::TextComponent,
        uuid::Uuid,
        world::{
            BlockPos as WitBlockPos, BoundingBox as WitBoundingBox, Entity, HostEntity,
            HostEntityWithStore, LivingEntity as WitLivingEntity, Mob as WitMob,
            RayTraceBlockResult as WitRayTraceBlockResult,
            RayTraceEntityResult as WitRayTraceEntityResult, RaycastResult as WitRaycastResult,
            World,
        },
    },
    wit::v0_2::uuid::UuidExt,
    wit::v0_2::world::to_wasm_block_direction,
};
use pumpkin_data::entity::EntityPose as InternalEntityPose;

impl Host for PluginHostState {}
impl entity_types::Host for PluginHostState {}

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
    async fn drop(&mut self, rep: Resource<Entity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_id(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<u32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity.get_entity().entity_id as u32)
        })
    }

    async fn get_uuid(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Uuid> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(Uuid::to_wit(&entity.get_entity().entity_uuid))
        })
    }

    async fn get_type(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<entity_types::EntityType> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            let original_name = entity.get_entity().entity_type.resource_name;
            to_wit_entity_type(original_name)
        })
    }

    async fn get_position(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Position> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(to_wasm_position(entity.get_entity().pos.load()))
        })
    }

    async fn get_world(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Resource<World>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            let world = entity.get_entity().world.load_full();
            state
                .add(world)
                .map_err(|_| wasmtime::Error::msg("failed to add world resource"))
        })
    }

    async fn get_yaw(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<f32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity.get_entity().yaw.load())
        })
    }

    async fn get_pitch(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<f32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity.get_entity().pitch.load())
        })
    }

    async fn get_head_yaw(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<f32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity.get_entity().head_yaw.load())
        })
    }

    async fn is_on_ground(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .on_ground
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn is_sneaking(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .sneaking
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn is_sprinting(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .sprinting
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn is_invisible(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .invisible
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn is_glowing(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .glowing
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn set_velocity(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        velocity: Position,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity
                .get_entity()
                .velocity
                .store(pumpkin_util::math::vector3::Vector3::new(
                    velocity.0, velocity.1, velocity.2,
                ));
            Ok(())
        })
    }

    async fn get_velocity(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Position> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(to_wasm_position(entity.get_entity().velocity.load()))
        })
    }

    async fn set_sneaking(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        sneaking: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity.get_entity().set_sneaking(sneaking);
            Ok(())
        })
    }

    async fn set_sprinting(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        sprinting: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity.get_entity().set_sprinting(sprinting);
            Ok(())
        })
    }

    async fn is_swimming(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .swimming
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn set_invisible(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        invisible: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity.get_entity().set_invisible(invisible);
            Ok(())
        })
    }

    async fn set_glowing(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        glowing: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity.get_entity().set_glowing(glowing);
            Ok(())
        })
    }

    async fn is_fall_flying(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .fall_flying
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn set_fall_flying(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        fall_flying: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity.get_entity().set_fall_flying(fall_flying);
            Ok(())
        })
    }

    async fn is_on_fire(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .fire_ticks
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0)
        })
    }

    async fn set_on_fire(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        on_fire: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity.get_entity().set_on_fire(on_fire);
            Ok(())
        })
    }

    async fn get_pose(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<EntityPose> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(map_entity_pose(entity.get_entity().pose.load()))
        })
    }

    async fn get_name(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            let name = entity.get_name();
            state
                .add(name)
                .map_err(|_| wasmtime::Error::msg("failed to add text component resource"))
        })
    }

    async fn set_custom_name(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        name: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let text = state.take(name)?;
            let entity_base = state.get(&entity)?;
            entity_base.get_entity().set_custom_name(text);
            Ok(())
        })
    }

    async fn get_custom_name(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<TextComponent>>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            let name = entity.get_entity().custom_name.load();
            if let Some(name) = name.as_ref() {
                Ok(Some(state.add(name.clone()).map_err(|_| {
                    wasmtime::Error::msg("failed to add text component resource")
                })?))
            } else {
                Ok(None)
            }
        })
    }

    async fn set_custom_name_visible(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        visible: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity.get_entity().set_custom_name_visible(visible);
            Ok(())
        })
    }

    async fn is_custom_name_visible(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .custom_name_visible
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn is_invulnerable(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .invulnerable
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn set_invulnerable(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        invulnerable: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity
                .get_entity()
                .invulnerable
                .store(invulnerable, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        })
    }

    async fn get_fire_ticks(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .fire_ticks
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn set_fire_ticks(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        ticks: i32,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity
                .get_entity()
                .fire_ticks
                .store(ticks, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        })
    }

    async fn get_fall_distance(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<f32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_living_entity()
                .map_or(0.0, |living| living.fall_distance.load()))
        })
    }

    async fn set_fall_distance(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        distance: f32,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            if let Some(living) = entity.get_living_entity() {
                living.fall_distance.store(distance);
            }
            Ok(())
        })
    }

    async fn is_silent(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity.get_entity().is_silent())
        })
    }

    async fn set_silent(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        silent: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity.get_entity().set_silent(silent);
            Ok(())
        })
    }

    async fn has_gravity(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(!entity.get_entity().has_no_gravity())
        })
    }

    async fn set_has_gravity(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        gravity: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity.get_entity().set_has_no_gravity(!gravity);
            Ok(())
        })
    }

    async fn get_eye_height(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<f32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity.get_entity().entity_dimension.load().eye_height)
        })
    }

    async fn get_eye_position(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Position> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(to_wasm_position(entity.get_eye_pos()))
        })
    }

    async fn get_nearby_entities(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        x: f64,
        y: f64,
        z: f64,
    ) -> wasmtime::Result<Vec<Resource<Entity>>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?.clone();
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
                        state
                            .add(e)
                            .map_err(|_| wasmtime::Error::msg("failed to add entity resource"))?,
                    );
                }
            }
            Ok(result)
        })
    }

    async fn get_vehicle(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<Entity>>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?.clone();
            let vehicle = entity
                .get_entity()
                .vehicle
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(v) = vehicle.as_ref() {
                Ok(Some(state.add(Arc::clone(v)).map_err(|_| {
                    wasmtime::Error::msg("failed to add entity resource")
                })?))
            } else {
                Ok(None)
            }
        })
    }

    async fn get_passengers(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Vec<Resource<Entity>>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?.clone();
            let passengers = entity
                .get_entity()
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut result = Vec::new();
            for p in passengers.iter() {
                result.push(
                    state
                        .add(Arc::clone(p))
                        .map_err(|_| wasmtime::Error::msg("failed to add entity resource"))?,
                );
            }
            Ok(result)
        })
    }

    async fn get_bounding_box(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<WitBoundingBox> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            let bb = entity.get_entity().bounding_box.load();
            Ok(WitBoundingBox {
                min: to_wasm_position(bb.min),
                max: to_wasm_position(bb.max),
            })
        })
    }

    async fn is_in_water(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .touching_water
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn is_in_lava(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .touching_lava
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn get_ticks_lived(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .age
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn set_ticks_lived(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        ticks: i32,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity
                .get_entity()
                .age
                .store(ticks, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        })
    }

    async fn get_width(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<f32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity.get_entity().entity_dimension.load().width)
        })
    }

    async fn get_height(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<f32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity.get_entity().entity_dimension.load().height)
        })
    }

    async fn set_rotation(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        yaw: f32,
        pitch: f32,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity.get_entity().set_rotation(yaw, pitch);
            Ok(())
        })
    }

    async fn has_visual_fire(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .has_visual_fire
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn set_visual_fire(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        visual_fire: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity.get_entity().set_on_fire(visual_fire);
            Ok(())
        })
    }

    async fn get_portal_cooldown(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<u32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity
                .get_entity()
                .portal_cooldown
                .load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    async fn set_portal_cooldown(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        cooldown: u32,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity
                .get_entity()
                .portal_cooldown
                .store(cooldown, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        })
    }

    async fn get_remaining_air(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            Ok(entity.get_player().map_or(0, |player| {
                player
                    .breath_manager
                    .air_supply
                    .load(std::sync::atomic::Ordering::Relaxed)
            }))
        })
    }

    async fn set_remaining_air(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        air: i32,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            if let Some(player) = entity.get_player() {
                player
                    .breath_manager
                    .air_supply
                    .store(air, std::sync::atomic::Ordering::Relaxed);
                player.breath_manager.send_air_supply(player);
            }
            Ok(())
        })
    }

    async fn get_max_air(
        _accessor: &Accessor<PluginHostState, Self>,
        _entity: Resource<Entity>,
    ) -> wasmtime::Result<i32> {
        Ok(crate::entity::breath::MAX_AIR)
    }

    async fn remove(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
            entity.get_entity().remove();
            Ok(())
        })
    }

    async fn raycast(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        max_distance: f64,
        fluid_handling: bool,
    ) -> wasmtime::Result<Option<WitRaycastResult>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
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
        })
    }

    async fn ray_trace_block(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        max_distance: f64,
        include_fluids: bool,
    ) -> wasmtime::Result<Option<WitRayTraceBlockResult>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&entity)?;
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
        })
    }

    async fn ray_trace_entity(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        max_distance: f64,
    ) -> wasmtime::Result<Option<WitRayTraceEntityResult>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity_base = state.get(&entity)?;
            let start = entity_base.get_eye_pos();
            let direction = entity_base.get_looking_vector();
            let end = start + direction * max_distance;
            let world = entity_base.get_entity().world.load_full();
            let self_id = entity_base.get_entity().entity_id;

            let hits = world.ray_trace_entities(start, end);
            for (hit_entity, hit_pos, distance) in hits {
                if hit_entity.get_entity().entity_id != self_id {
                    let entity_res = state
                        .add(hit_entity)
                        .map_err(|_| wasmtime::Error::msg("failed to add entity resource"))?;
                    return Ok(Some(WitRayTraceEntityResult {
                        entity: entity_res,
                        hit_pos: to_wasm_position(hit_pos),
                        distance,
                    }));
                }
            }

            Ok(None)
        })
    }

    async fn get_target_entity(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        max_distance: f64,
    ) -> wasmtime::Result<Option<Resource<Entity>>> {
        let res = Self::ray_trace_entity(accessor, entity, max_distance).await?;
        Ok(res.map(|r| r.entity))
    }

    async fn set_custom_data(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<Entity>,
        namespace: String,
        key: String,
        value: WitNbtTree,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&this)?;
            let base_entity = entity.get_entity();
            let tag = super::common::from_wit_nbt_tree(&value).map_err(wasmtime::Error::msg)?;
            base_entity.set_custom_data(&namespace, &key, tag);
            Ok(())
        })
    }

    async fn get_custom_data(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<Entity>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<Option<WitNbtTree>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&this)?;
            let base_entity = entity.get_entity();
            let tag = base_entity.get_custom_data(&namespace, &key);
            Ok(tag.map(super::common::to_wit_nbt_tree))
        })
    }

    async fn remove_custom_data(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<Entity>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&this)?;
            let base_entity = entity.get_entity();
            base_entity.remove_custom_data(&namespace, &key);
            Ok(())
        })
    }

    async fn has_custom_data(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<Entity>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&this)?;
            let base_entity = entity.get_entity();
            Ok(base_entity.has_custom_data(&namespace, &key))
        })
    }

    async fn as_living(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<WitLivingEntity>>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&this)?.clone();
            if entity.get_living_entity().is_some() {
                Ok(Some(state.add(entity)?))
            } else {
                Ok(None)
            }
        })
    }

    async fn as_mob(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<WitMob>>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&this)?.clone();
            if entity.get_mob().is_some() {
                Ok(Some(state.add(entity)?))
            } else {
                Ok(None)
            }
        })
    }

    async fn is_living(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&this)?;
            Ok(entity.get_living_entity().is_some())
        })
    }

    async fn is_mob(
        accessor: &Accessor<PluginHostState, Self>,
        this: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&this)?;
            Ok(entity.get_mob().is_some())
        })
    }
    async fn teleport(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        pos: Position,
        world: Resource<World>,
    ) -> wasmtime::Result<()> {
        let (entity, world) = accessor.with(|mut host| -> wasmtime::Result<_> {
            let state = host.get();
            let world = state.take(world)?;
            let entity = state.get(&entity)?.clone();
            Ok((entity, world))
        })?;
        let pos = Vector3::new(pos.0, pos.1, pos.2);
        run_blocking(accessor, move || entity.teleport(pos, None, None, world)).await
    }

    async fn set_swimming(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        swimming: bool,
    ) -> wasmtime::Result<()> {
        let entity = accessor.get_res(&entity)?;
        run_blocking(accessor, move || {
            entity.get_entity().set_swimming(swimming);
        })
        .await
    }

    async fn set_vehicle(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        vehicle: Option<Resource<Entity>>,
    ) -> wasmtime::Result<()> {
        let (entity, vehicle) = accessor.with(|mut host| -> wasmtime::Result<_> {
            let state = host.get();
            let entity = state.get(&entity)?.clone();
            let vehicle = vehicle.map(|vehicle| state.take(vehicle)).transpose()?;
            Ok((entity, vehicle))
        })?;
        run_blocking(accessor, move || {
            let current_vehicle = entity
                .get_entity()
                .vehicle
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            if let Some(current_vehicle) = current_vehicle {
                current_vehicle
                    .get_entity()
                    .remove_passenger(entity.get_entity().entity_id);
            }
            if let Some(vehicle) = vehicle {
                vehicle
                    .get_entity()
                    .add_passenger(Arc::clone(&vehicle), entity);
            }
        })
        .await
    }

    async fn add_passenger(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        passenger: Resource<Entity>,
    ) -> wasmtime::Result<()> {
        let (entity, passenger) = accessor.with(|mut host| -> wasmtime::Result<_> {
            let state = host.get();
            Ok((state.get(&entity)?.clone(), state.take(passenger)?))
        })?;
        run_blocking(accessor, move || {
            entity
                .get_entity()
                .add_passenger(Arc::clone(&entity), passenger);
        })
        .await
    }

    async fn remove_passenger(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
        passenger: Resource<Entity>,
    ) -> wasmtime::Result<()> {
        let (entity, passenger_id) = accessor.with(|mut host| -> wasmtime::Result<_> {
            let state = host.get();
            let entity = state.get(&entity)?.clone();
            let passenger = state.take(passenger)?;
            Ok((entity, passenger.get_entity().entity_id))
        })?;
        run_blocking(accessor, move || {
            entity.get_entity().remove_passenger(passenger_id);
        })
        .await
    }

    async fn eject_passengers(
        accessor: &Accessor<PluginHostState, Self>,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<()> {
        let entity = accessor.get_res(&entity)?;
        run_blocking(accessor, move || {
            let passenger_ids: Vec<i32> = entity
                .get_entity()
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .iter()
                .map(|passenger| passenger.get_entity().entity_id)
                .collect();
            for passenger_id in passenger_ids {
                entity.get_entity().remove_passenger(passenger_id);
            }
        })
        .await
    }
}

pub(crate) fn to_wit_entity_type(name: &str) -> wasmtime::Result<entity_types::EntityType> {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    let index = pumpkin_data::entity::EntityType::ALL
        .binary_search_by_key(&name, |e| e.resource_name)
        .map_err(|_| wasmtime::Error::msg(format!("Unknown entity type: {name}")))?;

    // SAFETY: The WIT enum is generated with variants matching EntityType::ALL in alphabetical order.
    Ok(unsafe { std::mem::transmute::<u8, entity_types::EntityType>(index as u8) })
}

pub(crate) fn from_wit_entity_type(
    entity_type: entity_types::EntityType,
) -> wasmtime::Result<&'static pumpkin_data::entity::EntityType> {
    let index = entity_type as usize;
    pumpkin_data::entity::EntityType::ALL
        .get(index)
        .copied()
        .ok_or_else(|| wasmtime::Error::msg(format!("Invalid entity type index: {index}")))
}
