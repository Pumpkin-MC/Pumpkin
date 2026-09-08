//! Embedded `GameTest` test-instance resources.
//!
//! Test instances are datapack registry entries stored under
//! `data/<namespace>/test_instance/*.json`. They are separate from structure
//! templates: a test instance references a structure, but is not itself a
//! structure template and does not belong in [`crate::generation::structure::template::TemplateCache`].

/// Vanilla's implicit namespace.
const DEFAULT_NAMESPACE: &str = "minecraft";

/// Canonicalizes a resource id to fully-qualified `namespace:path` form.
fn canonicalize(name: &str) -> String {
    if name.contains(':') {
        name.to_owned()
    } else {
        format!("{DEFAULT_NAMESPACE}:{name}")
    }
}

include!(concat!(env!("OUT_DIR"), "/test_instance_embeddings.rs"));

/// Returns the raw JSON for an embedded test instance.
///
/// `id` may be bare (`foo`) or namespaced (`minecraft:foo`, `pumpkin:foo`).
#[must_use]
pub fn json(id: &str) -> Option<&'static str> {
    get_test_instance_json(&canonicalize(id))
}

/// Returns all embedded test-instance resource ids.
///
/// Names are fully qualified, for example `pumpkin:creeper_should_run_from_cat`.
#[must_use]
#[allow(clippy::used_underscore_items)]
pub const fn all_names() -> &'static [&'static str] {
    _generated_all_test_instance_names()
}
