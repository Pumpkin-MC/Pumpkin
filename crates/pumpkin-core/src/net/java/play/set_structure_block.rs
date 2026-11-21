#[allow(clippy::wildcard_imports)]
use super::*;
use crate::block::entities::structure_block::{
    StructureBlockBlockEntity, mirror_from_packet_id, mode_from_packet_id, rotation_from_packet_id,
};
use pumpkin_data::translation::java as translation;
use pumpkin_protocol::java::server::play::SSetStructureBlock;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

fn apply_structure_block_settings(
    structure_block: &StructureBlockBlockEntity,
    world: &Arc<World>,
    packet: &SSetStructureBlock<'_>,
) {
    structure_block.set_mode(world, mode_from_packet_id(packet.mode.0));
    structure_block.set_structure_name(packet.name);
    structure_block.set_structure_pos(Vector3::new(
        i32::from(packet.offset_x),
        i32::from(packet.offset_y),
        i32::from(packet.offset_z),
    ));
    structure_block.set_structure_size(Vector3::new(
        i32::from(packet.size_x),
        i32::from(packet.size_y),
        i32::from(packet.size_z),
    ));
    structure_block.set_mirror_value(mirror_from_packet_id(packet.mirror.0));
    structure_block.set_rotation_value(rotation_from_packet_id(packet.rotation.0));
    *structure_block
        .metadata
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.metadata.to_string();
    *structure_block
        .ignore_entities
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.ignore_entities();
    *structure_block
        .strict
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.strict();
    *structure_block
        .show_air
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.show_air();
    *structure_block
        .show_bounding_box
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.show_bounding_box();
    *structure_block
        .integrity
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.integrity;
    *structure_block
        .seed
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.seed.0;
}

impl JavaClient {
    pub fn handle_set_structure_block(&self, player: &Player, packet: &SSetStructureBlock<'_>) {
        if !player.is_creative() || player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }

        let pos = packet.location;
        let world = player.world();
        let Some(block_entity) = world.get_block_entity(&pos) else {
            return;
        };
        let Some(structure_block) = block_entity
            .as_any()
            .downcast_ref::<StructureBlockBlockEntity>()
        else {
            return;
        };

        apply_structure_block_settings(structure_block, &world, packet);

        if structure_block.has_structure_name() {
            let structure_name = structure_block.get_structure_name();
            match packet.action.0 {
                SSetStructureBlock::ACTION_SAVE_AREA => {
                    let key = if structure_block.save_structure_area(&world) {
                        translation::STRUCTURE_BLOCK_SAVE_SUCCESS
                    } else {
                        translation::STRUCTURE_BLOCK_SAVE_FAILURE
                    };
                    player.send_system_message(&TextComponent::translate(
                        key,
                        vec![TextComponent::text(structure_name)],
                    ));
                }
                SSetStructureBlock::ACTION_LOAD_AREA => {
                    if !structure_block.is_structure_loadable(&world) {
                        player.send_system_message(&TextComponent::translate(
                            translation::STRUCTURE_BLOCK_LOAD_NOT_FOUND,
                            vec![TextComponent::text(structure_name)],
                        ));
                    } else if structure_block.place_structure_if_same_size(&world) {
                        player.send_system_message(&TextComponent::translate(
                            translation::STRUCTURE_BLOCK_LOAD_SUCCESS,
                            vec![TextComponent::text(structure_name)],
                        ));
                    } else {
                        player.send_system_message(&TextComponent::translate(
                            translation::STRUCTURE_BLOCK_LOAD_PREPARE,
                            vec![TextComponent::text(structure_name)],
                        ));
                    }
                }
                SSetStructureBlock::ACTION_SCAN_AREA => {
                    if structure_block.detect_size(&world) {
                        player.send_system_message(&TextComponent::translate(
                            translation::STRUCTURE_BLOCK_SIZE_SUCCESS,
                            vec![TextComponent::text(structure_name)],
                        ));
                    } else {
                        player.send_system_message(&TextComponent::translate(
                            translation::STRUCTURE_BLOCK_SIZE_FAILURE,
                            Vec::new(),
                        ));
                    }
                }
                _ => {}
            }
        } else {
            player.send_system_message(&TextComponent::translate(
                translation::STRUCTURE_BLOCK_INVALID_STRUCTURE_NAME,
                vec![TextComponent::text(packet.name.to_string())],
            ));
        }

        structure_block.mark_dirty();
        world.update_block_entity(&block_entity);
        let state_id = world.get_block_state_id(&pos);
        world.register_block_change(pos, state_id);
    }
}
