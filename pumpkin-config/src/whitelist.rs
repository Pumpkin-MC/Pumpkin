use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WhitelistEntry {
    pub uuid: Uuid,
    pub name: String,
}

impl WhitelistEntry {
    #[must_use]
    pub fn new(uuid: Uuid, name: String) -> Self {
        Self { uuid, name }
    }
}
