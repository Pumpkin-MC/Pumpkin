//! Packet ids of every supported Java version, for `pumpkin-protocol`'s `java-legacy` feature.

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use std::fs;

use crate::packet::{Packets, aliases, parse_packets};
use crate::version::JavaMinecraftVersion;

/// Output file, relative to the codegen manifest.
pub const OUT_FILE: &str = "../../crates/pumpkin-protocol/src/java/legacy/generated_ids.rs";

/// Older versions, oldest first, as stored in `assets/legacy_packets`. The current version is
/// read from `assets/packets.json` and appended last.
const LEGACY_VERSIONS: &[(JavaMinecraftVersion, &str)] = &[
    (JavaMinecraftVersion::V_1_7_2, "1_7_2"),
    (JavaMinecraftVersion::V_1_7_6, "1_7_6"),
    (JavaMinecraftVersion::V_1_8, "1_8"),
    (JavaMinecraftVersion::V_1_9, "1_9"),
    (JavaMinecraftVersion::V_1_9_1, "1_9_1"),
    (JavaMinecraftVersion::V_1_9_2, "1_9_2"),
    (JavaMinecraftVersion::V_1_9_3, "1_9_3"),
    (JavaMinecraftVersion::V_1_10, "1_10"),
    (JavaMinecraftVersion::V_1_11, "1_11"),
    (JavaMinecraftVersion::V_1_11_1, "1_11_1"),
    (JavaMinecraftVersion::V_1_12, "1_12"),
    (JavaMinecraftVersion::V_1_12_1, "1_12_1"),
    (JavaMinecraftVersion::V_1_12_2, "1_12_2"),
    (JavaMinecraftVersion::V_1_13, "1_13"),
    (JavaMinecraftVersion::V_1_13_1, "1_13_1"),
    (JavaMinecraftVersion::V_1_13_2, "1_13_2"),
    (JavaMinecraftVersion::V_1_14, "1_14"),
    (JavaMinecraftVersion::V_1_14_1, "1_14_1"),
    (JavaMinecraftVersion::V_1_14_2, "1_14_2"),
    (JavaMinecraftVersion::V_1_14_3, "1_14_3"),
    (JavaMinecraftVersion::V_1_14_4, "1_14_4"),
    (JavaMinecraftVersion::V_1_15, "1_15"),
    (JavaMinecraftVersion::V_1_15_1, "1_15_1"),
    (JavaMinecraftVersion::V_1_15_2, "1_15_2"),
    (JavaMinecraftVersion::V_1_16, "1_16"),
    (JavaMinecraftVersion::V_1_16_1, "1_16_1"),
    (JavaMinecraftVersion::V_1_16_2, "1_16_2"),
    (JavaMinecraftVersion::V_1_16_3, "1_16_3"),
    (JavaMinecraftVersion::V_1_16_4, "1_16_4"),
    (JavaMinecraftVersion::V_1_17, "1_17"),
    (JavaMinecraftVersion::V_1_17_1, "1_17_1"),
    (JavaMinecraftVersion::V_1_18, "1_18"),
    (JavaMinecraftVersion::V_1_18_2, "1_18_2"),
    (JavaMinecraftVersion::V_1_19, "1_19"),
    (JavaMinecraftVersion::V_1_19_1, "1_19_1"),
    (JavaMinecraftVersion::V_1_19_3, "1_19_3"),
    (JavaMinecraftVersion::V_1_19_4, "1_19_4"),
    (JavaMinecraftVersion::V_1_20, "1_20"),
    (JavaMinecraftVersion::V_1_20_2, "1_20_2"),
    (JavaMinecraftVersion::V_1_20_3, "1_20_3"),
    (JavaMinecraftVersion::V_1_20_5, "1_20_5"),
    (JavaMinecraftVersion::V_1_21, "1_21"),
    (JavaMinecraftVersion::V_1_21_2, "1_21_2"),
    (JavaMinecraftVersion::V_1_21_4, "1_21_4"),
    (JavaMinecraftVersion::V_1_21_5, "1_21_5"),
    (JavaMinecraftVersion::V_1_21_6, "1_21_6"),
    (JavaMinecraftVersion::V_1_21_7, "1_21_7"),
    (JavaMinecraftVersion::V_1_21_9, "1_21_9"),
    (JavaMinecraftVersion::V_1_21_11, "1_21_11"),
    (JavaMinecraftVersion::V_26_1, "26_1"),
    (JavaMinecraftVersion::V_26_2, "26_2"),
];

