//! Tags added at runtime, mostly so plugin items can join vanilla tags or form their own.
//!
//! There are two layers. Tags registered with [`register_tag`] (plugins) stay for the whole run.
//! Tags from datapacks are swapped as a whole by [`replace_datapack_tags`] on every datapack reload.
//!
//! Entries are kept as resource locations and only resolved when a membership check runs, so a tag
//! may name items that are registered later. The generated `Taggable::is_tagged_with` consults this
//! overlay next to the generated tag tables. The tags are not part of the client tag sync.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{PoisonError, RwLock};

use crate::tag::{RegistryKey, get_tag_ids};

const VANILLA_NAMESPACE_PREFIX: &str = "minecraft:";
/// Bounds `#tag` references so a cycle cannot recurse forever.
const MAX_TAG_DEPTH: u8 = 8;

type Tags = HashMap<(RegistryKey, String), Vec<String>>;

#[derive(Default)]
struct Overlay {
    plugin: Tags,
    datapack: Tags,
}

static OVERLAY: RwLock<Option<Overlay>> = RwLock::new(None);
/// Lets the common case, no runtime tags at all, skip the lock.
static HAS_TAGS: AtomicBool = AtomicBool::new(false);

fn normalize(location: &str) -> String {
    if location.contains(':') {
        location.to_string()
    } else {
        format!("{VANILLA_NAMESPACE_PREFIX}{location}")
    }
}

fn tag_name(tag: &str) -> String {
    normalize(tag.strip_prefix('#').unwrap_or(tag))
}

fn add_entries(
    tags: &mut Tags,
    registry: RegistryKey,
    tag: &str,
    entries: impl IntoIterator<Item = String>,
) {
    let values = tags.entry((registry, tag_name(tag))).or_default();
    for entry in entries {
        let entry = match entry.strip_prefix('#') {
            Some(reference) => format!("#{}", normalize(reference)),
            None => normalize(&entry),
        };
        if !values.contains(&entry) {
            values.push(entry);
        }
    }
}

/// Adds entries to a tag, creating it if needed. An entry is a resource location, or `#location`
/// for another tag. Entries of an existing tag, vanilla or runtime, are kept.
pub fn register_tag(registry: RegistryKey, tag: &str, entries: impl IntoIterator<Item = String>) {
    let mut guard = OVERLAY.write().unwrap_or_else(PoisonError::into_inner);
    let overlay = guard.get_or_insert_with(Overlay::default);
    add_entries(&mut overlay.plugin, registry, tag, entries);
    HAS_TAGS.store(true, Ordering::Release);
}

/// Replaces all datapack tags of a registry with `tags` (tag name and its entries).
pub fn replace_datapack_tags(
    registry: RegistryKey,
    tags: impl IntoIterator<Item = (String, Vec<String>)>,
) {
    let mut guard = OVERLAY.write().unwrap_or_else(PoisonError::into_inner);
    let overlay = guard.get_or_insert_with(Overlay::default);
    overlay.datapack.retain(|(key, _), _| *key != registry);
    for (tag, entries) in tags {
        add_entries(&mut overlay.datapack, registry, &tag, entries);
    }
    HAS_TAGS.store(true, Ordering::Release);
}

/// Whether the entry is in the runtime tag. `None` when no runtime tag has that name.
///
/// `location` is the entry's resource location, `id` its registry id (for generated tags that a
/// runtime tag references with `#`).
#[must_use]
pub fn is_member(registry: RegistryKey, tag: &str, location: &str, id: u16) -> Option<bool> {
    if !HAS_TAGS.load(Ordering::Acquire) {
        return None;
    }
    let guard = OVERLAY.read().unwrap_or_else(PoisonError::into_inner);
    let overlay = guard.as_ref()?;
    let tag = tag_name(tag);
    let key = (registry, tag.clone());
    if !overlay.plugin.contains_key(&key) && !overlay.datapack.contains_key(&key) {
        return None;
    }
    Some(contains(
        overlay,
        registry,
        &tag,
        &normalize(location),
        id,
        0,
    ))
}

