use proc_macro2::TokenStream;
use pumpkin_nbt::compound::NbtCompound;

use crate::version::MinecraftVersion;

mod block_state;
mod item_id;

#[allow(clippy::type_complexity)]
pub fn build() -> Vec<(fn() -> TokenStream, &'static str)> {
    vec![
        (block_state::build, "block_state_remap.rs"),
        (item_id::build, "item_id_remap.rs"),
    ]
}

pub struct MappingNode<'a, P> {
    pub version: MinecraftVersion,
    pub value: P,
    pub child: Option<&'a Self>,
}

pub struct Remapper<P, R> {
    pub version: MinecraftVersion,
    pub remapper: fn(&R, &R) -> R,
    pub serializer: fn(&P) -> R,
}

impl<P, R> Remapper<P, R> {
    pub fn process(&self, mappings: &MappingNode<'_, P>) -> Vec<(MinecraftVersion, R)> {
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

pub struct ParsedMappings {
    pub mapped_size: usize,
    pub forward: Vec<i32>,
}

impl ParsedMappings {
    pub fn parse_mapping_file(path: &str, section: &str) -> Option<Self> {
        use pumpkin_nbt::Nbt;
        use pumpkin_nbt::deserializer::NbtReadHelper;
        use std::fs;
        use std::io::Cursor;

        let bytes = fs::read(path).unwrap_or_else(|_| panic!("Failed to read {path}"));
        let mut reader = NbtReadHelper::new(Cursor::new(bytes));
        let nbt =
            Nbt::read(&mut reader).unwrap_or_else(|_| panic!("Failed to parse NBT at {path}"));

        let mappings = nbt.root_tag.get_compound(section)?;
        // .unwrap_or_else(|| panic!("Missing `{section}` compound in {path}"));

        Some(Self::parse_mappings(mappings, path, section))
    }

    fn parse_mappings(mappings: &NbtCompound, path: &str, section: &str) -> Self {
        let mapped_size = mappings
            .get_int("mappedSize")
            .unwrap_or_else(|| panic!("Missing `{section}.mappedSize` in {path}"));
        let strategy = mappings
            .get_byte("id")
            .unwrap_or_else(|| panic!("Missing `{section}.id` in {path}"));

        let forward = match strategy {
            // Direct
            0 => mappings
                .get_int_array("val")
                .unwrap_or_else(|| panic!("Missing `{section}.val` for direct mapping in {path}"))
                .to_vec(),
            // Shifts
            1 => {
                let shifts_at = mappings.get_int_array("at").unwrap_or_else(|| {
                    panic!("Missing `{section}.at` for shift mapping in {path}")
                });
                let shifts_to = mappings.get_int_array("to").unwrap_or_else(|| {
                    panic!("Missing `{section}.to` for shift mapping in {path}")
                });
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
                    let mut mapped_id = shifts_to[index];
                    for id in *from..to {
                        result[id as usize] = mapped_id;
                        mapped_id += 1;
                    }
                }

                result
            }
            // Changes
            2 => {
                let changes_at = mappings.get_int_array("at").unwrap_or_else(|| {
                    panic!("Missing `{section}.at` for change mapping in {path}")
                });
                let values = mappings.get_int_array("val").unwrap_or_else(|| {
                    panic!("Missing `{section}.val` for change mapping in {path}")
                });
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

    pub fn invert_with_default_to_u16(&self, name: &str) -> Vec<u16> {
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
}
