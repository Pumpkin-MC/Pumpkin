use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct ServerLinksConfig {
    pub enabled: bool,
    pub bug_report: String,
    pub support: String,
    pub status: String,
    pub feedback: String,
    pub community: String,
    pub website: String,
    pub forums: String,
    pub news: String,
    pub announcements: String,
    pub custom: HashMap<String, String>,
}

impl Default for ServerLinksConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bug_report: "https://github.com/Pumpkin-MC/Pumpkin/issues".to_owned(),
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
