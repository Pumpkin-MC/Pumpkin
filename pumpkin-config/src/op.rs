use pumpkin_util::permission::PermissionLvl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an operator (admin) on the server.
///
/// Includes their UUID, name, permission level, and whether they bypass the player limit.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Op {
    /// The UUID of the operator.
    pub uuid: Uuid,
    /// The name of the operator.
    pub name: String,
    /// The permission level assigned to this operator.
    pub level: PermissionLvl,
    /// Whether this operator bypasses the server's player limit.
    pub bypasses_player_limit: bool,
}

impl Op {
    /// Creates a new operator with the given properties.
    ///
    /// # Arguments
    /// * `uuid` – The UUID of the operator.
    /// * `name` – The name of the operator.
    /// * `level` – The permission level assigned to the operator.
    /// * `bypasses_player_limit` – Whether the operator can bypass the server's player limit.
    #[must_use]
    pub const fn new(
        uuid: Uuid,
        name: String,
        level: PermissionLvl,
        bypasses_player_limit: bool,
    ) -> Self {
        Self {
            uuid,
            name,
            level,
            bypasses_player_limit,
        }
    }
}
