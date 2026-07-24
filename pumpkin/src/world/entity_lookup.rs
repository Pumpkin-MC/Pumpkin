//! Vanilla-aligned entity storage.
//!
//! Ground truth (26.2 `EntityLookup` / `EntityTickList`):
//! - `EntityLookup`: `Int2ObjectMap byId` + `Map UUID byUuid` — O(1) add/remove/get
//! - `EntityTickList`: separate active map; iteration does **not** clone on every
//!   spawn/remove; tick walks the map (or a double-buffered view)
//!
//! Pumpkin previously used `ArcSwap<Vec<Arc<dyn EntityBase>>>` and RCU-cloned the
//! full vector on every spawn/remove (O(n) alloc). That is correct but scales
//! poorly past ~150–180 entities.
//!
//! This type mirrors vanilla `EntityLookup`: dual maps, O(1) mutations. Tick and
//! bulk queries take a snapshot of `Arc` handles once (pointer copies only).

use std::sync::Arc;

use dashmap::DashMap;
use uuid::Uuid;

use crate::entity::EntityBase;

/// Snapshot of live entities for iteration (tick, AABB queries, etc.).
///
/// Cheap to build: clones `Arc` handles only, not entity state.
#[derive(Clone, Default)]
pub struct EntitySnapshot(pub Vec<Arc<dyn EntityBase>>);

impl EntitySnapshot {
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<dyn EntityBase>> {
        self.0.iter()
    }
}

impl std::ops::Deref for EntitySnapshot {
    type Target = [Arc<dyn EntityBase>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Live entity index — vanilla `EntityLookup` analogue.
///
/// Thread-safe via `DashMap`. UUID is authoritative for identity (duplicate UUID
/// is rejected, same as vanilla's "Duplicate entity UUID" guard).
pub struct EntityLookup {
    by_id: DashMap<i32, Arc<dyn EntityBase>>,
    by_uuid: DashMap<Uuid, Arc<dyn EntityBase>>,
}

impl Default for EntityLookup {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityLookup {
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_id: DashMap::new(),
            by_uuid: DashMap::new(),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.by_uuid.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_uuid.is_empty()
    }

    /// Snapshot for tick / full iteration (vanilla `getAllEntities` / tick list).
    #[must_use]
    pub fn snapshot(&self) -> EntitySnapshot {
        EntitySnapshot(self.by_id.iter().map(|e| e.value().clone()).collect())
    }

    /// Compatibility alias used by call sites that previously did `entities.load()`.
    #[must_use]
    pub fn load(&self) -> EntitySnapshot {
        self.snapshot()
    }

    #[must_use]
    pub fn get_by_id(&self, id: i32) -> Option<Arc<dyn EntityBase>> {
        self.by_id.get(&id).map(|e| e.value().clone())
    }

    #[must_use]
    pub fn get_by_uuid(&self, uuid: Uuid) -> Option<Arc<dyn EntityBase>> {
        self.by_uuid.get(&uuid).map(|e| e.value().clone())
    }

    #[must_use]
    pub fn contains_uuid(&self, uuid: Uuid) -> bool {
        self.by_uuid.contains_key(&uuid)
    }

    /// Insert entity. Returns `false` if UUID already present (vanilla duplicate guard).
    pub fn add(&self, entity: Arc<dyn EntityBase>) -> bool {
        let base = entity.get_entity();
        let uuid = base.entity_uuid;
        let id = base.entity_id;
        if self.by_uuid.contains_key(&uuid) {
            return false;
        }
        self.by_uuid.insert(uuid, entity.clone());
        self.by_id.insert(id, entity);
        true
    }

    /// Remove by UUID. Returns the removed entity if it was present.
    pub fn remove_uuid(&self, uuid: Uuid) -> Option<Arc<dyn EntityBase>> {
        let entity = self.by_uuid.remove(&uuid)?.1;
        let id = entity.get_entity().entity_id;
        self.by_id.remove(&id);
        Some(entity)
    }

    /// Remove by entity identity (UUID).
    pub fn remove(&self, entity: &dyn EntityBase) -> Option<Arc<dyn EntityBase>> {
        self.remove_uuid(entity.get_entity().entity_uuid)
    }

    /// Remove all entities matching `should_remove`, returning those removed.
    pub fn drain_if(
        &self,
        mut should_remove: impl FnMut(&Arc<dyn EntityBase>) -> bool,
    ) -> Vec<Arc<dyn EntityBase>> {
        let mut removed = Vec::new();
        // Collect UUIDs first so we don't hold DashMap guards across remove.
        let uuids: Vec<Uuid> = self
            .by_uuid
            .iter()
            .filter_map(|entry| {
                if should_remove(entry.value()) {
                    Some(*entry.key())
                } else {
                    None
                }
            })
            .collect();
        for uuid in uuids {
            if let Some(entity) = self.remove_uuid(uuid) {
                removed.push(entity);
            }
        }
        removed
    }

    /// Extend with many entities (chunk load path). Skips duplicate UUIDs.
    pub fn extend(&self, entities: impl IntoIterator<Item = Arc<dyn EntityBase>>) {
        for entity in entities {
            let _ = self.add(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    // Structural unit tests live with integration coverage; EntityBase is heavy
    // to construct here. Smoke: empty lookup length.
    #[test]
    fn empty_lookup() {
        let lookup = super::EntityLookup::new();
        assert!(lookup.is_empty());
        assert_eq!(lookup.len(), 0);
        assert!(lookup.snapshot().is_empty());
    }
}
