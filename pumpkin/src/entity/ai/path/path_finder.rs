//! A* pathfinding algorithm implementation.
//!
//! This module provides the core pathfinding algorithm using A* search,
//! capable of finding paths between positions in a Minecraft world.

use std::collections::HashSet;

use pumpkin_util::math::position::BlockPos;

use super::binary_heap::BinaryHeap;
use super::node::Node;
use super::node_evaluator::NodeEvaluator;
use super::path::Path;
use super::target::Target;

/// Fudging factor for heuristic calculation.
///
/// This value slightly inflates the heuristic to speed up pathfinding
/// at the cost of potentially finding slightly suboptimal paths.
const FUDGING: f32 = 1.5;

/// Maximum number of neighbors per node.
const MAX_NEIGHBORS: usize = 32;

/// A* pathfinder for entity navigation.
///
/// Uses the A* algorithm to find paths from a starting position to
/// one or more target positions, considering terrain costs and obstacles.
pub struct PathFinder {
    /// Maximum number of nodes to visit before giving up
    max_visited_nodes: usize,
    /// The open set (priority queue of nodes to explore)
    open_set: BinaryHeap,
    /// All nodes created during pathfinding (for reconstruction)
    all_nodes: Vec<Node>,
}

impl PathFinder {
    /// Creates a new pathfinder with the specified maximum visited nodes.
    #[must_use]
    pub fn new(max_visited_nodes: usize) -> Self {
        Self {
            max_visited_nodes,
            open_set: BinaryHeap::new(),
            all_nodes: Vec::new(),
        }
    }

    /// Sets the maximum number of nodes to visit.
    pub fn set_max_visited_nodes(&mut self, max_visited_nodes: usize) {
        self.max_visited_nodes = max_visited_nodes;
    }

    /// Finds a path from the mob's current position to one of the target positions.
    ///
    /// # Arguments
    /// * `evaluator` - The node evaluator to use for pathfinding
    /// * `context` - The pathfinding context
    /// * `targets` - Set of target block positions
    /// * `max_distance` - Maximum pathfinding distance from start
    /// * `reach_range` - Distance at which a target is considered reached
    /// * `max_visited_multiplier` - Multiplier for max visited nodes
    ///
    /// # Returns
    /// An optional `Path` if one was found, or `None` if no path exists.
    pub async fn find_path<E: NodeEvaluator + ?Sized>(
        &mut self,
        evaluator: &mut E,
        context: &mut super::node_evaluator::PathfindingContext,
        targets: &HashSet<BlockPos>,
        max_distance: f32,
        reach_range: i32,
        max_visited_multiplier: f32,
    ) -> Option<Path> {
        // Clear state
        self.open_set.clear();
        self.all_nodes.clear();

        // Get start node
        let start_node = evaluator.get_start()?.clone();

        // Create targets
        let mut target_set: Vec<Target> = targets
            .iter()
            .map(|pos| evaluator.get_target(context, pos.0.x, pos.0.y, pos.0.z))
            .collect();

        if target_set.is_empty() {
            return None;
        }

        // Run A* algorithm
        self.find_path_internal(
            evaluator,
            context,
            start_node,
            &mut target_set,
            max_distance,
            reach_range,
            max_visited_multiplier,
        ).await
    }

