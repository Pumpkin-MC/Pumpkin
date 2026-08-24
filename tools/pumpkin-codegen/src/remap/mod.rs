use proc_macro2::TokenStream;
use pumpkin_nbt::compound::NbtCompound;

use crate::version::JavaMinecraftVersion;

mod argument_type;
mod attribute;
mod block_entity_type;
mod block_state;
mod custom_stat;
mod data_component_type;
mod enchantment;
mod entity_id;
mod environment_attribute;
mod item_id;
mod menu_id;
mod painting_variant;
mod particle_id;
mod recipe_serializer;
mod slot_display;
mod sound_id;

/// Returns the list of remap builder functions paired with their output file names.
#[allow(clippy::type_complexity)]
pub fn build() -> Vec<(fn() -> TokenStream, &'static str)> {
    vec![
        (argument_type::build, "argument_type_id_remap.rs"),
        (attribute::build, "attribute_id_remap.rs"),
        (block_entity_type::build, "block_entity_type_id_remap.rs"),
        (block_state::build, "block_state_remap.rs"),
        (custom_stat::build, "custom_stat_id_remap.rs"),
        (
            data_component_type::build,
            "data_component_type_id_remap.rs",
        ),
        (enchantment::build, "enchantment_id_remap.rs"),
        (entity_id::build, "entity_id_remap.rs"),
        (
            environment_attribute::build,
            "environment_attribute_id_remap.rs",
        ),
        (item_id::build, "item_id_remap.rs"),
        (menu_id::build, "menu_id_remap.rs"),
        (painting_variant::build, "painting_variant_id_remap.rs"),
        (particle_id::build, "particle_id_remap.rs"),
        (recipe_serializer::build, "recipe_serializer_id_remap.rs"),
        (slot_display::build, "slot_display_id_remap.rs"),
        (sound_id::build, "sound_id_remap.rs"),
    ]
}

