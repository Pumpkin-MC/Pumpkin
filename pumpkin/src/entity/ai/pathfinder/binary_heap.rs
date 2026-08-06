use rustc_hash::{FxBuildHasher, FxHashMap};

use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::pathfinder::node::{Coordinate, Node};

// Binary heap implementation that uses the node's f score as the node's value
// The node's position in the heap is stored in `node.heap_idx`, this is just copying vanilla
// behavior, I'm not sure it's necessary. Infact, it's always going to be 0 when popping so it's
// only use is when peeking into the heap. Possibly could be removed?

#[derive(Debug, Clone)]
pub struct BinaryHeap {
    heap: Vec<Option<Node>>,
    position_map: FxHashMap<Vector3<i32>, usize>,
    size: usize,
}

impl BinaryHeap {
    #[must_use]
    pub fn new() -> Self {
        let mut heap = Vec::with_capacity(1024);
        heap.push(None);

        Self {
            heap,
            position_map: FxHashMap::default(),
            size: 0,
        }
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let mut heap = Vec::with_capacity(capacity + 1);
        heap.push(None);

        Self {
            heap,
            position_map: FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher),
            size: 0,
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.size
    }

    pub fn clear(&mut self) {
        self.heap.clear();
        self.heap.push(None);
        self.position_map.clear();
        self.size = 0;
    }

    pub fn insert(&mut self, mut node: Node) {
        self.size += 1;

        if self.heap.len() <= self.size {
            self.heap.resize(self.size * 2, None);
        }

        node.heap_idx = self.size as i32;
        self.position_map.insert(node.as_vector3(), self.size);

        self.heap[self.size] = Some(node);
        self.bubble_up(self.size);
    }

    pub fn pop(&mut self) -> Option<Node> {
        if self.is_empty() {
            return None;
        }

        let min_node = self.heap[1].take()?;
        self.position_map.remove(&min_node.as_vector3());

        if self.size == 1 {
            self.size = 0;
            return Some(min_node);
        }

        if let Some(mut last_node) = self.heap[self.size].take() {
            last_node.heap_idx = 1;
            self.position_map.insert(last_node.as_vector3(), 1);
            self.heap[1] = Some(last_node);
        }

        self.size -= 1;
        self.bubble_down(1);

        Some(min_node)
    }

    #[must_use]
    pub fn peek(&self) -> Option<&Node> {
        if self.is_empty() {
            None
        } else {
            self.heap[1].as_ref()
        }
    }

    pub fn change_cost(&mut self, coords: &dyn Coordinate, new_f_score: f32) {
        if let Some(&index) = self.position_map.get(&coords.as_vector3())
            && let Some(ref mut node) = self.heap[index]
        {
            let old_f = node.f;
            node.f = new_f_score;

            if new_f_score < old_f {
                self.bubble_up(index);
            } else if new_f_score > old_f {
                self.bubble_down(index);
            }
        }
    }

    pub fn contains(&self, coords: &dyn Coordinate) -> bool {
        self.position_map.contains_key(&coords.as_vector3())
    }

    /// Get a reference to the node at the given coordinates, if it exists in the heap.
    pub fn get_node(&self, coords: &dyn Coordinate) -> Option<&Node> {
        self.position_map
            .get(&coords.as_vector3())
            .and_then(|&index| self.heap[index].as_ref())
    }

    /// Updates an existing node's fields and reorders the heap.
    /// This is used when we find a better path to an already-open node.
    pub fn update_node(&mut self, coords: &dyn Coordinate, updated: Node) {
        if let Some(&index) = self.position_map.get(&coords.as_vector3())
            && let Some(ref mut node) = self.heap[index]
        {
            let old_f = node.f;
            let heap_idx = node.heap_idx;
            *node = updated;
            node.heap_idx = heap_idx;

            if node.f < old_f {
                self.bubble_up(index);
            } else if node.f > old_f {
                self.bubble_down(index);
            }
        }
    }

