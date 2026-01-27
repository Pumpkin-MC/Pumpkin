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
