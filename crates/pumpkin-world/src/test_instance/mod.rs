//! Embedded `GameTest` test-instance resources.
//!
//! Test instances are datapack registry entries stored under
//! `data/<namespace>/test_instance/*.json`. They are separate from structure
//! templates: a test instance references a structure, but is not itself a
//! structure template and does not belong in [`crate::generation::structure::template::TemplateCache`].

/// Returns the raw JSON for an embedded test instance.
///
/// `id` may be bare (`foo`) or namespaced (`minecraft:foo`, `pumpkin:foo`).
#[must_use]
pub fn json(id: &str) -> Option<&'static str> {
    pumpkin_data::test_instance::get_test_instance_json(id)
}

/// Returns all embedded test-instance resource ids.
///
/// Names are fully qualified, for example `pumpkin:creeper_should_run_from_cat`.
#[must_use]
pub const fn all_names() -> &'static [&'static str] {
    pumpkin_data::test_instance::all_test_instance_names()
}