#[macro_export]
macro_rules! remap_nodes {
    ($remapper:expr) => {{
        let node_1_7_6 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_7_6,
            value: "../../assets/viarewind/data/mappings-1.8to1.7.10.nbt",
            child: None,
        };
        let node_1_8 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_8,
            value: "../../assets/viarewind/data/mappings-1.9.4to1.8.nbt",
            child: Some(&node_1_7_6),
        };
        let node_1_9 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_9,
            value: "../../assets/viabackwards/data/mappings-1.10to1.9.4.nbt",
            child: Some(&node_1_8),
        };
        let node_1_10 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_10,
            value: "../../assets/viabackwards/data/mappings-1.11to1.10.nbt",
            child: Some(&node_1_9),
        };
        let node_1_11 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_11,
            value: "../../assets/viabackwards/data/mappings-1.12to1.11.nbt",
            child: Some(&node_1_10),
        };
        let node_1_12 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_12,
            value: "../../assets/viabackwards/data/mappings-1.13to1.12.nbt",
            child: Some(&node_1_11),
        };
        let node_1_13 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_13,
            value: "../../assets/viabackwards/data/mappings-1.13.2to1.13.nbt",
            child: Some(&node_1_12),
        };
        let node_1_13_2 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_13_2,
            value: "../../assets/viabackwards/data/mappings-1.14to1.13.2.nbt",
            child: Some(&node_1_13),
        };
        let node_1_14 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_14,
            value: "../../assets/viabackwards/data/mappings-1.15to1.14.nbt",
            child: Some(&node_1_13_2),
        };
        let node_1_15 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_15,
            value: "../../assets/viabackwards/data/mappings-1.16to1.15.nbt",
            child: Some(&node_1_14),
        };
        let node_1_16 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_16,
            value: "../../assets/viabackwards/data/mappings-1.16.2to1.16.nbt",
            child: Some(&node_1_15),
        };
        let node_1_16_2 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_16_2,
            value: "../../assets/viabackwards/data/mappings-1.17to1.16.2.nbt",
            child: Some(&node_1_16),
        };
        let node_1_17 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_17,
            value: "../../assets/viabackwards/data/mappings-1.18to1.17.nbt",
            child: Some(&node_1_16_2),
        };
        let node_1_18 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_18,
            value: "../../assets/viabackwards/data/mappings-1.19to1.18.nbt",
            child: Some(&node_1_17),
        };
        let node_1_19 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_19,
            value: "../../assets/viabackwards/data/mappings-1.19.3to1.19.nbt",
            child: Some(&node_1_18),
        };
        let node_1_19_3 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_19_3,
            value: "../../assets/viabackwards/data/mappings-1.19.4to1.19.3.nbt",
            child: Some(&node_1_19),
        };
        let node_1_19_4 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_19_4,
            value: "../../assets/viabackwards/data/mappings-1.20to1.19.4.nbt",
            child: Some(&node_1_19_3),
        };
        let node_1_20 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_20,
            value: "../../assets/viabackwards/data/mappings-1.20.2to1.20.nbt",
            child: Some(&node_1_19_4),
        };
        let node_1_20_2 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_20_2,
            value: "../../assets/viabackwards/data/mappings-1.20.3to1.20.2.nbt",
            child: Some(&node_1_20),
        };
        let node_1_20_3 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_20_3,
            value: "../../assets/viabackwards/data/mappings-1.20.5to1.20.3.nbt",
            child: Some(&node_1_20_2),
        };
        let node_1_20_5 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_20_5,
            value: "../../assets/viabackwards/data/mappings-1.21to1.20.5.nbt",
            child: Some(&node_1_20_3),
        };
        let node_1_21 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_21,
            value: "../../assets/viabackwards/data/mappings-1.21.2to1.21.nbt",
            child: Some(&node_1_20_5),
        };
        let node_1_21_2 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_21_2,
            value: "../../assets/viabackwards/data/mappings-1.21.4to1.21.2.nbt",
            child: Some(&node_1_21),
        };
        let node_1_21_4 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_21_4,
            value: "../../assets/viabackwards/data/mappings-1.21.5to1.21.4.nbt",
            child: Some(&node_1_21_2),
        };
        let node_1_21_5 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_21_5,
            value: "../../assets/viabackwards/data/mappings-1.21.6to1.21.5.nbt",
            child: Some(&node_1_21_4),
        };
        let node_1_21_6 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_21_6,
            value: "../../assets/viabackwards/data/mappings-1.21.7to1.21.6.nbt",
            child: Some(&node_1_21_5),
        };
        let node_1_21_7 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_21_7,
            value: "../../assets/viabackwards/data/mappings-1.21.9to1.21.7.nbt",
            child: Some(&node_1_21_6),
        };
        let node_1_21_9 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_21_9,
            value: "../../assets/viabackwards/data/mappings-1.21.11to1.21.9.nbt",
            child: Some(&node_1_21_7),
        };
        let node_1_21_11 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_1_21_11,
            value: "../../assets/viabackwards/data/mappings-26.1to1.21.11.nbt",
            child: Some(&node_1_21_9),
        };
        let node_26_1 = $crate::remap::MappingNode {
            version: $crate::version::JavaMinecraftVersion::V_26_1,
            value: "../../assets/viabackwards/data/mappings-26.2to26.1.nbt",
            child: Some(&node_1_21_11),
        };
        $remapper.process(&node_26_1)
    }};
}

/// A node in a linked chain of ViaVersion mapping files, each describing how IDs changed
/// between consecutive Minecraft versions.
pub struct MappingNode<'a, P> {
    /// The Minecraft version this node represents.
    pub version: JavaMinecraftVersion,
    /// The path to (or data of) the ViaVersion NBT mapping file for this version hop.
    pub value: P,
    /// The previous version node in the chain, or `None` if this is the oldest supported version.
    pub child: Option<&'a Self>,
}

/// Drives the recursive processing of a [`MappingNode`] chain, composing intermediate mappings
/// into per-version translation tables.
pub struct Remapper<P, R> {
    /// The target (latest) version that all older mappings are translated toward.
    pub version: JavaMinecraftVersion,
    /// Combines the current-version mapping with a child mapping into a composed mapping.
    pub remapper: fn(&R, &R) -> R,
    /// Converts the raw path/data `P` stored in a [`MappingNode`] into the mapping type `R`.
    pub serializer: fn(&P) -> R,
}

