use pumpkin_util::version::JavaMinecraftVersion;
use serde::{Deserialize, Serialize};

/// Controls which Minecraft client versions are allowed to join the server.
///
/// This only restricts versions that the server's protocol implementation already
/// supports (see `LOWEST_SUPPORTED_MC_VERSION`..=`CURRENT_MC_VERSION`). It cannot add
/// support for versions the server doesn't implement.
#[derive(Deserialize, Serialize, Clone, Default, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum VersionAccessMode {
    /// Allow every version the server protocol implementation supports.
    #[default]
    Any,
    /// Only allow clients that are on the newest supported version.
    Latest,
    /// Only allow clients whose version falls within `min_version`..=`max_version`.
    Range,
    /// Only allow the versions listed in `versions`.
    Allowlist,
    /// Allow every supported version except the ones listed in `versions`.
    Denylist,
}

/// Friendly, human-configurable version gating and Server List Ping (status/motd)
/// version-text customization for Java Edition clients.
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct VersionsConfig {
    /// Which strategy to use to decide whether a client may join. See [`VersionAccessMode`].
    pub mode: VersionAccessMode,
    /// The list of versions used by `allowlist`/`denylist` mode, e.g. `["1.21", "1.20.5"]`.
    pub versions: Vec<String>,
    /// Inclusive lower bound used by `range` mode, e.g. `"1.20.5"`.
    pub min_version: Option<String>,
    /// Inclusive upper bound used by `range` mode, e.g. `"1.21.9"`.
    pub max_version: Option<String>,
    /// Custom message shown to players who try to join with a disallowed version.
    /// If unset, a sensible default message describing the rule is generated.
    pub disconnect_message: Option<String>,
    /// Overrides the `version.name` text shown in the multiplayer server list (Server List
    /// Ping / status response) for clients that ARE allowed to join. If unset, the default
    /// "min-max" text is used.
    pub status_name: Option<String>,
    /// Overrides the `version.name` text shown in the multiplayer server list for clients
    /// that are NOT allowed to join (e.g. to show a custom "Outdated" label). If unset, the
    /// client's own version-mismatch handling (greyed-out entry) is used.
    pub status_name_disallowed: Option<String>,
}

impl Default for VersionsConfig {
    fn default() -> Self {
        Self {
            mode: VersionAccessMode::Any,
            versions: Vec::new(),
            min_version: None,
            max_version: None,
            disconnect_message: None,
            status_name: None,
            status_name_disallowed: None,
        }
    }
}

impl VersionsConfig {
    /// Parses `versions` into recognized [`JavaMinecraftVersion`]s, silently skipping any
    /// entries that don't match a known version string.
    #[must_use]
    pub fn parse_list(&self) -> Vec<JavaMinecraftVersion> {
        self.versions
            .iter()
            .filter_map(|v| v.parse::<JavaMinecraftVersion>().ok())
            .collect()
    }

    /// Returns whether a client on `version` is allowed to join, given the server's `latest`
    /// supported version.
    #[must_use]
    pub fn is_allowed(&self, version: JavaMinecraftVersion, latest: JavaMinecraftVersion) -> bool {
        match self.mode {
            VersionAccessMode::Any => true,
            VersionAccessMode::Latest => version == latest,
            VersionAccessMode::Range => {
                let min = self
                    .min_version
                    .as_deref()
                    .and_then(|s| s.parse::<JavaMinecraftVersion>().ok());
                let max = self
                    .max_version
                    .as_deref()
                    .and_then(|s| s.parse::<JavaMinecraftVersion>().ok());
                match (min, max) {
                    (Some(min), Some(max)) => version >= min && version <= max,
                    (Some(min), None) => version >= min,
                    (None, Some(max)) => version <= max,
                    (None, None) => true,
                }
            }
            VersionAccessMode::Allowlist => self.parse_list().contains(&version),
            VersionAccessMode::Denylist => !self.parse_list().contains(&version),
        }
    }

