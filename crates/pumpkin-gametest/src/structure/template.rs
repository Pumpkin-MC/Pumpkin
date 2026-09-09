use pumpkin_data::{Block, BlockStateId};
use pumpkin_nbt::NbtCompound;

use crate::error::{GameTestError, GameTestResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TestBlockMode {
    Start,
    Log,
    Fail,
    Accept,
}

impl TestBlockMode {
    #[must_use]
    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Log => "log",
            Self::Fail => "fail",
            Self::Accept => "accept",
        }
    }

    #[must_use]
    pub const fn from_serialized_name(name: &str) -> Option<Self> {
        match name.as_bytes() {
            b"start" => Some(Self::Start),
            b"log" => Some(Self::Log),
            b"fail" => Some(Self::Fail),
            b"accept" => Some(Self::Accept),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameTestStructureBlock {
    pub position: [i32; 3],
    pub state: BlockStateId,
    pub nbt: Option<NbtCompound>,
    pub test_mode: Option<TestBlockMode>,
}

#[derive(Clone, Debug)]
pub struct GameTestStructureTemplate {
    size: [i32; 3],
    blocks: Vec<GameTestStructureBlock>,
}

#[derive(Clone, Copy, Debug)]
struct PaletteEntry {
    state: BlockStateId,
    test_mode: Option<TestBlockMode>,
}

impl GameTestStructureTemplate {
    pub fn from_nbt(structure: &NbtCompound) -> GameTestResult<Self> {
        let size = read_vec3(structure, "size")?;
        if size.iter().any(|axis| *axis <= 0) {
            return Err(invalid_structure(format!(
                "Structure has invalid size {size:?}"
            )));
        }

        let palette = resolve_palette(structure)?;
        let blocks = structure
            .get_list("blocks")
            .ok_or_else(|| invalid_structure("Structure is missing 'blocks'"))?;
        let mut parsed_blocks = Vec::with_capacity(blocks.len());

        // Validate the complete structure before changing the world so malformed NBT
        // cannot leave a half-placed test behind.
        for (index, block) in blocks.iter().enumerate() {
            let block = block.extract_compound().ok_or_else(|| {
                invalid_structure(format!("Structure block {index} is not a compound"))
            })?;
            let position = read_vec3(block, "pos")?;
            if position[0] < 0
                || position[1] < 0
                || position[2] < 0
                || position[0] >= size[0]
                || position[1] >= size[1]
                || position[2] >= size[2]
            {
                return Err(invalid_structure(format!(
                    "Structure block {index} position {position:?} is outside size {size:?}"
                )));
            }

            let state_index = block.get_int("state").ok_or_else(|| {
                invalid_structure(format!(
                    "Structure block {index} is missing integer 'state'"
                ))
            })?;
            let state_index = usize::try_from(state_index).map_err(|_| {
                invalid_structure(format!("Structure block {index} has negative state index"))
            })?;
            let palette_entry = palette.get(state_index).copied().ok_or_else(|| {
                invalid_structure(format!(
                    "Structure block {index} references missing palette state {state_index}"
                ))
            })?;

            parsed_blocks.push(GameTestStructureBlock {
                position,
                state: palette_entry.state,
                nbt: block.get_compound("nbt").cloned(),
                test_mode: palette_entry.test_mode,
            });
        }

        Ok(Self {
            size,
            blocks: parsed_blocks,
        })
    }

    #[must_use]
    pub const fn size(&self) -> [i32; 3] {
        self.size
    }

    #[must_use]
    pub fn blocks(&self) -> &[GameTestStructureBlock] {
        &self.blocks
    }

    #[must_use]
    pub const fn block_count(&self) -> usize {
        self.blocks.len()
    }
}

fn read_vec3(compound: &NbtCompound, name: &str) -> GameTestResult<[i32; 3]> {
    let values = compound
        .get_list(name)
        .ok_or_else(|| invalid_structure(format!("Structure is missing '{name}' int list")))?;
    let [x, y, z] = values else {
        return Err(invalid_structure(format!(
            "Structure '{name}' must contain exactly three integers"
        )));
    };

    Ok([
        x.extract_int().ok_or_else(|| {
            invalid_structure(format!("Structure '{name}' contains a non-integer value"))
        })?,
        y.extract_int().ok_or_else(|| {
            invalid_structure(format!("Structure '{name}' contains a non-integer value"))
        })?,
        z.extract_int().ok_or_else(|| {
            invalid_structure(format!("Structure '{name}' contains a non-integer value"))
        })?,
    ])
}

fn resolve_palette(structure: &NbtCompound) -> GameTestResult<Vec<PaletteEntry>> {
    let palette = structure
        .get_list("palette")
        .ok_or_else(|| invalid_structure("Structure is missing 'palette'"))?;
    let mut states = Vec::with_capacity(palette.len());

    for (index, entry) in palette.iter().enumerate() {
        let entry = entry
            .extract_compound()
            .ok_or_else(|| invalid_structure(format!("Palette entry {index} is not a compound")))?;
        let name = entry
            .get_string("Name")
            .ok_or_else(|| invalid_structure(format!("Palette entry {index} is missing 'Name'")))?;
        let block = Block::from_name(name).ok_or_else(|| {
            invalid_structure(format!("Unknown block '{name}' in structure palette"))
        })?;

        let mut test_mode = None;
        let state = if let Some(properties) = entry.get_compound("Properties") {
            let mut property_pairs = Vec::with_capacity(properties.child_tags.len());
            for (property_name, property_value) in &properties.child_tags {
                let property_value = property_value.extract_string().ok_or_else(|| {
                    invalid_structure(format!(
                        "Block '{name}' property '{property_name}' in palette entry {index} is not a string"
                    ))
                })?;
                if block == &Block::TEST_BLOCK && property_name.as_ref() == "mode" {
                    test_mode = Some(TestBlockMode::from_serialized_name(property_value).ok_or_else(
                        || {
                            invalid_structure(format!(
                                "Unknown test block mode '{property_value}' in palette entry {index}"
                            ))
                        },
                    )?);
                }
                property_pairs.push((property_name.as_ref(), property_value));
            }

            block
                .state_from_properties(&property_pairs)
                .ok_or_else(|| {
                    invalid_structure(format!(
                        "No Pumpkin block state matches palette entry {index} for '{name}'"
                    ))
                })?
        } else {
            block.default_state
        };

        if block == &Block::TEST_BLOCK && test_mode.is_none() {
            // TestBlockMode.START is the first/default enum value in the 26.2 server source.
            test_mode = Some(TestBlockMode::Start);
        }

        states.push(PaletteEntry {
            state: state.id,
            test_mode,
        });
    }

    Ok(states)
}

fn invalid_structure(message: impl Into<String>) -> GameTestError {
    GameTestError::InvalidStructure(message.into())
}