fn contains(
    overlay: &Overlay,
    registry: RegistryKey,
    tag: &str,
    location: &str,
    id: u16,
    depth: u8,
) -> bool {
    if depth > MAX_TAG_DEPTH {
        return false;
    }
    let key = (registry, tag.to_string());
    [&overlay.plugin, &overlay.datapack]
        .into_iter()
        .filter_map(|tags| tags.get(&key))
        .flatten()
        .any(|entry| match entry.strip_prefix('#') {
            Some(reference) => {
                let in_generated = get_tag_ids(registry, reference)
                    .or_else(|| {
                        reference
                            .strip_prefix(VANILLA_NAMESPACE_PREFIX)
                            .and_then(|short| get_tag_ids(registry, short))
                    })
                    .is_some_and(|ids| ids.contains(&id));
                in_generated || contains(overlay, registry, reference, location, id, depth + 1)
            }
            None => entry == location,
        })
}

/// All tags of a registry for the client tag sync, the generated ones extended by the runtime
/// ones. `resolve` maps an entry's resource location to its registry id, unknown entries are
/// skipped. `None` when there are no runtime tags, then the generated tables are complete.
#[must_use]
pub fn merged_tags(
    registry: RegistryKey,
    resolve: impl Fn(&str) -> Option<u16>,
) -> Option<Vec<(String, Vec<u16>)>> {
    if !HAS_TAGS.load(Ordering::Acquire) {
        return None;
    }
    let guard = OVERLAY.read().unwrap_or_else(PoisonError::into_inner);
    let overlay = guard.as_ref()?;
    let runtime_names: Vec<&String> = [&overlay.plugin, &overlay.datapack]
        .into_iter()
        .flat_map(|tags| tags.keys())
        .filter(|(key, _)| *key == registry)
        .map(|(_, name)| name)
        .collect();
    if runtime_names.is_empty() {
        return None;
    }

    let mut merged: Vec<(String, Vec<u16>)> = crate::tag::get_latest_map(registry)
        .entries()
        .map(|(name, tag)| ((*name).to_string(), tag.1.to_vec()))
        .collect();
    for name in runtime_names {
        let mut ids = Vec::new();
        collect_ids(overlay, registry, name, &resolve, &mut ids, 0);
        match merged.iter_mut().find(|(existing, _)| existing == name) {
            Some((_, existing)) => {
                for id in ids {
                    if !existing.contains(&id) {
                        existing.push(id);
                    }
                }
            }
            None => {
                ids.sort_unstable();
                ids.dedup();
                merged.push((name.clone(), ids));
            }
        }
    }
    Some(merged)
}

