use crate::wit::pumpkin::plugin::item_stack::ItemStack;
use crate::wit::pumpkin::plugin::player::{
    BanIpOptions, BanPlayerOptions, BedrockDisconnectReason, BedrockKickOptions, JavaKickOptions,
    Player, SocketTeardownPolicy,
};
use crate::wit::pumpkin::plugin::text::TextComponent;

/// Extension trait providing batch access and helper utilities for player ender chest inventories.
pub trait PlayerEnderChestExt {
    /// Returns all 27 slots of the player's ender chest.
    fn get_all_ender_chest_items(&self) -> Vec<Option<ItemStack>>;

    /// Sets all 27 slots of the player's ender chest from an iterator.
    fn set_all_ender_chest_items(&self, items: impl IntoIterator<Item = Option<ItemStack>>);
}

impl PlayerEnderChestExt for Player {
    fn get_all_ender_chest_items(&self) -> Vec<Option<ItemStack>> {
        (0..27)
            .map(|slot| self.get_ender_chest_item(slot))
            .collect()
    }

    fn set_all_ender_chest_items(&self, items: impl IntoIterator<Item = Option<ItemStack>>) {
        for (slot, item) in items.into_iter().take(27).enumerate() {
            self.set_ender_chest_item(slot as u8, item);
        }
    }
}

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

/// Extension trait providing typed cooldown helpers for `Player`.
pub trait PlayerCooldownExt {
    /// Sets a client-side item cooldown overlay using any item key or `Item` enum.
    fn set_cooldown(&self, item: impl crate::item::IntoItemKey, ticks: i32);

    /// Returns the remaining cooldown ticks for an item, if active.
    fn get_cooldown(&self, item: impl crate::item::IntoItemKey) -> Option<i32>;

    /// Checks if an item is currently on cooldown.
    fn has_cooldown(&self, item: impl crate::item::IntoItemKey) -> bool;
}

impl PlayerCooldownExt for Player {
    fn set_cooldown(&self, item: impl crate::item::IntoItemKey, ticks: i32) {
        self.set_item_cooldown(&item.into_item_key(), ticks);
    }

    fn get_cooldown(&self, item: impl crate::item::IntoItemKey) -> Option<i32> {
        self.get_item_cooldown(&item.into_item_key())
    }

    fn has_cooldown(&self, item: impl crate::item::IntoItemKey) -> bool {
        self.has_item_cooldown(&item.into_item_key())
    }
}

#[cfg(test)]
mod tests {
    use crate::{CustomStatistic, StatisticCategory};

    #[test]
    fn statistic_types() {
        assert_eq!(StatisticCategory::Mined as u8, 0);
        assert_eq!(StatisticCategory::Crafted as u8, 1);
        assert_eq!(StatisticCategory::Custom as u8, 8);

        assert_eq!(CustomStatistic::LeaveGame as u8, 0);
        assert_eq!(CustomStatistic::PlayTime as u8, 1);
        assert_eq!(CustomStatistic::Deaths as u8, 32);
        assert_eq!(CustomStatistic::PlayerKills as u8, 35);
    }
}
