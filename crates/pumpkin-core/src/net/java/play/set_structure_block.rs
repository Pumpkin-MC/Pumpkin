#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_data::block_properties::{StructureBlockLikeProperties, StructureblockMode};
use pumpkin_data::block_rotation::Mirror;
use pumpkin_data::translation::java;
use pumpkin_gametest::{GameTestRotation, GameTestStructureTemplate};
use pumpkin_protocol::java::server::play::SSetStructureBlock;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;

use crate::block::entities::structure_block::StructureBlockBlockEntity;
use crate::server::structure_editor::{capture_structure, place_editor_structure};

impl JavaClient {
    #[expect(
        clippy::too_many_lines,
        reason = "structure editor packet updates all saved fields"
    )]
    pub fn handle_set_structure_block(
        &self,
        player: &Arc<Player>,
        packet: &SSetStructureBlock<'_>,
    ) {
        if !player.has_client_loaded()
            || !player.is_creative()
            || player.permission_lvl.load() < PermissionLvl::Two
        {
            return;
        }
        player.update_last_action_time();
        let world = player.world();
        let Some(entity) = world.get_block_entity(&packet.location) else {
            return;
        };
        let Some(structure) = entity.as_any().downcast_ref::<StructureBlockBlockEntity>() else {
            return;
        };
        let (mode, mode_name) = match packet.mode.0 {
            SSetStructureBlock::MODE_SAVE => (StructureblockMode::Save, "SAVE"),
            SSetStructureBlock::MODE_LOAD => (StructureblockMode::Load, "LOAD"),
            SSetStructureBlock::MODE_CORNER => (StructureblockMode::Corner, "CORNER"),
            SSetStructureBlock::MODE_DATA => (StructureblockMode::Data, "DATA"),
            _ => return,
        };
        let (mirror, block_mirror) = match packet.mirror.0 {
            0 => ("NONE", Mirror::None),
            1 => ("LEFT_RIGHT", Mirror::LeftRight),
            2 => ("FRONT_BACK", Mirror::FrontBack),
            _ => return,
        };
        let (rotation, block_rotation) = match packet.rotation.0 {
            0 => ("NONE", GameTestRotation::None),
            1 => ("CLOCKWISE_90", GameTestRotation::Clockwise90),
            2 => ("CLOCKWISE_180", GameTestRotation::Clockwise180),
            3 => ("COUNTERCLOCKWISE_90", GameTestRotation::Counterclockwise90),
            _ => return,
        };
        *structure
            .name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.name.to_string();
        *structure
            .metadata
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.metadata.to_string();
        *structure
            .mode
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = mode_name.to_string();
        *structure
            .mirror
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = mirror.to_string();
        *structure
            .rotation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = rotation.to_string();
        *structure
            .pos_x
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = i32::from(packet.offset_x);
        *structure
            .pos_y
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = i32::from(packet.offset_y);
        *structure
            .pos_z
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = i32::from(packet.offset_z);
        *structure
            .size_x
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = i32::from(packet.size_x);
        *structure
            .size_y
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = i32::from(packet.size_y);
        *structure
            .size_z
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = i32::from(packet.size_z);
        *structure
            .ignore_entities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.ignore_entities();
        *structure
            .strict
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.strict();
        *structure
            .show_air
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.show_air();
        *structure
            .show_bounding_box
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.show_bounding_box();
        *structure
            .integrity
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.integrity;
        *structure
            .seed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.seed.0;

        let mut properties =
            StructureBlockLikeProperties::from_state_id(world.get_block_state_id(&packet.location));
        properties.mode = mode;
        world.set_block_state(
            &packet.location,
            properties.to_state_id(&Block::STRUCTURE_BLOCK),
            BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
        );
        world.update_block_entity(&entity);

        if packet.name.is_empty() {
            player.send_system_message(&TextComponent::translate_cross(
                java::STRUCTURE_BLOCK_INVALID_STRUCTURE_NAME,
                java::STRUCTURE_BLOCK_INVALID_STRUCTURE_NAME,
                [TextComponent::text(packet.name.to_string())],
            ));
            return;
        }
        if packet.action.0 == SSetStructureBlock::ACTION_SAVE_AREA {
            if mode_name != "SAVE" {
                player.send_system_message(&TextComponent::translate_cross(
                    java::STRUCTURE_BLOCK_SAVE_FAILURE,
                    java::STRUCTURE_BLOCK_SAVE_FAILURE,
                    [TextComponent::text(packet.name.to_string())],
                ));
                return;
            }
            let player = player.clone();
            let Some(server) = world.server.upgrade() else {
                return;
            };
            let name = packet.name.to_string();
            let origin = BlockPos::new(
                packet.location.0.x + i32::from(packet.offset_x),
                packet.location.0.y + i32::from(packet.offset_y),
                packet.location.0.z + i32::from(packet.offset_z),
            );
            let size = [
                i32::from(packet.size_x),
                i32::from(packet.size_y),
                i32::from(packet.size_z),
            ];
            let include_entities = !packet.ignore_entities();
            server.clone().spawn_task(async move {
                let saved =
                    match capture_structure(&world, origin, size, include_entities, false).await {
                        Ok(template) => server
                            .datapack_manager
                            .save_generated_structure(&name, template)
                            .await
                            .is_ok(),
                        Err(error) => {
                            warn!("Failed to capture structure '{name}': {error}");
                            false
                        }
                    };
                let key = if saved {
                    java::STRUCTURE_BLOCK_SAVE_SUCCESS
                } else {
                    java::STRUCTURE_BLOCK_SAVE_FAILURE
                };
                player.send_system_message(&TextComponent::translate_cross(
                    key,
                    key,
                    [TextComponent::text(name)],
                ));
            });
        } else if packet.action.0 == SSetStructureBlock::ACTION_LOAD_AREA {
            let player = player.clone();
            let Some(server) = world.server.upgrade() else {
                return;
            };
            let name = packet.name.to_string();
            let saved_size = [
                i32::from(packet.size_x),
                i32::from(packet.size_y),
                i32::from(packet.size_z),
            ];
            let origin = BlockPos::new(
                packet.location.0.x + i32::from(packet.offset_x),
                packet.location.0.y + i32::from(packet.offset_y),
                packet.location.0.z + i32::from(packet.offset_z),
            );
            let ignore_entities = packet.ignore_entities();
            let strict = packet.strict();
            let integrity = packet.integrity;
            let seed = packet.seed.0;
            let location = packet.location;
            server.clone().spawn_task(async move {
                let template = server
                    .datapack_manager
                    .load_structure(&name)
                    .await
                    .and_then(|nbt| {
                        GameTestStructureTemplate::from_nbt(&nbt).map_err(|error| error.to_string())
                    });
                if !world
                    .get_block_entity(&location)
                    .is_some_and(|current| Arc::ptr_eq(&current, &entity))
                {
                    return;
                }
                let key = match template {
                    Ok(template) if mode_name == "LOAD" && template.size() != saved_size => {
                        if let Some(structure) =
                            entity.as_any().downcast_ref::<StructureBlockBlockEntity>()
                        {
                            let [x, y, z] = template.size();
                            *structure
                                .size_x
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner) = x;
                            *structure
                                .size_y
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner) = y;
                            *structure
                                .size_z
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner) = z;
                            world.update_block_entity(&entity);
                        }
                        java::STRUCTURE_BLOCK_LOAD_PREPARE
                    }
                    Ok(template) if mode_name == "LOAD" => {
                        match place_editor_structure(
                            world,
                            &template,
                            origin,
                            block_rotation,
                            block_mirror,
                            ignore_entities,
                            strict,
                            integrity,
                            seed,
                        )
                        .await
                        {
                            Ok(()) => java::STRUCTURE_BLOCK_LOAD_SUCCESS,
                            Err(error) => {
                                warn!("Failed to place structure '{name}': {error}");
                                java::STRUCTURE_BLOCK_LOAD_NOT_FOUND
                            }
                        }
                    }
                    Ok(_) => java::STRUCTURE_BLOCK_LOAD_NOT_FOUND,
                    Err(error) => {
                        warn!("Failed to load structure '{name}': {error}");
                        java::STRUCTURE_BLOCK_LOAD_NOT_FOUND
                    }
                };
                player.send_system_message(&TextComponent::translate_cross(
                    key,
                    key,
                    [TextComponent::text(name)],
                ));
            });
        }
    }
}
