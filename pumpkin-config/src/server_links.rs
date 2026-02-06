use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for server-related links.
///
/// Controls default URLs for bug reports, support, community, and other resources,
/// as well as allowing custom links.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct ServerLinksConfig {
    /// Whether server links are enabled.
    pub enabled: bool,

    /// URL for reporting bugs.
    pub bug_report: String,

    /// URL for support resources.
    pub support: String,

    /// URL for server status.
    pub status: String,

    /// URL for player feedback.
    pub feedback: String,

    /// URL for the community page.
    pub community: String,

    /// URL for the official website.
    pub website: String,

    /// URL for forums.
    pub forums: String,

    /// URL for news updates.
    pub news: String,

    /// URL for announcements.
    pub announcements: String,

    /// Custom key-value links.
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
