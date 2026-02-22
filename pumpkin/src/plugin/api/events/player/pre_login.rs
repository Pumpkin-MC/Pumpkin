use pumpkin_macros::{Event, cancellable};
use uuid::Uuid;

/// An async event that occurs before a player logs in.
///
/// Plugins may set a disallow result and kick message to block the login.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerPreLoginEvent {
    /// The player's username.
    pub name: String,

    /// The player's UUID (offline or online mode based on server config).
    pub player_uuid: Uuid,

    /// The connecting address, as a string.
    pub address: String,

    /// Login result, "ALLOWED" to permit login.
    pub result: String,

    /// Kick message shown when disallowed.
    pub kick_message: String,
}

impl PlayerPreLoginEvent {
    #[must_use]
    #[expect(clippy::missing_const_for_fn)]
    pub fn new(
        name: String,
        player_uuid: Uuid,
        address: String,
        result: String,
        kick_message: String,
    ) -> Self {
        Self {
            name,
            player_uuid,
            address,
            result,
            kick_message,
            cancelled: false,
        }
    }
}
