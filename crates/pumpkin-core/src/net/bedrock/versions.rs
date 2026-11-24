use std::collections::{BTreeMap, BTreeSet};
use std::sync::{PoisonError, RwLock};

use pumpkin_util::version::BedrockMinecraftVersion;
use pumpkin_world::CURRENT_BEDROCK_MC_PROTOCOL;

/// Thread-safe registry for Bedrock Minecraft protocol versions supported by plugins.
#[derive(Debug, Default)]
pub struct BedrockVersions {
    protocols: RwLock<BTreeMap<String, BTreeSet<u32>>>,
}

impl BedrockVersions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the supported Bedrock network protocol numbers for a plugin.
    ///
    /// Any previous registration for this plugin name is replaced.
    pub fn register(&self, plugin_name: &str, protocols: impl IntoIterator<Item = u32>) {
        let mut guard = self
            .protocols
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        let set = guard.entry(plugin_name.to_string()).or_default();
        set.clear();
        for protocol in protocols {
            if protocol > 0 {
                set.insert(protocol);
            }
        }
    }

    /// Unregisters all protocols for a plugin (e.g. on plugin unload).
    pub fn unregister(&self, plugin_name: &str) {
        let mut guard = self
            .protocols
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        guard.remove(plugin_name);
    }

    /// Returns whether any plugin has explicitly registered this protocol number.
    #[must_use]
    pub fn is_registered(&self, protocol: u32) -> bool {
        if protocol == 0 {
            return false;
        }
        let guard = self
            .protocols
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        guard.values().any(|set| set.contains(&protocol))
    }

    /// Checks if a known Bedrock version enum is admitted by either the server's current
    /// protocol or any loaded translation plugin.
    #[must_use]
    pub fn admits(&self, version: BedrockMinecraftVersion) -> bool {
        let protocol = version.protocol_version();
        protocol > 0 && self.admits_protocol(protocol as u32)
    }

    /// Checks if a client Bedrock protocol number is admitted by either the server's current
    /// protocol or any loaded translation plugin.
    #[must_use]
    pub fn admits_protocol(&self, protocol: u32) -> bool {
        if protocol == 0 {
            return false;
        }
        if protocol == CURRENT_BEDROCK_MC_PROTOCOL {
            return true;
        }
        self.is_registered(protocol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admits_current_bedrock_protocol() {
        let registry = BedrockVersions::new();
        assert!(registry.admits_protocol(CURRENT_BEDROCK_MC_PROTOCOL));
        assert!(registry.admits(BedrockMinecraftVersion::V_1_26_60));
    }

    #[test]
    fn admits_registered_protocol() {
        let registry = BedrockVersions::new();
        assert!(!registry.admits(BedrockMinecraftVersion::V_1_21));
        assert!(!registry.admits_protocol(671));
        assert!(!registry.admits_protocol(766));

        registry.register("bedrock_mv", [671, 766, 2193]);
        assert!(registry.admits(BedrockMinecraftVersion::V_1_21));
        assert!(registry.admits(BedrockMinecraftVersion::V_1_26_51));
        assert!(registry.admits_protocol(671));
        assert!(registry.admits_protocol(766));
        assert!(!registry.admits(BedrockMinecraftVersion::Unknown));
        assert!(!registry.admits_protocol(0));

        registry.unregister("bedrock_mv");
        assert!(!registry.admits_protocol(671));
        assert!(registry.admits_protocol(CURRENT_BEDROCK_MC_PROTOCOL));
    }

    #[test]
    fn register_replaces_previous_list() {
        let registry = BedrockVersions::new();
        registry.register("test_plugin", [671]);
        assert!(registry.admits_protocol(671));

        registry.register("test_plugin", [2193]);
        assert!(!registry.admits_protocol(671));
        assert!(registry.admits_protocol(2193));
    }
}
