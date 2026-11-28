use std::sync::{Arc, RwLock, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::boundingbox::BoundingBox;
use rustc_hash::FxHashMap;
use uuid::Uuid;

use super::entity_list::{EntityList, Registry};
use crate::entity::EntityBase;
use crate::entity::mob::has_game_event_listener;
use crate::entity::player::Player;
use crossbeam::atomic::AtomicCell;

type PlayerRegistry = Vec<Arc<Player>>;
type Cell = [i32; 3];

const CELL_SIZE: f64 = 4.0;
// This limits index allocation, not entity size or query coverage.
const MAX_ENTITY_CELLS: u64 = 64;

#[derive(Clone, Copy, PartialEq, Eq)]
struct CellRange {
    min: Cell,
    max: Cell,
}

impl CellRange {
    fn of(area: &BoundingBox) -> Option<Self> {
        let min = [area.min.x, area.min.y, area.min.z];
        let max = [area.max.x, area.max.y, area.max.z];
        if min
            .iter()
            .zip(max)
            .any(|(min, max)| !min.is_finite() || !max.is_finite() || *min > max)
        {
            return None;
        }
        Some(Self {
            min: min.map(|n| (n / CELL_SIZE).floor() as i32),
            max: max.map(|n| (n / CELL_SIZE).floor() as i32),
        })
    }

    fn count(self) -> u64 {
        self.min
            .into_iter()
            .zip(self.max)
            .fold(1u64, |count, (min, max)| {
                count.saturating_mul((i64::from(max) - i64::from(min) + 1) as u64)
            })
    }

    fn cells(self) -> impl Iterator<Item = Cell> {
        (self.min[0]..=self.max[0]).flat_map(move |x| {
            (self.min[1]..=self.max[1])
                .flat_map(move |y| (self.min[2]..=self.max[2]).map(move |z| [x, y, z]))
        })
    }

    fn contains(self, cell: Cell) -> bool {
        (0..3).all(|axis| self.min[axis] <= cell[axis] && cell[axis] <= self.max[axis])
    }
}

/// Non-player entities by key. Keys grow with registry order: a push appends one, a removal
/// leaves a hole, so key order is registry order and a spawn or removal updates the index in
/// place instead of rebuilding it. Any other change to the registry rebuilds it.
#[derive(Default)]
struct Index {
    /// The [`EntityList`] version this index describes; `None` until the next rebuild.
    version: Option<u64>,
    entities: Vec<Weak<dyn EntityBase>>,
    alive: Vec<bool>,
    ids: Vec<i32>,
    entity_uuids: Vec<Uuid>,
    live: usize,
    /// An id or UUID is registered twice: removals then rebuild, to keep the lookup rules.
    duplicates: bool,
    slots: FxHashMap<i32, usize>,
    uuids: FxHashMap<Uuid, usize>,
    ranges: Vec<Option<CellRange>>,
    /// Copy of each entity's bounds, so exact tests skip the entity itself.
    bounds: Vec<AtomicCell<BoundingBox>>,
    types: Vec<&'static EntityType>,
    /// Mobs with game event listeners, in list order.
    listeners: Vec<usize>,
    /// Mobs, in list order.
    mobs: Vec<usize>,
    cells: FxHashMap<Cell, Vec<usize>>,
    oversized: Vec<usize>,
    #[cfg(test)]
    rebuilds: usize,
}

impl Index {
    fn rebuild(&mut self, registry: &Registry, version: u64) {
        self.entities.clear();
        self.alive.clear();
        self.ids.clear();
        self.entity_uuids.clear();
        self.live = 0;
        self.duplicates = false;
        self.slots.clear();
        self.uuids.clear();
        self.ranges.clear();
        self.cells.clear();
        self.oversized.clear();
        self.types.clear();
        self.bounds.clear();
        self.listeners.clear();
        self.mobs.clear();
        for registered in registry {
            self.push(registered);
        }
        self.version = Some(version);
        #[cfg(test)]
        {
            self.rebuilds += 1;
        }
    }

    /// Appends `registered` as the last entity in registry order.
    fn push(&mut self, registered: &Arc<dyn EntityBase>) {
        let key = self.entities.len();
        let entity = registered.get_entity();
        // The last id wins and the first UUID, like the rebuild always did.
        if self.slots.insert(entity.entity_id, key).is_some() {
            self.duplicates = true;
        }
        match self.uuids.entry(entity.entity_uuid) {
            std::collections::hash_map::Entry::Occupied(_) => self.duplicates = true,
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(key);
            }
        }
        self.entities.push(Arc::downgrade(registered));
        self.alive.push(true);
        self.ids.push(entity.entity_id);
        self.entity_uuids.push(entity.entity_uuid);
        self.types.push(entity.entity_type);
        if let Some(mob) = registered.get_mob() {
            self.mobs.push(key);
            if has_game_event_listener(mob) {
                self.listeners.push(key);
            }
        }
        let bounds = entity.bounding_box.load();
        self.bounds.push(AtomicCell::new(bounds));
        let range = Self::entity_range(&bounds);
        self.ranges.push(range);
        self.insert(key, range);
        self.live += 1;
    }

    /// Takes the entity with `key` out, leaving a hole so later keys keep their order.
    fn remove(&mut self, key: usize) {
        if !self.alive[key] {
            return;
        }
        self.alive[key] = false;
        self.live -= 1;
        let released: Weak<dyn EntityBase> = Weak::<crate::entity::marker::MarkerEntity>::new();
        self.entities[key] = released;
        if self.slots.get(&self.ids[key]) == Some(&key) {
            self.slots.remove(&self.ids[key]);
        }
        if self.uuids.get(&self.entity_uuids[key]) == Some(&key) {
            self.uuids.remove(&self.entity_uuids[key]);
        }
        self.unlink(key, self.ranges[key]);
        self.ranges[key] = None;
        for list in [&mut self.listeners, &mut self.mobs] {
            if let Ok(position) = list.binary_search(&key) {
                list.remove(position);
            }
        }
    }

    /// Holes left by removals, past which a removal compacts the keys with a rebuild.
    fn too_sparse(&self) -> bool {
        self.entities.len() - self.live > self.live.max(1024)
    }

    fn entity_range(area: &BoundingBox) -> Option<CellRange> {
        CellRange::of(area).filter(|range| range.count() <= MAX_ENTITY_CELLS)
    }

    fn insert(&mut self, slot: usize, range: Option<CellRange>) {
        if let Some(range) = range {
            for cell in range.cells() {
                self.cells.entry(cell).or_default().push(slot);
            }
        } else {
            self.oversized.push(slot);
        }
    }

    fn unlink(&mut self, slot: usize, range: Option<CellRange>) {
        if let Some(range) = range {
            for cell in range.cells() {
                if let std::collections::hash_map::Entry::Occupied(mut entry) =
                    self.cells.entry(cell)
                {
                    entry.get_mut().retain(|member| *member != slot);
                    if entry.get().is_empty() {
                        entry.remove();
                    }
                }
            }
        } else {
            self.oversized.retain(|member| *member != slot);
        }
    }

    fn update(&mut self, entity_id: i32, area: &BoundingBox) {
        let Some(&slot) = self.slots.get(&entity_id) else {
            return;
        };
        self.bounds[slot].store(*area);
        let next = Self::entity_range(area);
        let previous = self.ranges[slot];
        if previous == next {
            return;
        }
        self.unlink(slot, previous);
        self.ranges[slot] = next;
        self.insert(slot, next);
    }

    fn intersects(&self, slot: usize, area: &BoundingBox) -> bool {
        area.intersects(&self.bounds[slot].load())
    }

    fn entity(&self, slot: usize) -> Option<Arc<dyn EntityBase>> {
        self.entities[slot].upgrade()
    }

    /// Slots in registry order whose cells touch `area`.
    fn candidates(&self, area: &BoundingBox) -> Vec<usize> {
        let Some(range) = CellRange::of(area) else {
            return (0..self.types.len())
                .filter(|slot| self.alive[*slot])
                .collect();
        };
        let mut slots: Vec<usize> = self.oversized.clone();
        if range.count() <= self.cells.len() as u64 {
            for cell in range.cells() {
                if let Some(members) = self.cells.get(&cell) {
                    slots.extend_from_slice(members);
                }
            }
        } else {
            for (&cell, members) in &self.cells {
                if range.contains(cell) {
                    slots.extend_from_slice(members);
                }
            }
        }
        // Keep the registry order, including entities spanning several cells.
        slots.sort_unstable();
        slots.dedup();
        slots
    }
}

