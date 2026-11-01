use std::path::Path;

use pumpkin_config::op;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{LoadJSONConfiguration, SaveJSONConfiguration};

#[derive(Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct OperatorConfig {
    pub ops: Vec<op::Op>,
}

impl OperatorConfig {
    #[must_use]
    pub fn get_entry(&self, uuid: &Uuid) -> Option<&op::Op> {
        self.ops.iter().find(|entry| entry.uuid.eq(uuid))
    }
}

impl LoadJSONConfiguration for OperatorConfig {
    fn get_path() -> &'static Path {
        Path::new("ops.json")
    }
    fn validate(&self) {
        // TODO: Validate the operator configuration
    }
}

impl SaveJSONConfiguration for OperatorConfig {}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::PermissionLvl;

    const TEST_UUID: &str = "00000000-0000-0000-0000-000000000001";
    const SECOND_TEST_UUID: &str = "00000000-0000-0000-0000-000000000002";

    #[test]
    fn loads_vanilla_and_legacy_operator_entries() {
        for entry in [
            format!(
                r#"[{{"uuid":"{TEST_UUID}","name":"Operator","level":4,"bypassesPlayerLimit":true}}]"#
            ),
            format!(
                r#"[{{"uuid":"{TEST_UUID}","name":"Operator","level":4,"bypasses_player_limit":true}}]"#
            ),
        ] {
            let config: OperatorConfig = serde_json::from_str(&entry).unwrap();

            assert_eq!(config.ops.len(), 1);
            assert_eq!(config.ops[0].uuid, TEST_UUID.parse::<Uuid>().unwrap());
            assert_eq!(config.ops[0].level, PermissionLvl::Four);
            assert!(config.ops[0].bypasses_player_limit);
        }
    }

    #[test]
    fn defaults_missing_level_and_bypasses_player_limit() {
        let config: OperatorConfig = serde_json::from_str(&format!(
            r#"[
                {{"uuid":"{TEST_UUID}","name":"Operator","level":4}},
                {{"uuid":"{SECOND_TEST_UUID}","name":"Operator","bypassesPlayerLimit":true}}
            ]"#
        ))
        .unwrap();

        assert_eq!(config.ops.len(), 2);
        assert_eq!(config.ops[0].level, PermissionLvl::Four);
        assert!(!config.ops[0].bypasses_player_limit);
        assert_eq!(config.ops[1].level, PermissionLvl::Zero);
        assert!(config.ops[1].bypasses_player_limit);
    }
}
