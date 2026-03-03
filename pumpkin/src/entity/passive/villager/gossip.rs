use uuid::Uuid;

/// Type of gossip entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GossipType {
    MajorPositive,
    MinorPositive,
    MinorNegative,
    MajorNegative,
    Trading,
}

impl GossipType {
    /// Reputation weight per gossip value point.
    #[must_use]
    pub fn weight(&self) -> i32 {
        match self {
            Self::MajorPositive => 5,
            Self::MinorPositive => 1,
            Self::MinorNegative => -1,
            Self::MajorNegative => -5,
            Self::Trading => 1,
        }
    }

    /// Maximum value this gossip type can reach.
    #[must_use]
    pub fn max_value(&self) -> i32 {
        match self {
            Self::MajorPositive => 100,
            Self::MinorPositive => 200,
            Self::MinorNegative => 200,
            Self::MajorNegative => 100,
            Self::Trading => 25,
        }
    }

    /// How much this gossip decays per in-game day.
    #[must_use]
    pub fn decay_per_day(&self) -> i32 {
        match self {
            Self::MajorPositive => 0,
            Self::MinorPositive => 2,
            Self::MinorNegative => 20,
            Self::MajorNegative => 10,
            Self::Trading => 2,
        }
    }

    /// NBT name for serialization.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::MajorPositive => "major_positive",
            Self::MinorPositive => "minor_positive",
            Self::MinorNegative => "minor_negative",
            Self::MajorNegative => "major_negative",
            Self::Trading => "trading",
        }
    }

    /// Parse from NBT name string.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "major_positive" => Some(Self::MajorPositive),
            "minor_positive" => Some(Self::MinorPositive),
            "minor_negative" => Some(Self::MinorNegative),
            "major_negative" => Some(Self::MajorNegative),
            "trading" => Some(Self::Trading),
            _ => None,
        }
    }

    /// Maximum value that can be shared between villagers (gossip spreading).
    #[must_use]
    pub fn share_max(&self) -> i32 {
        match self {
            Self::MajorPositive => 20,
            Self::MinorPositive => 20,
            Self::MinorNegative => 20,
            Self::MajorNegative => 10,
            Self::Trading => 0, // Trading gossip doesn't spread
        }
    }
}

/// A gossip entry about a specific player.
#[derive(Debug, Clone)]
pub struct GossipEntry {
    pub gossip_type: GossipType,
    pub target: Uuid,
    pub value: i32,
}

/// Container for all gossip entries of a villager.
#[derive(Debug, Clone, Default)]
pub struct GossipContainer {
    pub entries: Vec<GossipEntry>,
}

impl GossipContainer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add gossip about a player. Clamps to max_value.
    pub fn add(&mut self, gossip_type: GossipType, target: Uuid, amount: i32) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.gossip_type == gossip_type && e.target == target)
        {
            entry.value = (entry.value + amount).min(gossip_type.max_value());
        } else {
            self.entries.push(GossipEntry {
                gossip_type,
                target,
                value: amount.min(gossip_type.max_value()),
            });
        }
    }

    /// Get total reputation for a player (sum of value * weight for all gossip types).
    #[must_use]
    pub fn get_reputation(&self, player_uuid: &Uuid) -> i32 {
        self.entries
            .iter()
            .filter(|e| &e.target == player_uuid)
            .map(|e| e.value * e.gossip_type.weight())
            .sum()
    }

    /// Calculate price adjustment from reputation.
    /// Negative reputation = higher prices, positive = lower prices.
    #[must_use]
    pub fn get_price_adjustment(&self, player_uuid: &Uuid) -> i32 {
        let reputation = self.get_reputation(player_uuid);
        -(reputation / 5)
    }

    /// Decay all gossip entries by one day. Removes entries that reach 0.
    pub fn decay(&mut self) {
        for entry in &mut self.entries {
            let decay = entry.gossip_type.decay_per_day();
            entry.value = (entry.value - decay).max(0);
        }
        self.entries.retain(|e| e.value > 0);
    }

    /// Get gossip entries that can be shared with another villager.
    /// Returns entries with values clamped to share_max.
    #[must_use]
    pub fn get_shareable(&self) -> Vec<GossipEntry> {
        self.entries
            .iter()
            .filter(|e| e.gossip_type.share_max() > 0)
            .map(|e| GossipEntry {
                gossip_type: e.gossip_type,
                target: e.target,
                value: e.value.min(e.gossip_type.share_max()),
            })
            .collect()
    }

    /// Merge gossip from another villager (gossip spreading).
    pub fn merge_from(&mut self, other: &[GossipEntry]) {
        for entry in other {
            self.add(entry.gossip_type, entry.target, entry.value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation() {
        let mut container = GossipContainer::new();
        let uuid = Uuid::new_v4();

        container.add(GossipType::MinorPositive, uuid, 10);
        container.add(GossipType::Trading, uuid, 5);
        // reputation = 10*1 + 5*1 = 15
        assert_eq!(container.get_reputation(&uuid), 15);

        container.add(GossipType::MajorNegative, uuid, 3);
        // reputation = 10*1 + 5*1 + 3*(-5) = 15 - 15 = 0
        assert_eq!(container.get_reputation(&uuid), 0);
    }

    #[test]
    fn test_decay() {
        let mut container = GossipContainer::new();
        let uuid = Uuid::new_v4();

        container.add(GossipType::MinorNegative, uuid, 25);
        assert_eq!(container.entries.len(), 1);

        container.decay(); // decays by 20
        assert_eq!(container.entries[0].value, 5);

        container.decay(); // 5-20 = -15 -> clamped to 0, removed
        assert_eq!(container.entries.len(), 0);
    }

    #[test]
    fn test_clamping() {
        let mut container = GossipContainer::new();
        let uuid = Uuid::new_v4();

        container.add(GossipType::Trading, uuid, 100); // max is 25
        assert_eq!(container.entries[0].value, 25);

        container.add(GossipType::Trading, uuid, 100); // still max 25
        assert_eq!(container.entries[0].value, 25);
    }
}
