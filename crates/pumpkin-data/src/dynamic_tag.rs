//! Tags added at runtime, so plugin items and blocks can join vanilla tags or form their own.
//!
//! Plugin tags stay for the whole run, datapack tags are replaced on every reload. The generated
//! `Taggable::is_tagged_with` and `has_tag` consult this overlay, and it is part of the client tag
//! sync. Entries are resolved to ids when published, so a tag may name entries registered later.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, PoisonError, RwLock};

use arc_swap::ArcSwapOption;

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

/// The tags as the layers define them.
static OVERLAY: RwLock<Option<Overlay>> = RwLock::new(None);
/// The resolved tags that membership checks read.
static SNAPSHOT: ArcSwapOption<Snapshot> = ArcSwapOption::const_empty();
/// Lets the common case, no runtime tags, skip everything.
static HAS_TAGS: AtomicBool = AtomicBool::new(false);

/// The resolved runtime tags of items and blocks, by registry and normalized tag name.
#[derive(Default)]
struct Snapshot {
    tags: HashMap<RegistryKey, HashMap<Box<str>, Box<[u16]>>>,
}

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

/// Registry id of an entry, for registries that can have runtime tags.
fn resolve(registry: RegistryKey, location: &str) -> Option<u16> {
    match registry {
        #[cfg(feature = "item")]
        RegistryKey::Item => crate::item::Item::from_registry_key(location).map(|item| item.id),
        #[cfg(feature = "block")]
        RegistryKey::Block => crate::Block::from_name(location).map(|block| block.id.as_u16()),
        _ => None,
    }
}

/// Whether tags of this registry are resolved to ids and published.
const fn has_ids(registry: RegistryKey) -> bool {
    matches!(registry, RegistryKey::Item | RegistryKey::Block)
}

fn publish(overlay: &Overlay) {
    let mut snapshot = Snapshot::default();
    for registry in [RegistryKey::Item, RegistryKey::Block] {
        let names: Vec<&String> = [&overlay.plugin, &overlay.datapack]
            .into_iter()
            .flat_map(|tags| tags.keys())
            .filter(|(key, _)| *key == registry)
            .map(|(_, name)| name)
            .collect();
        if names.is_empty() {
            continue;
        }
        let tags = snapshot.tags.entry(registry).or_default();
        for name in names {
            let mut ids = Vec::new();
            collect_ids(overlay, registry, name, &mut ids, 0);
            ids.sort_unstable();
            ids.dedup();
            tags.insert(name.as_str().into(), ids.into_boxed_slice());
        }
    }
    SNAPSHOT.store(Some(Arc::new(snapshot)));
    HAS_TAGS.store(true, Ordering::Release);
}

/// Adds entries to a tag, creating it if needed. An entry is a resource location, or `#location`
/// for another tag. Entries of an existing tag, vanilla or runtime, are kept.
pub fn register_tag(registry: RegistryKey, tag: &str, entries: impl IntoIterator<Item = String>) {
    let mut guard = OVERLAY.write().unwrap_or_else(PoisonError::into_inner);
    let overlay = guard.get_or_insert_with(Overlay::default);
    add_entries(&mut overlay.plugin, registry, tag, entries);
    publish(overlay);
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
    publish(overlay);
}

/// Publishes the tags again so entries registered after their tag resolve. Called when the item
/// and block registries close.
pub fn refresh() {
    let guard = OVERLAY.read().unwrap_or_else(PoisonError::into_inner);
    if let Some(overlay) = guard.as_ref() {
        publish(overlay);
    }
}

/// Whether the id is in the runtime tag of an item or block registry, `None` when no runtime tag
/// has that name. `tag` may be `#`-prefixed and without the `minecraft:` namespace.
#[must_use]
pub fn contains_id(registry: RegistryKey, tag: &str, id: u16) -> Option<bool> {
    if !HAS_TAGS.load(Ordering::Acquire) {
        return None;
    }
    let snapshot = SNAPSHOT.load();
    let tags = snapshot.as_ref()?.tags.get(&registry)?;
    let tag = tag.strip_prefix('#').unwrap_or(tag);
    let ids = if tag.contains(':') {
        tags.get(tag)?
    } else {
        // The vanilla namespace is implied. A stack buffer avoids a heap allocation.
        let mut buffer = [0u8; 96];
        let length = VANILLA_NAMESPACE_PREFIX.len() + tag.len();
        match buffer.get_mut(..length) {
            Some(name) => {
                name[..VANILLA_NAMESPACE_PREFIX.len()]
                    .copy_from_slice(VANILLA_NAMESPACE_PREFIX.as_bytes());
                name[VANILLA_NAMESPACE_PREFIX.len()..].copy_from_slice(tag.as_bytes());
                tags.get(std::str::from_utf8(name).ok()?)?
            }
            None => tags.get(normalize(tag).as_str())?,
        }
    };
    Some(ids.binary_search(&id).is_ok())
}