fn collect_ids(
    overlay: &Overlay,
    registry: RegistryKey,
    tag: &str,
    resolve: &impl Fn(&str) -> Option<u16>,
    ids: &mut Vec<u16>,
    depth: u8,
) {
    if depth > MAX_TAG_DEPTH {
        return;
    }
    let key = (registry, tag.to_string());
    for entry in [&overlay.plugin, &overlay.datapack]
        .into_iter()
        .filter_map(|tags| tags.get(&key))
        .flatten()
    {
        match entry.strip_prefix('#') {
            Some(reference) => {
                if let Some(generated) = get_tag_ids(registry, reference) {
                    ids.extend_from_slice(generated);
                }
                collect_ids(overlay, registry, reference, resolve, ids, depth + 1);
            }
            None => ids.extend(resolve(entry)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Item;
    use crate::tag::Taggable;

    #[test]
    fn runtime_tags_extend_and_define_tags() {
        // Tag names are unique to this test, the overlay is global.
        register_tag(
            RegistryKey::Item,
            "test_tags:gems",
            [
                "test_tags:ruby".to_string(),
                "minecraft:diamond".to_string(),
            ],
        );
        register_tag(
            RegistryKey::Item,
            "#test_tags:wrapper",
            [
                "#test_tags:gems".to_string(),
                "#minecraft:swords".to_string(),
            ],
        );
        // Joins a generated tag.
        register_tag(
            RegistryKey::Item,
            "minecraft:swords",
            ["test_tags:ruby_sword".to_string()],
        );

        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:gems", "test_tags:ruby", 0),
            Some(true)
        );
        assert_eq!(
            is_member(RegistryKey::Item, "#test_tags:gems", "minecraft:coal", 0),
            Some(false)
        );
        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:none", "x:y", 0),
            None
        );

        assert_eq!(Item::DIAMOND.is_tagged_with("test_tags:gems"), Some(true));
        assert_eq!(Item::COAL.is_tagged_with("test_tags:gems"), Some(false));
        assert_eq!(
            Item::DIAMOND_SWORD.is_tagged_with("test_tags:wrapper"),
            Some(true)
        );
        assert_eq!(Item::COAL.is_tagged_with("#test_tags:wrapper"), Some(false));
        // Generated membership is unchanged.
        assert_eq!(
            Item::DIAMOND_SWORD.is_tagged_with("minecraft:swords"),
            Some(true)
        );
        assert_eq!(Item::COAL.is_tagged_with("swords"), Some(false));
        assert_eq!(Item::COAL.is_tagged_with("test_tags:undefined"), None);
    }

    #[test]
    fn datapack_tags_are_replaced_per_reload() {
        replace_datapack_tags(
            RegistryKey::Item,
            [(
                "test_tags:pack".to_string(),
                vec!["test_tags:a".to_string()],
            )],
        );
        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:pack", "test_tags:a", 0),
            Some(true)
        );
        replace_datapack_tags(
            RegistryKey::Item,
            [(
                "test_tags:pack".to_string(),
                vec!["test_tags:b".to_string()],
            )],
        );
        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:pack", "test_tags:a", 0),
            Some(false)
        );
        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:pack", "test_tags:b", 0),
            Some(true)
        );
        // Plugin tags survive a datapack reload.
        register_tag(
            RegistryKey::Item,
            "test_tags:kept",
            ["test_tags:a".to_string()],
        );
        replace_datapack_tags(RegistryKey::Item, []);
        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:kept", "test_tags:a", 0),
            Some(true)
        );
        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:pack", "test_tags:b", 0),
            None
        );
    }

    #[test]
    fn merged_tags_contain_runtime_item_ids() {
        use crate::item::Item;
        let gem = Item::register_dynamic(crate::item_registry::ItemRegistration {
            key: "test_sync:gem".to_string(),
            components: Vec::new(),
            max_stack_size: None,
        })
        .unwrap();
        register_tag(
            RegistryKey::Item,
            "test_sync:gems",
            ["test_sync:gem".to_string(), "minecraft:diamond".to_string()],
        );
        register_tag(
            RegistryKey::Item,
            "minecraft:swords",
            ["test_sync:gem".to_string()],
        );
        register_tag(
            RegistryKey::Item,
            "test_sync:wrapper",
            [
                "#test_sync:gems".to_string(),
                "#minecraft:pickaxes".to_string(),
            ],
        );
        let merged = merged_tags(RegistryKey::Item, |key| {
            Item::from_registry_key(key).map(|item| item.id)
        })
        .unwrap();
        let ids = |name: &str| &merged.iter().find(|(n, _)| n == name).unwrap().1;
        assert!(ids("test_sync:gems").contains(&gem.id));
        assert!(ids("test_sync:gems").contains(&Item::DIAMOND.id));
        // Vanilla tag keeps its ids and gains the item.
        assert!(ids("minecraft:swords").contains(&Item::DIAMOND_SWORD.id));
        assert!(ids("minecraft:swords").contains(&gem.id));
        assert!(ids("test_sync:wrapper").contains(&Item::DIAMOND_PICKAXE.id));
        assert!(ids("test_sync:wrapper").contains(&gem.id));
    }

    #[test]
    fn self_referencing_tags_terminate() {
        register_tag(
            RegistryKey::Item,
            "test_tags:loop",
            ["#test_tags:loop".to_string()],
        );
        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:loop", "minecraft:coal", 0),
            Some(false)
        );
    }
}
