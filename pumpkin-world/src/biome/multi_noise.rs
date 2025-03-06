use std::{cmp::Ordering, collections::HashMap};

use pumpkin_data::chunk::Biome;
use serde::{Deserialize, Deserializer, Serialize};

use crate::dimension::Dimension;
pub fn to_long(float: f32) -> i64 {
    (float * 1000.0) as i64
}
#[derive(Clone, Serialize, Deserialize)]
pub struct NoiseValuePoint {
    pub temperature: i64,
    pub humidity: i64,
    pub continentalness: i64,
    pub erosion: i64,
    pub depth: i64,
    pub weirdness: i64,
}

#[derive(Clone, Deserialize)]
pub struct NoiseHypercube {
    pub temperature: ParameterRange,
    pub erosion: ParameterRange,
    pub depth: ParameterRange,
    pub continentalness: ParameterRange,
    pub weirdness: ParameterRange,
    pub humidity: ParameterRange,
    pub offset: i64,
}

impl NoiseHypercube {
    pub fn to_parameters(&self) -> [ParameterRange; 7] {
        [
            self.temperature,
            self.humidity,
            self.continentalness,
            self.erosion,
            self.depth,
            self.weirdness,
            ParameterRange {
                min: self.offset,
                max: self.offset,
            },
        ]
    }
}

#[derive(Clone, Copy)]
pub struct ParameterRange {
    pub min: i64,
    pub max: i64,
}

impl<'de> Deserialize<'de> for ParameterRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let arr: [f32; 2] = Deserialize::deserialize(deserializer)?;
        Ok(ParameterRange {
            min: to_long(arr[0]),
            max: to_long(arr[1]),
        })
    }
}

impl ParameterRange {
    fn get_distance(&self, noise: i64) -> i64 {
        let l = noise - self.max;
        let m = self.min - noise;
        if l > 0 { l } else { m.max(0) }
    }

