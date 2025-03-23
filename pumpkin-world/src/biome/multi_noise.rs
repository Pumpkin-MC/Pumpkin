use std::cmp::Ordering;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

pub fn to_long(float: f32) -> i64 {
    (float * 10000.0) as i64
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
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
    pub offset: f32,
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
                min: to_long(self.offset),
                max: to_long(self.offset),
            },
        ]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ParameterRange {
    pub min: i64,
    pub max: i64,
}

impl<'de> Deserialize<'de> for ParameterRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        match value {
            Value::Array(arr) if arr.len() == 2 => {
                let min = arr[0]
                    .as_f64()
                    .ok_or_else(|| serde::de::Error::custom("Expected float"))?
                    as f32;
                assert!(min >= -2.0);
                let max = arr[1]
                    .as_f64()
                    .ok_or_else(|| serde::de::Error::custom("Expected float"))?
                    as f32;
                assert!(max <= 2.0);
                assert!(min < max, "min is more max");
                Ok(ParameterRange {
                    min: to_long(min),
                    max: to_long(max),
                })
            }
            Value::Number(num) if num.is_f64() => {
                let val = num
                    .as_f64()
                    .ok_or_else(|| serde::de::Error::custom("Expected float"))?
                    as f32;
                let converted_val = to_long(val);
                Ok(ParameterRange {
                    min: converted_val,
                    max: converted_val,
                })
            }
            _ => Err(serde::de::Error::custom(
                "Expected array of two floats or a single float",
            )),
        }
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

#[derive(Clone)]
/// T = Biome
pub struct SearchTree<T: Clone> {
    pub root: TreeNode<T>,
}

impl<T: Clone> SearchTree<T> {
    pub fn create(entries: Vec<(T, &NoiseHypercube)>) -> Self {
        assert!(!entries.is_empty(), "entries cannot be empty");

        let leaves: Vec<TreeNode<T>> = entries
            .into_iter()
            .map(|(value, hypercube)| TreeNode::new_leaf(value, hypercube.to_parameters()))
            .collect();

        SearchTree {
            root: create_node(7, leaves),
        }
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

fn create_node<T: Clone>(parameter_number: usize, mut sub_tree: Vec<TreeNode<T>>) -> TreeNode<T> {
    assert!(
        !sub_tree.is_empty(),
        "Need at least one child to build a node"
    );
    if sub_tree.len() == 1 {
        return sub_tree.first().unwrap().clone();
    }
    if sub_tree.len() <= 6 {
        let mut sorted_sub_tree = sub_tree;
        sorted_sub_tree.sort_by_key(|a| calculate_bounds_sum(a.bounds()));
        let bounds = get_enclosing_parameters(&sorted_sub_tree);
        return TreeNode::Branch {
            children: sorted_sub_tree,
            bounds,
        };
    }
    let mut best_range_sum = i64::MAX;
    let mut best_param = 0;
    let mut best_batched = Vec::new();

    for param_idx in 0..parameter_number {
        sort_tree(&mut sub_tree, parameter_number, param_idx, false);
        let batched_tree = get_batched_tree(sub_tree.clone());
        let range_sum: i64 = batched_tree
            .iter()
            .map(|node| get_range_length_sum(node.bounds()))
            .sum();

        if best_range_sum > range_sum {
            best_range_sum = range_sum;
            best_param = param_idx;
            best_batched = batched_tree;
        }
    }

    sort_tree(&mut best_batched, parameter_number, best_param, true);

    let children: Vec<TreeNode<T>> = best_batched
        .into_iter()
        .map(|batch| create_node(parameter_number, batch.children()))
        .collect();

    let bounds = get_enclosing_parameters(&children);
    TreeNode::Branch { children, bounds }
}

fn create_node_comparator<T: Clone>(
    current_parameter: usize,
    abs: bool,
) -> impl Fn(&TreeNode<T>, &TreeNode<T>) -> Ordering {
    move |a: &TreeNode<T>, b: &TreeNode<T>| {
        let range_a = &a.bounds()[current_parameter];
        let range_b = &b.bounds()[current_parameter];

        let mid_a = (range_a.min + range_a.max) / 2;
        let mid_b = (range_b.min + range_b.max) / 2;

        let val_a = if abs { mid_a.abs() } else { mid_a };
        let val_b = if abs { mid_b.abs() } else { mid_b };

        val_a.cmp(&val_b)
    }
}

fn sort_tree<T: Clone>(
    sub_tree: &mut [TreeNode<T>],
    parameter_number: usize,
    current_parameter: usize,
    abs: bool,
) {
    sub_tree.sort_by(|a, b| {
        let mut comparator = create_node_comparator(current_parameter, abs);

        for i in 1..parameter_number {
            let next_parameter = (current_parameter + i) % parameter_number;

            let next_comparator = create_node_comparator(next_parameter, abs);

            let result = comparator(a, b);

            if result != Ordering::Equal {
                return result;
            }
            comparator = next_comparator;
        }

        comparator(a, b)
    });
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
                bounds: get_enclosing_parameters(&current_batch),
            });
            current_batch.clear();
        }
    }

    // Add the remaining nodes as the final batch
    if !current_batch.is_empty() {
        result.push(TreeNode::Branch {
            children: current_batch.clone(),
            bounds: get_enclosing_parameters(&current_batch),
        });
    }

    result
}

