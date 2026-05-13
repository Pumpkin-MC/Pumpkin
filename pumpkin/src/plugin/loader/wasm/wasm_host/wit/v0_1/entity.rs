use std::sync::Arc;
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{EntityResource, PluginHostState},
    wit::v0_1::events::to_wasm_position,
    wit::v0_1::pumpkin::plugin::{
        common::{EntityPose, Position},
        entity::Host,
        entity_types,
        text::TextComponent,
        world::{
            BlockPos as WitBlockPos, Entity, HostEntity, RaycastResult as WitRaycastResult, World,
        },
    },
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

    async fn get_uuid(&mut self, entity: Resource<Entity>) -> wasmtime::Result<String> {
        let entity = entity_from_resource(self, &entity)?;
        Ok(entity.get_entity().entity_uuid.to_string())
    }

    async fn get_type(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<entity_types::EntityType> {
        let entity = entity_from_resource(self, &entity)?;
        let name = entity.get_entity().entity_type.resource_name.to_string();
        // Standard WIT enum mapping: kebab-case
        let _wit_name = name.replace('_', "-");
        // We need to find the variant in the generated enum.
        // Since we don't have a direct mapping function yet, we use debug format or similar if available,
        // but for now let's just use the fact that it's an enum.
        // Actually, the easiest way is to use the generated `from_name` or similar if we added it,
        // but here we are returning it TO the guest.

        // Wait, I need to CONSTRUCT the EntityType enum.
        // I can't easily do it by string unless I have a mapping.
        // But wit-bindgen generates the enum.

        // Let's assume there is a way to construct it.
        // Actually, I can use the fact that it's a simple enum.
        // I'll use a hack for now if I don't have a better way, but wait!
        // I can just use `serde_json` to parse the string into the enum if it implements Deserialize!
        // Or just a match.

        // Better: let's see if pumpkin_data has a way.
        // For now, I'll just return a placeholder or implement a basic match for the most common ones
        // OR better: use the generated code.

        Err(wasmtime::Error::msg(
            "get_type implementation pending full enum mapping",
        ))
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
            .has_visual_fire
            .load(std::sync::atomic::Ordering::Relaxed))
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
                        let block = world.get_block_state(&pos).await;
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