    pub fn combine(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct BiomeEntries {
    pub nodes: HashMap<Dimension, HashMap<Biome, NoiseHypercube>>,
}

#[derive(Clone)]
/// T = Biome
pub struct SearchTree<T: Clone> {
    root: TreeNode<T>,
}

impl<T: Clone> SearchTree<T> {
    pub fn create(entries: Vec<(T, NoiseHypercube)>) -> Option<Self> {
        if entries.is_empty() {
            return None;
        }

        let leaves: Vec<TreeNode<T>> = entries
            .into_iter()
            .map(|(value, hypercube)| TreeNode::new_leaf(value, hypercube.to_parameters()))
            .collect();

        Some(SearchTree {
            root: create_node(leaves),
        })
    }

    pub fn get(
        &self,
        point: &NoiseValuePoint,
        last_result_node: &mut Option<TreeLeafNode<T>>,
    ) -> Option<T> {
        let point = &[
            point.temperature,
            point.humidity,
            point.continentalness,
            point.erosion,
            point.depth,
            point.weirdness,
            0,
        ];
        let result_node = self.root.get_node(point, last_result_node);
        let result = result_node.clone().map(|node| node.value);
        *last_result_node = result_node;
        result
    }
}

fn create_node<T: Clone>(sub_tree: Vec<TreeNode<T>>) -> TreeNode<T> {
    if sub_tree.is_empty() {
        panic!("Need at least one child to build a node");
    } else if sub_tree.len() == 1 {
        sub_tree.into_iter().next().unwrap()
    } else if sub_tree.len() <= 6 {
        let mut sorted_sub_tree = sub_tree;
        sorted_sub_tree.sort_by_key(|a| calculate_midpoint_sum(a));
        let bounds = calculate_bounds(&sorted_sub_tree);
        TreeNode::Branch {
            children: sorted_sub_tree,
            bounds,
        }
    } else {
        let best_split = (0..7)
            .map(|param_idx| {
                let mut sorted_sub_tree = sub_tree.clone();
                sort_tree(&mut sorted_sub_tree, param_idx, false);
                let batched_tree = get_batched_tree(sorted_sub_tree);

                let range_sum: i64 = batched_tree
                    .iter()
                    .map(|node| calculate_bounds_sum(node.bounds()))
                    .sum();

                (param_idx, batched_tree, range_sum)
            })
            .min_by_key(|(_, _, range_sum)| *range_sum)
            .unwrap();

        let (best_param, mut best_batched, _) = best_split;
        sort_tree(&mut best_batched, best_param, true);
        let children: Vec<TreeNode<T>> = best_batched
            .into_iter()
            .map(|batch| create_node(batch.children().to_vec()))
            .collect();
        let bounds = calculate_bounds(&children);
        TreeNode::Branch { children, bounds }
    }
}

fn sort_tree<T: Clone>(sub_tree: &mut [TreeNode<T>], parameter_offset: usize, abs: bool) {
    sub_tree.sort_by(|a, b| {
        for i in 0..7 {
            // Calculate the parameter index in cyclic order
            let current_param = (parameter_offset + i) % 7;

            // Get the midpoints for the current parameter
            let mid_a = get_midpoint(a, current_param);
            let mid_b = get_midpoint(b, current_param);

            // Apply absolute value if required
            let val_a = if abs { mid_a.abs() } else { mid_a };
            let val_b = if abs { mid_b.abs() } else { mid_b };

            match val_a.cmp(&val_b) {
                Ordering::Equal => continue,   // Move to the next parameter if equal
                non_equal => return non_equal, // Return the result if not equal
            }
        }

        Ordering::Equal // All parameters are equal
    });
}

fn get_midpoint<T: Clone>(node: &TreeNode<T>, parameter: usize) -> i64 {
    let range = &node.bounds()[parameter];
    (range.min + range.max) / 2
}

fn calculate_midpoint_sum<T: Clone>(node: &TreeNode<T>) -> i64 {
    node.bounds()
        .iter()
        .map(|range| ((range.min + range.max) / 2).abs())
        .sum()
}

fn get_batched_tree<T: Clone>(nodes: Vec<TreeNode<T>>) -> Vec<TreeNode<T>> {
    let mut result = Vec::new();
    let mut current_batch = Vec::new();

    // Calculate batch size based on the formula
    let node_count = nodes.len();
    let batch_size = (6.0f64.powf((node_count as f64 - 0.01).log(6.0).floor())) as usize;

    for node in nodes {
        current_batch.push(node);

        if current_batch.len() >= batch_size {
            result.push(TreeNode::Branch {
                children: current_batch.clone(),
                bounds: calculate_bounds(&current_batch),
            });
            current_batch.clear();
        }
    }

    // Add the remaining nodes as the final batch
    if !current_batch.is_empty() {
        result.push(TreeNode::Branch {
            children: current_batch.clone(),
            bounds: calculate_bounds(&current_batch),
        });
    }

    result
}

fn calculate_bounds<T: Clone>(nodes: &[TreeNode<T>]) -> [ParameterRange; 7] {
    let mut bounds = *nodes[0].bounds();

    for node in nodes.iter().skip(1) {
        for (i, range) in node.bounds().iter().enumerate() {
            bounds[i] = bounds[i].combine(range);
        }
    }

    bounds
}

fn calculate_bounds_sum(bounds: &[ParameterRange]) -> i64 {
    bounds.iter().map(|range| range.max - range.min).sum()
}

#[derive(Clone)]
pub enum TreeNode<T: Clone> {
    Leaf(TreeLeafNode<T>),
    Branch {
        children: Vec<TreeNode<T>>,
        bounds: [ParameterRange; 7],
    },
}

#[derive(Clone)]
pub struct TreeLeafNode<T: Clone> {
    value: T,
    point: [ParameterRange; 7],
}

impl<T: Clone> TreeNode<T> {
    pub fn new_leaf(value: T, point: [ParameterRange; 7]) -> Self {
        TreeNode::Leaf(TreeLeafNode { value, point })
    }

    // pub fn new_branch(children: Vec<TreeNode<T>>, bounds: [ParameterRange; 7]) -> Self {
    //     TreeNode::Branch { children, bounds }
    // }

    pub fn get_node(
        &self,
        point: &[i64; 7],
        alternative: &Option<TreeLeafNode<T>>,
    ) -> Option<TreeLeafNode<T>> {
        match self {
            Self::Leaf(node) => Some(node.clone()),
            Self::Branch { children, .. } => {
                let mut min = alternative
                    .as_ref()
                    .map(|node| squared_distance(&node.point, point))
                    .unwrap_or(i64::MAX);
                let mut tree_leaf_node = alternative.clone();
                for node in children {
                    let distance = squared_distance(node.bounds(), point);
                    if distance < min {
                        let tree_leaf_node2 = node
                            .get_node(point, alternative)
                            .expect("get_node should always return a value on a non empty tree");

                        let distance2 = squared_distance(&tree_leaf_node2.point, point);
                        if distance2 < min {
                            min = distance2;
                            tree_leaf_node = Some(tree_leaf_node2);
                        }
                    }
                }
                tree_leaf_node
            }
        }
    }

    pub fn bounds(&self) -> &[ParameterRange; 7] {
        match self {
            TreeNode::Leaf(TreeLeafNode { point, .. }) => point,
            TreeNode::Branch { bounds, .. } => bounds,
        }
    }

    pub fn children(self) -> Vec<TreeNode<T>> {
        match self {
            TreeNode::Leaf(TreeLeafNode { .. }) => vec![],
            TreeNode::Branch { children, .. } => children,
        }
    }
}

fn squared_distance(a: &[ParameterRange; 7], b: &[i64; 7]) -> i64 {
    a.iter().zip(b).map(|(a, b)| a.get_distance(*b)).sum()
}
