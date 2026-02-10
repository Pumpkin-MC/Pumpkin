use proc_macro2::{Literal, TokenStream};
use quote::quote;
use std::{fs, io::Cursor};

use pumpkin_nbt::{Nbt, compound::NbtCompound, deserializer::NbtReadHelper};

struct ParsedMappings {
    mapped_size: usize,
    forward: Vec<i32>,
}

fn parse_mapping_file(path: &str) -> ParsedMappings {
    let bytes = fs::read(path).unwrap_or_else(|_| panic!("Failed to read {path}"));
    let mut reader = NbtReadHelper::new(Cursor::new(bytes));
    let nbt = Nbt::read(&mut reader).unwrap_or_else(|_| panic!("Failed to parse NBT at {path}"));

    let blockstates = nbt
        .root_tag
        .get_compound("blockstates")
        .unwrap_or_else(|| panic!("Missing `blockstates` compound in {path}"));

    parse_blockstate_mappings(blockstates, path)
}

fn parse_blockstate_mappings(blockstates: &NbtCompound, path: &str) -> ParsedMappings {
    let mapped_size = blockstates
        .get_int("mappedSize")
        .unwrap_or_else(|| panic!("Missing `blockstates.mappedSize` in {path}"));
    let strategy = blockstates
        .get_byte("id")
        .unwrap_or_else(|| panic!("Missing `blockstates.id` in {path}"));

    let forward = match strategy {
        // Direct
        0 => blockstates
            .get_int_array("val")
            .unwrap_or_else(|| panic!("Missing `blockstates.val` for direct mapping in {path}"))
            .to_vec(),
        // Shifts
        1 => {
            let shifts_at = blockstates
                .get_int_array("at")
                .unwrap_or_else(|| panic!("Missing `blockstates.at` for shift mapping in {path}"));
            let shifts_to = blockstates
                .get_int_array("to")
                .unwrap_or_else(|| panic!("Missing `blockstates.to` for shift mapping in {path}"));
            let size = blockstates
                .get_int("size")
                .unwrap_or_else(|| panic!("Missing `blockstates.size` for shift mapping in {path}"))
                as usize;

            assert_eq!(
                shifts_at.len(),
                shifts_to.len(),
                "Shift mapping length mismatch in {path}"
            );

            let mut mappings = vec![-1; size];

            if !shifts_at.is_empty() && shifts_at[0] != 0 {
                for id in 0..shifts_at[0] {
                    mappings[id as usize] = id;
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
                    mappings[id as usize] = mapped_id;
                    mapped_id += 1;
                }
            }

            mappings
        }
        // Changes
        2 => {
            let changes_at = blockstates
                .get_int_array("at")
                .unwrap_or_else(|| panic!("Missing `blockstates.at` for change mapping in {path}"));
            let values = blockstates.get_int_array("val").unwrap_or_else(|| {
                panic!("Missing `blockstates.val` for change mapping in {path}")
            });
            let size = blockstates.get_int("size").unwrap_or_else(|| {
                panic!("Missing `blockstates.size` for change mapping in {path}")
            }) as usize;
            let fill_between = blockstates.get("nofill").is_none();

            assert_eq!(
                changes_at.len(),
                values.len(),
                "Change mapping length mismatch in {path}"
            );

            let mut mappings = vec![-1; size];
            let mut next_unhandled_id = 0;

            for (index, changed_id) in changes_at.iter().enumerate() {
                if fill_between {
                    for id in next_unhandled_id..*changed_id {
                        mappings[id as usize] = id;
                    }
                    next_unhandled_id = changed_id + 1;
                }
                mappings[*changed_id as usize] = values[index];
            }

            mappings
        }
        // Identity
        3 => {
            let size = blockstates.get_int("size").unwrap_or_else(|| {
                panic!("Missing `blockstates.size` for identity mapping in {path}")
            }) as usize;
            (0..size as i32).collect::<Vec<_>>()
        }
        _ => panic!("Unknown blockstate mapping strategy {strategy} in {path}"),
    };

    ParsedMappings {
        mapped_size: mapped_size as usize,
        forward,
    }
}

fn invert_to_u16(forward: &[i32], mapped_size: usize, name: &str) -> Vec<u16> {
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

    inverse
}

fn compose(first: &[u16], second: &[u16]) -> Vec<u16> {
    first
        .iter()
        .map(|id| second.get(usize::from(*id)).copied().unwrap_or(0))
        .collect()
}

pub fn build() -> TokenStream {
    let mappings_7_to_9 =
        parse_mapping_file("../assets/viaversion/data/mappings-1.21.7to1.21.9.nbt");
    let mappings_9_to_11 =
        parse_mapping_file("../assets/viaversion/data/mappings-1.21.9to1.21.11.nbt");

    let remap_9_to_7 = invert_to_u16(
        &mappings_7_to_9.forward,
        mappings_7_to_9.mapped_size,
        "1.21.9->1.21.7",
    );
    let remap_11_to_9 = invert_to_u16(
        &mappings_9_to_11.forward,
        mappings_9_to_11.mapped_size,
        "1.21.11->1.21.9",
    );
    let remap_11_to_7 = compose(&remap_11_to_9, &remap_9_to_7);

    let remap_11_to_9_tokens: Vec<Literal> = remap_11_to_9
        .into_iter()
        .map(Literal::u16_unsuffixed)
        .collect();
    let remap_11_to_7_tokens: Vec<Literal> = remap_11_to_7
        .into_iter()
        .map(Literal::u16_unsuffixed)
        .collect();

    quote! {
        use pumpkin_util::version::MinecraftVersion;

        pub static BLOCK_STATE_REMAP_1_21_11_TO_1_21_9: &[u16] = &[#(#remap_11_to_9_tokens),*];
        pub static BLOCK_STATE_REMAP_1_21_11_TO_1_21_7: &[u16] = &[#(#remap_11_to_7_tokens),*];

        #[must_use]
        pub fn remap_block_state_for_version(state_id: u16, version: MinecraftVersion) -> u16 {
            match version {
                MinecraftVersion::V_1_21_7 => BLOCK_STATE_REMAP_1_21_11_TO_1_21_7
                    .get(usize::from(state_id))
                    .copied()
                    .unwrap_or(0),
                MinecraftVersion::V_1_21_9 => BLOCK_STATE_REMAP_1_21_11_TO_1_21_9
                    .get(usize::from(state_id))
                    .copied()
                    .unwrap_or(0),
                _ => state_id,
            }
        }
    }
}