    /// Drain all nodes from the heap, returning them as a Vec.
    pub fn drain(&mut self) -> Vec<Node> {
        let nodes: Vec<Node> = self.heap[1..=self.size]
            .iter()
            .filter_map(|node_opt| *node_opt)
            .collect();
        self.clear();
        nodes
    }

    #[must_use]
    pub fn get_heap(&self) -> Vec<Node> {
        self.heap[1..=self.size]
            .iter()
            .filter_map(|node_opt| *node_opt)
            .collect()
    }

    fn bubble_up(&mut self, mut index: usize) {
        while index > 1 {
            let parent_index = index / 2;

            let should_swap = {
                if let (Some(node), Some(parent)) = (&self.heap[index], &self.heap[parent_index]) {
                    node.f < parent.f
                } else {
                    false
                }
            };

            if !should_swap {
                break;
            }

            self.swap_nodes(index, parent_index);
            index = parent_index;
        }
    }

    fn bubble_down(&mut self, mut index: usize) {
        loop {
            let left_child = index * 2;
            let right_child = index * 2 + 1;
            let mut smallest = index;

            if left_child <= self.size
                && let (Some(node), Some(left)) = (&self.heap[smallest], &self.heap[left_child])
                && left.f < node.f
            {
                smallest = left_child;
            }

            if right_child <= self.size
                && let (Some(node), Some(right)) = (&self.heap[smallest], &self.heap[right_child])
                && right.f < node.f
            {
                smallest = right_child;
            }

            if smallest == index {
                break;
            }

            self.swap_nodes(index, smallest);
            index = smallest;
        }
    }

    fn swap_nodes(&mut self, i: usize, j: usize) {
        if let Some(ref mut node_i) = self.heap[i] {
            node_i.heap_idx = j as i32;
            self.position_map.insert(node_i.as_vector3(), j);
        }
        if let Some(ref mut node_j) = self.heap[j] {
            node_j.heap_idx = i as i32;
            self.position_map.insert(node_j.as_vector3(), i);
        }

        self.heap.swap(i, j);
    }
}

