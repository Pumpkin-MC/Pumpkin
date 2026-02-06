use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an entry in the server whitelist.
///
/// Stores the player's UUID and username.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WhitelistEntry {
    /// The UUID of the whitelisted player.
    pub uuid: Uuid,

    /// The username of the whitelisted player.
    pub name: String,
}

impl WhitelistEntry {
    /// Creates a new whitelist entry with the given UUID and name.
    ///
    /// # Arguments
    /// * `uuid` – The UUID of the player.
    /// * `name` – The username of the player.
    #[must_use]
    pub const fn new(uuid: Uuid, name: String) -> Self {
        Self { uuid, name }
    }
}
