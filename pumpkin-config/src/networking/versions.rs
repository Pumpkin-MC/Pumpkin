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
    /// Allow every version the server protocol implementation supports
    /// (`LOWEST_SUPPORTED_MC_VERSION`..=`CURRENT_MC_VERSION`). This is the vanilla-like
    /// default: no restriction beyond what the server can already talk to.
    #[default]
    Any,
    /// Only allow clients that are on `CURRENT_MC_VERSION` -- the newest version *this
    /// server build* implements. Note this is tied to the server software's own version,
    /// not necessarily the newest version publicly available to players; if the server
    /// hasn't been updated yet (or was built against an upcoming/snapshot target), players
    /// on an officially "latest" client can still get rejected here. Prefer `range` with an
    /// explicit `min_version` if you want "recent versions only" without that surprise.
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
///
/// All the string fields below are intentionally always written out to `pumpkin.toml`
/// (even when empty) so operators can see every knob that exists without having to read
/// the source. An empty string means "not set / use the automatic default".
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct VersionsConfig {
    /// Which strategy to use to decide whether a client may join:
    /// `any` (default) | `latest` | `range` | `allowlist` | `denylist`.
    /// See [`VersionAccessMode`] for what each one does.
    pub mode: VersionAccessMode,
    /// The list of versions used by `allowlist`/`denylist` mode, e.g. `["1.21", "1.20.5"]`.
    /// Ignored by every other mode.
    pub versions: Vec<String>,
    /// Inclusive lower bound used by `range` mode, e.g. `"1.20.5"`. Leave empty to mean
    /// "no lower bound other than what the server supports". Ignored by every other mode.
    pub min_version: String,
    /// Inclusive upper bound used by `range` mode, e.g. `"1.21.9"`. Leave empty to mean
    /// "no upper bound other than what the server supports". Ignored by every other mode.
    pub max_version: String,
    /// Custom message shown to players who try to join with a disallowed version. Leave
    /// empty to auto-generate a message describing the active rule (e.g. which versions
    /// are allowed).
    pub disconnect_message: String,
    /// Overrides the `version.name` text shown in the multiplayer server list (Server List
    /// Ping / status response) for clients that ARE allowed to join. Leave empty to use the
    /// default "min-max" text.
    pub status_name: String,
    /// Overrides the `version.name` text shown in the multiplayer server list for clients
    /// that are NOT allowed to join (e.g. to show a custom "Outdated"/"Restricted" label).
    /// Leave empty to use the client's own version-mismatch handling (greyed-out entry).
    pub status_name_disallowed: String,
}

impl Default for VersionsConfig {
    fn default() -> Self {
        Self {
            mode: VersionAccessMode::Any,
            versions: Vec::new(),
            min_version: String::new(),
            max_version: String::new(),
            disconnect_message: String::new(),
            status_name: String::new(),
            status_name_disallowed: String::new(),
        }
    }
}

/// Treats an empty string the same as "not set".
fn non_empty(s: &str) -> Option<&str> {
    if s.is_empty() { None } else { Some(s) }
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
                let min = non_empty(&self.min_version)
                    .and_then(|s| s.parse::<JavaMinecraftVersion>().ok());
                let max = non_empty(&self.max_version)
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

    /// Returns the configured disconnect message, or `None` if it should be auto-generated.
    #[must_use]
    pub fn disconnect_message(&self) -> Option<&str> {
        non_empty(&self.disconnect_message)
    }

    /// Returns the configured "allowed" status name override, if any.
    #[must_use]
    pub fn status_name(&self) -> Option<&str> {
        non_empty(&self.status_name)
    }

    /// Returns the configured "disallowed" status name override, if any.
    #[must_use]
    pub fn status_name_disallowed(&self) -> Option<&str> {
        non_empty(&self.status_name_disallowed)
    }

    /// Builds a human-readable disconnect message describing the active rule, used when
    /// `disconnect_message` isn't set.
    #[must_use]
    pub fn default_disconnect_message(&self, latest: JavaMinecraftVersion) -> String {
        match self.mode {
            VersionAccessMode::Any => {
                "This server does not accept connections from your Minecraft version.".to_string()
            }
            VersionAccessMode::Latest => {
                format!("This server only accepts the latest Minecraft version ({latest}).")
            }
            VersionAccessMode::Range => {
                let min = non_empty(&self.min_version).unwrap_or("any");
                let max = non_empty(&self.max_version).unwrap_or("any");
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
    fn any_is_the_default_mode() {
        assert_eq!(VersionsConfig::default().mode, VersionAccessMode::Any);
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
            min_version: "1.20.5".to_string(),
            max_version: "1.21.4".to_string(),
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
    fn range_mode_empty_bound_is_open_ended() {
        let cfg = VersionsConfig {
            mode: VersionAccessMode::Range,
            min_version: "1.21".to_string(),
            max_version: String::new(),
            ..Default::default()
        };
        let latest = v("1.21.9");
        assert!(cfg.is_allowed(v("1.21.9"), latest));
        assert!(!cfg.is_allowed(v("1.20.5"), latest));
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
            min_version: "1.8".to_string(),
            max_version: "1.12".to_string(),
            ..Default::default()
        };
        assert!(range.default_disconnect_message(latest).contains("1.8"));
        assert!(range.default_disconnect_message(latest).contains("1.12"));

        let latest_only = VersionsConfig {
            mode: VersionAccessMode::Latest,
            ..Default::default()
        };
        assert!(
            latest_only
                .default_disconnect_message(latest)
                .contains("1.21.9")
        );
    }

    #[test]
    fn empty_strings_are_treated_as_unset() {
        let cfg = VersionsConfig::default();
        assert_eq!(cfg.disconnect_message(), None);
        assert_eq!(cfg.status_name(), None);
        assert_eq!(cfg.status_name_disallowed(), None);

        let cfg = VersionsConfig {
            disconnect_message: "custom".to_string(),
            status_name: "name".to_string(),
            status_name_disallowed: "nope".to_string(),
            ..Default::default()
        };
        assert_eq!(cfg.disconnect_message(), Some("custom"));
        assert_eq!(cfg.status_name(), Some("name"));
        assert_eq!(cfg.status_name_disallowed(), Some("nope"));
    }
}
