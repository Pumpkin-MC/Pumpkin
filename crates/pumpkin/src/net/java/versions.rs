use std::collections::BTreeMap;
use std::sync::Arc;

use arc_swap::ArcSwap;
use pumpkin_data::packet::{CURRENT_MC_VERSION, LOWEST_SUPPORTED_MC_VERSION};
use pumpkin_util::version::JavaMinecraftVersion;

/// Java versions the server admits: its own, plus the ones plugins registered to translate.
#[derive(Default)]
pub struct JavaVersions {
    /// By plugin name.
    registered: ArcSwap<BTreeMap<String, Arc<[JavaMinecraftVersion]>>>,
}

impl JavaVersions {
    /// Replaces `plugin`'s versions.
    pub fn register(&self, plugin: &str, versions: impl IntoIterator<Item = JavaMinecraftVersion>) {
        let versions: Arc<[_]> = versions
            .into_iter()
            .filter(|version| *version != JavaMinecraftVersion::Unknown)
            .collect();
        self.registered.rcu(|registered| {
            let mut registered = (**registered).clone();
            registered.insert(plugin.to_string(), versions.clone());
            registered
        });
    }

    pub fn unregister(&self, plugin: &str) {
        self.registered.rcu(|registered| {
            let mut registered = (**registered).clone();
            registered.remove(plugin);
            registered
        });
    }

    #[must_use]
    pub fn is_native(version: JavaMinecraftVersion) -> bool {
        (LOWEST_SUPPORTED_MC_VERSION..=CURRENT_MC_VERSION).contains(&version)
    }

    #[must_use]
    pub fn is_translated(&self, version: JavaMinecraftVersion) -> bool {
        self.registered
            .load()
            .values()
            .any(|versions| versions.contains(&version))
    }

    #[must_use]
    pub fn admits(&self, version: JavaMinecraftVersion) -> bool {
        Self::is_native(version) || self.is_translated(version)
    }

    /// The oldest admitted version.
    #[must_use]
    pub fn oldest(&self) -> JavaMinecraftVersion {
        self.registered
            .load()
            .values()
            .flat_map(|versions| versions.iter().copied())
            .fold(LOWEST_SUPPORTED_MC_VERSION, Ord::min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_versions_are_admitted_until_unregistered() {
        let versions = JavaVersions::default();
        assert!(versions.admits(CURRENT_MC_VERSION));
        assert!(!versions.admits(JavaMinecraftVersion::V_1_8));
        assert_eq!(versions.oldest(), LOWEST_SUPPORTED_MC_VERSION);

        versions.register(
            "multiversion",
            [JavaMinecraftVersion::V_1_8, JavaMinecraftVersion::Unknown],
        );
        assert!(versions.admits(JavaMinecraftVersion::V_1_8));
        assert!(!versions.admits(JavaMinecraftVersion::Unknown));
        assert_eq!(versions.oldest(), JavaMinecraftVersion::V_1_8);

        versions.unregister("multiversion");
        assert!(!versions.admits(JavaMinecraftVersion::V_1_8));
    }
}
