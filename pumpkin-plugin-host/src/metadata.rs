/// Struct representing metadata for a plugin.
///
/// This struct contains essential information about a plugin, including its name,
/// version, authors, and a description. It is generic over a lifetime `'s` to allow
/// for string slices that are valid for the lifetime of the plugin metadata.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// The name of the plugin.
    pub name: String,
    /// The version of the plugin.
    pub version: String,
    /// The authors of the plugin.
    pub authors: Vec<String>,
    /// A description of the plugin.
    pub description: String,
}
