//! Template-based structure piece matching vanilla Minecraft `TemplateStructurePiece`.
//!
//! This module provides `TemplatePiece`, which implements `StructurePieceBase`
//! to place blocks from a loaded structure template into the world.

use std::sync::Arc;

use pumpkin_data::{Mirror, Rotation};
use pumpkin_util::{
    math::{block_box::BlockBox, vector3::Vector3},
    random::RandomGenerator,
};

use crate::{
    ProtoChunk,
    generation::structure::{
        piece::StructurePieceType,
        structures::{StructurePiece, StructurePieceBase},
    },
    world::WorldPortalExt,
};

use super::{BlockStateResolver, PaletteEntry, StructurePlaceSettings, StructureTemplate};

/// A structure piece that places blocks from an NBT template matching vanilla `TemplateStructurePiece`.
///
/// This piece handles:
/// - Loading blocks from a `StructureTemplate`
/// - Applying rotation, mirror, and pivot transformations from `StructurePlaceSettings`
/// - Placing blocks at world coordinates
/// - Replacing jigsaw blocks with their `final_state`
/// - Skipping structure void blocks
pub struct TemplatePiece {
    /// The underlying structure piece metadata.
    pub piece: StructurePiece,

    /// The template identifier/name.
    pub template_name: String,

    /// The template to place.
    pub template: Arc<StructureTemplate>,

    /// Placement settings (rotation, mirror, pivot, processors, etc.).
    pub place_settings: StructurePlaceSettings,

    /// The origin position of the template in the world.
    pub template_position: Vector3<i32>,
}

impl TemplatePiece {
    /// Creates a new template piece from the given `template` at `origin` with legacy rotation & mirror.
    #[must_use]
    pub fn new(
        template: Arc<StructureTemplate>,
        rotation: Rotation,
        mirror: Mirror,
        origin: Vector3<i32>,
        piece_type: StructurePieceType,
    ) -> Self {
        let place_settings = StructurePlaceSettings::new()
            .set_rotation(rotation)
            .set_mirror(mirror);
        let bounding_box = template.get_bounding_box(&place_settings, origin);

        Self {
            piece: StructurePiece::new(piece_type, bounding_box, 0),
            template_name: String::new(),
            template,
            place_settings,
            template_position: origin,
        }
    }