#[derive(Default)]
struct PlayerIndex {
    registry: Weak<PlayerRegistry>,
    slots: FxHashMap<i32, usize>,
    uuids: FxHashMap<Uuid, usize>,
    ranges: Vec<Option<CellRange>>,
    bounds: Vec<AtomicCell<BoundingBox>>,
    cells: FxHashMap<Cell, Vec<usize>>,
    oversized: Vec<usize>,
}

impl PlayerIndex {
    fn matches(&self, registry: &Arc<PlayerRegistry>) -> bool {
        self.registry.as_ptr() == Arc::as_ptr(registry)
    }

    fn rebuild(&mut self, registry: &Arc<PlayerRegistry>) {
        self.slots.clear();
        self.uuids.clear();
        self.ranges.clear();
        self.cells.clear();
        self.oversized.clear();
        self.bounds.clear();
        for (slot, player) in registry.iter().enumerate() {
            let entity = player.get_entity();
            self.slots.insert(entity.entity_id, slot);
            self.uuids.entry(player.gameprofile.id).or_insert(slot);
            let bounds = entity.bounding_box.load();
            self.bounds.push(AtomicCell::new(bounds));
            let range = Index::entity_range(&bounds);
            self.ranges.push(range);
            self.insert(slot, range);
        }
        self.registry = Arc::downgrade(registry);
    }

