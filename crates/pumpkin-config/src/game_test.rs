use serde::{Deserialize, Serialize};

/// This configuration controls the behavior of the Pumpkin `GameTest` framework.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct GameTestConfig {
    /// The name of the example datapack to load for example tests.
    pub example_pack_name: String,
    /// Whether to load example tests from the `pumpkin-unit-test-example` datapack.    
    pub load_example_tests: bool,
}

impl Default for GameTestConfig {
    fn default() -> Self {
        Self {
            example_pack_name: "pumpkin-unit-test-example".to_string(),
            load_example_tests: true,
        }
    }
}
