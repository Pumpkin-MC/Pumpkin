use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ResourcePackConfig {
    pub enabled: bool,
    /// The path to the resource pack.
    pub url: String,
    /// The SHA1 hash (40) of the resource pack.
    pub sha1: String,
    /// Custom prompt Text component, Leave blank for none
    pub message: String,
    /// Will force the Player to accept the resource pack
    pub force: bool,
}

impl ResourcePackConfig {
    pub fn validate(&self) {
        assert_eq!(
            !self.url.is_empty(),
            !self.sha1.is_empty(),
            "Resource Pack path or Sha1 hash is missing"
        );
        assert!(
            self.sha1.len() <= 40,
            "Resource pack sha1 hash is too long (max. 40)"
        )
    }
}
