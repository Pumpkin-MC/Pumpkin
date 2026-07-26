//! NBT Structure Template System
//!
//! This module provides functionality for loading and placing Minecraft structure
//! templates from `.nbt` files. This enables exact vanilla structure matching and
//! dramatically simplifies implementing structures like igloos, shipwrecks, villages, etc.
//!
//! # Architecture
//!
//! - [`StructureTemplate`]: Represents a loaded NBT template with size, palette, and blocks
//! - [`TemplatePiece`]: A structure piece that places blocks from a template
//! - [`Rotation`] and [`Mirror`]: Transform positions and block properties
//! - [`TemplateCache`]: Lazy-loading cache for embedded template files
//!
//! # Example Usage
//!
//! ```ignore
//! use pumpkin_world::generation::structure::template::{TemplateCache, TemplatePiece};
//! use pumpkin_data::Rotation;
//!
//! // Load a template from the cache
//! let template = TemplateCache::get("igloo/top").expect("Template not found");
//!
//! // Create a piece to place the template
//! let piece = TemplatePiece::new(template, rotation, mirror, position);
//! ```

mod block_state_resolver;
mod cache;
pub mod processor;
mod structure_template;
mod template_piece;

use pumpkin_data::Mirror;
use pumpkin_data::Rotation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomImpl, hash_block_pos, legacy_rand::LegacyRand};

use crate::ProtoChunk;

pub use block_state_resolver::BlockStateResolver;
pub use cache::{
    TemplateCache, get_pool_elements, get_processor_list_json, get_template,
    get_template_pool_json, global_cache,
};
pub use processor::StructureProcessor;
pub use pumpkin_data::{Mirror as BlockMirror, Rotation as BlockRotation};
pub use structure_template::{PaletteEntry, StructureTemplate, TemplateBlock, TemplateEntity};
pub use template_piece::TemplatePiece;

