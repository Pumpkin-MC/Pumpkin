//! Template-based structure piece for placing NBT templates.
//!
//! This module provides `TemplatePiece`, which implements `StructurePieceBase`
//! to place blocks from a loaded structure template into the world.

use std::sync::Arc;

use pumpkin_util::{
    math::{block_box::BlockBox, vector3::Vector3},
    random::RandomGenerator,
};
use tracing::debug;

use crate::{
    ProtoChunk,
    generation::structure::{
        piece::StructurePieceType,
        structures::{StructurePiece, StructurePieceBase},
    },
};

use super::{BlockMirror, BlockRotation, BlockStateResolver, StructureTemplate};

/// A structure piece that places blocks from an NBT template.
///
/// This piece handles:
/// - Loading blocks from a `StructureTemplate`
/// - Applying rotation and mirror transformations
/// - Placing blocks at world coordinates
/// - Skipping structure void blocks
#[derive(Clone)]
pub struct TemplatePiece {
    /// The underlying structure piece metadata.
    pub piece: StructurePiece,

    /// The template to place.
    pub template: Arc<StructureTemplate>,

    /// Rotation to apply when placing.
    pub rotation: BlockRotation,

    /// Mirror to apply when placing.
    pub mirror: BlockMirror,
}

impl TemplatePiece {
    /// Creates a new template piece from the given `template` at `origin`.
    ///
    /// The `rotation` and `mirror` transforms are applied when placing blocks.
    /// The `piece_type` identifies this piece within the structure system.
    #[must_use]
    pub fn new(
        template: Arc<StructureTemplate>,
        rotation: BlockRotation,
        mirror: BlockMirror,
        origin: Vector3<i32>,
        piece_type: StructurePieceType,
    ) -> Self {
        // Calculate the bounding box based on template size and rotation
        let rotated_size = rotation.transform_size(template.size);

        let bounding_box = BlockBox::new(
            origin.x,
            origin.y,
            origin.z,
            origin.x + rotated_size.x - 1,
            origin.y + rotated_size.y - 1,
            origin.z + rotated_size.z - 1,
        );

        Self {
            piece: StructurePiece::new(piece_type, bounding_box, 0),
            template,
            rotation,
            mirror,
        }
    }

    /// Creates a template piece with a specific chain length.
    #[must_use]
    pub fn with_chain_length(
        template: Arc<StructureTemplate>,
        rotation: BlockRotation,
        mirror: BlockMirror,
        origin: Vector3<i32>,
        piece_type: StructurePieceType,
        chain_length: u32,
    ) -> Self {
        let mut piece = Self::new(template, rotation, mirror, origin, piece_type);
        piece.piece.chain_length = chain_length;
        piece
    }

    /// Returns the template's size after applying rotation.
    #[must_use]
    pub fn rotated_size(&self) -> Vector3<i32> {
        self.rotation.transform_size(self.template.size)
    }

    /// Transforms a template-relative position to world coordinates.
    fn transform_pos(&self, local_pos: Vector3<i32>) -> Vector3<i32> {
        // First apply mirror
        let mirrored = self.mirror.transform_pos(local_pos, self.template.size);

        // Then apply rotation
        let rotated = self.rotation.transform_pos(mirrored, self.template.size);

        // Add world offset
        Vector3::new(
            self.piece.bounding_box.min.x + rotated.x,
            self.piece.bounding_box.min.y + rotated.y,
            self.piece.bounding_box.min.z + rotated.z,
        )
    }

    /// Checks if a block name is structure void (should not be placed).
    fn is_structure_void(name: &str) -> bool {
        name == "minecraft:structure_void" || name == "structure_void"
    }

    /// Places all blocks from the template into the chunk.
    fn place_blocks(&self, chunk: &mut ProtoChunk) {
        let box_limit = self.piece.bounding_box;

        for block in &self.template.blocks {
            let palette_entry = &self.template.palette[block.state as usize];

            // Skip structure void blocks
            if Self::is_structure_void(&palette_entry.name) {
                continue;
            }

            // Resolve the block state with rotation/mirror
            let state = match BlockStateResolver::resolve(palette_entry, self.rotation, self.mirror)
            {
                Some(s) => s,
                None => {
                    debug!("Failed to resolve block: {}", palette_entry.name);
                    continue;
                }
            };

            // Transform position to world coordinates
            let world_pos = self.transform_pos(block.pos);

            // Check bounds
            if !box_limit.contains_pos(&world_pos) {
                continue;
            }

            // Place the block
            chunk.set_block_state(world_pos.x, world_pos.y, world_pos.z, state);

            // TODO: Handle block entity data (block.nbt)
            // This would involve creating a block entity at this position
            // and populating it with the NBT data
        }
    }
}

impl StructurePieceBase for TemplatePiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        self.place_blocks(chunk);

        // TODO: Spawn entities from template
        // This would involve iterating over self.template.entities
        // and spawning them at the transformed positions
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }
}