fn get_enclosing_parameters<T: Clone>(nodes: &[TreeNode<T>]) -> [ParameterRange; 7] {
    assert!(!nodes.is_empty(), "SubTree needs at least one child");
    let mut bounds = *nodes[0].bounds();
    for node in nodes.iter().skip(1) {
        for (i, range) in node.bounds().iter().enumerate() {
            bounds[i] = bounds[i].combine(range);
        }
    }
    bounds
}

fn get_range_length_sum(bounds: &[ParameterRange]) -> i64 {
    bounds
        .iter()
        .map(|range| (range.max - range.min).abs())
        .sum()
}

fn calculate_bounds_sum(bounds: &[ParameterRange]) -> i64 {
    bounds
        .iter()
        .map(|range| ((range.min + range.max) / 2).abs())
        .sum()
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TreeNode<T: Clone> {
    Leaf(TreeLeafNode<T>),
    Branch {
        children: Vec<TreeNode<T>>,
        bounds: [ParameterRange; 7],
    },
}

#[derive(Clone, PartialEq, Eq, Debug)]
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
    fn is_leaf(&self, node: &TreeLeafNode<T>) -> bool {
        match self {
            TreeNode::Leaf(leaf) => leaf.point == node.point,
            TreeNode::Branch { .. } => false,
        }
    }

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
                    if min <= distance {
                        continue;
                    }
                    let tree_leaf_node2 = node
                        .get_node(point, &tree_leaf_node)
                        .expect("get_node should always return a value on a non empty tree");

                    let n = if node.is_leaf(&tree_leaf_node2) {
                        distance
                    } else {
                        squared_distance(&tree_leaf_node2.point, point)
                    };

                    if min <= n {
                        continue;
                    }

                    min = n;
                    tree_leaf_node = Some(tree_leaf_node2)
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
    a.iter()
        .zip(b)
        .map(|(a, b)| {
            let distance = a.get_distance(*b);
            distance * distance
        })
        .sum()
}

#[cfg(test)]
mod test {
    use crate::biome::multi_noise::{TreeNode, create_node};

    use super::{NoiseHypercube, ParameterRange};

    #[test]
    fn test_create_node_single_leaf() {
        let hypercube = NoiseHypercube {
            temperature: ParameterRange { min: 0, max: 10 },
            humidity: ParameterRange { min: 0, max: 10 },
            continentalness: ParameterRange { min: 0, max: 10 },
            erosion: ParameterRange { min: 0, max: 10 },
            depth: ParameterRange { min: 0, max: 10 },
            weirdness: ParameterRange { min: 0, max: 10 },
            offset: 0.0,
        };
        let leaves = vec![TreeNode::new_leaf(1, hypercube.to_parameters())];
        let node = create_node(7, leaves.clone());
        assert_eq!(node, leaves[0]);
    }

    #[test]
    fn test_create_node_multiple_leaves_small() {
        let hypercube1 = NoiseHypercube {
            temperature: ParameterRange { min: 0, max: 10 },
            humidity: ParameterRange { min: 0, max: 10 },
            continentalness: ParameterRange { min: 0, max: 10 },
            erosion: ParameterRange { min: 0, max: 10 },
            depth: ParameterRange { min: 0, max: 10 },
            weirdness: ParameterRange { min: 0, max: 10 },
            offset: 0.0,
        };
        let hypercube2 = NoiseHypercube {
            temperature: ParameterRange { min: 10, max: 20 },
            humidity: ParameterRange { min: 10, max: 20 },
            continentalness: ParameterRange { min: 10, max: 20 },
            erosion: ParameterRange { min: 10, max: 20 },
            depth: ParameterRange { min: 10, max: 20 },
            weirdness: ParameterRange { min: 10, max: 20 },
            offset: 0.0,
        };
        let leaves = vec![
            TreeNode::new_leaf(1, hypercube1.to_parameters()),
            TreeNode::new_leaf(2, hypercube2.to_parameters()),
        ];
        let node = create_node(7, leaves.clone());
        if let TreeNode::Branch { children, .. } = node {
            assert_eq!(children.len(), 2);
            assert_eq!(children[0], leaves[0]);
            assert_eq!(children[1], leaves[1]);
        } else {
            panic!("Expected a branch node");
        }
    }
}
