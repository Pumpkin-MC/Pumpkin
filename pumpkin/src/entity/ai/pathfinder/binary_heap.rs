use std::collections::HashMap;

use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::pathfinder::node::{Coordinate, Node};

// Binary heap implementation that uses the node's f score as the node's value
// The node's position in the heap is stored in `node.heap_idx`, this is just copying vanilla
// behavior, I'm not sure it's necessary. Infact, it's always going to be 0 when popping so it's
// only use is when peeking into the heap. Possibly could be removed?

#[derive(Debug, Clone)]
pub struct BinaryHeap {
    heap: Vec<Option<Box<Node>>>,
    position_map: HashMap<Vector3<i32>, usize>,
    size: usize,
}

impl BinaryHeap {
    #[must_use]
    pub fn new() -> Self {
        let mut heap = Vec::with_capacity(1024);
        heap.push(None);

        Self {
            heap,
            position_map: HashMap::new(),
            size: 0,
        }
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let mut heap = Vec::with_capacity(capacity + 1);
        heap.push(None);

        Self {
            heap,
            position_map: HashMap::with_capacity(capacity),
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

        self.heap[self.size] = Some(Box::new(node));
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
            return Some(*min_node);
        }

        if let Some(mut last_node) = self.heap[self.size].take() {
            last_node.heap_idx = 1;
            self.position_map.insert(last_node.as_vector3(), 1);
            self.heap[1] = Some(last_node);
        }

        self.size -= 1;
        self.bubble_down(1);

        Some(*min_node)
    }

    #[must_use]
    pub fn peek(&self) -> Option<&Node> {
        if self.is_empty() {
            None
        } else {
            self.heap[1].as_deref()
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

    #[must_use]
    pub fn get_heap(&self) -> Vec<Node> {
        self.heap[1..=self.size]
            .iter()
            .filter_map(|node_opt| node_opt.as_ref().map(|n| n.as_ref().clone()))
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
