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

use pumpkin_data::Rotation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;

use crate::ProtoChunk;

pub use block_state_resolver::BlockStateResolver;
pub use cache::{TemplateCache, get_pool_elements, get_template, global_cache};
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
    processors: &[Box<dyn StructureProcessor>],
    chunk_box: Option<&pumpkin_util::math::block_box::BlockBox>,
) {
    let (rotated_ox, rotated_oz) = rotation.rotate_offset(offset.0, offset.1);
    let world_x = origin.x + rotated_ox;
    let world_z = origin.z + rotated_oz;

    for block in &template.blocks {
        let palette_entry = &template.palette[block.state as usize];

        // Skip structure void blocks
        if palette_entry.name == "minecraft:structure_void" {
            continue;
        }

        // Skip air blocks when using IGNORE_AIR processor (e.g. nether fossils)
        if skip_air && palette_entry.name == "minecraft:air" {
            continue;
        }

        // Resolve block state with rotation applied to directional properties
        let Some(mut state) =
            BlockStateResolver::resolve(palette_entry, rotation, Default::default())
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

        // Apply processors
        for processor in processors {
            state = processor.process(chunk, world_pos, state);
        }

        chunk.set_block_state(wx, wy, wz, state);

        // Create block entities for interactive blocks (furnaces, chests, etc.)
        let block_entity_id = get_block_entity_id(&palette_entry.name);
        if block.nbt.is_some() || block_entity_id.is_some() {
            let block_entity_id = block_entity_id.unwrap_or(&palette_entry.name);
            let mut block_entity_nbt = NbtCompound::new();

            block_entity_nbt.put_string("id", block_entity_id.to_string());
            block_entity_nbt.put_int("x", wx);
            block_entity_nbt.put_int("y", wy);
            block_entity_nbt.put_int("z", wz);

            if let Some(template_nbt) = &block.nbt {
                for (key, value) in &template_nbt.child_tags {
                    if key.as_ref() != "x"
                        && key.as_ref() != "y"
                        && key.as_ref() != "z"
                        && key.as_ref() != "id"
                    {
                        block_entity_nbt
                            .child_tags
                            .insert(key.clone(), value.clone());
                    }
                }
            }

            chunk.add_block_entity(block_entity_nbt);
        }
    }
}

/// Returns the block entity ID for blocks that require one, or None if not needed.
fn get_block_entity_id(block_name: &str) -> Option<&'static str> {
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
