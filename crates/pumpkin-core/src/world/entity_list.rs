//! The world's non-player entity list.
//!
//! Readers get an immutable snapshot (`load`), as with a plain `ArcSwap<Vec<_>>`. Spawns and
//! removals no longer copy the whole list each: they are queued, applied to the entity index
//! at once, and folded into the list by the next reader with a single copy. A chain reaction
//! that spawns and removes thousands of entities within a tick, between two reads of the
//! list, therefore copies it once instead of once per entity.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use arc_swap::{ArcSwap, Guard};
use rustc_hash::{FxHashMap, FxHashSet};
use uuid::Uuid;

use super::entity_index::EntityIndex;
use crate::entity::EntityBase;

pub type Registry = Vec<Arc<dyn EntityBase>>;

enum Change {
    Push(Arc<dyn EntityBase>),
    /// Every entity with this UUID queued or listed before the change.
    Remove(Uuid),
}

pub struct EntityList {
    published: ArcSwap<Registry>,
    /// Changes not yet in `published`, oldest first. Held while publishing, so a change
    /// and its index update are never split by a flush.
    pending: Mutex<Vec<Change>>,
    dirty: AtomicBool,
    /// Bumped by every change to the logical list (published plus pending). The entity
    /// index records the version it describes.
    version: AtomicU64,
}

impl Default for EntityList {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityList {
    #[must_use]
    pub fn new() -> Self {
        Self {
            published: ArcSwap::from_pointee(Vec::new()),
            pending: Mutex::new(Vec::new()),
            dirty: AtomicBool::new(false),
            version: AtomicU64::new(0),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Change>> {
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The current list, with every queued spawn and removal applied.
    pub fn load(&self) -> Guard<Arc<Registry>> {
        if self.dirty.load(Ordering::Acquire) {
            self.flush(&mut self.lock());
        }
        self.published.load()
    }

    /// Like [`Self::load`], as an owned `Arc`.
    pub fn load_full(&self) -> Arc<Registry> {
        Guard::into_inner(self.load())
    }

    /// The current list and the version it has, read together.
    pub(crate) fn snapshot(&self) -> (Arc<Registry>, u64) {
        let mut pending = self.lock();
        self.flush(&mut pending);
        (
            self.published.load_full(),
            self.version.load(Ordering::Acquire),
        )
    }

    pub(crate) fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }

    /// Replaces the whole list. Queued changes are dropped: they happened before.
    pub fn store(&self, entities: Arc<Registry>) {
        let mut pending = self.lock();
        pending.clear();
        // Published first: a reader that still sees `dirty` waits on the lock, one that no
        // longer does reads the replacement, never the list without its queued changes.
        self.published.store(entities);
        self.dirty.store(false, Ordering::Release);
        self.version.fetch_add(1, Ordering::AcqRel);
    }

    /// Replaces the list with `update(list)`. The entity index rebuilds on its next query.
    pub fn rcu(&self, mut update: impl FnMut(&Arc<Registry>) -> Registry) {
        let mut pending = self.lock();
        self.flush(&mut pending);
        let next = update(&self.published.load_full());
        self.published.store(Arc::new(next));
        self.version.fetch_add(1, Ordering::AcqRel);
    }

    /// Appends `entity` and indexes it.
    pub fn push(&self, entity: Arc<dyn EntityBase>, index: &EntityIndex) {
        let mut pending = self.lock();
        let version = self.version.load(Ordering::Acquire);
        self.queued();
        index.pushed(version, &entity);
        pending.push(Change::Push(entity));
        self.version.store(version + 1, Ordering::Release);
    }

    /// Takes every entity with `uuid` out of the list and the index.
    pub fn remove(&self, uuid: Uuid, index: &EntityIndex) {
        let mut pending = self.lock();
        let version = self.version.load(Ordering::Acquire);
        self.queued();
        index.removed(version, uuid);
        pending.push(Change::Remove(uuid));
        self.version.store(version + 1, Ordering::Release);
    }

    /// Marks the list dirty before the change becomes visible through the index (a new
    /// version): whoever sees the change in a lookup then reads a list that has it.
    fn queued(&self) {
        self.dirty.store(true, Ordering::Release);
    }

    /// Folds the queued changes into the published list with one copy.
    fn flush(&self, pending: &mut Vec<Change>) {
        if pending.is_empty() {
            return;
        }
        let changes = std::mem::take(pending);
        let next = apply(&self.published.load(), changes);
        self.published.store(Arc::new(next));
        self.dirty.store(false, Ordering::Release);
    }
}

/// `list` with `changes` applied in order.
fn apply(list: &Registry, changes: Vec<Change>) -> Registry {
    let pushes = changes
        .iter()
        .filter(|change| matches!(change, Change::Push(_)))
        .count();
    let mut next = Vec::with_capacity(list.len() + pushes);
    next.extend_from_slice(list);
    // UUIDs removed by some change, and the positions pushed after their last removal.
    let mut removed: FxHashSet<Uuid> = FxHashSet::default();
    let mut kept: FxHashMap<Uuid, Vec<usize>> = FxHashMap::default();
    for change in changes {
        match change {
            Change::Push(entity) => {
                let uuid = entity.get_entity().entity_uuid;
                if removed.contains(&uuid) {
                    kept.entry(uuid).or_default().push(next.len());
                }
                next.push(entity);
            }
            Change::Remove(uuid) => {
                removed.insert(uuid);
                kept.remove(&uuid);
            }
        }
    }
    if removed.is_empty() {
        return next;
    }
    let mut position = 0;
    next.retain(|entity| {
        let uuid = entity.get_entity().entity_uuid;
        let keep = !removed.contains(&uuid)
            || kept
                .get(&uuid)
                .is_some_and(|positions| positions.contains(&position));
        position += 1;
        keep
    });
    next
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::{boundingbox::BoundingBox, vector3::Vector3};

    use crate::entity::{EntityBase, ai::test_support::world, r#type::from_type};

    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.0 >> 33
        }
    }

    fn ids(entities: &[Arc<dyn EntityBase>]) -> Vec<i32> {
        entities.iter().map(|e| e.get_entity().entity_id).collect()
    }

    /// Queued spawns and removals, including an entity removed then added again before the
    /// list is read, give the list a naive copy-per-change model gives, and the index sees
    /// them before the list is folded.
    #[tokio::test]
    async fn queued_changes_match_a_copy_per_change_model() -> Result<(), Box<dyn std::error::Error>>
    {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let everything =
            BoundingBox::new(Vector3::new(-1e6, -1e6, -1e6), Vector3::new(1e6, 1e6, 1e6));
        let mut model: Vec<Arc<dyn EntityBase>> = Vec::new();
        let mut rng = Lcg(42);
        for round in 0..300 {
            for _ in 0..rng.next() % 6 {
                let roll = rng.next() % 10;
                if roll < 5 || model.is_empty() {
                    let pos =
                        Vector3::new((rng.next() % 64) as f64, 64.0, (rng.next() % 64) as f64);
                    let entity = from_type(&EntityType::COW, pos, &world, uuid::Uuid::new_v4());
                    world.add_entity_silent(entity.clone());
                    model.push(entity);
                } else {
                    let victim = model.remove((rng.next() as usize) % model.len());
                    let uuid = victim.get_entity().entity_uuid;
                    world.entities.remove(uuid, &world.entity_index);
                    // Re-registered under the same UUID before anyone reads the list.
                    if roll == 9 {
                        world.entities.push(victim.clone(), &world.entity_index);
                        model.push(victim);
                    }
                }
            }
            // The index answers before the list is folded.
            assert_eq!(ids(&world.get_entities_at_box(&everything)), ids(&model));
            for entity in &model {
                let id = entity.get_entity().entity_id;
                assert!(
                    world.get_entity_by_id(id).is_some(),
                    "round {round}: {id} missing"
                );
            }
            if round % 3 == 0 {
                assert_eq!(ids(&world.entities.load()), ids(&model), "round {round}");
            }
        }
        assert_eq!(ids(&world.entities.load()), ids(&model));
        Ok(())
    }

    /// `store` and `rcu` replace the list; the index follows on its next query.
    #[tokio::test]
    async fn store_and_rcu_reset_the_index() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let a = from_type(
            &EntityType::COW,
            Vector3::new(0.0, 64.0, 0.0),
            &world,
            uuid::Uuid::new_v4(),
        );
        let b = from_type(
            &EntityType::COW,
            Vector3::new(2.0, 64.0, 0.0),
            &world,
            uuid::Uuid::new_v4(),
        );
        world.add_entity_silent(a.clone());
        // Queued, then dropped by the store.
        world.add_entity_silent(b.clone());
        world.entities.store(Arc::new(vec![a.clone()]));
        assert!(world.get_entity_by_id(b.get_entity().entity_id).is_none());
        assert_eq!(ids(&world.entities.load()), ids(std::slice::from_ref(&a)));
        world.add_entity_silent(b.clone());
        world
            .entities
            .rcu(|list| list.iter().rev().cloned().collect());
        assert_eq!(ids(&world.entities.load()), ids(&[b.clone(), a.clone()]));
        let everything =
            BoundingBox::new(Vector3::new(-1e6, -1e6, -1e6), Vector3::new(1e6, 1e6, 1e6));
        assert_eq!(ids(&world.get_entities_at_box(&everything)), ids(&[b, a]));
        Ok(())
    }

