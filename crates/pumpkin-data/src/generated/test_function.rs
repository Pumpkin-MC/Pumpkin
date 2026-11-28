/* This file is generated. Do not edit manually. */
#[allow(clippy::too_many_lines)]
#[allow(clippy::match_same_arms)]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::match_single_binding)]
#[must_use]
pub fn get_test_function_source(path: &str) -> Option<&'static str> {
    match path {
        "pumpkin:test_environment_setup" => Some(include_str!(
            "../../../../assets/tests/datapacks/pumpkin-unit-test/data/pumpkin/function/test_environment_setup.mcfunction"
        )),
        "pumpkin:test_environment_teardown" => Some(include_str!(
            "../../../../assets/tests/datapacks/pumpkin-unit-test/data/pumpkin/function/test_environment_teardown.mcfunction"
        )),
        _ => None,
    }
}
#[must_use]
#[allow(clippy::too_many_lines, clippy::large_stack_arrays)]
pub const fn all_test_function_names() -> &'static [&'static str] {
    &[
        "pumpkin:test_environment_setup",
        "pumpkin:test_environment_teardown",
    ]
}
