use std::collections::HashMap;

use super::{Tag, TagEntry};
use crate::DatapackError;
use crate::Identifier;

/// An unresolved tag stores raw entries (elements + tag references) before resolution.
#[derive(Debug, Clone)]
struct UnresolvedTag {
    entries: Vec<TagEntry>,
}

/// Runtime tag registry that handles datapack tag merging and resolution.
///
/// Tags are stored per registry (e.g., "block", "item", "`entity_type`").
/// Each registry maps tag IDs to their resolved element IDs.
///
/// Datapack tags are merged on top of Pumpkin's static compile-time tags.
/// Since Pumpkin accesses static tags via `Taggable::is_tagged_with()`, the
/// runtime registry serves as an additional source of truth.
#[derive(Debug, Clone)]
pub struct TagRegistry {
    /// Unresolved raw entries per (registry, `tag_id`).
    unresolved: HashMap<(String, Identifier), UnresolvedTag>,
    /// Resolved tags per (registry, `tag_id`).
    resolved: HashMap<(String, Identifier), Tag>,
}

impl TagRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            unresolved: HashMap::new(),
            resolved: HashMap::new(),
        }
    }

    /// Add or replace entries for a tag in a registry.
    pub fn add_or_replace(&mut self, registry: &str, tag_id: Identifier, entries: Vec<TagEntry>) {
        self.unresolved
            .insert((registry.to_string(), tag_id), UnresolvedTag { entries });
    }

    /// Append entries to an existing tag, or create a new one.
    pub fn add_or_append(&mut self, registry: &str, tag_id: Identifier, entries: Vec<TagEntry>) {
        let key = (registry.to_string(), tag_id);
        let entry = self.unresolved.entry(key).or_insert_with(|| UnresolvedTag {
            entries: Vec::new(),
        });
        entry.entries.extend(entries);
    }

    /// Get the resolved values for a tag in a specific registry.
    #[must_use]
    pub fn get_tag_values(&self, registry: &str, tag_id: &Identifier) -> Option<&[Identifier]> {
        self.resolved
            .get(&(registry.to_string(), tag_id.clone()))
            .map(|t| t.entries.as_slice())
    }

    /// Return the total number of resolved tags across all registries.
    #[must_use]
    pub fn tag_count(&self) -> usize {
        self.resolved.len()
    }

    /// Check if a specific element has a given tag in a registry.
    #[must_use]
    pub fn is_tagged(&self, registry: &str, element_id: &Identifier, tag_id: &Identifier) -> bool {
        self.get_tag_values(registry, tag_id)
            .is_some_and(|values| values.contains(element_id))
    }

    /// Check if an element is tagged using both static (compile-time) and dynamic (datapack) tags.
    ///
    /// This bridges Pumpkin's static `Taggable::is_tagged_with()` and the runtime tag registry.
    /// Call this when you need to check if something has a tag that might come from a datapack.
    ///
    /// # Arguments
    /// * `registry` - The tag registry key (e.g., "block", "item", "`entity_type`")
    /// * `element_key` - The element's registry key (e.g., "minecraft:stone")
    /// * `tag_name` - The tag name (e.g., "`minecraft:stone_ore_replaceables`")
    /// * `static_check` - Closure that checks the static tag (e.g., `|tag| item.is_tagged_with(tag)`)
    #[must_use]
    pub fn is_tagged_bridge(
        &self,
        registry: &str,
        element_key: &str,
        tag_name: &str,
        static_check: impl FnOnce(&str) -> Option<bool>,
    ) -> bool {
        // Check static tags first
        if static_check(tag_name) == Some(true) {
            return true;
        }
        // Fall through to dynamic tags
        let Ok(tag_id) = crate::Identifier::parse(tag_name) else {
            return false;
        };
        let Ok(element_id) = crate::Identifier::parse(element_key) else {
            return false;
        };
        self.is_tagged(registry, &element_id, &tag_id)
    }

    /// Resolve all tag references topologically.
    pub fn resolve_all(&mut self) -> Result<(), DatapackError> {
        const MAX_ITERATIONS: u32 = 100;

        let keys: Vec<(String, Identifier)> = self.unresolved.keys().cloned().collect();
        let mut errors = Vec::new();

        // Build a dependency graph and resolve iteratively
        let mut changed = true;
        let mut iteration = 0;

        while changed && iteration < MAX_ITERATIONS {
            changed = false;
            iteration += 1;

            for key in &keys {
                let Some(unresolved) = self.unresolved.get(key) else {
                    continue;
                };

                let mut resolved_ids = Vec::new();
                let mut fully_resolved = true;

                for entry in &unresolved.entries {
                    match entry {
                        TagEntry::Element(id, _required) => {
                            resolved_ids.push(id.clone());
                        }
                        TagEntry::TagReference(ref_id, required) => {
                            // Try to find the referenced tag within the same registry
                            let ref_key = (key.0.clone(), ref_id.clone());
                            if let Some(ref_tag) = self.resolved.get(&ref_key) {
                                resolved_ids.extend(ref_tag.entries.iter().cloned());
                            } else if *required {
                                fully_resolved = false;
                            }
                            // If not required and not resolved, skip silently
                        }
                    }
                }

                if fully_resolved {
                    let new_tag = Tag::new(resolved_ids);
                    let existing = self.resolved.get(key);
                    if existing.is_none_or(|e| e.entries != new_tag.entries) {
                        self.resolved.insert(key.clone(), new_tag);
                        changed = true;
                    }
                }
            }
        }

        if iteration >= MAX_ITERATIONS {
            // Report unresolved tags
            for key in keys {
                if !self.resolved.contains_key(&key) {
                    errors.push(format!(
                        "Tag {}:{} could not be fully resolved (circular or missing dependencies)",
                        key.0, key.1
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(DatapackError::Validation(errors))
        }
    }

    /// Replace all tags with a new set (used during reload).
    pub fn replace_with(&mut self, other: Self) {
        self.unresolved = other.unresolved;
        self.resolved = other.resolved;
    }
}

impl Default for TagRegistry {
    fn default() -> Self {
        Self::new()
    }
}