    /// A spawner and a reader race. Whatever the reader finds through the index is in the
    /// list it reads next, and a type predicate that reads the list does not deadlock with
    /// the spawns (it runs after the index lock is released).
    #[tokio::test(flavor = "multi_thread")]
    async fn readers_racing_spawns_see_consistent_lists() -> Result<(), Box<dyn std::error::Error>>
    {
        const SPAWNS: usize = 3_000;
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let everything =
            BoundingBox::new(Vector3::new(-1e6, -1e6, -1e6), Vector3::new(1e6, 1e6, 1e6));
        let entities: Vec<_> = (0..SPAWNS)
            .map(|i| {
                let pos = Vector3::new((i % 64) as f64, 64.0, (i / 64) as f64);
                from_type(&EntityType::COW, pos, &world, uuid::Uuid::new_v4())
            })
            .collect();
        let ids: Vec<i32> = entities.iter().map(|e| e.get_entity().entity_id).collect();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                for entity in &entities {
                    world.add_entity_silent(entity.clone());
                }
            });
            scope.spawn(|| {
                let mut seen = 0;
                while seen < SPAWNS {
                    if world.get_entity_by_id(ids[seen]).is_some() {
                        let list = world.entities.load();
                        assert!(
                            list.iter().any(|e| e.get_entity().entity_id == ids[seen]),
                            "entity {} found by id but missing from the list",
                            ids[seen]
                        );
                        seen += 1;
                    }
                    let _ = world.get_entities_of_type_at_box(&everything, |_| {
                        !world.entities.load().is_empty()
                    });
                }
            });
        });
        assert_eq!(world.entities.load().len(), SPAWNS);
        Ok(())
    }
}
