//! Structure template loading from NBT files.
//!
//! This module handles parsing vanilla Minecraft structure template files (.nbt)
//! into a runtime representation that can be placed in the world.

use std::io::Cursor;
use std::path::Path;

use pumpkin_nbt::{
    compound::NbtCompound,
    nbt_compress::{
        read_gzip_compound_tag, write_gzip_compound_tag, write_gzip_compound_tag_to_bytes,
    },
    tag::NbtTag,
};
use pumpkin_util::math::vector3::Vector3;
use thiserror::Error;

/// Current world data version, written into saved structure NBT (`NbtUtils.addCurrentDataVersion`).
const STRUCTURE_DATA_VERSION: i32 = crate::chunk::format::anvil::WORLD_DATA_VERSION;

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

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
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
    /// The block name (e.g., "`minecraft:stone_bricks`").
    pub name: String,

    /// Block state properties (e.g., [("facing", "north"), ("lit", "false")]).
    pub properties: Vec<(String, String)>,
}

impl PaletteEntry {
    /// Creates a new palette entry with no properties.
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self {
            name,
            properties: Vec::new(),
        }
    }

    /// Creates a new palette entry with the given properties.
    #[must_use]
    pub const fn with_properties(name: String, properties: Vec<(String, String)>) -> Self {
        Self { name, properties }
    }

    /// Parses a block state string (e.g. "`minecraft:oak_log`[axis=x]") into a palette entry.
    #[must_use]
    pub fn from_string(s: &str) -> Self {
        s.find('[').map_or_else(
            || Self::new(s.to_string()),
            |bracket_pos| {
                let name = s[..bracket_pos].to_string();
                let end_bracket = s.rfind(']').unwrap_or(s.len());
                let props_str = &s[bracket_pos + 1..end_bracket];
                let properties = props_str
                    .split(',')
                    .filter_map(|p| {
                        let mut parts = p.split('=');
                        let key = parts.next()?.trim().to_string();
                        let value = parts.next()?.trim().to_string();
                        Some((key, value))
                    })
                    .collect();
                Self { name, properties }
            },
        )
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

/// A single block read from the world by a capture routine, before palette assignment.
///
/// Mirrors vanilla's `StructureTemplate.StructureBlockInfo` prior to `SimplePalette` interning.
#[derive(Debug, Clone)]
pub struct CapturedBlock {
    /// Position relative to the capture region's minimum corner.
    pub pos: Vector3<i32>,

    /// The block's name and properties (pre-palette-dedup).
    pub palette_entry: PaletteEntry,

    /// Optional block-entity NBT (`id` + fields, no position).
    pub nbt: Option<NbtCompound>,

    /// Whether this block occupies a full unit cube (used only to mirror vanilla's
    /// full/other/block-entity ordering in the saved block list; does not affect placement).
    pub full_cube: bool,
}

impl StructureTemplate {
    /// Builds a template from blocks captured out of a live world (`StructureTemplate.fillFromWorld`
    /// equivalent), interning identical blocks into a shared palette the same way vanilla's
    /// `SimplePalette.idFor` does.
    ///
    /// `blocks` need not be pre-sorted or deduplicated. Ordering mirrors vanilla's
    /// `buildInfoList`: blocks with no NBT and a full collision cube first, other plain blocks
    /// second, blocks carrying NBT last, each group sorted by (y, x, z). Vanilla additionally
    /// excludes blocks with a "dynamic shape" from the full-cube group; this codebase has no
    /// equivalent check, so `full_cube` alone decides the grouping.
    #[must_use]
    pub fn from_captured(
        size: Vector3<i32>,
        mut blocks: Vec<CapturedBlock>,
        entities: Vec<TemplateEntity>,
    ) -> Self {
        blocks.sort_by_key(|b| (b.pos.y, b.pos.x, b.pos.z));

        let (mut with_nbt, without_nbt): (Vec<_>, Vec<_>) =
            blocks.into_iter().partition(|b| b.nbt.is_some());
        let (mut full, mut other): (Vec<_>, Vec<_>) =
            without_nbt.into_iter().partition(|b| b.full_cube);

        let mut ordered = Vec::with_capacity(full.len() + other.len() + with_nbt.len());
        ordered.append(&mut full);
        ordered.append(&mut other);
        ordered.append(&mut with_nbt);

        let mut palette: Vec<PaletteEntry> = Vec::new();
        let mut find_index = |entry: &PaletteEntry| -> u32 {
            palette
                .iter()
                .position(|existing| {
                    existing.name == entry.name && existing.properties == entry.properties
                })
                .map_or_else(
                    || {
                        palette.push(entry.clone());
                        (palette.len() - 1) as u32
                    },
                    |idx| idx as u32,
                )
        };

        let blocks = ordered
            .into_iter()
            .map(|captured| TemplateBlock {
                pos: captured.pos,
                state: find_index(&captured.palette_entry),
                nbt: captured.nbt,
            })
            .collect();

        Self {
            size,
            palette,
            blocks,
            entities,
        }
    }

    /// Serializes this template to an NBT compound matching vanilla's on-disk structure format
    /// (`StructureTemplate.save`). The inverse of `from_nbt_compound`.
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut root = NbtCompound::new();

        let palette_list = self
            .palette
            .iter()
            .map(|entry| {
                let mut entry_compound = NbtCompound::new();
                entry_compound.put_string("Name", entry.name.clone());
                if !entry.properties.is_empty() {
                    let mut props = NbtCompound::new();
                    for (key, value) in &entry.properties {
                        props.put_string(key, value.clone());
                    }
                    entry_compound.put_compound("Properties", props);
                }
                NbtTag::Compound(entry_compound)
            })
            .collect();
        root.put("palette", NbtTag::List(palette_list));

        let blocks_list = self
            .blocks
            .iter()
            .map(|block| {
                let mut block_compound = NbtCompound::new();
                block_compound.put(
                    "pos",
                    NbtTag::List(vec![
                        NbtTag::Int(block.pos.x),
                        NbtTag::Int(block.pos.y),
                        NbtTag::Int(block.pos.z),
                    ]),
                );
                block_compound.put_int("state", block.state as i32);
                if let Some(nbt) = &block.nbt {
                    block_compound.put_compound("nbt", nbt.clone());
                }
                NbtTag::Compound(block_compound)
            })
            .collect();
        root.put("blocks", NbtTag::List(blocks_list));

        let entities_list = self
            .entities
            .iter()
            .map(|entity| {
                let mut entity_compound = NbtCompound::new();
                entity_compound.put(
                    "pos",
                    NbtTag::List(vec![
                        entity.pos.x.into(),
                        entity.pos.y.into(),
                        entity.pos.z.into(),
                    ]),
                );
                entity_compound.put(
                    "blockPos",
                    NbtTag::List(vec![
                        NbtTag::Int(entity.block_pos.x),
                        NbtTag::Int(entity.block_pos.y),
                        NbtTag::Int(entity.block_pos.z),
                    ]),
                );
                entity_compound.put_compound("nbt", entity.nbt.clone());
                NbtTag::Compound(entity_compound)
            })
            .collect();
        root.put("entities", NbtTag::List(entities_list));

        root.put(
            "size",
            NbtTag::List(vec![
                NbtTag::Int(self.size.x),
                NbtTag::Int(self.size.y),
                NbtTag::Int(self.size.z),
            ]),
        );
        root.put_int("DataVersion", STRUCTURE_DATA_VERSION);

        root
    }

    /// Serializes this template to gzip-compressed NBT bytes, the exact on-disk format of a
    /// vanilla `.nbt` structure file.
    ///
    /// # Errors
    ///
    /// Returns an error if NBT serialization fails.
    pub fn to_nbt_bytes(&self) -> Result<Vec<u8>, TemplateError> {
        Ok(write_gzip_compound_tag_to_bytes(self.to_nbt_compound())?)
    }

    /// Writes this template to `path` as gzip-compressed NBT, creating parent directories as
    /// needed. Mirrors `StructureTemplateManager.save(Path, StructureTemplate, boolean)`
    /// (the non-text branch).
    ///
    /// # Errors
    ///
    /// Returns an error if a parent directory can't be created, the file can't be opened for
    /// writing, or serialization fails.
    pub fn save_to_path(&self, path: &Path) -> Result<(), TemplateError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(path)?;
        write_gzip_compound_tag(self.to_nbt_compound(), file)?;
        Ok(())
    }

    /// Loads a structure template from a gzip-compressed NBT file on disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the file can't be opened or the NBT data is invalid.
    pub fn load_from_path(path: &Path) -> Result<Self, TemplateError> {
        let file = std::fs::File::open(path)?;
        let compound = read_gzip_compound_tag(file)?;
        Self::from_nbt_compound(&compound)
    }

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
        let palette_list = if let Some(list) = compound.get_list("palette") {
            list
        } else if let Some(palettes) = compound.get_list("palettes") {
            if let Some(NbtTag::List(list)) = palettes.first() {
                list
            } else {
                return Err(TemplateError::MissingField("palette"));
            }
        } else {
            return Err(TemplateError::MissingField("palette"));
        };

        let mut palette = Vec::with_capacity(palette_list.len());

        for tag in palette_list {
            let NbtTag::Compound(entry_compound) = tag else {
                return Err(TemplateError::InvalidFieldType("palette entry"));
            };

            let name = entry_compound
                .get_string("Name")
                .ok_or(TemplateError::MissingField("palette.Name"))?
                .to_string();

            let properties =
                entry_compound
                    .get_compound("Properties")
                    .map_or_else(Vec::new, |props_compound| {
                        props_compound
                            .child_tags
                            .iter()
                            .filter_map(|(key, value)| {
                                if let NbtTag::String(v) = value {
                                    Some((key.to_string(), v.to_string()))
                                } else {
                                    None
                                }
                            })
                            .collect()
                    });

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
            let NbtTag::Compound(block_compound) = tag else {
                return Err(TemplateError::InvalidFieldType("blocks entry"));
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
                .ok_or(TemplateError::MissingField("blocks.state"))? as u32;

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
        let Some(entities_list) = compound.get_list("entities") else {
            return Ok(Vec::new());
        };

        let mut entities = Vec::with_capacity(entities_list.len());

        for tag in entities_list {
            let NbtTag::Compound(entity_compound) = tag else {
                return Err(TemplateError::InvalidFieldType("entities entry"));
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
    pub const fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Returns whether this template has any block entity data.
    #[must_use]
    pub fn has_block_entities(&self) -> bool {
        self.blocks.iter().any(|b| b.nbt.is_some())
    }

    /// Returns whether this template has any entities.
    #[must_use]
    pub const fn has_entities(&self) -> bool {
        !self.entities.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_entry_creation() {
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

    #[test]
    fn from_captured_interns_duplicate_palette_entries() {
        let stone = PaletteEntry::new("minecraft:stone".to_string());
        let blocks = vec![
            CapturedBlock {
                pos: Vector3::new(0, 0, 0),
                palette_entry: stone.clone(),
                nbt: None,
                full_cube: true,
            },
            CapturedBlock {
                pos: Vector3::new(1, 0, 0),
                palette_entry: stone,
                nbt: None,
                full_cube: true,
            },
        ];
        let template = StructureTemplate::from_captured(Vector3::new(2, 1, 1), blocks, Vec::new());
        assert_eq!(template.palette.len(), 1);
        assert_eq!(template.blocks.len(), 2);
        assert_eq!(template.blocks[0].state, template.blocks[1].state);
    }

    #[test]
    fn from_captured_orders_block_entities_last() {
        let mut nbt = NbtCompound::new();
        nbt.put_string("id", "minecraft:chest".to_string());
        let blocks = vec![
            CapturedBlock {
                pos: Vector3::new(0, 0, 0),
                palette_entry: PaletteEntry::new("minecraft:chest".to_string()),
                nbt: Some(nbt),
                full_cube: false,
            },
            CapturedBlock {
                pos: Vector3::new(0, 0, 0),
                palette_entry: PaletteEntry::new("minecraft:stone".to_string()),
                nbt: None,
                full_cube: true,
            },
            CapturedBlock {
                pos: Vector3::new(0, 0, 0),
                palette_entry: PaletteEntry::new("minecraft:oak_stairs".to_string()),
                nbt: None,
                full_cube: false,
            },
        ];
        let template = StructureTemplate::from_captured(Vector3::new(1, 1, 1), blocks, Vec::new());
        assert_eq!(
            template.palette[template.blocks[0].state as usize].name,
            "minecraft:stone"
        );
        assert_eq!(
            template.palette[template.blocks[1].state as usize].name,
            "minecraft:oak_stairs"
        );
        assert_eq!(
            template.palette[template.blocks[2].state as usize].name,
            "minecraft:chest"
        );
        assert!(template.blocks[2].nbt.is_some());
    }

    #[test]
    fn nbt_roundtrip_preserves_blocks_palette_entities_and_size() {
        let mut chest_nbt = NbtCompound::new();
        chest_nbt.put_string("id", "minecraft:chest".to_string());

        let blocks = vec![
            CapturedBlock {
                pos: Vector3::new(0, 0, 0),
                palette_entry: PaletteEntry::with_properties(
                    "minecraft:oak_stairs".to_string(),
                    vec![("facing".to_string(), "north".to_string())],
                ),
                nbt: None,
                full_cube: false,
            },
            CapturedBlock {
                pos: Vector3::new(1, 0, 0),
                palette_entry: PaletteEntry::new("minecraft:chest".to_string()),
                nbt: Some(chest_nbt),
                full_cube: false,
            },
        ];
        let entities = vec![TemplateEntity {
            pos: Vector3::new(0.5, 0.0, 0.5),
            block_pos: Vector3::new(0, 0, 0),
            nbt: {
                let mut nbt = NbtCompound::new();
                nbt.put_string("id", "minecraft:zombie".to_string());
                nbt
            },
        }];

        let template = StructureTemplate::from_captured(Vector3::new(2, 1, 1), blocks, entities);
        let bytes = template.to_nbt_bytes().expect("serialization failed");
        let loaded = StructureTemplate::from_nbt_bytes(&bytes).expect("deserialization failed");

        assert_eq!(loaded.size, Vector3::new(2, 1, 1));
        assert_eq!(loaded.blocks.len(), 2);
        assert_eq!(loaded.entities.len(), 1);
        assert_eq!(
            loaded.entities[0].nbt.get_string("id"),
            Some("minecraft:zombie")
        );

        let stairs_block = loaded
            .blocks
            .iter()
            .find(|b| b.pos == Vector3::new(0, 0, 0))
            .unwrap();
        let stairs_entry = &loaded.palette[stairs_block.state as usize];
        assert_eq!(stairs_entry.name, "minecraft:oak_stairs");
        assert_eq!(
            stairs_entry.properties,
            vec![("facing".to_string(), "north".to_string())]
        );

        let chest_block = loaded
            .blocks
            .iter()
            .find(|b| b.pos == Vector3::new(1, 0, 0))
            .unwrap();
        assert!(chest_block.nbt.is_some());
        assert_eq!(
            chest_block.nbt.as_ref().unwrap().get_string("id"),
            Some("minecraft:chest")
        );
    }

    #[test]
    fn filesystem_roundtrip_preserves_template() {
        let blocks = vec![CapturedBlock {
            pos: Vector3::new(0, 0, 0),
            palette_entry: PaletteEntry::new("minecraft:stone".to_string()),
            nbt: None,
            full_cube: true,
        }];
        let template = StructureTemplate::from_captured(Vector3::new(1, 1, 1), blocks, Vec::new());

        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir
            .path()
            .join("generated")
            .join("minecraft")
            .join("structure")
            .join("my_house.nbt");

        template.save_to_path(&path).expect("save_to_path failed");
        assert!(path.exists());

        let loaded = StructureTemplate::load_from_path(&path).expect("load_from_path failed");
        assert_eq!(loaded.size, template.size);
        assert_eq!(loaded.blocks.len(), template.blocks.len());
        assert_eq!(loaded.palette[0].name, "minecraft:stone");
    }
}