    /// Builds a human-readable disconnect message describing the active rule, used when
    /// `disconnect_message` isn't set.
    #[must_use]
    pub fn default_disconnect_message(&self, latest: JavaMinecraftVersion) -> String {
        match self.mode {
            VersionAccessMode::Any => {
                "This server does not accept connections from your Minecraft version."
                    .to_string()
            }
            VersionAccessMode::Latest => {
                format!("This server only accepts the latest Minecraft version ({latest}).")
            }
            VersionAccessMode::Range => {
                let min = self.min_version.as_deref().unwrap_or("any");
                let max = self.max_version.as_deref().unwrap_or("any");
                format!("This server only accepts Minecraft versions between {min} and {max}.")
            }
            VersionAccessMode::Allowlist => format!(
                "This server only accepts these Minecraft versions: {}.",
                self.versions.join(", ")
            ),
            VersionAccessMode::Denylist => format!(
                "This server does not accept these Minecraft versions: {}.",
                self.versions.join(", ")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> JavaMinecraftVersion {
        s.parse().unwrap()
    }

    #[test]
    fn any_mode_allows_everything() {
        let cfg = VersionsConfig::default();
        assert!(cfg.is_allowed(v("1.20.5"), v("1.21.9")));
        assert!(cfg.is_allowed(v("1.21.9"), v("1.21.9")));
    }

    #[test]
    fn latest_mode_only_allows_latest() {
        let cfg = VersionsConfig {
            mode: VersionAccessMode::Latest,
            ..Default::default()
        };
        let latest = v("1.21.9");
        assert!(cfg.is_allowed(latest, latest));
        assert!(!cfg.is_allowed(v("1.21.7"), latest));
    }

    #[test]
    fn range_mode_bounds_inclusive() {
        let cfg = VersionsConfig {
            mode: VersionAccessMode::Range,
            min_version: Some("1.20.5".to_string()),
            max_version: Some("1.21.4".to_string()),
            ..Default::default()
        };
        let latest = v("1.21.9");
        assert!(cfg.is_allowed(v("1.20.5"), latest));
        assert!(cfg.is_allowed(v("1.21.4"), latest));
        assert!(cfg.is_allowed(v("1.21"), latest));
        assert!(!cfg.is_allowed(v("1.21.5"), latest));
        assert!(!cfg.is_allowed(v("1.19.4"), latest));
    }

    #[test]
    fn allowlist_only_allows_listed_versions() {
        let cfg = VersionsConfig {
            mode: VersionAccessMode::Allowlist,
            versions: vec!["1.21".to_string(), "1.20.5".to_string()],
            ..Default::default()
        };
        let latest = v("1.21.9");
        assert!(cfg.is_allowed(v("1.21"), latest));
        assert!(cfg.is_allowed(v("1.20.5"), latest));
        assert!(!cfg.is_allowed(v("1.21.9"), latest));
    }

    #[test]
    fn denylist_blocks_listed_versions_only() {
        let cfg = VersionsConfig {
            mode: VersionAccessMode::Denylist,
            versions: vec!["1.20.5".to_string()],
            ..Default::default()
        };
        let latest = v("1.21.9");
        assert!(!cfg.is_allowed(v("1.20.5"), latest));
        assert!(cfg.is_allowed(v("1.21.9"), latest));
        assert!(cfg.is_allowed(v("1.21"), latest));
    }

    #[test]
    fn version_round_trips_through_display_and_fromstr() {
        for s in ["1.7.2", "1.8", "1.12.2", "1.16", "1.20.5", "1.21.9", "26.2"] {
            assert_eq!(v(s).to_string(), s);
        }
        assert!("not-a-version".parse::<JavaMinecraftVersion>().is_err());
    }

    #[test]
    fn default_disconnect_messages_are_descriptive() {
        let latest = v("1.21.9");
        let range = VersionsConfig {
            mode: VersionAccessMode::Range,
            min_version: Some("1.8".to_string()),
            max_version: Some("1.12".to_string()),
            ..Default::default()
        };
        assert!(range.default_disconnect_message(latest).contains("1.8"));
        assert!(range.default_disconnect_message(latest).contains("1.12"));

        let latest_only = VersionsConfig {
            mode: VersionAccessMode::Latest,
            ..Default::default()
        };
        assert!(latest_only.default_disconnect_message(latest).contains("1.21.9"));
    }
}