/// Connection states in the order of the generated lookup tables.
const STATES: [&str; 5] = ["handshake", "status", "login", "config", "play"];

/// Packets renamed in a later version, as `(current row, older row)`, per direction and state.
const SERVERBOUND_RENAMES: &[(&str, &str, &str)] = &[
    ("play", "COMMAND_SUGGESTION", "COMMAND_SUGGESTIONS"),
    ("play", "TELEPORT_TO_ENTITY", "SPECTATE_ENTITY"),
    ("play", "PUNCH", "SWING"),
];
const CLIENTBOUND_RENAMES: &[(&str, &str, &str)] = &[
    ("login", "LOGIN_FINISHED", "GAME_PROFILE"),
    ("play", "PLAYER_CHAT", "CHAT"),
    ("play", "PLAYER_ROTATION", "MOVE_PLAYER_ROT"),
    ("play", "SET_HELD_SLOT", "SET_CARRIED_ITEM"),
];

/// state -> packet name -> id per version column (`-1` when absent).
type Rows = BTreeMap<String, BTreeMap<String, Vec<i16>>>;

pub(crate) fn build() -> TokenStream {
    let mut columns: Vec<Packets> = LEGACY_VERSIONS
        .iter()
        .map(|(_, stem)| read(&format!("../../assets/legacy_packets/{stem}_packets.json")))
        .collect();
    columns.push(read("../../assets/packets.json"));
    let versions = columns.len();
    let version_count = Literal::usize_unsuffixed(versions);

    let serverbound = rows(&columns, true);
    let clientbound = rows(&columns, false);

    let serverbound_modules = phase_modules(&serverbound, true);
    let clientbound_modules = phase_modules(&clientbound, false);
    let (serverbound_tables, serverbound_statics) =
        lookup_tables(&serverbound, SERVERBOUND_RENAMES, versions, true);
    let (clientbound_tables, clientbound_statics) =
        lookup_tables(&clientbound, CLIENTBOUND_RENAMES, versions, false);

    quote! {
        /// Number of version columns, oldest first; the last one is the current version.
        pub const VERSIONS: usize = #version_count;

        /// A packet's id in every version, `-1` where the version doesn't have it.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct PacketId(pub [i16; VERSIONS]);

        pub mod serverbound {
            #serverbound_modules
        }

        pub mod clientbound {
            #clientbound_modules
        }

        #serverbound_statics
        #clientbound_statics

        /// Client id to current id, per state and version.
        pub(super) static SERVERBOUND_TO_CURRENT: [[&[i16]; VERSIONS]; 5] = [#serverbound_tables];

        /// Current id to client id, per state and version.
        pub(super) static CLIENTBOUND_FROM_CURRENT: [[&[i16]; VERSIONS]; 5] = [#clientbound_tables];
    }
}

fn read(path: &str) -> Packets {
    let content =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read packet JSON: {path}"));
    parse_packets(path, &content)
}

fn rows(columns: &[Packets], is_serverbound: bool) -> Rows {
    let mut rows = Rows::new();
    for (column, Packets(phases)) in columns.iter().enumerate() {
        for (phase, data) in phases {
            let packets = if is_serverbound {
                &data.serverbound
            } else {
                &data.clientbound
            };
            let state = if phase == "configuration" {
                "config"
            } else {
                phase.as_str()
            };
            for (full_name, info) in packets {
                let name = full_name.strip_prefix("minecraft:").unwrap_or(full_name);
                let ids = rows
                    .entry(state.to_string())
                    .or_default()
                    .entry(name.replace(['/', '-'], "_").to_uppercase())
                    .or_insert_with(|| vec![-1; columns.len()]);
                ids[column] = i16::try_from(info.protocol_id).expect("packet id fits i16");
            }
        }
    }
    rows
}

fn phase_modules(rows: &Rows, is_serverbound: bool) -> TokenStream {
    let aliases = aliases(is_serverbound);
    let empty = BTreeMap::new();
    let mut output = TokenStream::new();
    for state in STATES {
        let packets = rows.get(state).unwrap_or(&empty);
        let mut consts = TokenStream::new();
        for (name, ids) in packets {
            let ident = format_ident!("{name}");
            let ids = ids.iter().map(|id| Literal::i16_unsuffixed(*id));
            consts.extend(quote! {
                pub const #ident: super::super::PacketId = super::super::PacketId([#(#ids),*]);
            });
        }
        for (alias, target) in aliases.get(state).into_iter().flatten() {
            if packets.contains_key(*target) && !packets.contains_key(*alias) {
                let alias = format_ident!("{alias}");
                let target = format_ident!("{target}");
                consts.extend(quote! {
                    pub const #alias: super::super::PacketId = #target;
                });
            }
        }
        let state = format_ident!("{state}");
        output.extend(quote! {
            pub mod #state {
                #consts
            }
        });
    }
    output
}

/// Dense id tables per state and version; identical tables are emitted once.
fn lookup_tables(
    rows: &Rows,
    renames: &[(&str, &str, &str)],
    versions: usize,
    is_serverbound: bool,
) -> (TokenStream, TokenStream) {
    let current = versions - 1;
    let empty = BTreeMap::new();
    let prefix = if is_serverbound {
        "SERVERBOUND"
    } else {
        "CLIENTBOUND"
    };
    let mut unique: Vec<Vec<i16>> = Vec::new();
    let mut per_state = Vec::new();
    for state in STATES {
        let packets = rows.get(state).unwrap_or(&empty);
        let mut per_version = Vec::new();
        for column in 0..versions {
            let mut table = Vec::<i16>::new();
            for ids in packets.values() {
                let (current_id, client_id) = (ids[current], ids[column]);
                if current_id == -1 || client_id == -1 {
                    continue;
                }
                if is_serverbound {
                    set(&mut table, client_id, current_id);
                } else {
                    set(&mut table, current_id, client_id);
                }
            }
            for (_, now, older) in renames.iter().filter(|(s, ..)| *s == state) {
                let (Some(now), Some(older)) = (packets.get(*now), packets.get(*older)) else {
                    continue;
                };
                let (current_id, client_id) = (now[current], older[column]);
                if current_id == -1 || client_id == -1 {
                    continue;
                }
                if is_serverbound {
                    set(&mut table, client_id, current_id);
                } else {
                    set(&mut table, current_id, client_id);
                }
            }
            let index = unique.iter().position(|t| *t == table).unwrap_or_else(|| {
                unique.push(table);
                unique.len() - 1
            });
            let ident = format_ident!("{prefix}_{index}");
            per_version.push(quote!(&#ident));
        }
        per_state.push(quote!([#(#per_version),*]));
    }
    let statics = unique.iter().enumerate().map(|(index, table)| {
        let ident = format_ident!("{prefix}_{index}");
        let len = Literal::usize_unsuffixed(table.len());
        let table = table.iter().map(|id| Literal::i16_unsuffixed(*id));
        quote!(static #ident: [i16; #len] = [#(#table),*];)
    });
    (quote!(#(#per_state),*), quote!(#(#statics)*))
}

/// Fills `key` unless an earlier packet already claimed it.
fn set(table: &mut Vec<i16>, key: i16, value: i16) {
    let key = key as usize;
    if table.len() <= key {
        table.resize(key + 1, -1);
    }
    if table[key] == -1 {
        table[key] = value;
    }
}
