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
        use std::collections::HashSet;

        let mut seen_uuids = HashSet::new();
        let mut seen_names = HashSet::new();

        for op in &self.ops {
            if op.uuid.is_nil() {
                tracing::warn!("Operator entry has nil UUID: {}", op.name);
            } else if !seen_uuids.insert(op.uuid) {
                tracing::warn!("Duplicate operator UUID: {}", op.uuid);
            }

            if op.name.is_empty() {
                tracing::warn!("Operator entry has empty name: {}", op.uuid);
            } else if !seen_names.insert(op.name.to_lowercase()) {
                tracing::warn!("Duplicate operator name (case-insensitive): {}", op.name);
            }

            if op.level as u8 > 4 {
                tracing::warn!("Operator {} has invalid permission level: {}", op.name, op.level as u8);
            }
        }
    }
}

impl SaveJSONConfiguration for OperatorConfig {}
