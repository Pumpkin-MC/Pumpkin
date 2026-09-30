use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

use crate::version::JavaMinecraftVersion;

/// The newest protocol version used as the fallback for unknown versions in `PacketId::to_id`.
const LATEST_VERSION: JavaMinecraftVersion = JavaMinecraftVersion::V_26_3;

/// Represents the protocol_id object within the JSON.
#[derive(Deserialize)]
pub struct PacketInfo {
    pub protocol_id: i32,
}

/// Represents the mapping from packet direction (serverbound / clientbound) to packets.
#[derive(Deserialize, Default)]
pub struct PhaseData {
    #[serde(default)]
    pub serverbound: BTreeMap<String, PacketInfo>,
    #[serde(default)]
    pub clientbound: BTreeMap<String, PacketInfo>,
}

/// Raw deserialization shape for a single versioned packet mapping file.
#[derive(Deserialize)]
pub struct Packets(pub BTreeMap<String, PhaseData>);

/// Compact dump used by `assets/packets.json` / `26_3_packets.json`.
#[derive(Deserialize)]
struct CompactPackets {
    #[serde(default)]
    serverbound: BTreeMap<String, BTreeMap<String, i32>>,
    #[serde(default)]
    clientbound: BTreeMap<String, BTreeMap<String, i32>>,
}

fn canonicalize_phase(phase: &str) -> String {
    if phase == "config" {
        "configuration".to_string()
    } else {
        phase.to_string()
    }
}

fn canonicalize_packet_name(name: String) -> String {
    if name.contains(':') {
        name
    } else {
        format!("minecraft:{name}")
    }
}

fn parse_packets(path: &str, content: &str) -> Packets {
    if let Ok(parsed) = serde_json::from_str::<Packets>(content) {
        return parsed;
    }

    let compact: CompactPackets =
        serde_json::from_str(content).unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"));

    let mut phases = BTreeMap::<String, PhaseData>::new();
    for (phase, packets) in compact.serverbound {
        let phase = canonicalize_phase(&phase);
        let entry = phases.entry(phase).or_default();
        entry.serverbound = packets
            .into_iter()
            .map(|(name, id)| {
                (
                    canonicalize_packet_name(name),
                    PacketInfo { protocol_id: id },
                )
            })
            .collect();
    }
    for (phase, packets) in compact.clientbound {
        let phase = canonicalize_phase(&phase);
        let entry = phases.entry(phase).or_default();
        entry.clientbound = packets
            .into_iter()
            .map(|(name, id)| {
                (
                    canonicalize_packet_name(name),
                    PacketInfo { protocol_id: id },
                )
            })
            .collect();
    }

    Packets(phases)
}

/// Generates the `TokenStream` for the `PacketId` struct, `CURRENT_MC_VERSION`, and
/// all `serverbound`/`clientbound` packet ID constants.
pub(crate) fn build() -> TokenStream {
    let assets = [(JavaMinecraftVersion::V_26_3, "26_3_packets.json")];

    // Parse available packet files into a BTreeMap keyed by JavaMinecraftVersion
    let mut versions = BTreeMap::new();
    for (ver, file) in assets {
        let path = "../../assets/packets.json".to_string();

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Failed to read packet JSON file: {path}"));
        let parsed = parse_packets(&path, &content);

        versions.insert(ver, parsed);
    }

    // Generate PacketId struct definition and impl blocks dynamically based on versions
    let packet_id_struct = generate_struct(&versions);
    let serverbound_modules = generate_phase_modules(&versions, true);
    let clientbound_modules = generate_phase_modules(&versions, false);

    quote!(
        use pumpkin_util::version::JavaMinecraftVersion;

        pub const CURRENT_MC_VERSION: JavaMinecraftVersion = #LATEST_VERSION;
        pub const LOWEST_SUPPORTED_MC_VERSION: JavaMinecraftVersion = #LATEST_VERSION;

        #packet_id_struct

        // We place the constants directly into these phase modules
        pub mod serverbound {
            #serverbound_modules
        }

        pub mod clientbound {
            #clientbound_modules
        }
    )
}