    fn insert(&mut self, slot: usize, range: Option<CellRange>) {
        if let Some(range) = range {
            for cell in range.cells() {
                self.cells.entry(cell).or_default().push(slot);
            }
        } else {
            self.oversized.push(slot);
        }
    }

    fn update(&mut self, entity_id: i32, area: &BoundingBox) {
        let Some(&slot) = self.slots.get(&entity_id) else {
            return;
        };
        self.bounds[slot].store(*area);
        let next = Index::entity_range(area);
        let previous = self.ranges[slot];
        if previous == next {
            return;
        }
        if let Some(previous) = previous {
            for cell in previous.cells() {
                if let std::collections::hash_map::Entry::Occupied(mut entry) =
                    self.cells.entry(cell)
                {
                    entry.get_mut().retain(|member| *member != slot);
                    if entry.get().is_empty() {
                        entry.remove();
                    }
                }
            }
        } else {
            self.oversized.retain(|member| *member != slot);
        }
        self.ranges[slot] = next;
        self.insert(slot, next);
    }

    fn intersects(&self, slot: usize, area: &BoundingBox) -> bool {
        area.intersects(&self.bounds[slot].load())
    }

    fn candidates(&self, area: &BoundingBox) -> Vec<usize> {
        let Some(range) = CellRange::of(area) else {
            return (0..self.bounds.len()).collect();
        };
        let mut slots: Vec<usize> = self.oversized.clone();
        if range.count() <= self.cells.len() as u64 {
            for cell in range.cells() {
                if let Some(members) = self.cells.get(&cell) {
                    slots.extend(members.iter().copied());
                }
            }
        } else {
            for (&cell, members) in &self.cells {
                if range.contains(cell) {
                    slots.extend(members.iter().copied());
                }
            }
        }
        slots.sort_unstable();
        slots.dedup();
        slots
    }
}

/// Live broad phase for the world's non-player entity queries.
#[derive(Default)]
pub struct EntityIndex {
    inner: RwLock<Index>,
    players: RwLock<PlayerIndex>,
}

impl EntityIndex {
    /// Runs `query` on an index that describes `list`, rebuilding it first if it does not.
    fn with_index<T>(&self, list: &EntityList, query: impl FnOnce(&Index) -> T) -> T {
        let index = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if index.version == Some(list.version()) {
            return query(&index);
        }
        drop(index);
        // Read before locking the index: a spawn holds the list's lock, then the index's.
        let (registry, version) = list.snapshot();
        let mut index = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if index.version != Some(list.version()) {
            index.rebuild(&registry, version);
        }
        query(&index)
    }

