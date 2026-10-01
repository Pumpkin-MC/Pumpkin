use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{collections::BTreeMap, fs, sync::LazyLock};

type DataComponentRegistry = BTreeMap<String, u8>;

static DATA_COMPONENT_REGISTRY: LazyLock<Result<DataComponentRegistry, String>> =
    LazyLock::new(|| {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/data_component.json");
        let source = fs::read_to_string(path)
            .map_err(|error| format!("failed to read data_component.json: {error}"))?;
        serde_json::from_str(&source)
            .map_err(|error| format!("failed to parse data_component.json: {error}"))
    });

/// Reports whether `name` is an entry in the generated data-component registry.
///
/// The registry is loaded once from the same asset used to generate `DataComponent`,
/// so unknown and custom-namespace names cannot be accepted by a separate list. The
/// result preserves I/O and parse failures for the caller to report.
#[must_use]
pub(crate) fn is_registered_component(name: &str) -> Result<bool, String> {
    DATA_COMPONENT_REGISTRY
        .as_ref()
        .map(|registry| registry.contains_key(name))
        .map_err(Clone::clone)
}

/// Generates the `TokenStream` for the `DataComponent` enum and its ID/name conversion methods.
pub fn build() -> TokenStream {
    let data_component: BTreeMap<String, u8> =
        serde_json::from_str(&fs::read_to_string("../../assets/data_component.json").unwrap())
            .expect("Failed to parse data_component.json");

    let mut enum_variants = TokenStream::new();
    let mut id_to_enum = TokenStream::new();
    let mut enum_to_name = TokenStream::new();
    let mut name_to_enum = TokenStream::new();
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
    }

    quote! {
        use crate::data_component_impl::*;

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_registered_component;

    /// Canonical Minecraft component names are accepted by the registry lookup.
    #[test]
    fn recognizes_registered_components() {
        assert_eq!(is_registered_component("minecraft:custom_name"), Ok(true));
        assert_eq!(is_registered_component("minecraft:container"), Ok(true));
    }

    /// Unknown names and names from another namespace produce successful negative lookups.
    #[test]
    fn rejects_unknown_and_custom_namespace_components() {
        assert_eq!(
            is_registered_component("minecraft:not_a_component"),
            Ok(false)
        );
        assert_eq!(is_registered_component("example:custom_name"), Ok(false));
        assert_eq!(is_registered_component("custom_name"), Ok(false));
    }
}
