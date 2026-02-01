//! Binary heap implementation for A* pathfinding.
//!
//! This module provides a min-heap optimized for pathfinding operations,
//! supporting efficient insertion, removal, and cost updates.

use super::node::Node;

/// A binary min-heap for pathfinding nodes.
///
/// This heap maintains nodes ordered by their f-score (total estimated cost),
/// with lower f-scores having higher priority. Nodes track their position
/// in the heap for efficient updates.
pub struct BinaryHeap {
    /// The heap storage
    heap: Vec<Node>,
    /// Current number of elements
    size: usize,
}

impl BinaryHeap {
    /// Creates a new empty binary heap with default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(128)
    }

    /// Creates a new empty binary heap with specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            heap: Vec::with_capacity(capacity),
            size: 0,
        }
    }

    /// Inserts a node into the heap.
    ///
    /// The node's `heap_idx` will be updated to reflect its position.
    ///
    /// # Panics
    /// Panics if the node is already in a heap (heap_idx >= 0).
    pub fn insert(&mut self, mut node: Node) -> usize {
        assert!(node.heap_idx < 0, "Node is already in a heap");

        // Grow if necessary
        if self.size >= self.heap.len() {
            self.heap
                .resize(self.heap.len().max(1) * 2, Node::new(0, 0, 0));
        }

        let idx = self.size;
        node.heap_idx = idx as i32;
        self.heap[idx] = node;
        self.size += 1;
        self.up_heap(idx);
        idx
    }

    /// Clears the heap, removing all nodes.
    pub fn clear(&mut self) {
        self.size = 0;
    }

    /// Returns a reference to the node with the lowest f-score.
    ///
    /// Returns `None` if the heap is empty.
    #[must_use]
    pub fn peek(&self) -> Option<&Node> {
        if self.size > 0 {
            Some(&self.heap[0])
        } else {
            None
        }
    }

    /// Removes and returns the node with the lowest f-score.
    ///
    /// Returns `None` if the heap is empty.
    pub fn pop(&mut self) -> Option<Node> {
        if self.size == 0 {
            return None;
        }

        let mut result = self.heap[0].clone();
        result.heap_idx = -1;

        self.size -= 1;
        if self.size > 0 {
            self.heap[0] = self.heap[self.size].clone();
            self.heap[0].heap_idx = 0;
            self.down_heap(0);
        }

        Some(result)
    }

    /// Removes a specific node from the heap.
    ///
    /// The node's `heap_idx` will be set to -1.
    pub fn remove(&mut self, idx: usize) {
        if idx >= self.size {
            return;
        }

        let old_f = self.heap[idx].f;
        self.heap[idx].heap_idx = -1;

        self.size -= 1;
        if idx < self.size {
            self.heap[idx] = self.heap[self.size].clone();
            self.heap[idx].heap_idx = idx as i32;

            if self.heap[idx].f < old_f {
                self.up_heap(idx);
            } else {
                self.down_heap(idx);
            }
        }
    }

    /// Updates the cost of a node and rebalances the heap.
    ///
    /// Call this after changing the f-score of a node in the heap.
    pub fn change_cost(&mut self, idx: usize, new_f: f32) {
        if idx >= self.size {
            return;
        }

        let old_f = self.heap[idx].f;
        self.heap[idx].f = new_f;

        if new_f < old_f {
            self.up_heap(idx);
        } else {
            self.down_heap(idx);
        }
    }

    /// Returns the current number of nodes in the heap.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.size
    }

    /// Returns whether the heap is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Returns a reference to a node by index.
    #[must_use]
    pub fn get(&self, idx: usize) -> Option<&Node> {
        if idx < self.size {
            Some(&self.heap[idx])
        } else {
            None
        }
    }

    /// Returns a mutable reference to a node by index.
    #[must_use]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Node> {
        if idx < self.size {
            Some(&mut self.heap[idx])
        } else {
            None
        }
    }

    /// Returns all nodes currently in the heap.
    #[must_use]
    pub fn get_heap(&self) -> Vec<Node> {
        self.heap[..self.size].to_vec()
    }

    /// Moves a node up the heap until the heap property is restored.
    fn up_heap(&mut self, mut idx: usize) {
        let node = self.heap[idx].clone();
        let f = node.f;

        while idx > 0 {
            let parent_idx = (idx - 1) >> 1;
            let parent = &self.heap[parent_idx];

            if f >= parent.f {
                break;
            }

            self.heap[idx] = parent.clone();
            self.heap[idx].heap_idx = idx as i32;
            idx = parent_idx;
        }

        self.heap[idx] = node;
        self.heap[idx].heap_idx = idx as i32;
    }

    /// Moves a node down the heap until the heap property is restored.
    fn down_heap(&mut self, mut idx: usize) {
        let node = self.heap[idx].clone();
        let f = node.f;

        loop {
            let left_idx = 1 + (idx << 1);
            let right_idx = left_idx + 1;

            if left_idx >= self.size {
                break;
            }

            let left_f = self.heap[left_idx].f;
            let (min_child_idx, min_child_f) = if right_idx >= self.size {
                (left_idx, left_f)
            } else {
                let right_f = self.heap[right_idx].f;
                if left_f < right_f {
                    (left_idx, left_f)
                } else {
                    (right_idx, right_f)
                }
            };

            if min_child_f >= f {
                break;
            }

            self.heap[idx] = self.heap[min_child_idx].clone();
            self.heap[idx].heap_idx = idx as i32;
            idx = min_child_idx;
        }

        self.heap[idx] = node;
        self.heap[idx].heap_idx = idx as i32;
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

    #[test]
    fn test_insert_and_pop() {
        let mut heap = BinaryHeap::new();

        let mut n1 = Node::new(0, 0, 0);
        n1.f = 5.0;
        let mut n2 = Node::new(1, 0, 0);
        n2.f = 3.0;
        let mut n3 = Node::new(2, 0, 0);
        n3.f = 7.0;

        heap.insert(n1);
        heap.insert(n2);
        heap.insert(n3);

        assert_eq!(heap.len(), 3);

        let popped = heap.pop().unwrap();
        assert_eq!(popped.f, 3.0);
        assert_eq!(popped.x, 1);

        let popped = heap.pop().unwrap();
        assert_eq!(popped.f, 5.0);

        let popped = heap.pop().unwrap();
        assert_eq!(popped.f, 7.0);

        assert!(heap.is_empty());
    }

    #[test]
    fn test_change_cost() {
        let mut heap = BinaryHeap::new();

        let mut n1 = Node::new(0, 0, 0);
        n1.f = 5.0;
        let mut n2 = Node::new(1, 0, 0);
        n2.f = 3.0;

        heap.insert(n1);
        let idx = heap.insert(n2);

        // Change n2's cost to be higher than n1
        heap.change_cost(idx, 10.0);

        let popped = heap.pop().unwrap();
        assert_eq!(popped.x, 0); // n1 should come first now
    }
}
