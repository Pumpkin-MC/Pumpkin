use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for server-related links.
///
/// Controls default URLs for bug reports, support, community, and other resources,
/// as well as allowing custom links.
#[derive(Deserialize, Serialize, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct ServerLinksConfig {
    /// Whether server links are enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// URL for reporting bugs.
    #[toml(default = Self::default().bug_report)]
    pub bug_report: String,
    /// URL for support resources.
    #[toml(default = Self::default().support)]
    pub support: String,
    /// URL for server status.
    #[toml(default = Self::default().status)]
    pub status: String,
    /// URL for player feedback.
    #[toml(default = Self::default().feedback)]
    pub feedback: String,
    /// URL for the community page.
    #[toml(default = Self::default().community)]
    pub community: String,
    /// URL for the official website.
    #[toml(default = Self::default().website)]
    pub website: String,
    /// URL for forums.
    #[toml(default = Self::default().forums)]
    pub forums: String,
    /// URL for news updates.
    #[toml(default = Self::default().news)]
    pub news: String,
    /// URL for announcements.
    #[toml(default = Self::default().announcements)]
    pub announcements: String,
    /// Custom key-value links.
    #[toml(default = Self::default().custom)]
    pub custom: HashMap<String, String>,
}

impl Default for ServerLinksConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bug_report: "https://github.com/Pumpkin-MC/Pumpkin/issues".to_string(),
            support: String::new(),
            status: String::new(),
            feedback: String::new(),
            community: String::new(),
            website: String::new(),
            forums: String::new(),
            news: String::new(),
            announcements: String::new(),
            custom: HashMap::default(),
        }
    }
}