    fn with_players<T>(
        &self,
        registry: &Arc<PlayerRegistry>,
        query: impl FnOnce(&PlayerIndex) -> T,
    ) -> T {
        let index = self
            .players
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if index.matches(registry) {
            return query(&index);
        }
        drop(index);
        let mut index = self
            .players
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !index.matches(registry) {
            index.rebuild(registry);
        }
        query(&index)
    }

    pub fn query(
        &self,
        list: &EntityList,
        area: BoundingBox,
    ) -> impl Iterator<Item = Arc<dyn EntityBase>> + use<> {
        self.query_types(list, area, |_| true)
    }

    /// Like [`Self::query`], limited to entity types that pass `matches`.
    pub fn query_types<F: Fn(&'static EntityType) -> bool>(
        &self,
        list: &EntityList,
        area: BoundingBox,
        matches: F,
    ) -> impl Iterator<Item = Arc<dyn EntityBase>> + use<F> {
        // `matches` runs after the index lock is released: a predicate that reads the world
        // (and so the entity list) must not wait on a spawn that waits on this lock.
        let entities: Vec<_> = self.with_index(list, |index| {
            index
                .candidates(&area)
                .into_iter()
                .filter(|&slot| index.intersects(slot, &area))
                .filter_map(|slot| Some((index.types[slot], index.entity(slot)?)))
                .collect()
        });
        entities
            .into_iter()
            .filter(move |(entity_type, _)| matches(entity_type))
            .map(|(_, entity)| entity)
    }

    pub fn query_players(
        &self,
        registry: Arc<PlayerRegistry>,
        area: BoundingBox,
    ) -> impl Iterator<Item = Arc<Player>> + use<> {
        let slots = self.with_players(&registry, |index| {
            let mut slots = index.candidates(&area);
            slots.retain(|&slot| index.intersects(slot, &area));
            slots
        });
        slots.into_iter().map(move |slot| registry[slot].clone())
    }

    /// Mobs with game event listeners whose body overlaps `area`, in list order.
    pub fn listeners(
        &self,
        list: &EntityList,
        area: BoundingBox,
    ) -> impl Iterator<Item = Arc<dyn EntityBase>> + use<> {
        let entities: Vec<_> = self.with_index(list, |index| {
            index
                .listeners
                .iter()
                .copied()
                .filter(|&slot| index.intersects(slot, &area))
                .filter_map(|slot| index.entity(slot))
                .collect()
        });
        entities.into_iter()
    }

    /// Every mob, in list order, without walking the other entities.
    pub fn mobs(&self, list: &EntityList) -> Vec<Arc<dyn EntityBase>> {
        self.with_index(list, |index| {
            index
                .mobs
                .iter()
                .filter_map(|&slot| index.entity(slot))
                .collect()
        })
    }

    pub fn by_id(&self, list: &EntityList, id: i32) -> Option<Arc<dyn EntityBase>> {
        self.with_index(list, |index| {
            index.slots.get(&id).and_then(|&slot| index.entity(slot))
        })
    }

    /// The first entity with `uuid` in registry order, without a scan.
    pub fn by_uuid(&self, list: &EntityList, uuid: Uuid) -> Option<Arc<dyn EntityBase>> {
        self.with_index(list, |index| {
            index.uuids.get(&uuid).and_then(|&slot| index.entity(slot))
        })
    }

    /// The list at `version` gets `entity` appended: indexes it in place when the index
    /// describes that version, otherwise leaves the next query to rebuild it.
    pub(super) fn pushed(&self, version: u64, entity: &Arc<dyn EntityBase>) {
        let mut index = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if index.version != Some(version) {
            return;
        }
        index.push(entity);
        index.version = Some(version + 1);
    }

    /// The list at `version` loses the entities with `uuid`: updates the index in place when
    /// it describes that version, otherwise leaves the next query to rebuild it. Duplicate
    /// registrations and too many holes also leave it to a rebuild.
    pub(super) fn removed(&self, version: u64, uuid: Uuid) {
        let mut index = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if index.version != Some(version) {
            return;
        }
        if index.duplicates {
            index.version = None;
            return;
        }
        if let Some(&slot) = index.uuids.get(&uuid) {
            index.remove(slot);
        }
        index.version = if index.too_sparse() {
            None
        } else {
            Some(version + 1)
        };
    }