/// Whether the entry is in the runtime tag, `None` when no runtime tag has that name. Items and
/// blocks are checked by `id`, other registries by `location`.
#[must_use]
pub fn is_member(registry: RegistryKey, tag: &str, location: &str, id: u16) -> Option<bool> {
    if has_ids(registry) {
        return contains_id(registry, tag, id);
    }
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

/// All tags of a registry for the client tag sync: the generated ones plus the runtime ones.
/// `None` when there are no runtime tags.
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
        collect_ids_with(overlay, registry, name, &resolve, &mut ids, 0);
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

fn collect_ids(overlay: &Overlay, registry: RegistryKey, tag: &str, ids: &mut Vec<u16>, depth: u8) {
    collect_ids_with(
        overlay,
        registry,
        tag,
        &|location| resolve(registry, location),
        ids,
        depth,
    );
}

fn collect_ids_with(
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
                collect_ids_with(overlay, registry, reference, resolve, ids, depth + 1);
            }
            None => ids.extend(resolve(entry)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Item;
    use crate::item_registry::ItemRegistration;
    use crate::tag::Taggable;

    /// Registers (or finds) a plugin item and returns its id.
    fn item(key: &str) -> u16 {
        Item::register_dynamic(ItemRegistration {
            key: key.to_string(),
            components: Vec::new(),
            max_stack_size: None,
        })
        .expect("test item registers")
        .id
    }

    #[test]
    fn runtime_tags_extend_and_define_tags() {
        // Tag names are unique to this test, the overlay is global.
        let ruby = item("test_tags:ruby");
        item("test_tags:ruby_sword");
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
        register_tag(
            RegistryKey::Item,
            "minecraft:swords",
            ["test_tags:ruby_sword".to_string()],
        );

        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:gems", "test_tags:ruby", ruby),
            Some(true)
        );
        assert_eq!(
            is_member(
                RegistryKey::Item,
                "#test_tags:gems",
                "minecraft:coal",
                Item::COAL.id
            ),
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
        assert_eq!(
            Item::DIAMOND_SWORD.is_tagged_with("minecraft:swords"),
            Some(true)
        );
        assert_eq!(Item::COAL.is_tagged_with("swords"), Some(false));
        assert_eq!(Item::COAL.is_tagged_with("test_tags:undefined"), None);
    }

    #[test]
    fn datapack_tags_are_replaced_per_reload() {
        let a = item("test_tags:a");
        let b = item("test_tags:b");
        replace_datapack_tags(
            RegistryKey::Item,
            [(
                "test_tags:pack".to_string(),
                vec!["test_tags:a".to_string()],
            )],
        );
        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:pack", "test_tags:a", a),
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
            is_member(RegistryKey::Item, "test_tags:pack", "test_tags:a", a),
            Some(false)
        );
        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:pack", "test_tags:b", b),
            Some(true)
        );
        register_tag(
            RegistryKey::Item,
            "test_tags:kept",
            ["test_tags:a".to_string()],
        );
        replace_datapack_tags(RegistryKey::Item, []);
        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:kept", "test_tags:a", a),
            Some(true)
        );
        assert_eq!(
            is_member(RegistryKey::Item, "test_tags:pack", "test_tags:b", b),
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
            is_member(
                RegistryKey::Item,
                "test_tags:loop",
                "minecraft:coal",
                Item::COAL.id
            ),
            Some(false)
        );
    }

    #[test]
    fn membership_accepts_every_spelling_of_a_tag() {
        let stone_gem = item("test_spelling:gem");
        register_tag(
            RegistryKey::Item,
            "minecraft:test_spelling_vanilla",
            ["test_spelling:gem".to_string()],
        );
        for spelling in [
            "minecraft:test_spelling_vanilla",
            "test_spelling_vanilla",
            "#minecraft:test_spelling_vanilla",
            "#test_spelling_vanilla",
        ] {
            assert_eq!(
                contains_id(RegistryKey::Item, spelling, stone_gem),
                Some(true),
                "{spelling}"
            );
            assert_eq!(
                contains_id(RegistryKey::Item, spelling, Item::COAL.id),
                Some(false),
                "{spelling}"
            );
        }
        assert_eq!(
            contains_id(RegistryKey::Item, "test_spelling_none", 0),
            None
        );
    }

    #[test]
    fn entries_registered_after_their_tag_are_resolved_by_a_refresh() {
        register_tag(
            RegistryKey::Item,
            "test_late:tag",
            ["test_late:gem".to_string()],
        );
        assert_eq!(
            contains_id(RegistryKey::Item, "test_late:tag", Item::COAL.id),
            Some(false)
        );
        let gem = item("test_late:gem");
        refresh();
        assert_eq!(
            contains_id(RegistryKey::Item, "test_late:tag", gem),
            Some(true)
        );
    }
}
