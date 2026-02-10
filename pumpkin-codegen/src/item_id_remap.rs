use proc_macro2::{Literal, TokenStream};
use quote::quote;
use std::{fs, io::Cursor};

use pumpkin_nbt::{Nbt, compound::NbtCompound, deserializer::NbtReadHelper};

struct ParsedMappings {
    mapped_size: usize,
    forward: Vec<i32>,
}

fn parse_mapping_file(path: &str, section: &str) -> ParsedMappings {
    let bytes = fs::read(path).unwrap_or_else(|_| panic!("Failed to read {path}"));
    let mut reader = NbtReadHelper::new(Cursor::new(bytes));
    let nbt = Nbt::read(&mut reader).unwrap_or_else(|_| panic!("Failed to parse NBT at {path}"));

    let mappings = nbt
        .root_tag
        .get_compound(section)
        .unwrap_or_else(|| panic!("Missing `{section}` compound in {path}"));

    parse_mappings(mappings, path, section)
}

fn parse_mappings(mappings: &NbtCompound, path: &str, section: &str) -> ParsedMappings {
    let mapped_size = mappings
        .get_int("mappedSize")
        .unwrap_or_else(|| panic!("Missing `{section}.mappedSize` in {path}"));
    let strategy = mappings
        .get_byte("id")
        .unwrap_or_else(|| panic!("Missing `{section}.id` in {path}"));

    let forward =
        match strategy {
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

    ParsedMappings {
        mapped_size: mapped_size as usize,
        forward,
    }
}

fn invert_with_default_to_u16(forward: &[i32], mapped_size: usize, name: &str) -> Vec<u16> {
    let mut inverse = vec![0u16; mapped_size];
    let mut seen = vec![false; mapped_size];

    for (old_id, mapped_id) in forward.iter().enumerate() {
        let Ok(mapped_id) = usize::try_from(*mapped_id) else {
            continue;
        };
        if mapped_id >= mapped_size || seen[mapped_id] {
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

fn forward_with_default_to_u16(forward: &[i32], name: &str) -> Vec<u16> {
    forward
        .iter()
        .enumerate()
        .map(|(old_id, mapped_id)| {
            let mapped = usize::try_from(*mapped_id).ok().unwrap_or(old_id);
            u16::try_from(mapped).unwrap_or_else(|_| {
                u16::try_from(old_id).unwrap_or_else(|_| {
                    panic!("{name}: mapped id {mapped} and fallback id {old_id} do not fit in u16")
                })
            })
        })
        .collect()
}

fn compose_with_default(first: &[u16], second: &[u16]) -> Vec<u16> {
    first
        .iter()
        .map(|id| second.get(usize::from(*id)).copied().unwrap_or(*id))
        .collect()
}

pub fn build() -> TokenStream {
    let mappings_7_to_9 = parse_mapping_file(
        "../assets/viaversion/data/mappings-1.21.7to1.21.9.nbt",
        "items",
    );
    let mappings_9_to_11 = parse_mapping_file(
        "../assets/viaversion/data/mappings-1.21.9to1.21.11.nbt",
        "items",
    );

    let remap_9_to_7 = invert_with_default_to_u16(
        &mappings_7_to_9.forward,
        mappings_7_to_9.mapped_size,
        "1.21.9->1.21.7 items",
    );
    let remap_7_to_9 =
        forward_with_default_to_u16(&mappings_7_to_9.forward, "1.21.7->1.21.9 items");
    let remap_11_to_9 = invert_with_default_to_u16(
        &mappings_9_to_11.forward,
        mappings_9_to_11.mapped_size,
        "1.21.11->1.21.9 items",
    );
    let remap_9_to_11 =
        forward_with_default_to_u16(&mappings_9_to_11.forward, "1.21.9->1.21.11 items");
    let remap_11_to_7 = compose_with_default(&remap_11_to_9, &remap_9_to_7);
    let remap_7_to_11 = compose_with_default(&remap_7_to_9, &remap_9_to_11);

    let remap_11_to_9_tokens: Vec<Literal> = remap_11_to_9
        .into_iter()
        .map(Literal::u16_unsuffixed)
        .collect();
    let remap_9_to_11_tokens: Vec<Literal> = remap_9_to_11
        .into_iter()
        .map(Literal::u16_unsuffixed)
        .collect();
    let remap_11_to_7_tokens: Vec<Literal> = remap_11_to_7
        .into_iter()
        .map(Literal::u16_unsuffixed)
        .collect();
    let remap_7_to_11_tokens: Vec<Literal> = remap_7_to_11
        .into_iter()
        .map(Literal::u16_unsuffixed)
        .collect();

    quote! {
        use pumpkin_util::version::MinecraftVersion;

        pub static ITEM_ID_REMAP_1_21_11_TO_1_21_9: &[u16] = &[#(#remap_11_to_9_tokens),*];
        pub static ITEM_ID_REMAP_1_21_11_TO_1_21_7: &[u16] = &[#(#remap_11_to_7_tokens),*];
        pub static ITEM_ID_REMAP_1_21_9_TO_1_21_11: &[u16] = &[#(#remap_9_to_11_tokens),*];
        pub static ITEM_ID_REMAP_1_21_7_TO_1_21_11: &[u16] = &[#(#remap_7_to_11_tokens),*];

        #[must_use]
        pub fn remap_item_id_for_version(item_id: u16, version: MinecraftVersion) -> u16 {
            match version {
                MinecraftVersion::V_1_21_7 => ITEM_ID_REMAP_1_21_11_TO_1_21_7
                    .get(usize::from(item_id))
                    .copied()
                    .unwrap_or(item_id),
                MinecraftVersion::V_1_21_9 => ITEM_ID_REMAP_1_21_11_TO_1_21_9
                    .get(usize::from(item_id))
                    .copied()
                    .unwrap_or(item_id),
                _ => item_id,
            }
        }

        #[must_use]
        pub fn remap_item_id_from_version(item_id: u16, version: MinecraftVersion) -> u16 {
            match version {
                MinecraftVersion::V_1_21_7 => ITEM_ID_REMAP_1_21_7_TO_1_21_11
                    .get(usize::from(item_id))
                    .copied()
                    .unwrap_or(item_id),
                MinecraftVersion::V_1_21_9 => ITEM_ID_REMAP_1_21_9_TO_1_21_11
                    .get(usize::from(item_id))
                    .copied()
                    .unwrap_or(item_id),
                _ => item_id,
            }
        }
    }
}