impl<P, R> Remapper<P, R> {
    /// Recursively processes the [`MappingNode`] chain and returns a list of `(version, mapping)` pairs.
    ///
    /// # Returns
    /// A `Vec` where each entry contains a [`JavaMinecraftVersion`] and its composed mapping relative
    /// to `self.version`.
    pub fn process(&self, mappings: &MappingNode<'_, P>) -> Vec<(JavaMinecraftVersion, R)> {
        let current_mapping = (self.serializer)(&mappings.value);
        let mut remap = if let Some(child) = mappings.child {
            let mut res = self.process(child);
            for (_, remap) in &mut res {
                let new_mapping = (self.remapper)(&current_mapping, remap);
                *remap = new_mapping;
            }
            res
        } else {
            Vec::new()
        };
        remap.push((mappings.version, current_mapping));
        remap
    }
}

/// A decoded ViaVersion ID mapping with a forward translation table.
pub struct ParsedMappings {
    /// Number of IDs in the mapped (newer) version's namespace.
    pub mapped_size: usize,
    /// Forward mapping: index is the old ID, value is the new ID (`-1` means unmapped).
    pub forward: Vec<i32>,
}

impl ParsedMappings {
    /// Reads and parses a ViaVersion NBT mapping file, extracting the named section.
    ///
    /// # Arguments
    /// - `path` – Path to the `.nbt` mapping file.
    /// - `section` – Name of the compound section to extract (e.g. `"blockstates"`, `"items"`).
    ///
    /// # Returns
    /// `Some(ParsedMappings)` if the section exists, or `None` if the section is absent.
    pub fn parse_mapping_file(path: &str, section: &str) -> Option<Self> {
        use pumpkin_nbt::Nbt;
        use pumpkin_nbt::deserializer::NbtReadHelperJava;
        use std::fs;
        use std::io::Cursor;

        let bytes = fs::read(path).unwrap_or_else(|_| panic!("Failed to read {path}"));
        let mut reader = NbtReadHelperJava::new(Cursor::new(bytes));
        let nbt =
            Nbt::read(&mut reader).unwrap_or_else(|_| panic!("Failed to parse NBT at {path}"));

        let mappings = nbt.root_tag.get_compound(section)?;
        // .unwrap_or_else(|| panic!("Missing `{section}` compound in {path}"));

        Some(Self::parse_mappings(mappings, path, section))
    }