impl Default for BinaryHeap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::math::position::BlockPos;

    fn node_at(x: i32, y: i32, z: i32, f: f32) -> Node {
        let mut node = Node::new(BlockPos::new(x, y, z));
        node.f = f;
        node
    }

    /// Popping repeatedly must yield nodes in ascending `f` order (min-heap).
    #[test]
    fn pop_returns_ascending_f_order() {
        let mut heap = BinaryHeap::new();
        let fs = [5.0, 1.0, 4.0, 2.0, 8.0, 3.0, 7.0, 6.0, 0.0, 9.0];
        for (i, &f) in fs.iter().enumerate() {
            heap.insert(node_at(i as i32, 0, 0, f));
        }
        assert_eq!(heap.len(), fs.len());

        let mut popped = Vec::new();
        while let Some(node) = heap.pop() {
            popped.push(node.f);
        }
        let mut sorted = fs.to_vec();
        sorted.sort_by(f32::total_cmp);
        assert_eq!(popped, sorted);
        assert!(heap.is_empty());
    }

    #[test]
    fn peek_returns_minimum_without_removing() {
        let mut heap = BinaryHeap::new();
        heap.insert(node_at(0, 0, 0, 3.0));
        heap.insert(node_at(1, 0, 0, 1.0));
        heap.insert(node_at(2, 0, 0, 2.0));

        assert_eq!(heap.peek().unwrap().f, 1.0);
        assert_eq!(heap.len(), 3);
        assert_eq!(heap.pop().unwrap().f, 1.0);
    }

    #[test]
    fn contains_and_get_node_track_membership() {
        let mut heap = BinaryHeap::new();
        heap.insert(node_at(4, 5, 6, 2.0));

        let pos = BlockPos::new(4, 5, 6);
        assert!(heap.contains(&pos));
        assert_eq!(heap.get_node(&pos).unwrap().f, 2.0);
        assert!(!heap.contains(&BlockPos::new(0, 0, 0)));

        heap.pop();
        assert!(!heap.contains(&pos));
    }

    /// Lowering a node's cost must bubble it to the front (decrease-key).
    #[test]
    fn change_cost_decrease_promotes_node() {
        let mut heap = BinaryHeap::new();
        heap.insert(node_at(0, 0, 0, 1.0));
        heap.insert(node_at(1, 0, 0, 2.0));
        let target = BlockPos::new(2, 0, 0);
        heap.insert(node_at(2, 0, 0, 9.0));

        heap.change_cost(&target, 0.5);

        assert_eq!(heap.pop().unwrap().pos, target);
    }

    /// Raising the current minimum's cost must sink it (increase-key).
    #[test]
    fn change_cost_increase_demotes_node() {
        let mut heap = BinaryHeap::new();
        let low = BlockPos::new(0, 0, 0);
        heap.insert(node_at(0, 0, 0, 1.0));
        heap.insert(node_at(1, 0, 0, 2.0));
        heap.insert(node_at(2, 0, 0, 3.0));

        heap.change_cost(&low, 8.0);

        // 2.0 is now the smallest; the demoted node must come out last.
        assert_eq!(heap.pop().unwrap().f, 2.0);
        assert_eq!(heap.pop().unwrap().f, 3.0);
        assert_eq!(heap.pop().unwrap().pos, low);
    }

    #[test]
    fn change_cost_on_absent_coord_is_noop() {
        let mut heap = BinaryHeap::new();
        heap.insert(node_at(0, 0, 0, 1.0));
        heap.change_cost(&BlockPos::new(9, 9, 9), 0.0);
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.pop().unwrap().f, 1.0);
    }

    #[test]
    fn update_node_replaces_fields_and_reorders() {
        let mut heap = BinaryHeap::new();
        heap.insert(node_at(0, 0, 0, 1.0));
        let coord = BlockPos::new(1, 0, 0);
        heap.insert(node_at(1, 0, 0, 5.0));

        let mut better = node_at(1, 0, 0, 0.1);
        better.g = 42.0;
        heap.update_node(&coord, better);

        let top = heap.pop().unwrap();
        assert_eq!(top.pos, coord);
        assert_eq!(top.g, 42.0);
    }

    #[test]
    fn clear_and_drain_empty_the_heap() {
        let mut heap = BinaryHeap::new();
        for i in 0..5 {
            heap.insert(node_at(i, 0, 0, i as f32));
        }
        let drained = heap.drain();
        assert_eq!(drained.len(), 5);
        assert!(heap.is_empty());
        assert!(!heap.contains(&BlockPos::new(0, 0, 0)));

        heap.insert(node_at(0, 0, 0, 1.0));
        heap.clear();
        assert!(heap.is_empty());
        assert!(heap.pop().is_none());
    }

    /// After an interleaved workload, every heap slot must still be indexed by
    /// `position_map`, and lookups must return the node actually stored there.
    #[test]
    fn position_map_stays_consistent_with_heap() {
        let mut heap = BinaryHeap::new();
        for i in 0..40 {
            heap.insert(node_at(i, 0, 0, ((i * 7) % 13) as f32));
        }
        for _ in 0..10 {
            heap.pop();
        }
        heap.change_cost(&BlockPos::new(35, 0, 0), 0.0);

        for node in heap.get_heap() {
            let stored = heap
                .get_node(&node.pos)
                .expect("every live node must be indexed by position_map");
            assert_eq!(stored.pos, node.pos);
            assert_eq!(stored.f, node.f);
        }
    }

    /// Larger deterministic workload: the heap must fully sort by `f` on drain.
    #[test]
    fn stress_insert_pop_is_fully_sorted() {
        let mut heap = BinaryHeap::new();
        let mut expected = Vec::new();
        // Deterministic spread of costs across distinct positions.
        for i in 0..200 {
            let f = ((i * 31 + 7) % 101) as f32;
            heap.insert(node_at(i, i % 5, i % 3, f));
            expected.push(f);
        }
        expected.sort_by(f32::total_cmp);

        let mut popped = Vec::new();
        while let Some(node) = heap.pop() {
            popped.push(node.f);
        }
        assert_eq!(popped, expected);
    }
}
