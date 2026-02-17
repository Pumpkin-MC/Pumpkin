//! Structure template loading from NBT files.
//!
//! This module handles parsing vanilla Minecraft structure template files (.nbt)
//! into a runtime representation that can be placed in the world.

use std::io::Cursor;

use pumpkin_nbt::{compound::NbtCompound, nbt_compress::read_gzip_compound_tag, tag::NbtTag};
use pumpkin_util::math::vector3::Vector3;
use thiserror::Error;

/// Errors that can occur when loading a structure template.
#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Failed to decompress NBT: {0}")]
    NbtError(#[from] pumpkin_nbt::Error),

    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    #[error("Invalid field type for {0}")]
    InvalidFieldType(&'static str),

    #[error("Invalid palette index: {0}")]
    InvalidPaletteIndex(u32),
}

/// A loaded structure template from an NBT file.
///
/// Structure templates contain a palette of block states and a list of blocks
/// referencing that palette. This allows for efficient storage and the ability
/// to transform (rotate/mirror) the template when placing.
#[derive(Debug, Clone)]
pub struct StructureTemplate {
    /// The dimensions of the template (width, height, depth).
    pub size: Vector3<i32>,

    /// The palette of block states used in this template.
    pub palette: Vec<PaletteEntry>,

    /// The blocks in this template, referencing palette indices.
    pub blocks: Vec<TemplateBlock>,

    /// Entities to spawn when placing this template (future use).
    pub entities: Vec<TemplateEntity>,
}

/// A single entry in the template's block palette.
///
/// Each palette entry defines a block type with optional properties.
#[derive(Debug, Clone)]
pub struct PaletteEntry {
    /// The block name (e.g., "minecraft:stone_bricks").
    pub name: String,

    /// Block state properties (e.g., [("facing", "north"), ("lit", "false")]).
    pub properties: Vec<(String, String)>,
}

impl PaletteEntry {
    /// Creates a new palette entry with no properties.
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            properties: Vec::new(),
        }
    }

    /// Creates a new palette entry with the given properties.
    #[must_use]
    pub fn with_properties(name: String, properties: Vec<(String, String)>) -> Self {
        Self { name, properties }
    }
}

/// A single block placement in the template.
#[derive(Debug, Clone)]
pub struct TemplateBlock {
    /// Position relative to the template origin.
    pub pos: Vector3<i32>,

    /// Index into the template's palette.
    pub state: u32,

    /// Optional NBT data for block entities (chests, signs, etc.).
    pub nbt: Option<NbtCompound>,
}

/// An entity to spawn when placing the template.
#[derive(Debug, Clone)]
pub struct TemplateEntity {
    /// Position relative to the template origin.
    pub pos: Vector3<f64>,

    /// Block position (snapped to grid).
    pub block_pos: Vector3<i32>,

    /// Entity NBT data.
    pub nbt: NbtCompound,
}

impl StructureTemplate {
    /// Loads a structure template from gzipped NBT bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the NBT data is invalid or missing required fields.
    pub fn from_nbt_bytes(bytes: &[u8]) -> Result<Self, TemplateError> {
        let compound = read_gzip_compound_tag(Cursor::new(bytes))?;
        Self::from_nbt_compound(&compound)
    }

    /// Loads a structure template from a parsed NBT compound.
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing or have invalid types.
    pub fn from_nbt_compound(compound: &NbtCompound) -> Result<Self, TemplateError> {
        // Parse size
        let size = Self::parse_size(compound)?;

        // Parse palette
        let palette = Self::parse_palette(compound)?;

        // Parse blocks
        let blocks = Self::parse_blocks(compound, palette.len())?;

        // Parse entities (optional)
        let entities = Self::parse_entities(compound).unwrap_or_default();

        Ok(Self {
            size,
            palette,
            blocks,
            entities,
        })
    }

    fn parse_size(compound: &NbtCompound) -> Result<Vector3<i32>, TemplateError> {
        let size_list = compound
            .get_list("size")
            .ok_or(TemplateError::MissingField("size"))?;

        if size_list.len() != 3 {
            return Err(TemplateError::InvalidFieldType("size"));
        }

        let x = Self::extract_int(&size_list[0]).ok_or(TemplateError::InvalidFieldType("size"))?;
        let y = Self::extract_int(&size_list[1]).ok_or(TemplateError::InvalidFieldType("size"))?;
        let z = Self::extract_int(&size_list[2]).ok_or(TemplateError::InvalidFieldType("size"))?;

        Ok(Vector3::new(x, y, z))
    }

