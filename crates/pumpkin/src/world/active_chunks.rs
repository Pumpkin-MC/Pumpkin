use pumpkin_util::math::vector2::Vector2;
use rustc_hash::{FxHashMap, FxHashSet};
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct ActivePlayerArea {
    pub center: Vector2<i32>,
    pub simulation_distance: i32,
}

#[derive(Default)]
pub(super) struct ActiveChunkTracker {
    pub players: FxHashMap<Uuid, ActivePlayerArea>,
    pub loaded_active_chunks: FxHashSet<Vector2<i32>>,
    watcher_counts: FxHashMap<Vector2<i32>, u32>,
    forced_chunks: FxHashSet<Vector2<i32>>,
}

impl ActiveChunkTracker {
    fn add_chunk(
        &mut self,
        pos: Vector2<i32>,
        active_chunks: &mut FxHashSet<Vector2<i32>>,
        newly_active: &mut Vec<Vector2<i32>>,
    ) {
        let count = self.watcher_counts.entry(pos).or_default();
        *count += 1;
        if *count == 1 {
            active_chunks.insert(pos);
            newly_active.push(pos);
        }
    }

    fn remove_chunk(&mut self, pos: Vector2<i32>, active_chunks: &mut FxHashSet<Vector2<i32>>) {
        let Some(count) = self.watcher_counts.get_mut(&pos) else {
            return;
        };
        *count -= 1;
        if *count == 0 {
            self.watcher_counts.remove(&pos);
            self.loaded_active_chunks.remove(&pos);
            active_chunks.remove(&pos);
        }
    }

    fn add_area(
        &mut self,
        area: ActivePlayerArea,
        active_chunks: &mut FxHashSet<Vector2<i32>>,
        newly_active: &mut Vec<Vector2<i32>>,
    ) {
        for dx in -area.simulation_distance..=area.simulation_distance {
            for dz in -area.simulation_distance..=area.simulation_distance {
                self.add_chunk(area.center.add_raw(dx, dz), active_chunks, newly_active);
            }
        }
    }

    fn remove_area(&mut self, area: ActivePlayerArea, active_chunks: &mut FxHashSet<Vector2<i32>>) {
        for dx in -area.simulation_distance..=area.simulation_distance {
            for dz in -area.simulation_distance..=area.simulation_distance {
                self.remove_chunk(area.center.add_raw(dx, dz), active_chunks);
            }
        }
    }

    pub fn update_player(
        &mut self,
        id: Uuid,
        area: ActivePlayerArea,
        active_chunks: &mut FxHashSet<Vector2<i32>>,
        newly_active: &mut Vec<Vector2<i32>>,
    ) {
        let previous = self.players.insert(id, area);
        match previous {
            None => self.add_area(area, active_chunks, newly_active),
            Some(previous) if previous != area => {
                for dx in -previous.simulation_distance..=previous.simulation_distance {
                    for dz in -previous.simulation_distance..=previous.simulation_distance {
                        let pos = previous.center.add_raw(dx, dz);
                        if (pos.x - area.center.x).abs() > area.simulation_distance
                            || (pos.y - area.center.y).abs() > area.simulation_distance
                        {
                            self.remove_chunk(pos, active_chunks);
                        }
                    }
                }
                for dx in -area.simulation_distance..=area.simulation_distance {
                    for dz in -area.simulation_distance..=area.simulation_distance {
                        let pos = area.center.add_raw(dx, dz);
                        if (pos.x - previous.center.x).abs() > previous.simulation_distance
                            || (pos.y - previous.center.y).abs() > previous.simulation_distance
                        {
                            self.add_chunk(pos, active_chunks, newly_active);
                        }
                    }
                }
            }
            Some(_) => {}
        }
    }

    pub fn remove_player(&mut self, id: Uuid, active_chunks: &mut FxHashSet<Vector2<i32>>) {
        if let Some(area) = self.players.remove(&id) {
            self.remove_area(area, active_chunks);
        }
    }

    pub fn sync_forced_chunks(
        &mut self,
        forced_chunks: &FxHashSet<Vector2<i32>>,
        active_chunks: &mut FxHashSet<Vector2<i32>>,
        newly_active: &mut Vec<Vector2<i32>>,
    ) {
        let removed: Vec<_> = self
            .forced_chunks
            .difference(forced_chunks)
            .copied()
            .collect();
        let added: Vec<_> = forced_chunks
            .difference(&self.forced_chunks)
            .copied()
            .collect();
        for pos in removed {
            self.remove_chunk(pos, active_chunks);
        }
        for pos in added {
            self.add_chunk(pos, active_chunks, newly_active);
        }
        self.forced_chunks.clone_from(forced_chunks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(x: i32, z: i32, simulation_distance: i32) -> ActivePlayerArea {
        ActivePlayerArea {
            center: Vector2::new(x, z),
            simulation_distance,
        }
    }

    #[test]
    fn follows_player_boundary_crossing() {
        let id = Uuid::from_u128(1);
        let mut tracker = ActiveChunkTracker::default();
        let mut active = FxHashSet::default();
        let mut newly_active = Vec::new();

        tracker.update_player(id, area(0, 0, 1), &mut active, &mut newly_active);
        assert_eq!(active.len(), 9);
        newly_active.clear();

        tracker.update_player(id, area(1, 0, 1), &mut active, &mut newly_active);

        assert_eq!(active.len(), 9);
        assert_eq!(newly_active.len(), 3);
        assert!(!active.contains(&Vector2::new(-1, 0)));
        assert!(active.contains(&Vector2::new(2, 0)));
    }

    #[test]
    fn removes_chunks_when_player_leaves() {
        let id = Uuid::from_u128(1);
        let mut tracker = ActiveChunkTracker::default();
        let mut active = FxHashSet::default();
        let mut newly_active = Vec::new();

        tracker.update_player(id, area(0, 0, 1), &mut active, &mut newly_active);
        tracker.remove_player(id, &mut active);

        assert!(active.is_empty());
        assert!(tracker.watcher_counts.is_empty());
    }

    #[test]
    fn keeps_overlaps_active_until_both_players_leave() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let mut tracker = ActiveChunkTracker::default();
        let mut active = FxHashSet::default();
        let mut newly_active = Vec::new();

        tracker.update_player(first, area(0, 0, 1), &mut active, &mut newly_active);
        tracker.update_player(second, area(0, 0, 1), &mut active, &mut newly_active);
        tracker.remove_player(first, &mut active);

        assert_eq!(active.len(), 9);
        assert!(tracker.watcher_counts.values().all(|count| *count == 1));

        tracker.remove_player(second, &mut active);
        assert!(active.is_empty());
    }
}
