use serde::{Deserialize, Serialize};

/// Configuration for server resource pack distribution.
///
/// Controls whether a resource pack is offered or enforced,
/// along with its metadata and client prompt behaviour.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ResourcePackConfig {
    /// Whether the resource pack system is enabled.
    pub enabled: bool,
    /// The URL to the resource pack.
    pub url: String,
    /// The SHA1 hash (40 characters) of the resource pack.
    pub sha1: String,
    /// Custom prompt text component shown to players; leave blank for none.
    pub prompt_message: String,
    /// Whether players are forced to accept the resource pack.
    pub force: bool,
}

impl ResourcePackConfig {
    pub fn validate(&self) {
        if !self.enabled {
            return;
        }

        assert_eq!(
            !self.url.is_empty(),
            !self.sha1.is_empty(),
            "Resource pack path or SHA1 hash is missing"
        );

        let hash_len = self.sha1.len();

        assert_eq!(
            hash_len, 40,
            "Resource pack SHA1 hash is the wrong length (should be 40, is {hash_len})"
        );
    }
}