    #[cfg(test)]
    fn rebuilds(&self) -> usize {
        self.inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .rebuilds
    }

    pub fn player_by_id(&self, registry: &Arc<PlayerRegistry>, id: i32) -> Option<Arc<Player>> {
        let slot = self.with_players(registry, |index| index.slots.get(&id).copied())?;
        Some(registry[slot].clone())
    }

    pub fn player_by_uuid(
        &self,
        registry: &Arc<PlayerRegistry>,
        uuid: Uuid,
    ) -> Option<Arc<Player>> {
        let slot = self.with_players(registry, |index| index.uuids.get(&uuid).copied())?;
        Some(registry[slot].clone())
    }

    pub fn set_bounds(&self, entity_id: i32, bounds: &AtomicCell<BoundingBox>, area: BoundingBox) {
        let index = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let next = Index::entity_range(&area);
        match index.slots.get(&entity_id) {
            None => {
                bounds.store(area);
                return;
            }
            Some(&slot) if index.ranges[slot] == next => {
                index.bounds[slot].store(area);
                bounds.store(area);
                return;
            }
            Some(_) => {}
        }
        drop(index);
        let mut index = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        index.update(entity_id, &area);
        bounds.store(area);
    }

    pub fn set_player_bounds(
        &self,
        entity_id: i32,
        bounds: &AtomicCell<BoundingBox>,
        area: BoundingBox,
    ) {
        let index = self
            .players
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let next = Index::entity_range(&area);
        match index.slots.get(&entity_id) {
            None => {
                bounds.store(area);
                return;
            }
            Some(&slot) if index.ranges[slot] == next => {
                index.bounds[slot].store(area);
                bounds.store(area);
                return;
            }
            Some(_) => {}
        }
        drop(index);
        let mut index = self
            .players
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        index.update(entity_id, &area);
        bounds.store(area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{
        Entity, ai::test_support::world, interaction::InteractionEntity, passive::cow::CowEntity,
    };
    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;

    fn ids(world: &super::super::World, area: &BoundingBox) -> Vec<i32> {
        world
            .get_entities_at_box(area)
            .iter()
            .map(|entity| entity.get_entity().entity_id)
            .collect()
    }

    #[tokio::test]
    async fn queries_follow_moves_resizes_and_registry_replacement()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(-4.7, 64.0, 0.0),
            &EntityType::COW,
        ));
        world.entities.store(Arc::new(vec![body.clone()]));
        let area = BoundingBox::new(
            Vector3::new(-4.2, 64.0, -0.1),
            Vector3::new(-4.1, 65.0, 0.1),
        );
        assert!(ids(&world, &area).is_empty());
        assert!(
            world
                .get_entity_by_uuid(body.get_entity().entity_uuid)
                .is_some()
        );
        // A bound change within the same grid cells still changes exact membership.
        body.get_entity().set_pos(Vector3::new(-4.6, 64.0, 0.0));
        assert_eq!(ids(&world, &area), [body.get_entity().entity_id]);
        body.get_entity().set_pos(Vector3::new(8.0, 64.0, 0.0));
        assert!(ids(&world, &area).is_empty());
        body.get_entity().bounding_box.store(BoundingBox::new(
            Vector3::new(-5.0, 64.0, -13.0),
            Vector3::new(21.0, 65.0, 13.0),
        ));
        assert_eq!(ids(&world, &area), [body.get_entity().entity_id]);
        body.get_entity()
            .set_pose(pumpkin_data::entity::EntityPose::Dying);
        assert!(ids(&world, &area).is_empty());
        let replacement = InteractionEntity::new(Entity::new(
            world.clone(),
            Vector3::new(-4.0, 64.0, 0.0),
            &EntityType::INTERACTION,
        ));
        world.entities.store(Arc::new(vec![replacement.clone()]));
        assert_eq!(ids(&world, &area), [replacement.entity.entity_id]);
        assert!(
            world
                .get_entity_by_id(body.get_entity().entity_id)
                .is_none()
        );
        assert!(
            world
                .get_entity_by_uuid(body.get_entity().entity_uuid)
                .is_none()
        );
        assert!(
            world
                .get_entity_by_uuid(replacement.entity.entity_uuid)
                .is_some()
        );
        world.entities.store(Arc::new(Vec::new()));
        let removed = Arc::downgrade(&replacement);
        assert!(ids(&world, &area).is_empty());
        drop(replacement);
        assert!(removed.upgrade().is_none());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn queries_keep_order_and_coverage_across_cells_and_oversized_bounds()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let near = InteractionEntity::new(Entity::new(
            world.clone(),
            Vector3::new(-0.1, 64.0, 0.0),
            &EntityType::INTERACTION,
        ));
        let wide = InteractionEntity::new(Entity::new(
            world.clone(),
            Vector3::new(100.0, 64.0, 0.0),
            &EntityType::INTERACTION,
        ));
        wide.set_width(220.0);
        world
            .entities
            .store(Arc::new(vec![wide.clone(), near.clone()]));
        let area = BoundingBox::new(Vector3::new(-1.0, 63.0, -1.0), Vector3::new(1.0, 66.0, 1.0));
        assert_eq!(
            ids(&world, &area),
            [wide.entity.entity_id, near.entity.entity_id]
        );
        let huge = BoundingBox::new(
            Vector3::new(-1e20, -1e20, -1e20),
            Vector3::new(1e20, 1e20, 1e20),
        );
        assert_eq!(
            ids(&world, &huge),
            [wide.entity.entity_id, near.entity.entity_id]
        );
        // Registry order can change independently of bounds, including island spawn ordering.
        world
            .entities
            .store(Arc::new(vec![near.clone(), wide.clone()]));
        assert_eq!(
            ids(&world, &area),
            [near.entity.entity_id, wide.entity.entity_id]
        );
        wide.set_width(1.0);
        assert_eq!(ids(&world, &area), [near.entity.entity_id]);
        let mut found = Vec::new();
        world.extend_entities_in_box_where(&mut found, usize::MAX, area, |_| true);
        assert_eq!(
            found
                .iter()
                .map(|entity| entity.get_entity().entity_id)
                .collect::<Vec<_>>(),
            [near.entity.entity_id]
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn transferred_bounds_update_the_destination_index()
    -> Result<(), Box<dyn std::error::Error>> {
        let source_folder = tempfile::tempdir()?;
        let destination_folder = tempfile::tempdir()?;
        let source = world(&source_folder);
        let destination = world(&destination_folder);
        let body = CowEntity::new(Entity::new(
            source.clone(),
            Vector3::new(1.0, 64.0, 0.0),
            &EntityType::COW,
        ));
        let area = BoundingBox::new(Vector3::new(0.0, 63.0, -1.0), Vector3::new(2.0, 66.0, 1.0));
        source.entities.store(Arc::new(vec![body.clone()]));
        assert_eq!(ids(&source, &area), [body.get_entity().entity_id]);
        source.entities.store(Arc::new(Vec::new()));
        body.get_entity().set_world(destination.clone());
        destination.entities.store(Arc::new(vec![body.clone()]));
        assert_eq!(ids(&destination, &area), [body.get_entity().entity_id]);
        body.get_entity().set_pos(Vector3::new(20.0, 64.0, 0.0));
        assert!(ids(&source, &area).is_empty());
        assert!(ids(&destination, &area).is_empty());
        assert_eq!(
            ids(&destination, &area.expand(20.0, 0.0, 0.0)),
            [body.get_entity().entity_id]
        );
        source.level.shutdown().await;
        destination.level.shutdown().await;
        Ok(())
    }
}

#[cfg(test)]
mod incremental_tests {
    use std::sync::Arc;

    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::{boundingbox::BoundingBox, vector3::Vector3};

