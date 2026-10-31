//! Thin Rust views of keys resolved by the shared helper component.

#[allow(clippy::all, clippy::pedantic, clippy::nursery, missing_docs)]
mod bindings {
    wit_bindgen::generate!({
        path: "../pumpkin-plugin-wit/helpers",
        world: "helper-client",
        type_section_suffix: "pumpkin-helpers",
    });
}

use bindings::pumpkin::helpers::registry_keys;
pub use bindings::pumpkin::helpers::registry_keys::qualify_key as qualify;

macro_rules! known_key {
    ($name:ident, $lookup:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            /// Resolves a qualified or unqualified name using the helper component's data.
            /// Returns `None` for unknown names, including custom registry entries.
            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                registry_keys::$lookup(name).map(Self)
            }

            /// Resolves a registry key using the helper component's data.
            #[must_use]
            pub fn from_registry_key(key: &str) -> Option<Self> {
                Self::from_name(key)
            }

            /// Returns the unqualified vanilla name.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.strip_prefix("minecraft:").unwrap_or(&self.0)
            }

            /// Returns the canonical namespaced registry key.
            #[must_use]
            pub const fn resource_location(&self) -> &str {
                self.0.as_str()
            }

            pub(crate) fn into_registry_key(self) -> String {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.resource_location())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}

known_key!(
    Item,
    item_key,
    "A known item key from the shared helper component's extracted data."
);
known_key!(
    BlockType,
    block_key,
    "A known block key from the shared helper component's extracted data."
);
