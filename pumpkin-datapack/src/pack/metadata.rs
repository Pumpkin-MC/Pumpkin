use serde::Deserialize;
use std::collections::HashMap;

use super::format::{FormatRange, PackCompatibility, PackFormat};

/// Parsed `pack.mcmeta` file.
#[derive(Debug, Clone, Deserialize)]
pub struct PackMcmeta {
    pub pack: PackSection,
    #[serde(default)]
    pub features: Option<FeaturesSection>,
    #[serde(default)]
    pub overlays: Option<OverlaysSection>,
    #[serde(default)]
    pub filter: Option<FilterSection>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackSection {
    /// Can be a plain string or a `TextComponent` JSON object.
    #[serde(default)]
    pub description: serde_json::Value,
    /// Vanilla `pack_format` (e.g. 81). Not present in all packs; some use `min_format`/`max_format`.
    #[serde(default = "default_pack_format")]
    pub pack_format: u32,
    /// Supported format range (object or number); alternative to `min_format`/`max_format`.
    #[serde(default)]
    pub supported_formats: Option<serde_json::Value>,
    /// Alternative minimum format using simple major version numbers.
    #[serde(default)]
    pub min_format: Option<serde_json::Value>,
    /// Alternative maximum format using simple major version numbers.
    #[serde(default)]
    pub max_format: Option<serde_json::Value>,
}

const fn default_pack_format() -> u32 {
    81
}

impl PackMcmeta {
    /// Resolve the supported format range from `pack_format` and optional `supported_formats`.
    #[must_use]
    pub fn supported_formats(&self) -> FormatRange {
        self.pack.supported_formats.as_ref().map_or_else(
            || FormatRange::Single(PackFormat::new(self.pack.pack_format, 0)),
            |v| parse_format_range(v, self.pack.pack_format),
        )
    }

    /// Compute compatibility with the server version.
    #[must_use]
    pub fn compatibility(&self) -> PackCompatibility {
        PackCompatibility::check(self.pack.pack_format, &self.supported_formats())
    }

    /// Return the list of overlay directories whose format range matches
    /// `PackFormat::CURRENT`. These directories should be checked before the
    /// root data directory when serving resources.
    #[must_use]
    pub fn matching_overlay_dirs(&self) -> Vec<String> {
        let Some(overlays) = &self.overlays else {
            return Vec::new();
        };
        let current = PackFormat::CURRENT;
        overlays
            .entries
            .iter()
            .filter(|entry| entry.formats_range().matches(&current))
            .map(|entry| entry.directory.clone())
            .collect()
    }
}

fn parse_format_range(v: &serde_json::Value, fallback_major: u32) -> FormatRange {
    match v {
        serde_json::Value::Number(n) => FormatRange::Single(PackFormat::new(
            n.as_u64().unwrap_or(u64::from(fallback_major)) as u32,
            0,
        )),
        serde_json::Value::Object(map) => {
            let min = parse_version_pair(map.get("min_inclusive"), fallback_major);
            let max = parse_version_pair(map.get("max_inclusive"), fallback_major);
            if min == max {
                FormatRange::Single(min)
            } else {
                FormatRange::Range { min, max }
            }
        }
        _ => FormatRange::Single(PackFormat::new(fallback_major, 0)),
    }
}

fn parse_version_pair(v: Option<&serde_json::Value>, fallback_major: u32) -> PackFormat {
    match v {
        Some(serde_json::Value::Number(n)) => {
            PackFormat::new(n.as_u64().unwrap_or(u64::from(fallback_major)) as u32, 0)
        }
        Some(serde_json::Value::Array(arr)) if arr.len() >= 2 => {
            let major = arr[0].as_u64().unwrap_or(u64::from(fallback_major)) as u32;
            let minor = arr[1].as_u64().unwrap_or(0) as u32;
            PackFormat::new(major, minor)
        }
        _ => PackFormat::new(fallback_major, 0),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeaturesSection {
    pub enabled: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OverlaysSection {
    pub entries: Vec<OverlayEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OverlayEntry {
    #[serde(default)]
    pub formats: Option<FormatRangeValue>,
    /// Alternative min format using simple major version numbers.
    /// Used when packs supply `min_format`/`max_format` fields instead of the standard `formats`.
    #[serde(default)]
    pub min_format: Option<u32>,
    /// Alternative max format using simple major version numbers.
    #[serde(default)]
    pub max_format: Option<u32>,
    pub directory: String,
}

impl OverlayEntry {
    /// Convert the parsed format information into a `FormatRange`.
    /// Supports both the standard `formats` field and a simplified
    /// `min_format`/`max_format` fallback.
    #[must_use]
    pub fn formats_range(&self) -> FormatRange {
        // Standard `formats` field takes priority
        if let Some(ref formats) = self.formats {
            return match formats {
                FormatRangeValue::Single(v) => FormatRange::Single(PackFormat::new(*v, 0)),
                FormatRangeValue::Array(arr) if arr.len() >= 2 => FormatRange::Range {
                    min: PackFormat::new(arr[0], 0),
                    max: PackFormat::new(arr[1], 0),
                },
                FormatRangeValue::Object(map) => {
                    let min = parse_version_pair(map.get("min_inclusive"), 0);
                    let max = parse_version_pair(map.get("max_inclusive"), 0);
                    if min == max {
                        FormatRange::Single(min)
                    } else {
                        FormatRange::Range { min, max }
                    }
                }
                FormatRangeValue::Array(_) => FormatRange::Range {
                    min: PackFormat::new(0, 0),
                    max: PackFormat::new(u32::MAX, u32::MAX),
                },
            };
        }

        // Fall back to min_format/max_format fields
        let min = self
            .min_format
            .map_or(PackFormat::new(0, 0), |m| PackFormat::new(m, 0));
        let max = self
            .max_format
            .map_or(PackFormat::new(u32::MAX, u32::MAX), |m| {
                PackFormat::new(m, 0)
            });
        if min == max {
            FormatRange::Single(min)
        } else {
            FormatRange::Range { min, max }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum FormatRangeValue {
    Single(u32),
    Array(Vec<u32>),
    Object(HashMap<String, serde_json::Value>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilterSection {
    pub block: Vec<String>,
}
