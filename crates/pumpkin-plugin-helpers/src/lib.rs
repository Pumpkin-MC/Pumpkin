//! Language-agnostic helpers for plugins
//!
//! ## Usage
//!
//! Build for `wasm32-wasip2`, then use `pumpkin-plugin-compose` to embed this component
//! into a plugin that imports `pumpkin:helpers/registry-keys@0.1.0`.

mod generated;

#[allow(clippy::all, clippy::pedantic, clippy::nursery, missing_docs)]
mod bindings {
    wit_bindgen::generate!({path: "../pumpkin-plugin-wit/helpers", world: "helpers"});
}

struct Helpers;

fn qualify(key: &str) -> String {
    if key.contains(':') {
        key.to_string()
    } else {
        format!("minecraft:{key}")
    }
}

fn lookup(names: &[&str], name: &str) -> Option<String> {
    let key = qualify(name);
    names.binary_search(&key.as_str()).ok().map(|_| key)
}

impl bindings::exports::pumpkin::helpers::registry_keys::Guest for Helpers {
    fn qualify_key(key: String) -> String {
        qualify(&key)
    }
    fn item_key(name: String) -> Option<String> {
        lookup(generated::ITEM_KEYS, &name)
    }
    fn block_key(name: String) -> Option<String> {
        lookup(generated::BLOCK_KEYS, &name)
    }
}

#[cfg(target_family = "wasm")]
bindings::export!(Helpers with_types_in bindings);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_keys_accept_qualified_and_unqualified_names() {
        for name in ["diamond", "minecraft:diamond"] {
            assert_eq!(
                lookup(generated::ITEM_KEYS, name).as_deref(),
                Some("minecraft:diamond")
            );
        }
        assert_eq!(
            lookup(generated::BLOCK_KEYS, "stone").as_deref(),
            Some("minecraft:stone")
        );
        assert_eq!(lookup(generated::ITEM_KEYS, "custom:diamond"), None);
        assert_eq!(lookup(generated::BLOCK_KEYS, "Minecraft:stone"), None);
        assert_eq!(lookup(generated::ITEM_KEYS, ""), None);
        assert_eq!(qualify("custom:item"), "custom:item");
    }

    #[test]
    fn lookup_tables_are_sorted_and_unique() {
        for keys in [generated::ITEM_KEYS, generated::BLOCK_KEYS] {
            assert!(!keys.is_empty());
            assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
            assert!(keys.iter().all(|key| key.starts_with("minecraft:")));
        }
    }
}
