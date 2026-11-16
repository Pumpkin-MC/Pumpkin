use std::collections::{BTreeMap, BTreeSet};
use std::sync::{PoisonError, RwLock};

use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_util::version::JavaMinecraftVersion;

/// Thread-safe registry for Java Minecraft client versions supported by plugins.
#[derive(Debug, Default)]
pub struct JavaVersions {
    versions: RwLock<BTreeMap<String, BTreeSet<JavaMinecraftVersion>>>,
}

impl JavaVersions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the supported versions for a plugin.
    ///
    /// Any previous version registration for this plugin name is replaced.
    pub fn register(
        &self,
        plugin_name: &str,
        versions: impl IntoIterator<Item = JavaMinecraftVersion>,
    ) {
        let mut guard = self
            .versions
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        let set = guard.entry(plugin_name.to_string()).or_default();
        set.clear();
        for version in versions {
            if version != JavaMinecraftVersion::Unknown {
                set.insert(version);
            }
        }
    }

    /// Unregisters all versions for a plugin (e.g. on plugin unload).
    pub fn unregister(&self, plugin_name: &str) {
        let mut guard = self
            .versions
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        guard.remove(plugin_name);
    }

    /// Returns whether any plugin has explicitly registered this version.
    #[must_use]
    pub fn is_registered(&self, version: JavaMinecraftVersion) -> bool {
        if version == JavaMinecraftVersion::Unknown {
            return false;
        }
        let guard = self.versions.read().unwrap_or_else(PoisonError::into_inner);
        guard.values().any(|set| set.contains(&version))
    }

    /// Checks if a client version is admitted by either the server's current version
    /// or any loaded translation plugin.
    #[must_use]
    pub fn admits(&self, version: JavaMinecraftVersion) -> bool {
        if version == JavaMinecraftVersion::Unknown {
            return false;
        }
        if version == CURRENT_MC_VERSION {
            return true;
        }
        self.is_registered(version)
    }

    /// Checks if a client protocol number is admitted by either the server's current version
    /// or any loaded translation plugin.
    #[must_use]
    pub fn admits_protocol(&self, protocol: u32) -> bool {
        let version = JavaMinecraftVersion::from_protocol(protocol);
        if version != JavaMinecraftVersion::Unknown && self.admits(version) {
            return true;
        }

        let guard = self.versions.read().unwrap_or_else(PoisonError::into_inner);
        guard.values().any(|set| {
            set.iter().any(|v| {
                let p = v.protocol_version();
                p >= 0 && (p as u32) == protocol
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admits_current_mc_version() {
        let registry = JavaVersions::new();
        assert!(registry.admits(CURRENT_MC_VERSION));
        assert!(registry.admits_protocol(CURRENT_MC_VERSION.protocol_version() as u32));
    }

    #[test]
    fn admits_registered_version() {
        let registry = JavaVersions::new();
        assert!(!registry.admits(JavaMinecraftVersion::V_1_20_2));
        assert!(
            !registry.admits_protocol(JavaMinecraftVersion::V_1_20_2.protocol_version() as u32)
        );

        registry.register(
            "multiversion",
            vec![JavaMinecraftVersion::V_1_20_2, JavaMinecraftVersion::V_1_8],
        );
        assert!(registry.admits(JavaMinecraftVersion::V_1_20_2));
        assert!(registry.admits(JavaMinecraftVersion::V_1_8));
        assert!(registry.admits_protocol(JavaMinecraftVersion::V_1_20_2.protocol_version() as u32));
        assert!(registry.admits_protocol(JavaMinecraftVersion::V_1_8.protocol_version() as u32));

        // Unknown is never admitted
        assert!(!registry.admits(JavaMinecraftVersion::Unknown));

        // Unregister plugin
        registry.unregister("multiversion");
        assert!(!registry.admits(JavaMinecraftVersion::V_1_20_2));
        assert!(!registry.admits(JavaMinecraftVersion::V_1_8));
        assert!(registry.admits(CURRENT_MC_VERSION));
    }

    #[test]
    fn register_replaces_previous_list() {
        let registry = JavaVersions::new();
        registry.register("test_plugin", vec![JavaMinecraftVersion::V_1_8]);
        assert!(registry.admits(JavaMinecraftVersion::V_1_8));

        registry.register("test_plugin", vec![JavaMinecraftVersion::V_1_20_2]);
        assert!(!registry.admits(JavaMinecraftVersion::V_1_8));
        assert!(registry.admits(JavaMinecraftVersion::V_1_20_2));
    }
}
