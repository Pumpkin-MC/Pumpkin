//! Template-based structure piece for placing NBT templates.
//!
//! This module provides `TemplatePiece`, which implements `StructurePieceBase`
//! to place blocks from a loaded structure template into the world.

use std::sync::Arc;

use pumpkin_data::{Mirror, Rotation};
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
    world::WorldPortalExt,
};

use super::{BlockStateResolver, StructureTemplate};

/// A structure piece that places blocks from an NBT template.
///
/// This piece handles:
/// - Loading blocks from a `StructureTemplate`
/// - Applying rotation and mirror transformations
/// - Placing blocks at world coordinates
/// - Skipping structure void blocks
pub struct TemplatePiece {
    /// The underlying structure piece metadata.
    pub piece: StructurePiece,

    /// The template to place.
    pub template: Arc<StructureTemplate>,

    /// Rotation to apply when placing.
    pub rotation: Rotation,

    /// Mirror to apply when placing.
    pub mirror: Mirror,
}

impl TemplatePiece {
    /// Creates a new template piece from the given `template` at `origin`.
    ///
    /// The `rotation` and `mirror` transforms are applied when placing blocks.
    /// The `piece_type` identifies this piece within the structure system.
    #[must_use]
    pub fn new(
        template: Arc<StructureTemplate>,
        rotation: Rotation,
        mirror: Mirror,
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
        rotation: Rotation,
        mirror: Mirror,
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
    fn place_blocks(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        let box_limit = self.piece.bounding_box;

        for block in &self.template.blocks {
            let palette_entry = &self.template.palette[block.state as usize];

            // Skip structure void blocks
            if Self::is_structure_void(&palette_entry.name) {
                continue;
            }

            // Resolve the block state with rotation/mirror
            let Some(state) =
                BlockStateResolver::resolve(palette_entry, self.rotation, self.mirror)
            else {
                debug!("Failed to resolve block: {}", palette_entry.name);
                continue;
            };

            // Transform position to world coordinates
            let world_pos = self.transform_pos(block.pos);

            // Check bounds against both the piece bounding box and the current chunk box
            if !box_limit.contains_pos(&world_pos) || !chunk_box.contains_pos(&world_pos) {
                continue;
            }

            // Place the block
            chunk.set_block_state(world_pos.x, world_pos.y, world_pos.z, state);

            let block_entity_id = super::get_block_entity_id(&palette_entry.name);
            if block.nbt.is_some() || block_entity_id.is_some() {
                let block_entity_id = block_entity_id.unwrap_or(&palette_entry.name);
                let mut placed_nbt = pumpkin_nbt::compound::NbtCompound::new();

                placed_nbt.put_string("id", block_entity_id.to_string());
                placed_nbt.put_int("x", world_pos.x);
                placed_nbt.put_int("y", world_pos.y);
                placed_nbt.put_int("z", world_pos.z);

                if let Some(template_nbt) = &block.nbt {
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
                    use pumpkin_util::random::{
                        RandomImpl, hash_block_pos, legacy_rand::LegacyRand,
                    };
                    let mut random =
                        LegacyRand::from_seed(
                            hash_block_pos(world_pos.x, world_pos.y, world_pos.z) as u64,
                        );
                    placed_nbt.put_long("LootTableSeed", random.next_i64());
                }

                chunk.add_block_entity(placed_nbt);
            }
        }
    }
}

impl StructurePieceBase for TemplatePiece {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        self.place_blocks(chunk, chunk_box);
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }
}
