use crate::wit::pumpkin::plugin::player::{
    BanIpOptions, BanPlayerOptions, BedrockDisconnectReason, BedrockKickOptions, JavaKickOptions,
    SocketTeardownPolicy,
};
use crate::wit::pumpkin::plugin::text::TextComponent;

impl JavaKickOptions {
    /// Creates a new `JavaKickOptions` with the given reason and default settings.
    #[must_use]
    pub fn new(reason: TextComponent) -> Self {
        Self {
            reason,
            log_to_console: true,
            teardown_policy: SocketTeardownPolicy::Graceful,
        }
    }
}

impl Default for BedrockKickOptions {
    fn default() -> Self {
        Self {
            reason: BedrockDisconnectReason::Kicked,
            message: String::new(),
            skip_message: false,
            filtered_message: String::new(),
            log_to_console: true,
            teardown_policy: SocketTeardownPolicy::Graceful,
        }
    }
}

impl BedrockKickOptions {
    /// Creates a new `BedrockKickOptions` with the given reason, message, and default settings.
    #[must_use]
    pub fn new(reason: BedrockDisconnectReason, message: impl Into<String>) -> Self {
        Self {
            reason,
            message: message.into(),
            ..Default::default()
        }
    }
}

impl Default for BanPlayerOptions {
    fn default() -> Self {
        Self {
            reason: None,
            source: None,
            expires_at_utc: None,
            duration_seconds: None,
            kick_if_online: true,
            log_to_console: true,
        }
    }
}

impl BanPlayerOptions {
    /// Creates a new permanent ban with an optional reason and default settings.
    #[must_use]
    pub fn new(reason: Option<TextComponent>) -> Self {
        Self {
            reason,
            ..Default::default()
        }
    }

    /// Creates a temporary ban with a specific duration in seconds.
    #[must_use]
    pub fn temporary(reason: Option<TextComponent>, duration_seconds: u64) -> Self {
        Self {
            reason,
            duration_seconds: Some(duration_seconds),
            ..Default::default()
        }
    }
}

impl Default for BanIpOptions {
    fn default() -> Self {
        Self {
            reason: None,
            source: None,
            expires_at_utc: None,
            duration_seconds: None,
            kick_matching_players: true,
            log_to_console: true,
        }
    }
}

impl BanIpOptions {
    /// Creates a new permanent IP ban with an optional reason and default settings.
    #[must_use]
    pub fn new(reason: Option<TextComponent>) -> Self {
        Self {
            reason,
            ..Default::default()
        }
    }

    /// Creates a temporary IP ban with a specific duration in seconds.
    #[must_use]
    pub fn temporary(reason: Option<TextComponent>, duration_seconds: u64) -> Self {
        Self {
            reason,
            duration_seconds: Some(duration_seconds),
            ..Default::default()
        }
    }
}