    /// Creates a new template piece with full `StructurePlaceSettings` matching vanilla `TemplateStructurePiece`.
    #[must_use]
    pub fn from_settings(
        template: Arc<StructureTemplate>,
        template_name: String,
        place_settings: StructurePlaceSettings,
        origin: Vector3<i32>,
        piece_type: StructurePieceType,
    ) -> Self {
        let bounding_box = template.get_bounding_box(&place_settings, origin);

        Self {
            piece: StructurePiece::new(piece_type, bounding_box, 0),
            template_name,
            template,
            place_settings,
            template_position: origin,
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
        self.place_settings
            .get_rotation()
            .transform_size(self.template.size)
    }

    /// Gets the rotation configured in place settings.
    #[must_use]
    pub const fn get_rotation(&self) -> Rotation {
        self.place_settings.get_rotation()
    }

    /// Gets a reference to the loaded template.
    #[must_use]
    pub const fn template(&self) -> &Arc<StructureTemplate> {
        &self.template
    }

    /// Gets the world template position.
    #[must_use]
    pub const fn template_position(&self) -> Vector3<i32> {
        self.template_position
    }

    /// Gets a reference to the structure placement settings.
    #[must_use]
    pub const fn place_settings(&self) -> &StructurePlaceSettings {
        &self.place_settings
    }

    /// Gets a mutable reference to the structure placement settings.
    pub const fn place_settings_mut(&mut self) -> &mut StructurePlaceSettings {
        &mut self.place_settings
    }

    /// Transforms a template-relative position to world coordinates.
    #[must_use]
    pub fn transform_pos(&self, local_pos: Vector3<i32>) -> Vector3<i32> {
        StructureTemplate::transform_block_pos(
            local_pos,
            self.place_settings.get_mirror(),
            self.place_settings.get_rotation(),
            self.place_settings.get_rotation_pivot(),
        ) + self.template_position
    }

    /// Moves the template position and bounding box by `(dx, dy, dz)`.
    pub fn move_piece(&mut self, dx: i32, dy: i32, dz: i32) {
        self.piece.translate(dx, dy, dz);
        self.template_position += Vector3::new(dx, dy, dz);
    }

    /// Checks if a block name is structure void (should not be placed).
    fn is_structure_void(name: &str) -> bool {
        name == "minecraft:structure_void" || name == "structure_void"
    }

    /// Places all blocks from the template into the chunk matching vanilla `postProcess`.
    fn place_blocks(&mut self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        self.place_settings = self
            .place_settings
            .clone()
            .set_bounding_box(Some(*chunk_box));
        self.piece.bounding_box = self
            .template
            .get_bounding_box(&self.place_settings, self.template_position);
        let box_limit = self.piece.bounding_box;

        for block in &self.template.blocks {
            let palette_entry = &self.template.palette[block.state as usize];

            // Structure blocks are data markers
            if palette_entry.name == "minecraft:structure_block" {
                continue;
            }

            let mut placed_entry = palette_entry.clone();
            let mut block_entity_nbt = block.nbt.clone();

            // Jigsaw blocks are replaced with final_state
            if palette_entry.name == "minecraft:jigsaw" {
                let final_state = block_entity_nbt
                    .as_ref()
                    .and_then(|nbt| nbt.get_string("final_state"))
                    .unwrap_or("minecraft:air");
                placed_entry = PaletteEntry::from_string(final_state);
                block_entity_nbt = None;
            }

            // Skip structure void blocks
            if Self::is_structure_void(&placed_entry.name) {
                continue;
            }

            // Resolve the block state with rotation/mirror
            let Some(mut state) = BlockStateResolver::resolve(
                &placed_entry,
                self.place_settings.get_rotation(),
                self.place_settings.get_mirror(),
            ) else {
                continue;
            };

            // Transform position to world coordinates
            let world_pos = self.transform_pos(block.pos);

            // Check bounds against both the piece bounding box and the current chunk box
            if !box_limit.contains_pos(&world_pos) || !chunk_box.contains_pos(&world_pos) {
                continue;
            }

            // Handle waterlogging if enabled
            if self.place_settings.should_apply_waterlogging()
                && chunk.get_block_state(&world_pos).to_block_id() == pumpkin_data::Block::WATER.id
                && let Some((_, waterlogged)) = placed_entry
                    .properties
                    .iter_mut()
                    .find(|(name, _)| name == "waterlogged")
            {
                *waterlogged = "true".to_string();
                if let Some(waterlogged_state) = BlockStateResolver::resolve(
                    &placed_entry,
                    self.place_settings.get_rotation(),
                    self.place_settings.get_mirror(),
                ) {
                    state = waterlogged_state;
                }
            }

            // Place the block
            chunk.set_block_state(world_pos.x, world_pos.y, world_pos.z, state);

            let final_block = pumpkin_data::Block::from_id(state.id.to_block_id());
            let block_entity_id = super::get_block_entity_id(final_block.name);
            if block_entity_nbt.is_some() || block_entity_id.is_some() {
                let fallback_id = block_entity_id.unwrap_or(final_block.name);
                let mut placed_nbt = pumpkin_nbt::compound::NbtCompound::new();

                placed_nbt.put_string("id", fallback_id.to_string());
                placed_nbt.put_int("x", world_pos.x);
                placed_nbt.put_int("y", world_pos.y);
                placed_nbt.put_int("z", world_pos.z);

                if let Some(template_nbt) = &block_entity_nbt {
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

    fn translate(&mut self, x: i32, y: i32, z: i32) {
        self.move_piece(x, y, z);
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }
}