    /// Decodes a ViaVersion mapping compound into a forward ID translation table.
    fn parse_mappings(mappings: &NbtCompound, path: &str, section: &str) -> Self {
        let mapped_size = mappings
            .get_int("mappedSize")
            .unwrap_or_else(|| panic!("Missing `{section}.mappedSize` in {path}"));
        let strategy = mappings
            .get_byte("id")
            .unwrap_or_else(|| panic!("Missing `{section}.id` in {path}"));

        let forward = match strategy {
            // Direct
            0 => {
                if let Some(val) = mappings.get_int_array("val") {
                    val.to_vec()
                } else if let Some(val_bytes) = mappings.get_byte_array("val") {
                    let size = mappings.get_int("size").unwrap_or(mapped_size) as usize;
                    let bytes: &[u8] = unsafe {
                        std::slice::from_raw_parts(val_bytes.as_ptr().cast::<u8>(), val_bytes.len())
                    };
                    let mut cursor = std::io::Cursor::new(bytes);
                    let mut values = Vec::with_capacity(size);
                    let mut prev = 0i32;
                    for _ in 0..size {
                        prev += Self::read_zigzag_var_int(&mut cursor).unwrap_or(0);
                        values.push(prev);
                    }
                    values
                } else {
                    panic!("Missing `{section}.val` for direct mapping in {path}");
                }
            }
            // Shifts
            1 => {
                let (shifts_at, shifts_to) = if let (Some(at), Some(to)) =
                    (mappings.get_int_array("at"), mappings.get_int_array("to"))
                {
                    (at.to_vec(), to.to_vec())
                } else if let Some(val_bytes) = mappings.get_byte_array("val") {
                    Self::read_at_value_pairs(val_bytes)
                } else {
                    panic!(
                        "Missing `{section}.at`/`to` or `{section}.val` for shift mapping in {path}"
                    );
                };

                let size = mappings.get_int("size").unwrap_or_else(|| {
                    panic!("Missing `{section}.size` for shift mapping in {path}")
                }) as usize;

                assert_eq!(
                    shifts_at.len(),
                    shifts_to.len(),
                    "Shift mapping length mismatch in {path}"
                );

                let mut result = vec![-1; size];

                if !shifts_at.is_empty() && shifts_at[0] != 0 {
                    for id in 0..shifts_at[0] {
                        result[id as usize] = id;
                    }
                }

                for (index, from) in shifts_at.iter().enumerate() {
                    let to = if index + 1 == shifts_at.len() {
                        size as i32
                    } else {
                        shifts_at[index + 1]
                    };
                    for (mapped_id, id) in (shifts_to[index]..).zip(*from..to) {
                        result[id as usize] = mapped_id;
                    }
                }

                result
            }
            // Changes
            2 => {
                let (changes_at, values) = if let (Some(at), Some(val)) =
                    (mappings.get_int_array("at"), mappings.get_int_array("val"))
                {
                    (at.to_vec(), val.to_vec())
                } else if let Some(val_bytes) = mappings.get_byte_array("val") {
                    Self::read_at_value_pairs(val_bytes)
                } else {
                    panic!("Missing `{section}.at`/`val` for change mapping in {path}");
                };

                let size = mappings.get_int("size").unwrap_or_else(|| {
                    panic!("Missing `{section}.size` for change mapping in {path}")
                }) as usize;
                let fill_between = mappings.get("nofill").is_none();

                assert_eq!(
                    changes_at.len(),
                    values.len(),
                    "Change mapping length mismatch in {path}"
                );

                let mut result = vec![-1; size];
                let mut next_unhandled_id = 0;

                for (index, changed_id) in changes_at.iter().enumerate() {
                    if fill_between {
                        for id in next_unhandled_id..*changed_id {
                            result[id as usize] = id;
                        }
                        next_unhandled_id = changed_id + 1;
                    }
                    result[*changed_id as usize] = values[index];
                }

                if fill_between {
                    for id in next_unhandled_id..size as i32 {
                        result[id as usize] = id;
                    }
                }

                result
            }
            // Identity
            3 => {
                let size = mappings.get_int("size").unwrap_or_else(|| {
                    panic!("Missing `{section}.size` for identity mapping in {path}")
                }) as usize;
                (0..size as i32).collect::<Vec<_>>()
            }
            _ => panic!("Unknown {section} mapping strategy {strategy} in {path}"),
        };

        Self {
            mapped_size: mapped_size as usize,
            forward,
        }
    }

    fn read_var_int(cursor: &mut std::io::Cursor<&[u8]>) -> Option<i32> {
        use std::io::Read;
        let mut num_read = 0;
        let mut result = 0i32;
        loop {
            let mut byte = [0u8; 1];
            if cursor.read_exact(&mut byte).is_err() {
                return None;
            }
            let b = byte[0];
            let value = (b & 0b0111_1111) as i32;
            result |= value << (7 * num_read);
            num_read += 1;
            if num_read > 5 {
                return None;
            }
            if (b & 0b1000_0000) == 0 {
                break;
            }
        }
        Some(result)
    }

    fn read_zigzag_var_int(cursor: &mut std::io::Cursor<&[u8]>) -> Option<i32> {
        let value = Self::read_var_int(cursor)?;
        let unsigned = value as u32;
        Some(((unsigned >> 1) as i32) ^ (-((unsigned & 1) as i32)))
    }

