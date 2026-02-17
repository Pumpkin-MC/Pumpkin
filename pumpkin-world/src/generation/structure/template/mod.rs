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
//! - [`BlockRotation`] and [`BlockMirror`]: Transform positions and block properties
//! - [`TemplateCache`]: Lazy-loading cache for embedded template files
//!
//! # Example Usage
//!
//! ```ignore
//! use pumpkin_world::generation::structure::template::{TemplateCache, TemplatePiece, BlockRotation};
//!
//! // Load a template from the cache
//! let template = TemplateCache::get("igloo/top").expect("Template not found");
//!
//! // Create a piece to place the template
//! let piece = TemplatePiece::new(template, rotation, mirror, position);
//! ```

mod block_state_resolver;
mod cache;
mod rotation;
mod structure_template;
mod template_piece;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;

use crate::ProtoChunk;

pub use block_state_resolver::BlockStateResolver;
pub use cache::{TemplateCache, get_template, global_cache};
pub use rotation::{BlockMirror, BlockRotation};
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
/// `offset` is the un-rotated XZ offset from origin (x_offset, z_offset) - rotation is applied automatically.
pub fn place_template(
    chunk: &mut ProtoChunk,
    template: &StructureTemplate,
    origin: Vector3<i32>,
    offset: (i32, i32),
    rotation: BlockRotation,
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

        // Resolve block state with rotation applied to directional properties
        let Some(state) = BlockStateResolver::resolve(palette_entry, rotation, Default::default())
        else {
            continue;
        };

        // Rotate block position within template bounds
        let local_pos = rotation.transform_pos(block.pos, template.size);

        let wx = world_x + local_pos.x;
        let wy = origin.y + local_pos.y;
        let wz = world_z + local_pos.z;

        chunk.set_block_state(wx, wy, wz, state);

        // Create block entities for interactive blocks (furnaces, chests, etc.)
        let block_entity_id = get_block_entity_id(&palette_entry.name);
        if block.nbt.is_some() || block_entity_id.is_some() {
            let block_entity_id = block_entity_id.unwrap_or(&palette_entry.name);
            let mut block_entity_nbt = NbtCompound::new();

            block_entity_nbt.put_int("x", wx);
            block_entity_nbt.put_int("y", wy);
            block_entity_nbt.put_int("z", wz);
            block_entity_nbt.put_string("id", block_entity_id.to_string());

            if let Some(template_nbt) = &block.nbt {
                for (key, value) in &template_nbt.child_tags {
                    if key != "x" && key != "y" && key != "z" && key != "id" {
                        block_entity_nbt
                            .child_tags
                            .push((key.clone(), value.clone()));
                    }
                }
            }

            chunk.add_pending_block_entity(block_entity_nbt);
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
