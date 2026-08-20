use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use std::fs;

/// Generates the `TokenStream` for the `DataComponent` enum and its ID/name conversion methods.
pub fn build() -> TokenStream {
    let data_component: BTreeMap<String, u8> =
        serde_json::from_str(&fs::read_to_string("../../assets/data_component.json").unwrap())
            .expect("Failed to parse data_component.json");

    // Wire ids for the `minecraft:data_component_type` registry are not stable across protocol
    // versions the way most other registries are - unlike packets, they have no per-version JSON
    // dump in `assets/`, so this table has historically only tracked the latest (26.x) numbering.
    // Components inserted since then shift the ids of everything after their insertion point,
    // which silently corrupts decoding of any explicit component patch (e.g. a firework rocket's
    // `minecraft:fireworks`) sent by an older client. This legacy table (sourced from the
    // `minecraft:data_component_type` registry actually used by 1.21.11-and-earlier clients -
    // stable since the last data component addition before the 26.x snapshots) lets callers that
    // know they're talking to a pre-26.1 client translate correctly.
    let legacy_component: BTreeMap<String, u8> = serde_json::from_str(
        &fs::read_to_string("../../assets/data_component_legacy_1_21_11.json").unwrap(),
    )
    .expect("Failed to parse data_component_legacy_1_21_11.json");

    let mut enum_variants = TokenStream::new();
    let mut id_to_enum = TokenStream::new();
    let mut enum_to_name = TokenStream::new();
    let mut name_to_enum = TokenStream::new();
    let mut legacy_id_to_enum = TokenStream::new();
    let mut enum_to_legacy_id = TokenStream::new();
    let mut data_component_vec = data_component.iter().collect::<Vec<_>>();
    data_component_vec.sort_by_key(|(_, i)| **i);

    for (raw_name, raw_value) in &data_component_vec {
        let strip_name = raw_name
            .strip_prefix("minecraft:")
            .unwrap()
            .replace('/', "_");
        let pascal_case = format_ident!("{}", strip_name.to_pascal_case());

        // Enum variant

        enum_variants.extend(quote! {
            #pascal_case = #raw_value,
        });

        id_to_enum.extend(quote! {
            #raw_value => Some(Self::#pascal_case),
        });

        // TODO use phf
        name_to_enum.extend(quote! {
            #raw_name | #strip_name => Some(Self::#pascal_case),
        });

        // Enum -> &str
        enum_to_name.extend(quote! {
            Self::#pascal_case => #raw_name,
        });

        if let Some(legacy_id) = legacy_component.get(raw_name.as_str()) {
            legacy_id_to_enum.extend(quote! {
                #legacy_id => Some(Self::#pascal_case),
            });
            enum_to_legacy_id.extend(quote! {
                Self::#pascal_case => Some(#legacy_id),
            });
        } else {
            enum_to_legacy_id.extend(quote! {
                Self::#pascal_case => None,
            });
        }
    }

    quote! {
        use crate::data_component_impl::*;

        #[derive(Copy, Clone, Hash, PartialEq, Eq)]
        #[repr(u8)]
        pub enum DataComponent {
            #enum_variants
        }

        impl DataComponent {
            #[must_use]
            pub const fn to_id(self) -> u8 {
                self as u8
            }

            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub const fn try_from_id(id: u8) -> Option<Self> {
                match id {
                    #id_to_enum
                    _ => None,
                }
            }

            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub fn try_from_name(name: &str) -> Option<Self> {
                match name {
                    #name_to_enum
                    _ => None,
                }
            }

            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub const fn to_name(self) -> &'static str {
                match self {
                    #enum_to_name
                }
            }

            /// Resolves a wire component id using the legacy (pre-26.1, e.g. 1.21.11 and
            /// earlier) numbering, for decoding components sent by an older client.
            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub const fn try_from_id_legacy(id: u8) -> Option<Self> {
                match id {
                    #legacy_id_to_enum
                    _ => None,
                }
            }

            /// Returns this component's wire id under the legacy (pre-26.1, e.g. 1.21.11 and
            /// earlier) numbering, or `None` if the component didn't exist in that scheme yet.
            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub const fn to_id_legacy(self) -> Option<u8> {
                match self {
                    #enum_to_legacy_id
                }
            }
        }
    }
}