    fn read_at_value_pairs(val_bytes: &[i8]) -> (Vec<i32>, Vec<i32>) {
        let bytes: &[u8] =
            unsafe { std::slice::from_raw_parts(val_bytes.as_ptr().cast::<u8>(), val_bytes.len()) };
        let mut cursor = std::io::Cursor::new(bytes);
        let mut at = Vec::new();
        let mut values = Vec::new();
        let mut prev_at = -1i32;
        let mut prev_val = 0i32;
        while let Some(diff_at) = Self::read_var_int(&mut cursor) {
            let diff_val = Self::read_zigzag_var_int(&mut cursor).unwrap_or(0);
            prev_at = prev_at + 1 + diff_at;
            prev_val += diff_val;
            at.push(prev_at);
            values.push(prev_val);
        }
        (at, values)
    }

    /// Inverts the forward mapping into a reverse lookup table where index is the new ID and value
    /// is the corresponding old ID. Unmapped entries default to their own index cast to `u16`.
    ///
    /// # Arguments
    /// - `name` – Descriptive name used in panic messages for better diagnostics.
    ///
    /// # Returns
    /// A `Vec<u16>` of length `self.mapped_size` mapping new IDs back to old IDs.
    pub fn _invert_with_default_to_u16(&self, name: &str) -> Vec<u16> {
        let mut inverse = vec![0u16; self.mapped_size];
        let mut seen = vec![false; self.mapped_size];

        for (old_id, mapped_id) in self.forward.iter().enumerate() {
            let Ok(mapped_id) = usize::try_from(*mapped_id) else {
                continue;
            };
            if mapped_id >= self.mapped_size || seen[mapped_id] {
                continue;
            }

            let old_u16 = u16::try_from(old_id)
                .unwrap_or_else(|_| panic!("{name}: id {old_id} does not fit in u16"));
            inverse[mapped_id] = old_u16;
            seen[mapped_id] = true;
        }

        for (mapped_id, mapped_to) in inverse.iter_mut().enumerate() {
            if !seen[mapped_id] {
                *mapped_to = u16::try_from(mapped_id)
                    .unwrap_or_else(|_| panic!("{name}: id {mapped_id} does not fit in u16"));
            }
        }

        inverse
    }

    /// Converts the forward mapping directly to a u16 table.
    /// Used with ViaBackwards mappings which are already in new→old direction.
    pub fn to_u16(&self, name: &str) -> Vec<u16> {
        self.forward
            .iter()
            .map(|&id| {
                if id < 0 {
                    0 // unmapped → air
                } else if id > 0xFFFF {
                    // For pre-1.13 mappings where itemId was packed as (id << 16) | data
                    u16::try_from(id >> 16)
                        .unwrap_or_else(|_| panic!("{name}: id {id} does not fit in u16"))
                } else {
                    u16::try_from(id)
                        .unwrap_or_else(|_| panic!("{name}: id {id} does not fit in u16"))
                }
            })
            .collect()
    }

    /// Converts the forward mapping directly to a u32 table.
    /// Used with ViaBackwards mappings which are already in new→old direction.
    pub fn to_u32(&self, name: &str) -> Vec<u32> {
        self.forward
            .iter()
            .map(|&id| {
                if id < 0 {
                    0
                } else {
                    u32::try_from(id)
                        .unwrap_or_else(|_| panic!("{name}: id {id} does not fit in u32"))
                }
            })
            .collect()
    }
}

