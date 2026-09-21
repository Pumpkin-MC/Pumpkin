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

    #[test]
    fn loads_vanilla_camel_case_operator_entries() {
        let config: OperatorConfig = serde_json::from_str(
            r#"[{"uuid":"00000000-0000-0000-0000-000000000001","name":"Operator","level":4,"bypassesPlayerLimit":false}]"#,
        )
        .unwrap();

        assert_eq!(config.ops.len(), 1);
        assert_eq!(config.ops[0].level, pumpkin_util::PermissionLvl::Four);
        assert!(!config.ops[0].bypasses_player_limit);

        let serialized = serde_json::to_value(&config).unwrap();
        assert_eq!(serialized[0]["bypassesPlayerLimit"], false);
        assert!(serialized[0].get("bypasses_player_limit").is_none());
    }
}