    /// Internal A* pathfinding implementation.
    async fn find_path_internal<E: NodeEvaluator + ?Sized>(
        &mut self,
        evaluator: &mut E,
        context: &mut super::node_evaluator::PathfindingContext,
        mut start_node: Node,
        targets: &mut Vec<Target>,
        max_distance: f32,
        reach_range: i32,
        max_visited_multiplier: f32,
    ) -> Option<Path> {
        // Initialize start node
        start_node.g = 0.0;
        start_node.h = self.get_best_h(&start_node, targets);
        start_node.f = start_node.h;

        // Add start node to all_nodes and open set
        self.all_nodes.push(start_node.clone());
        let start_idx = self.all_nodes.len() - 1;
        self.open_set.insert(self.all_nodes[start_idx].clone());

        let mut reached_targets: HashSet<usize> = HashSet::new();
        let mut visited = 0;
        let max_visited = (self.max_visited_nodes as f32 * max_visited_multiplier) as usize;

        while !self.open_set.is_empty() {
            // Check if we've exceeded our visit budget
            visited += 1;
            if visited >= max_visited {
                break;
            }

            // Get the node with lowest f-score
            let current = self.open_set.pop()?;

            // Find index of current node in all_nodes
            let current_idx = self.find_node_index(&current)?;

            // Mark as closed
            self.all_nodes[current_idx].closed = true;

            // Check if we've reached any targets
            for (target_idx, target) in targets.iter_mut().enumerate() {
                let dist = current.distance_manhattan(&target.node);
                if dist <= reach_range as f32 {
                    target.set_reached();
                    reached_targets.insert(target_idx);
                }
            }

            // If we've reached at least one target, we're done
            if !reached_targets.is_empty() {
                break;
            }

            // Check if we're too far from start
            let dist_from_start = current.distance_to(&self.all_nodes[0]);
            if dist_from_start >= max_distance {
                continue;
            }

            // Get and process neighbors
            let neighbors = evaluator.get_neighbors(context, &current);

            for neighbor in neighbors.into_iter().take(MAX_NEIGHBORS) {
                // Calculate new costs
                let step_cost = self.distance(&current, &neighbor);
                let walked = current.walked_distance + step_cost;
                let new_g = current.g + step_cost + neighbor.cost_malus;

                // Check if this is a better path
                let existing_idx = self.find_or_add_node(neighbor.clone());

                let in_open = self.all_nodes[existing_idx].in_open_set();
                let old_g = self.all_nodes[existing_idx].g;

                if walked < max_distance && (!in_open || new_g < old_g) {
                    // Calculate new h before borrowing mutably
                    let new_h = self.get_best_h(&self.all_nodes[existing_idx], targets) * FUDGING;

                    // Update node
                    let existing = &mut self.all_nodes[existing_idx];
                    existing.came_from = Some(current_idx);
                    existing.g = new_g;
                    existing.h = new_h;
                    existing.walked_distance = walked;

                    if existing.in_open_set() {
                        // Update in heap
                        let heap_idx = existing.heap_idx as usize;
                        let new_f = existing.g + existing.h;
                        self.open_set.change_cost(heap_idx, new_f);
                    } else {
                        // Add to heap
                        existing.f = existing.g + existing.h;
                        let node_clone = existing.clone();
                        self.open_set.insert(node_clone);
                    }
                }
            }
        }

        // Find the best path
        if !reached_targets.is_empty() {
            // Pick the reached target with the shortest path
            let best_target_idx = reached_targets
                .iter()
                .min_by(|&&a, &&b| {
                    let path_a = targets[a].best_node_index.map(|i| self.all_nodes[i].g);
                    let path_b = targets[b].best_node_index.map(|i| self.all_nodes[i].g);
                    path_a.partial_cmp(&path_b).unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied()?;

            let target = &targets[best_target_idx];
            if let Some(best_idx) = target.best_node_index {
                return Some(self.reconstruct_path(&self.all_nodes[best_idx], target.as_block_pos(), true));
            }
        }

        // No target reached - return partial path to closest point
        let best_target = targets
            .iter()
            .filter(|t| t.best_node_index.is_some())
            .min_by(|a, b| {
                a.best_heuristic.partial_cmp(&b.best_heuristic).unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(target) = best_target {
            if let Some(best_idx) = target.best_node_index {
                return Some(self.reconstruct_path(&self.all_nodes[best_idx], target.as_block_pos(), false));
            }
        }

        None
    }

    /// Calculates the distance between two nodes.
    fn distance(&self, from: &Node, to: &Node) -> f32 {
        from.distance_to(to)
    }

    /// Gets the best heuristic (minimum distance to any target).
    fn get_best_h(&self, node: &Node, targets: &mut [Target]) -> f32 {
        let mut best = f32::MAX;
        let node_idx = self.all_nodes.len().saturating_sub(1);

        for target in targets.iter_mut() {
            let dist = node.distance_to(&target.node);
            target.update_best(dist, node_idx);
            best = best.min(dist);
        }

        best
    }

    /// Finds the index of a node in all_nodes.
    fn find_node_index(&self, node: &Node) -> Option<usize> {
        self.all_nodes
            .iter()
            .position(|n| n.x == node.x && n.y == node.y && n.z == node.z)
    }

    /// Finds an existing node or adds a new one, returning the index.
    fn find_or_add_node(&mut self, node: Node) -> usize {
        if let Some(idx) = self.find_node_index(&node) {
            idx
        } else {
            self.all_nodes.push(node);
            self.all_nodes.len() - 1
        }
    }

    /// Reconstructs the path from the end node back to start.
    fn reconstruct_path(&self, end: &Node, target: BlockPos, reached: bool) -> Path {
        let mut nodes = Vec::new();
        let mut current = end.clone();
        nodes.push(current.clone());

        while let Some(parent_idx) = current.came_from {
            current = self.all_nodes[parent_idx].clone();
            nodes.push(current.clone());
        }

        nodes.reverse();
        Path::new(nodes, target, reached)
    }
}

impl Default for PathFinder {
    fn default() -> Self {
        Self::new(256)
    }
}
