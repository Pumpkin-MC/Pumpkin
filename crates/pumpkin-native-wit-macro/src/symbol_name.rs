//! Symbol name encoding matching `wit-bindgen`'s native target conventions.

use wit_bindgen_core::symbol_name::make_external_component;
use wit_parser::{Function, Resolve, WorldId, WorldKey};

pub fn encode_component(s: &str) -> String {
    make_external_component(s)
}

pub fn import_symbol(module: &str, name: &str) -> String {
    if module.is_empty() || module == "$root" {
        encode_component(name)
    } else {
        format!("{}X00{}", encode_component(module), encode_component(name))
    }
}

pub fn func_import_symbol(resolve: &Resolve, key: &WorldKey, func: &Function) -> String {
    import_symbol(&resolve.name_world_key(key), &func.name)
}

pub fn canonical_export_name(module: Option<&str>, func_name: &str) -> String {
    module.map_or_else(
        || func_name.to_string(),
        |module| format!("{module}#{func_name}"),
    )
}

pub fn export_symbol(canonical_name: &str) -> String {
    encode_component(canonical_name)
}

pub fn post_return_symbol(canonical_name: &str) -> String {
    encode_component(&format!("cabi_post_{canonical_name}"))
}

pub fn export_field(canonical_name: &str) -> String {
    format!("__export_{}", export_symbol(canonical_name))
}

pub fn post_return_field(canonical_name: &str) -> String {
    format!("__post_return_{}", export_symbol(canonical_name))
}

/// Fully qualified `package/world` name, e.g. `pumpkin:plugin@0.1.0/plugin`.
pub fn world_wit_name(resolve: &Resolve, world: WorldId) -> String {
    let world = &resolve.worlds[world];
    let package = world
        .package
        .map(|id| resolve.packages[id].name.to_string())
        .unwrap_or_default();
    format!("{package}/{}", world.name)
}

/// World metadata symbol emitted by `wit-bindgen` on non-wasm targets.
pub fn world_metadata_symbol(resolve: &Resolve, world: WorldId) -> String {
    format!(
        "__wit_bindgen_world_{}",
        encode_component(&world_wit_name(resolve, world))
    )
}