    use crate::entity::{
        Entity, EntityBase, ai::test_support::world, interaction::InteractionEntity,
        passive::cow::CowEntity,
    };
    use crate::world::World;

    /// Small deterministic generator, so a failure replays.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.0 >> 33
        }
        fn coord(&mut self) -> f64 {
            (self.next() % 6400) as f64 / 100.0 - 32.0
        }
    }

    fn ids(entities: &[Arc<dyn EntityBase>]) -> Vec<i32> {
        entities.iter().map(|e| e.get_entity().entity_id).collect()
    }

    /// The index's answers against a linear scan of the entity list.
    fn assert_matches_scan(world: &World, rng: &mut Lcg) {
        let list = world.entities.load_full();
        for _ in 0..8 {
            let (x, z) = (rng.coord(), rng.coord());
            let area = BoundingBox::new(
                Vector3::new(x - 6.0, 60.0, z - 6.0),
                Vector3::new(x + 6.0, 70.0, z + 6.0),
            );
            let expected: Vec<i32> = list
                .iter()
                .filter(|e| area.intersects(&e.get_entity().bounding_box.load()))
                .map(|e| e.get_entity().entity_id)
                .collect();
            assert_eq!(ids(&world.get_entities_at_box(&area)), expected);
        }
        let mobs: Vec<i32> = list
            .iter()
            .filter(|e| e.get_mob().is_some())
            .map(|e| e.get_entity().entity_id)
            .collect();
        assert_eq!(ids(&world.entity_index.mobs(&world.entities)), mobs);
        for entity in list.iter() {
            let base = entity.get_entity();
            let by_id = world
                .get_entity_by_id(base.entity_id)
                .map(|e| e.get_entity().entity_id);
            assert_eq!(by_id, Some(base.entity_id));
            let by_uuid = world
                .get_entity_by_uuid(base.entity_uuid)
                .map(|e| e.get_entity().entity_uuid);
            assert_eq!(by_uuid, Some(base.entity_uuid));
        }
    }

    fn spawn(world: &Arc<World>, rng: &mut Lcg) -> Arc<dyn EntityBase> {
        let pos = Vector3::new(rng.coord(), 64.0, rng.coord());
        let entity: Arc<dyn EntityBase> = if rng.next().is_multiple_of(2) {
            CowEntity::new(Entity::new(world.clone(), pos, &EntityType::COW))
        } else {
            InteractionEntity::new(Entity::new(world.clone(), pos, &EntityType::INTERACTION))
        };
        world.spawn_entity(entity.clone());
        entity
    }

    /// Spawns and removals update the index in place: answers stay those of a linear scan,
    /// in list order, and the index is not rebuilt for each change.
    #[tokio::test]
    async fn spawns_and_removals_update_the_index_in_place()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let mut rng = Lcg(42);
        let mut live: Vec<Arc<dyn EntityBase>> = Vec::new();
        for _ in 0..50 {
            live.push(spawn(&world, &mut rng));
        }
        assert_matches_scan(&world, &mut rng);
        let rebuilds = world.entity_index.rebuilds();

        for step in 0..600 {
            if live.is_empty() || !rng.next().is_multiple_of(3) {
                live.push(spawn(&world, &mut rng));
            } else {
                let victim = live.swap_remove((rng.next() as usize) % live.len());
                world.remove_entity(victim.as_ref());
                assert!(
                    world
                        .get_entity_by_id(victim.get_entity().entity_id)
                        .is_none()
                );
            }
            // Entities move between queries, too.
            if let Some(mover) = live.get((rng.next() as usize) % live.len().max(1)) {
                mover
                    .get_entity()
                    .set_pos(Vector3::new(rng.coord(), 64.0, rng.coord()));
            }
            if step % 25 == 0 {
                assert_matches_scan(&world, &mut rng);
            }
        }
        assert_matches_scan(&world, &mut rng);
        assert_eq!(
            world.entity_index.rebuilds(),
            rebuilds,
            "spawning and removing must not rebuild the index"
        );

        // Enough removals compact the keys with one rebuild, and answers stay right.
        for _ in 0..1500 {
            live.push(spawn(&world, &mut rng));
        }
        while live.len() > 100 {
            let victim = live.pop().ok_or("empty")?;
            world.remove_entity(victim.as_ref());
        }
        assert_matches_scan(&world, &mut rng);
        assert!(world.entity_index.rebuilds() > rebuilds);

        // The same entity registered twice: a removal takes both out, like the list's retain.
        let twice = spawn(&world, &mut rng);
        world.spawn_entity_non_save(twice.clone());
        assert_matches_scan(&world, &mut rng);
        world.remove_entity(twice.as_ref());
        assert!(
            world
                .get_entity_by_id(twice.get_entity().entity_id)
                .is_none()
        );
        assert_matches_scan(&world, &mut rng);

        world.level.shutdown().await;
        Ok(())
    }
}