/// Returns all JavaMinecraftVersion variants that share the same protocol data mapping.
#[must_use]
pub fn version_patterns(ver: JavaMinecraftVersion) -> Vec<JavaMinecraftVersion> {
    match ver {
        JavaMinecraftVersion::V_1_7_6 => {
            vec![JavaMinecraftVersion::V_1_7_2, JavaMinecraftVersion::V_1_7_6]
        }
        JavaMinecraftVersion::V_1_8 => vec![JavaMinecraftVersion::V_1_8],
        JavaMinecraftVersion::V_1_9 => vec![
            JavaMinecraftVersion::V_1_9,
            JavaMinecraftVersion::V_1_9_1,
            JavaMinecraftVersion::V_1_9_2,
            JavaMinecraftVersion::V_1_9_3,
        ],
        JavaMinecraftVersion::V_1_10 => vec![JavaMinecraftVersion::V_1_10],
        JavaMinecraftVersion::V_1_11 => {
            vec![JavaMinecraftVersion::V_1_11, JavaMinecraftVersion::V_1_11_1]
        }
        JavaMinecraftVersion::V_1_12 => vec![
            JavaMinecraftVersion::V_1_12,
            JavaMinecraftVersion::V_1_12_1,
            JavaMinecraftVersion::V_1_12_2,
        ],
        JavaMinecraftVersion::V_1_13 => {
            vec![JavaMinecraftVersion::V_1_13, JavaMinecraftVersion::V_1_13_1]
        }
        JavaMinecraftVersion::V_1_13_2 => vec![JavaMinecraftVersion::V_1_13_2],
        JavaMinecraftVersion::V_1_14 => vec![
            JavaMinecraftVersion::V_1_14,
            JavaMinecraftVersion::V_1_14_1,
            JavaMinecraftVersion::V_1_14_2,
            JavaMinecraftVersion::V_1_14_3,
            JavaMinecraftVersion::V_1_14_4,
        ],
        JavaMinecraftVersion::V_1_15 => vec![
            JavaMinecraftVersion::V_1_15,
            JavaMinecraftVersion::V_1_15_1,
            JavaMinecraftVersion::V_1_15_2,
        ],
        JavaMinecraftVersion::V_1_16 => {
            vec![JavaMinecraftVersion::V_1_16, JavaMinecraftVersion::V_1_16_1]
        }
        JavaMinecraftVersion::V_1_16_2 => vec![
            JavaMinecraftVersion::V_1_16_2,
            JavaMinecraftVersion::V_1_16_3,
            JavaMinecraftVersion::V_1_16_4,
        ],
        JavaMinecraftVersion::V_1_17 => {
            vec![JavaMinecraftVersion::V_1_17, JavaMinecraftVersion::V_1_17_1]
        }
        JavaMinecraftVersion::V_1_18 => {
            vec![JavaMinecraftVersion::V_1_18, JavaMinecraftVersion::V_1_18_2]
        }
        JavaMinecraftVersion::V_1_19 => {
            vec![JavaMinecraftVersion::V_1_19, JavaMinecraftVersion::V_1_19_1]
        }
        JavaMinecraftVersion::V_1_19_3 => vec![JavaMinecraftVersion::V_1_19_3],
        JavaMinecraftVersion::V_1_19_4 => vec![JavaMinecraftVersion::V_1_19_4],
        JavaMinecraftVersion::V_1_20 => vec![JavaMinecraftVersion::V_1_20],
        JavaMinecraftVersion::V_1_20_2 => vec![JavaMinecraftVersion::V_1_20_2],
        JavaMinecraftVersion::V_1_20_3 => vec![JavaMinecraftVersion::V_1_20_3],
        JavaMinecraftVersion::V_1_20_5 => vec![JavaMinecraftVersion::V_1_20_5],
        JavaMinecraftVersion::V_1_21 => vec![JavaMinecraftVersion::V_1_21],
        JavaMinecraftVersion::V_1_21_2 => vec![JavaMinecraftVersion::V_1_21_2],
        JavaMinecraftVersion::V_1_21_4 => vec![JavaMinecraftVersion::V_1_21_4],
        JavaMinecraftVersion::V_1_21_5 => vec![JavaMinecraftVersion::V_1_21_5],
        JavaMinecraftVersion::V_1_21_6 => vec![JavaMinecraftVersion::V_1_21_6],
        JavaMinecraftVersion::V_1_21_7 => vec![JavaMinecraftVersion::V_1_21_7],
        JavaMinecraftVersion::V_1_21_9 => vec![JavaMinecraftVersion::V_1_21_9],
        JavaMinecraftVersion::V_1_21_11 => vec![JavaMinecraftVersion::V_1_21_11],
        JavaMinecraftVersion::V_26_1 => vec![JavaMinecraftVersion::V_26_1],
        JavaMinecraftVersion::V_26_2 => vec![JavaMinecraftVersion::V_26_2],
        _ => vec![ver],
    }
}