    fn parse_palette(compound: &NbtCompound) -> Result<Vec<PaletteEntry>, TemplateError> {
        let palette_list = compound
            .get_list("palette")
            .ok_or(TemplateError::MissingField("palette"))?;

        let mut palette = Vec::with_capacity(palette_list.len());

        for tag in palette_list {
            let entry_compound = match tag {
                NbtTag::Compound(c) => c,
                _ => return Err(TemplateError::InvalidFieldType("palette entry")),
            };

            let name = entry_compound
                .get_string("Name")
                .ok_or(TemplateError::MissingField("palette.Name"))?
                .to_string();

            let properties = if let Some(props_compound) = entry_compound.get_compound("Properties")
            {
                props_compound
                    .child_tags
                    .iter()
                    .filter_map(|(key, value)| {
                        if let NbtTag::String(v) = value {
                            Some((key.clone(), v.clone()))
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

            palette.push(PaletteEntry { name, properties });
        }

        Ok(palette)
    }

    fn parse_blocks(
        compound: &NbtCompound,
        palette_size: usize,
    ) -> Result<Vec<TemplateBlock>, TemplateError> {
        let blocks_list = compound
            .get_list("blocks")
            .ok_or(TemplateError::MissingField("blocks"))?;

        let mut blocks = Vec::with_capacity(blocks_list.len());

        for tag in blocks_list {
            let block_compound = match tag {
                NbtTag::Compound(c) => c,
                _ => return Err(TemplateError::InvalidFieldType("blocks entry")),
            };

            // Parse position
            let pos_list = block_compound
                .get_list("pos")
                .ok_or(TemplateError::MissingField("blocks.pos"))?;

            if pos_list.len() != 3 {
                return Err(TemplateError::InvalidFieldType("blocks.pos"));
            }

            let x =
                Self::extract_int(&pos_list[0]).ok_or(TemplateError::InvalidFieldType("pos"))?;
            let y =
                Self::extract_int(&pos_list[1]).ok_or(TemplateError::InvalidFieldType("pos"))?;
            let z =
                Self::extract_int(&pos_list[2]).ok_or(TemplateError::InvalidFieldType("pos"))?;

            // Parse state (palette index)
            let state = block_compound
                .get_int("state")
                .ok_or(TemplateError::MissingField("blocks.state"))?
                as u32;

            if state as usize >= palette_size {
                return Err(TemplateError::InvalidPaletteIndex(state));
            }

            // Parse optional NBT
            let nbt = block_compound.get_compound("nbt").cloned();

            blocks.push(TemplateBlock {
                pos: Vector3::new(x, y, z),
                state,
                nbt,
            });
        }

        Ok(blocks)
    }

    fn parse_entities(compound: &NbtCompound) -> Result<Vec<TemplateEntity>, TemplateError> {
        let entities_list = match compound.get_list("entities") {
            Some(list) => list,
            None => return Ok(Vec::new()),
        };

        let mut entities = Vec::with_capacity(entities_list.len());

        for tag in entities_list {
            let entity_compound = match tag {
                NbtTag::Compound(c) => c,
                _ => return Err(TemplateError::InvalidFieldType("entities entry")),
            };

            // Parse position (double coordinates)
            let pos_list = entity_compound
                .get_list("pos")
                .ok_or(TemplateError::MissingField("entities.pos"))?;

            if pos_list.len() != 3 {
                return Err(TemplateError::InvalidFieldType("entities.pos"));
            }

            let x = Self::extract_double(&pos_list[0])
                .ok_or(TemplateError::InvalidFieldType("entities.pos"))?;
            let y = Self::extract_double(&pos_list[1])
                .ok_or(TemplateError::InvalidFieldType("entities.pos"))?;
            let z = Self::extract_double(&pos_list[2])
                .ok_or(TemplateError::InvalidFieldType("entities.pos"))?;

            // Parse block position
            let block_pos_list = entity_compound
                .get_list("blockPos")
                .ok_or(TemplateError::MissingField("entities.blockPos"))?;

            if block_pos_list.len() != 3 {
                return Err(TemplateError::InvalidFieldType("entities.blockPos"));
            }

            let bx = Self::extract_int(&block_pos_list[0])
                .ok_or(TemplateError::InvalidFieldType("entities.blockPos"))?;
            let by = Self::extract_int(&block_pos_list[1])
                .ok_or(TemplateError::InvalidFieldType("entities.blockPos"))?;
            let bz = Self::extract_int(&block_pos_list[2])
                .ok_or(TemplateError::InvalidFieldType("entities.blockPos"))?;

            // Parse NBT
            let nbt = entity_compound
                .get_compound("nbt")
                .cloned()
                .ok_or(TemplateError::MissingField("entities.nbt"))?;

            entities.push(TemplateEntity {
                pos: Vector3::new(x, y, z),
                block_pos: Vector3::new(bx, by, bz),
                nbt,
            });
        }

        Ok(entities)
    }

    /// Helper to extract an i32 from various NBT numeric types.
    fn extract_int(tag: &NbtTag) -> Option<i32> {
        match tag {
            NbtTag::Byte(v) => Some(i32::from(*v)),
            NbtTag::Short(v) => Some(i32::from(*v)),
            NbtTag::Int(v) => Some(*v),
            NbtTag::Long(v) => Some(*v as i32),
            _ => None,
        }
    }

    /// Helper to extract an f64 from various NBT numeric types.
    fn extract_double(tag: &NbtTag) -> Option<f64> {
        match tag {
            NbtTag::Float(v) => Some(f64::from(*v)),
            NbtTag::Double(v) => Some(*v),
            NbtTag::Int(v) => Some(f64::from(*v)),
            NbtTag::Long(v) => Some(*v as f64),
            _ => None,
        }
    }

    /// Returns the total number of blocks in this template.
    #[must_use]
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Returns whether this template has any block entity data.
    #[must_use]
    pub fn has_block_entities(&self) -> bool {
        self.blocks.iter().any(|b| b.nbt.is_some())
    }

    /// Returns whether this template has any entities.
    #[must_use]
    pub fn has_entities(&self) -> bool {
        !self.entities.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_entry_creation() {
        let entry = PaletteEntry::new("minecraft:stone".to_string());
        assert_eq!(entry.name, "minecraft:stone");
        assert!(entry.properties.is_empty());

        let entry_with_props = PaletteEntry::with_properties(
            "minecraft:oak_stairs".to_string(),
            vec![
                ("facing".to_string(), "north".to_string()),
                ("half".to_string(), "bottom".to_string()),
            ],
        );
        assert_eq!(entry_with_props.properties.len(), 2);
    }
}
