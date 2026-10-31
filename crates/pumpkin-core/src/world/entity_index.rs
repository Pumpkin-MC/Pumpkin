use std::sync::{Arc, RwLock, Weak};

use pumpkin_util::math::boundingbox::BoundingBox;
use rustc_hash::FxHashMap;
use uuid::Uuid;

use crate::entity::EntityBase;
use crossbeam::atomic::AtomicCell;

type Registry = Vec<Arc<dyn EntityBase>>;
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

#[derive(Default)]
struct Index {
    registry: Weak<Registry>,
    slots: FxHashMap<i32, usize>,
    uuids: FxHashMap<Uuid, usize>,
    ranges: Vec<Option<CellRange>>,
    cells: FxHashMap<Cell, Vec<usize>>,
    oversized: Vec<usize>,
}

impl Index {
    fn matches(&self, registry: &Arc<Registry>) -> bool {
        self.registry.as_ptr() == Arc::as_ptr(registry)
    }

    fn rebuild(&mut self, registry: &Arc<Registry>) {
        self.slots.clear();
        self.uuids.clear();
        self.ranges.clear();
        self.cells.clear();
        self.oversized.clear();
        for (slot, entity) in registry.iter().enumerate() {
            let entity = entity.get_entity();
            self.slots.insert(entity.entity_id, slot);
            // The first in list order wins, like a linear search.
            self.uuids.entry(entity.entity_uuid).or_insert(slot);
            let range = Self::entity_range(&entity.bounding_box.load());
            self.ranges.push(range);
            self.insert(slot, range);
        }
        self.registry = Arc::downgrade(registry);
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

    fn update(&mut self, entity_id: i32, area: &BoundingBox) {
        let Some(&slot) = self.slots.get(&entity_id) else {
            return;
        };
        let next = Self::entity_range(area);
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

    fn candidates(&self, area: &BoundingBox, len: usize) -> Vec<usize> {
        let Some(range) = CellRange::of(area) else {
            return (0..len).collect();
        };
        let mut slots = self.oversized.clone();
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

/// Live broad phase for the world's non-player entity queries.
#[derive(Default)]
pub struct EntityIndex {
    inner: RwLock<Index>,
}

impl EntityIndex {
    fn with_index<T>(&self, registry: &Arc<Registry>, query: impl FnOnce(&Index) -> T) -> T {
        let index = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if index.matches(registry) {
            return query(&index);
        }
        drop(index);
        let mut index = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !index.matches(registry) {
            index.rebuild(registry);
        }
        query(&index)
    }

    pub fn query(
        &self,
        registry: Arc<Registry>,
        area: BoundingBox,
    ) -> impl Iterator<Item = Arc<dyn EntityBase>> + use<> {
        let slots = self.with_index(&registry, |index| index.candidates(&area, registry.len()));
        slots.into_iter().filter_map(move |slot| {
            let entity = &registry[slot];
            area.intersects(&entity.get_entity().bounding_box.load())
                .then(|| entity.clone())
        })
    }

    /// The first entity with `uuid` in registry order, without a scan.
    pub fn by_uuid(&self, registry: &Arc<Registry>, uuid: Uuid) -> Option<Arc<dyn EntityBase>> {
        let slot = self.with_index(registry, |index| index.uuids.get(&uuid).copied())?;
        Some(registry[slot].clone())
    }

    pub fn set_bounds(&self, entity_id: i32, bounds: &AtomicCell<BoundingBox>, area: BoundingBox) {
        let index = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let next = Index::entity_range(&area);
        if index
            .slots
            .get(&entity_id)
            .is_none_or(|&slot| index.ranges[slot] == next)
        {
            bounds.store(area);
            return;
        }
        drop(index);
        let mut index = self
            .inner
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