/// Places a template at a world origin with an un-rotated XZ offset.
///
/// All rotation is handled internally:
/// - The offset is rotated to position the template correctly
/// - Block positions within the template are rotated
/// - Directional block properties (facing, axis, etc.) are rotated
/// - Block entities are created from template NBT data
///
/// `origin` is the base world position (x, y, z).
/// `offset` is the un-rotated XZ offset from origin (`x_offset`, `z_offset`) - rotation is applied automatically.
#[allow(clippy::too_many_arguments)]
pub fn place_template(
    chunk: &mut ProtoChunk,
    template: &StructureTemplate,
    origin: Vector3<i32>,
    offset: (i32, i32),
    rotation: Rotation,
    skip_air: bool,
    apply_waterlogging: bool,
    processors: &[StructureProcessor],
    chunk_box: Option<&pumpkin_util::math::block_box::BlockBox>,
) {
    let (rotated_ox, rotated_oz) = rotation.rotate_offset(offset.0, offset.1);
    let world_x = origin.x + rotated_ox;
    let world_z = origin.z + rotated_oz;

    place_template_blocks(
        chunk,
        template,
        origin,
        world_x,
        world_z,
        rotation,
        skip_air,
        apply_waterlogging,
        processors,
        chunk_box,
    );
    place_template_entities(
        chunk, template, origin, world_x, world_z, rotation, chunk_box,
    );
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn place_template_blocks(
    chunk: &mut ProtoChunk,
    template: &StructureTemplate,
    origin: Vector3<i32>,
    world_x: i32,
    world_z: i32,
    rotation: Rotation,
    skip_air: bool,
    apply_waterlogging: bool,
    processors: &[StructureProcessor],
    chunk_box: Option<&pumpkin_util::math::block_box::BlockBox>,
) {
    for block in &template.blocks {
        let palette_entry = &template.palette[block.state as usize];

        // Structure blocks are data markers and structure void preserves the existing block.
        if palette_entry.name == "minecraft:structure_void"
            || palette_entry.name == "minecraft:structure_block"
        {
            continue;
        }

        // Skip air blocks when using IGNORE_AIR processor (e.g. nether fossils)
        if skip_air && palette_entry.name == "minecraft:air" {
            continue;
        }

        let mut block_entity_nbt = block.nbt.clone();
        let mut placed_entry = palette_entry.clone();

        // Jigsaw blocks are replaced during template processing, before block entities are
        // collected. Keeping this in the placement pipeline avoids stale jigsaw entities.
        if palette_entry.name == "minecraft:jigsaw" {
            let final_state = block_entity_nbt
                .as_ref()
                .and_then(|nbt| nbt.get_string("final_state"))
                .unwrap_or("minecraft:air");
            placed_entry = PaletteEntry::from_string(final_state);
            block_entity_nbt = None;
        }

        // Resolve block state with rotation applied to directional properties
        let Some(mut state) =
            BlockStateResolver::resolve(&placed_entry, rotation, Mirror::default())
        else {
            continue;
        };

        // Rotate block position within template bounds
        let local_pos = rotation.transform_pos(block.pos, template.size);

        let wx = world_x + local_pos.x;
        let wy = origin.y + local_pos.y;
        let wz = world_z + local_pos.z;

        if let Some(bbox) = chunk_box
            && (wx < bbox.min.x
                || wx > bbox.max.x
                || wy < bbox.min.y
                || wy > bbox.max.y
                || wz < bbox.min.z
                || wz > bbox.max.z)
        {
            continue;
        }

        let world_pos = Vector3::new(wx, wy, wz);

        if apply_waterlogging
            && chunk.get_block_state(&world_pos).to_block_id() == pumpkin_data::Block::WATER.id
            && let Some((_, waterlogged)) = placed_entry
                .properties
                .iter_mut()
                .find(|(name, _)| name == "waterlogged")
        {
            *waterlogged = "true".to_string();
            if let Some(waterlogged_state) =
                BlockStateResolver::resolve(&placed_entry, rotation, Mirror::default())
            {
                state = waterlogged_state;
            }
        }

        // Apply processors
        let mut should_place = true;
        for processor in processors {
            let Some(processed_state) = processor.process(chunk, world_pos, state) else {
                should_place = false;
                break;
            };
            state = processed_state;
        }
        if !should_place {
            continue;
        }

        chunk.set_block_state(wx, wy, wz, state);
        place_block_entity(chunk, &placed_entry, block_entity_nbt.as_ref(), wx, wy, wz);
    }
}

fn place_block_entity(
    chunk: &mut ProtoChunk,
    placed_entry: &PaletteEntry,
    block_entity_nbt: Option<&NbtCompound>,
    wx: i32,
    wy: i32,
    wz: i32,
) {
    let block_entity_id = get_block_entity_id(&placed_entry.name);
    if block_entity_nbt.is_none() && block_entity_id.is_none() {
        return;
    }
    let block_entity_id = block_entity_id.unwrap_or(&placed_entry.name);
    let mut placed_nbt = NbtCompound::new();

    placed_nbt.put_string("id", block_entity_id.to_string());
    placed_nbt.put_int("x", wx);
    placed_nbt.put_int("y", wy);
    placed_nbt.put_int("z", wz);

    if let Some(template_nbt) = block_entity_nbt {
        for (key, value) in &template_nbt.child_tags {
            if key.as_ref() != "x"
                && key.as_ref() != "y"
                && key.as_ref() != "z"
                && key.as_ref() != "id"
            {
                placed_nbt.child_tags.insert(key.clone(), value.clone());
            }
        }
    }

    if placed_nbt.get_string("LootTable").is_some()
        && placed_nbt.get_long("LootTableSeed").is_none()
    {
        let mut random = LegacyRand::from_seed(hash_block_pos(wx, wy, wz) as u64);
        placed_nbt.put_long("LootTableSeed", random.next_i64());
    }

    chunk.add_block_entity(placed_nbt);
}

fn place_template_entities(
    chunk: &mut ProtoChunk,
    template: &StructureTemplate,
    origin: Vector3<i32>,
    world_x: i32,
    world_z: i32,
    rotation: Rotation,
    chunk_box: Option<&pumpkin_util::math::block_box::BlockBox>,
) {
    // Spawn structure-template entities (villagers, iron golems, animals, …).
    // Vanilla places these when the structure piece is applied to the chunk.
    for entity in &template.entities {
        let local_block = rotation.transform_pos(entity.block_pos, template.size);
        let wx = world_x + local_block.x;
        let wy = origin.y + local_block.y;
        let wz = world_z + local_block.z;

        if let Some(bbox) = chunk_box
            && (wx < bbox.min.x
                || wx > bbox.max.x
                || wy < bbox.min.y
                || wy > bbox.max.y
                || wz < bbox.min.z
                || wz > bbox.max.z)
        {
            continue;
        }

        // Transform fractional position the same way as block coords.
        let local_pos = transform_entity_pos(rotation, entity.pos, template.size);
        let world_ex = f64::from(world_x) + local_pos.x;
        let world_ey = f64::from(origin.y) + local_pos.y;
        let world_ez = f64::from(world_z) + local_pos.z;

        let mut placed = entity.nbt.clone();
        // Template NBT already contains Pos/Motion/Rotation, so replace those
        // fields rather than using NbtCompound::put (which preserves old values).
        placed.child_tags.insert(
            "Pos".into(),
            NbtTag::List(vec![
                NbtTag::Double(world_ex),
                NbtTag::Double(world_ey),
                NbtTag::Double(world_ez),
            ]),
        );
        placed.child_tags.insert(
            "Motion".into(),
            NbtTag::List(vec![
                NbtTag::Double(0.0),
                NbtTag::Double(0.0),
                NbtTag::Double(0.0),
            ]),
        );
        let yaw = placed
            .get_list("Rotation")
            .and_then(|rotation| rotation.first())
            .and_then(NbtTag::extract_float)
            .unwrap_or(0.0);
        let pitch = placed
            .get_list("Rotation")
            .and_then(|rotation| rotation.get(1))
            .and_then(NbtTag::extract_float)
            .unwrap_or(0.0);
        let rotation_degrees = match rotation {
            Rotation::None => 0.0,
            Rotation::Clockwise90 => 90.0,
            Rotation::Rotate180 => 180.0,
            Rotation::CounterClockwise90 => 270.0,
        };
        let yaw = (yaw + 180.0).rem_euclid(360.0) - 180.0 + rotation_degrees;
        placed.child_tags.insert(
            "Rotation".into(),
            NbtTag::List(vec![NbtTag::Float(yaw), NbtTag::Float(pitch)]),
        );
        // Vanilla assigns a fresh UUID to every entity placed from a template.
        placed.child_tags.remove("UUID");
        chunk.add_entity(placed);
    }
}

/// Rotate an entity's relative double position within a template of `size`.
fn transform_entity_pos(rotation: Rotation, pos: Vector3<f64>, size: Vector3<i32>) -> Vector3<f64> {
    match rotation {
        Rotation::None => pos,
        Rotation::Clockwise90 => Vector3::new(f64::from(size.z) - pos.z, pos.y, pos.x),
        Rotation::Rotate180 => {
            Vector3::new(f64::from(size.x) - pos.x, pos.y, f64::from(size.z) - pos.z)
        }
        Rotation::CounterClockwise90 => Vector3::new(pos.z, pos.y, f64::from(size.x) - pos.x),
    }
}

/// Returns the block entity ID for blocks that require one, or None if not needed.
pub(crate) fn get_block_entity_id(block_name: &str) -> Option<&'static str> {
    match block_name {
        "minecraft:furnace" => Some("minecraft:furnace"),
        "minecraft:chest" => Some("minecraft:chest"),
        "minecraft:trapped_chest" => Some("minecraft:trapped_chest"),
        "minecraft:barrel" => Some("minecraft:barrel"),
        "minecraft:hopper" => Some("minecraft:hopper"),
        "minecraft:dropper" => Some("minecraft:dropper"),
        "minecraft:dispenser" => Some("minecraft:dispenser"),
        "minecraft:brewing_stand" => Some("minecraft:brewing_stand"),
        "minecraft:blast_furnace" => Some("minecraft:blast_furnace"),
        "minecraft:smoker" => Some("minecraft:smoker"),
        "minecraft:shulker_box" => Some("minecraft:shulker_box"),
        "minecraft:bed" => Some("minecraft:bed"),
        "minecraft:sign"
        | "minecraft:oak_sign"
        | "minecraft:spruce_sign"
        | "minecraft:birch_sign"
        | "minecraft:jungle_sign"
        | "minecraft:acacia_sign"
        | "minecraft:dark_oak_sign"
        | "minecraft:mangrove_sign"
        | "minecraft:cherry_sign"
        | "minecraft:bamboo_sign"
        | "minecraft:crimson_sign"
        | "minecraft:warped_sign" => Some("minecraft:sign"),
        "minecraft:hanging_sign" => Some("minecraft:hanging_sign"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Rotation, Vector3, transform_entity_pos};

    #[test]
    fn rotated_entity_position_stays_at_the_block_center() {
        let size = Vector3::new(5, 4, 7);
        let pos = Vector3::new(0.5, 1.0, 0.5);

        let clockwise = transform_entity_pos(Rotation::Clockwise90, pos, size);
        assert_eq!(clockwise, Vector3::new(6.5, 1.0, 0.5));

        let reversed = transform_entity_pos(Rotation::Rotate180, pos, size);
        assert_eq!(reversed, Vector3::new(4.5, 1.0, 6.5));

        let counter_clockwise = transform_entity_pos(Rotation::CounterClockwise90, pos, size);
        assert_eq!(counter_clockwise, Vector3::new(0.5, 1.0, 4.5));
    }
}