/// Generate the `PacketId` struct and impls.
fn generate_struct<T>(_versions: &BTreeMap<JavaMinecraftVersion, T>) -> TokenStream {
    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct PacketId(pub i32);

        impl PacketId {
            /// Converts the requested protocol version into the corresponding packet ID.
            #[must_use]
            pub const fn to_id(&self, _version: JavaMinecraftVersion) -> i32 {
                self.0
            }
        }

        impl PartialEq<i32> for PacketId {
            fn eq(&self, other: &i32) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<PacketId> for i32 {
            fn eq(&self, other: &PacketId) -> bool {
                *self == other.0
            }
        }

        impl From<PacketId> for i32 {
            fn from(id: PacketId) -> Self {
                id.0
            }
        }

        impl From<i32> for PacketId {
            fn from(id: i32) -> Self {
                Self(id)
            }
        }
    }
}

/// Generates modular phase submodules and `PacketId` constants for each phase.
fn generate_phase_modules(
    versions: &BTreeMap<JavaMinecraftVersion, Packets>,
    is_serverbound: bool,
) -> TokenStream {
    // phase -> const_name -> (ver -> id)
    let mut phase_packets =
        BTreeMap::<String, BTreeMap<String, BTreeMap<&JavaMinecraftVersion, i32>>>::new();

    for (ver, Packets(phases)) in versions {
        for (phase, phase_data) in phases {
            let packets = if is_serverbound {
                &phase_data.serverbound
            } else {
                &phase_data.clientbound
            };
            let phase_module = if phase == "configuration" {
                "config"
            } else {
                phase.as_str()
            };
            for (full_name, info) in packets {
                let name = full_name.strip_prefix("minecraft:").unwrap_or(full_name);
                let sanitized_name = name.replace(['/', '-'], "_").to_uppercase();
                phase_packets
                    .entry(phase_module.to_string())
                    .or_default()
                    .entry(sanitized_name)
                    .or_default()
                    .insert(ver, info.protocol_id);
            }
        }
    }

    // Define aliases per phase for backwards compatibility and PacketEvents parity
    let mut aliases: BTreeMap<&str, Vec<(&str, &str)>> = BTreeMap::new();
    if is_serverbound {
        aliases.insert(
            "handshake",
            vec![("HANDSHAKE", "INTENTION"), ("HANDSHAKING", "INTENTION")],
        );
        aliases.insert(
            "login",
            vec![
                ("LOGIN_START", "HELLO"),
                ("ENCRYPTION_RESPONSE", "KEY"),
                ("LOGIN_PLUGIN_RESPONSE", "CUSTOM_QUERY_ANSWER"),
            ],
        );
        aliases.insert(
            "play",
            vec![
                ("CHAT_MESSAGE", "CHAT"),
                ("TELEPORT_CONFIRM", "ACCEPT_TELEPORTATION"),
                ("SELECT_BUNDLE_ITEM", "BUNDLE_ITEM_SELECTED"),
                ("SET_DIFFICULTY", "CHANGE_DIFFICULTY"),
                ("CHUNK_BATCH_ACK", "CHUNK_BATCH_RECEIVED"),
                ("CLICK_CONTAINER_BUTTON", "CONTAINER_BUTTON_CLICK"),
                ("CLICK_CONTAINER", "CONTAINER_CLICK"),
                ("SLOT_STATE_CHANGE", "CONTAINER_SLOT_STATE_CHANGED"),
                ("INTERACT_ENTITY", "INTERACT"),
                ("GENERATE_STRUCTURE", "JIGSAW_GENERATE"),
                ("PLAYER_POSITION", "MOVE_PLAYER_POS"),
                ("PLAYER_POSITION_ROTATION", "MOVE_PLAYER_POS_ROT"),
                ("PLAYER_POSITION_AND_ROTATION", "MOVE_PLAYER_POS_ROT"),
                ("PLAYER_ROTATION", "MOVE_PLAYER_ROT"),
                ("PLAYER_FLYING", "MOVE_PLAYER_STATUS_ONLY"),
                ("STEER_BOAT", "PADDLE_BOAT"),
                ("PLAYER_DIGGING", "PLAYER_ACTION"),
                ("ENTITY_ACTION", "PLAYER_COMMAND"),
                ("SWING_ARM", "PUNCH"),
                ("ANIMATION", "PUNCH"),
                ("SWING", "PUNCH"),
                ("DEBUG_SAMPLE_SUBSCRIPTION", "DEBUG_SUBSCRIPTION_REQUEST"),
                ("PLAYER_BLOCK_PLACEMENT", "USE_ITEM_ON"),
                ("SPECTATE", "SPECTATOR_ACTION"),
                ("SPECTATE_ENTITY", "SPECTATOR_ACTION"),
            ],
        );
    } else {
        aliases.insert(
            "login",
            vec![
                ("LOGIN_SUCCESS", "LOGIN_FINISHED"),
                ("GAME_PROFILE", "LOGIN_FINISHED"),
                ("SET_COMPRESSION", "LOGIN_COMPRESSION"),
                ("LOGIN_DISCONNECT", "LOGIN_DISCONNECT"),
                ("ENCRYPTION_REQUEST", "HELLO"),
                ("LOGIN_PLUGIN_REQUEST", "CUSTOM_QUERY"),
            ],
        );
        aliases.insert(
            "play",
            vec![
                ("SET_CARRIED_ITEM", "SET_CURSOR_ITEM"),
                ("CHAT", "SYSTEM_CHAT"),
                ("BUNDLE", "BUNDLE_DELIMITER"),
                ("SPAWN_ENTITY", "ADD_ENTITY"),
                ("ENTITY_ANIMATION", "ANIMATE"),
                ("STATISTICS", "AWARD_STATS"),
                ("ACKNOWLEDGE_BLOCK_CHANGES", "BLOCK_CHANGED_ACK"),
                ("BLOCK_BREAK_ANIMATION", "BLOCK_DESTRUCTION"),
                ("BLOCK_ACTION", "BLOCK_EVENT"),
                ("BLOCK_CHANGE", "BLOCK_UPDATE"),
                ("BOSS_BAR", "BOSS_EVENT"),
                ("SERVER_DIFFICULTY", "CHANGE_DIFFICULTY"),
                ("CHUNK_BATCH_END", "CHUNK_BATCH_FINISHED"),
                ("CHUNK_BATCH_BEGIN", "CHUNK_BATCH_START"),
                ("TAB_COMPLETE", "COMMAND_SUGGESTIONS"),
                ("DECLARE_COMMANDS", "COMMANDS"),
                ("CLOSE_WINDOW", "CONTAINER_CLOSE"),
                ("WINDOW_ITEMS", "CONTAINER_SET_CONTENT"),
                ("WINDOW_PROPERTY", "CONTAINER_SET_DATA"),
                ("SET_SLOT", "CONTAINER_SET_SLOT"),
                ("PLUGIN_MESSAGE", "CUSTOM_PAYLOAD"),
                ("ENTITY_STATUS", "ENTITY_EVENT"),
                ("EXPLOSION", "EXPLODE"),
                ("UNLOAD_CHUNK", "FORGET_LEVEL_CHUNK"),
                ("CHANGE_GAME_STATE", "GAME_EVENT"),
                ("OPEN_HORSE_WINDOW", "MOUNT_SCREEN_OPEN"),
                ("INITIALIZE_WORLD_BORDER", "INITIALIZE_BORDER"),
                ("CHUNK_DATA", "LEVEL_CHUNK_WITH_LIGHT"),
                ("EFFECT", "LEVEL_EVENT"),
                ("PARTICLE", "LEVEL_PARTICLES"),
                ("JOIN_GAME", "LOGIN"),
                ("MAP_DATA", "MAP_ITEM_DATA"),
                ("ENTITY_RELATIVE_MOVE", "MOVE_ENTITY_POS"),
                ("ENTITY_RELATIVE_MOVE_AND_ROTATION", "MOVE_ENTITY_POS_ROT"),
                ("MOVE_MINECART", "MOVE_MINECART_ALONG_TRACK"),
                ("ENTITY_ROTATION", "MOVE_ENTITY_ROT"),
                ("OPEN_WINDOW", "OPEN_SCREEN"),
                ("DEBUG_PONG", "PONG_RESPONSE"),
                ("CRAFT_RECIPE_RESPONSE", "PLACE_GHOST_RECIPE"),
                ("CHAT_MESSAGE", "PLAYER_CHAT"),
                ("FACE_PLAYER", "PLAYER_LOOK_AT"),
                ("PLAYER_POSITION_AND_LOOK", "PLAYER_POSITION"),
                ("DESTROY_ENTITIES", "REMOVE_ENTITIES"),
                ("REMOVE_ENTITY_EFFECT", "REMOVE_MOB_EFFECT"),
                ("RESOURCE_PACK_REMOVE", "RESOURCE_PACK_POP"),
                ("RESOURCE_PACK_SEND", "RESOURCE_PACK_PUSH"),
                ("ENTITY_HEAD_LOOK", "ROTATE_HEAD"),
                ("MULTI_BLOCK_CHANGE", "SECTION_BLOCKS_UPDATE"),
                ("ACTION_BAR", "SET_ACTION_BAR_TEXT"),
                ("WORLD_BORDER_CENTER", "SET_BORDER_CENTER"),
                ("WORLD_BORDER_LERP_SIZE", "SET_BORDER_LERP_SIZE"),
                ("WORLD_BORDER_SIZE", "SET_BORDER_SIZE"),
                ("WORLD_BORDER_WARNING_DELAY", "SET_BORDER_WARNING_DELAY"),
                ("WORLD_BORDER_WARNING_REACH", "SET_BORDER_WARNING_DISTANCE"),
                ("UPDATE_VIEW_POSITION", "SET_CHUNK_CACHE_CENTER"),
                ("UPDATE_VIEW_DISTANCE", "SET_CHUNK_CACHE_RADIUS"),
                ("SPAWN_POSITION", "SET_DEFAULT_SPAWN_POSITION"),
                ("DISPLAY_SCOREBOARD", "SET_DISPLAY_OBJECTIVE"),
                ("ENTITY_METADATA", "SET_ENTITY_DATA"),
                ("ATTACH_ENTITY", "SET_ENTITY_LINK"),
                ("ENTITY_VELOCITY", "SET_ENTITY_MOTION"),
                ("ENTITY_EQUIPMENT", "SET_EQUIPMENT"),
                ("UPDATE_HEALTH", "SET_HEALTH"),
                ("HELD_ITEM_CHANGE", "SET_HELD_SLOT"),
                ("SCOREBOARD_OBJECTIVE", "SET_OBJECTIVE"),
                ("UPDATE_SCORE", "SET_SCORE"),
                ("UPDATE_SIMULATION_DISTANCE", "SET_SIMULATION_DISTANCE"),
                ("SET_TITLE_SUBTITLE", "SET_SUBTITLE_TEXT"),
                ("TIME_UPDATE", "SET_TIME"),
                ("SET_TITLE_TIMES", "SET_TITLES_ANIMATION"),
                ("ENTITY_SOUND_EFFECT", "SOUND_ENTITY"),
                ("SOUND_EFFECT", "SOUND"),
                ("CONFIGURATION_START", "START_CONFIGURATION"),
                ("SYSTEM_CHAT_MESSAGE", "SYSTEM_CHAT"),
                ("PLAYER_LIST_HEADER_AND_FOOTER", "TAB_LIST"),
                ("NBT_QUERY_RESPONSE", "TAG_QUERY"),
                ("COLLECT_ITEM", "TAKE_ITEM_ENTITY"),
                ("ENTITY_EFFECT", "UPDATE_MOB_EFFECT"),
                ("DECLARE_RECIPES", "UPDATE_RECIPES"),
                ("TAGS", "UPDATE_TAGS"),
            ],
        );
    }

    let mut output = TokenStream::new();
    let expected_phases = vec!["handshake", "status", "login", "config", "play"];

    for phase_name in expected_phases {
        let phase_ident = format_ident!("{}", phase_name);
        let mut consts_ts = TokenStream::new();
        let empty_map = BTreeMap::new();
        let packets_in_phase = phase_packets.get(phase_name).unwrap_or(&empty_map);

        for (name, values) in packets_in_phase {
            let id = values.get(&LATEST_VERSION).copied().unwrap_or(-1);
            let const_name = format_ident!("{}", name);
            consts_ts.extend(quote! {
                pub const #const_name: super::super::PacketId = super::super::PacketId(#id);
            });
        }

        // Add aliases for this phase
        if let Some(alias_list) = aliases.get(phase_name) {
            for (alias, target) in alias_list {
                let alias_ident = format_ident!("{}", alias);
                let target_ident = format_ident!("{}", target);
                if packets_in_phase.contains_key(*target) && !packets_in_phase.contains_key(*alias)
                {
                    consts_ts.extend(quote! {
                        pub const #alias_ident: super::super::PacketId = #target_ident;
                    });
                }
            }
        }

        output.extend(quote! {
            pub mod #phase_ident {
                #consts_ts
            }
        });
    }

    output
}
