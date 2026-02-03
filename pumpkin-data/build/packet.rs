use proc_macro2::TokenStream;
use pumpkin_util::version::MinecraftVersion;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

#[derive(Deserialize)]
pub struct Packets {
    version: u32,
    serverbound: BTreeMap<String, BTreeMap<String, i32>>,
    clientbound: BTreeMap<String, BTreeMap<String, i32>>,
}

pub(crate) fn build() -> TokenStream {
    // Paths to JSON assets (these are emitted as rerun-if-changed so Cargo rebuilds on changes)
    let paths = [
        "../assets/packet/1_21_11_packets.json",
        "../assets/packet/1_21_9_packets.json",
        "../assets/packet/1_21_7_packets.json",
    ];

    // Parse available packet files into a BTreeMap keyed by MinecraftVersion
    let mut versions: BTreeMap<MinecraftVersion, Packets> = BTreeMap::new();
    for path in paths {
        println!("cargo:rerun-if-changed={path}");
        let content = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read packet JSON file: {path}"));
        let parsed: Packets = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"));
        let version = MinecraftVersion::from_protocol(parsed.version);
        if versions.contains_key(&version) {
            panic!("Duplicate packet version {} in file {path}", parsed.version);
        }
        versions.insert(version, parsed);
    }

    // Choose the "latest" protocol baseline. This must be present in `versions`.
    let latest_version = MinecraftVersion::V_1_21_11;
    if !versions.contains_key(&latest_version) {
        panic!("Selected latest version {latest_version:?} was not among loaded packet files");
    }

    // Create identifiers for each version field based on MinecraftVersion::to_string()
    // We sanitize the string to create valid Rust identifiers but we base the name on to_string().
    let mut version_field_idents = Vec::new();
    for ver in versions.keys() {
        let raw = ver.to_string();
        let sanitized = sanitize_ident_from_string(&raw);
        // append `_id` to make intent explicit and avoid leading-digit issues
        let field_name = format!("{}_id", sanitized);
        let ident = format_ident!("{}", field_name);
        version_field_idents.push((*ver, raw, ident));
    }

    // Generate PacketId struct definition and impl blocks dynamically based on versions
    let packet_id_struct = generate_packet_id_struct(&version_field_idents, &latest_version);
    let serverbound_consts =
        generate_mapped_consts(&version_field_idents, &versions, &latest_version, true);
    let clientbound_consts =
        generate_mapped_consts(&version_field_idents, &versions, &latest_version, false);

    quote!(
        use pumpkin_util::version::MinecraftVersion;

        pub const CURRENT_MC_VERSION: MinecraftVersion = MinecraftVersion::V_1_21_11;
        pub const LOWEST_SUPPORTED_MC_VERSION: MinecraftVersion = MinecraftVersion::V_1_21_7;

        #packet_id_struct

        // We place the constants directly into these modules
        pub mod serverbound {
            #serverbound_consts
        }

        pub mod clientbound {
            #clientbound_consts
        }
    )
}

/// Sanitize a string obtained from `MinecraftVersion::to_string()` into a valid Rust identifier base.
///
/// This function:
/// - Replaces non-alphanumeric characters with underscores
/// - Collapses repeated underscores
/// - If the first character is not a valid identifier start, prefixes with `v_`
fn sanitize_ident_from_string(s: &str) -> String {
    // Replace non-alphanumeric with underscores
    let mut out = s
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();

    // Collapse repeated underscores
    while out.contains("__") {
        out = out.replace("__", "_");
    }

    // Trim leading/trailing underscores
    let out = out.trim_matches('_').to_string();

    // If empty or starts with a digit, prefix with "v_"
    let mut final_str = if out.is_empty() { "v".to_string() } else { out };

    if !final_str
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or(false)
    {
        final_str = format!("v_{}", final_str);
    }

    final_str
}

/// Generate the `PacketId` struct and impls (including `to_id`) dynamically based on available versions.
fn generate_packet_id_struct(
    version_field_idents: &[(MinecraftVersion, String, proc_macro2::Ident)],
    latest_version: &MinecraftVersion,
) -> TokenStream {
    // Build struct fields
    let mut struct_fields = TokenStream::new();
    for (_, _raw, ident) in version_field_idents {
        struct_fields.extend(quote! {
            pub #ident: i32,
        });
    }

    // Determine which ident corresponds to the latest version for equality impls
    let latest_field_ident = version_field_idents
        .iter()
        .find(|(v, _r, _i)| v == latest_version)
        .expect("latest version must be in version_field_idents")
        .2
        .clone();

    // Build match arms for to_id
    let mut match_arms = TokenStream::new();
    for (ver, _raw, ident) in version_field_idents {
        // Convert the version value into the enum variant ident (e.g. V_1_21_11)
        let variant_name = format!("{:?}", ver);
        let variant_ident = format_ident!("{}", variant_name);
        match_arms.extend(quote! {
            MinecraftVersion::#variant_ident => self.#ident,
        });
    }
    // default fallback: return latest
    match_arms.extend(quote! {
        _ => self.#latest_field_ident,
    });

    quote! {
        #[derive(Clone, Copy, Debug)]
        pub struct PacketId {
            #struct_fields
        }

        impl PacketId {
            /// Converts the requested protocol version into the corresponding packet ID.
            /// Returns -1 if the packet does not exist in that version.
            pub fn to_id(&self, version: MinecraftVersion) -> i32 {
                match version {
                    #match_arms
                }
            }
        }

        impl PartialEq<i32> for PacketId {
            fn eq(&self, other: &i32) -> bool {
                self.#latest_field_ident == *other
            }
        }

        impl PartialEq<PacketId> for i32 {
            fn eq(&self, other: &PacketId) -> bool {
                *self == other.#latest_field_ident
            }
        }
    }
}

/// Generate mapped constants where `latest_version` is considered the baseline.
/// `versions` must include the `latest_version` entry.
fn generate_mapped_consts(
    version_field_idents: &[(MinecraftVersion, String, proc_macro2::Ident)],
    versions: &BTreeMap<MinecraftVersion, Packets>,
    latest_version: &MinecraftVersion,
    is_serverbound: bool,
) -> TokenStream {
    let mut output = TokenStream::new();

    // Latest = baseline (use provided latest_version)
    let latest = versions
        .get(latest_version)
        .expect("Latest version not found in versions map");

    let latest_phases = if is_serverbound {
        &latest.serverbound
    } else {
        &latest.clientbound
    };

    for (phase, packets) in latest_phases {
        for name in packets.keys() {
            let sanitized_name = name.replace(['/', '-'], "_").to_uppercase();
            let const_name = format_ident!("{}_{}", phase.to_uppercase(), sanitized_name);

            // Build initialization values for each version field (in the same order as version_field_idents)
            let mut init_pairs = TokenStream::new();
            for (ver, _raw, field_ident) in version_field_idents {
                let map = versions
                    .get(ver)
                    .expect("version missing from versions map");
                let phase_map = if is_serverbound {
                    &map.serverbound
                } else {
                    &map.clientbound
                };
                let id = phase_map
                    .get(phase)
                    .and_then(|p| p.get(name))
                    .copied()
                    .unwrap_or(-1);
                init_pairs.extend(quote! {
                    #field_ident: #id,
                });
            }

            output.extend(quote! {
                pub const #const_name: super::PacketId = super::PacketId {
                    #init_pairs
                };
            });
        }
    }

    output
}
