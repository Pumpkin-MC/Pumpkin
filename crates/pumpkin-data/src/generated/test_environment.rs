/* This file is generated. Do not edit manually. */
#[allow(clippy::too_many_lines)]
#[allow(clippy::match_same_arms)]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::match_single_binding)]
#[must_use]
pub fn get_test_environment_json(path: &str) -> Option<&'static str> {
    match path {
        "minecraft:default" | "default" => Some(include_str!(
            "../../../../assets/datapack/data/minecraft/test_environment/default.json"
        )),
        "pumpkin:environment_with_function" => Some(include_str!(
            "../../../../assets/tests/datapacks/pumpkin-unit-test/data/pumpkin/test_environment/environment_with_function.json"
        )),
        _ => None,
    }
}
#[must_use]
#[allow(clippy::too_many_lines, clippy::large_stack_arrays)]
pub const fn all_test_environment_names() -> &'static [&'static str] {
    &["minecraft:default", "pumpkin:environment_with_function"]
}
