use std::cmp::Ordering;

use pumpkin_data::chunk::Biome;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, Map};

use super::noise::density::component_functions::SharedComponentReference;
use super::noise::density::NoisePos;

#[derive(Clone, Serialize, Deserialize)]
pub struct NoiseValuePoint {
    pub temperature: f64,
    pub erosion: f64,
    pub depth: f64,
    pub continents: f64,
    pub weirdness: f64,
    pub humidity: f64,
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

#[derive(Debug, Clone, Copy)]
pub struct ParameterRange {
    pub min: i64,
    pub max: i64,
}

impl ParameterRange {
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

impl<'de> Deserialize<'de> for ParameterRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let arr: [f64; 2] = Deserialize::deserialize(deserializer)?;
        Ok(ParameterRange {
            min: to_long(arr[0]),
            max: to_long(arr[1]),
        })
    }
}

fn to_long(float: f64) -> i64 {
    (float * 1000.0) as i64
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct BiomeEntries {
    #[serde_as(as = "Map<_, _>")]
    nodes: Vec<(Biome, NoiseValuePoint)>,
}

#[derive(Clone)]
pub struct MultiNoiseSampler {
    pub(crate) temperature: SharedComponentReference,
    pub(crate) erosion: SharedComponentReference,
    pub(crate) depth: SharedComponentReference,
    pub(crate) continents: SharedComponentReference,
    pub(crate) weirdness: SharedComponentReference,
    pub(crate) humidity: SharedComponentReference,
}

impl MultiNoiseSampler {
    pub fn sample(&self, pos: &NoisePos) -> NoiseValuePoint {
        NoiseValuePoint {
            temperature: self.temperature.sample(pos),
            erosion: self.erosion.sample(pos),
            depth: self.depth.sample(pos),
            continents: self.continents.sample(pos),
            weirdness: self.weirdness.sample(pos),
            humidity: self.humidity.sample(pos),
        }
    }
}

pub struct SearchTree<T: Clone> {
    root: TreeNode<T>,
}

#[derive(Clone, Debug)]
pub enum TreeNode<T: Clone> {
    Leaf {
        value: T,
        point: [ParameterRange; 7],
    },
    Branch {
        children: Vec<TreeNode<T>>,
        bounds: [ParameterRange; 7],
    },
}

impl<T: Clone> TreeNode<T> {
    pub fn create(entries: Vec<(NoiseHypercube, T)>) -> Self {
        let leaves: Vec<TreeNode<T>> = entries
            .into_iter()
            .map(|(hypercube, value)| TreeNode::Leaf {
                value,
                point: hypercube.to_parameters(),
            })
            .collect();

        Self::create_node(leaves)
    }

    /*fn get_resulting_node(
        &self,
        other_parameters: &[i64],
        distance_function: &dyn Fn(&TreeNode<T>, &[i64]) -> i64,
    ) -> Option<(T, [ParameterRange; 7])> {
        match self {
            TreeNode::Leaf { value, point } => (value, point),
            TreeNode::Branch { children, bounds } => {
                let mut closest_node = None;
                let mut smallest_distance = i64::MAX;

                for tree_node in sub_tree {
                    let distance = distance_function(tree_node, other_parameters);
                    if distance < smallest_distance {
                        if let Some(resulting_node) =
                            tree_node.get_resulting_node(other_parameters, distance_function)
                        {
                            let resulting_distance =
                                distance_function(&resulting_node, other_parameters);
                            if resulting_distance < smallest_distance {
                                smallest_distance = resulting_distance;
                                closest_node = Some(resulting_node);
                            }
                        } else {
                            smallest_distance = distance;
                            closest_node = tree_node.as_leaf();
                        }
                    }
                }

                closest_node
            }
        }
    }*/

    fn create_node(sub_tree: Vec<TreeNode<T>>) -> TreeNode<T> {
        if sub_tree.len() == 1 {
            return sub_tree.into_iter().next().unwrap();
        } else if sub_tree.len() <= 6 {
            let mut sorted_sub_tree = sub_tree;
            sorted_sub_tree.sort_by(|a, b| {
                let sum_a = Self::calculate_midpoint_sum(a);
                let sum_b = Self::calculate_midpoint_sum(b);
                sum_a.cmp(&sum_b)
            });
            let bounds = Self::calculate_bounds(&sorted_sub_tree);
            return TreeNode::Branch {
                children: sorted_sub_tree,
                bounds,
            };
        } else {
            // This is used to minimize the range each subtree takes, so that
            // the resulting tree is more balanced and efficient for searches.
            let best_split = (0..7)
                .map(|param_idx| {
                    let mut sorted_sub_tree = sub_tree.clone();
                    Self::sort_tree(&mut sorted_sub_tree, param_idx, false);
                    let batched_tree = Self::get_batched_tree(sorted_sub_tree);

                    let range_sum: i64 = batched_tree
                        .iter()
                        .map(|node| node.calculate_bounds_sum())
                        .sum();

                    (param_idx, batched_tree, range_sum)
                })
                .min_by(|(_, _, range_sum_a), (_, _, range_sum_b)| range_sum_a.cmp(range_sum_b));

            if let Some((best_param, mut best_batched, _)) = best_split {
                Self::sort_tree(&mut best_batched, best_param, true);
                let children: Vec<TreeNode<T>> = best_batched
                    .into_iter()
                    .map(|batch| Self::create_node(batch.children().to_vec()))
                    .collect();
                return TreeNode::Branch {
                    bounds: Self::calculate_bounds(&children),
                    children,
                };
            }
        }

        panic!("Failed to create a node");
    }

    fn sort_tree(sub_tree: &mut Vec<TreeNode<T>>, parameter_offset: usize, abs: bool) {
        sub_tree.sort_by(|a, b| {
            for i in 0..7 {
                // Calculate the parameter index in cyclic order
                let current_param = (parameter_offset + i) % 7;

                // Get the midpoints for the current parameter
                let mid_a = Self::get_midpoint(a, current_param);
                let mid_b = Self::get_midpoint(b, current_param);

                // Apply absolute value if required
                let val_a = if abs { mid_a.abs() } else { mid_a };
                let val_b = if abs { mid_b.abs() } else { mid_b };

                // Compare the values
                match val_a.cmp(&val_b) {
                    Ordering::Equal => continue,   // Move to the next parameter if equal
                    non_equal => return non_equal, // Return the result if not equal
                }
            }

            Ordering::Equal
        });
    }

    fn get_midpoint(&self, parameter: usize) -> i64 {
        let range = &self.bounds()[parameter];
        (range.min + range.max) / 2
    }

    fn get_batched_tree(nodes: Vec<TreeNode<T>>) -> Vec<TreeNode<T>> {
        if nodes.is_empty() {
            return Vec::new();
        }

        // Calculate the chunk size using the heuristic
        let node_count = nodes.len();
        let chunk_size = (6.0f64.powf((node_count as f64 - 0.01).log(6.0).floor())) as usize;

        nodes
            .chunks(chunk_size)
            .map(|chunk: &[TreeNode<T>]| TreeNode::Branch {
                children: chunk.to_vec(), // Convert the slice into a Vec<TreeNode<T>>
                bounds: Self::calculate_bounds(chunk), // Calculate bounds for this chunk
            })
            .collect()
    }

    fn calculate_midpoint_sum(&self) -> i64 {
        self.bounds()
            .iter()
            .map(|range| (range.min + range.max).abs() / 2)
            .sum()
    }

    fn calculate_bounds_sum(&self) -> i64 {
        self.bounds()
            .iter()
            .map(|range| range.max - range.min)
            .sum()
    }

    fn calculate_bounds(nodes: &[TreeNode<T>]) -> [ParameterRange; 7] {
        let mut bounds = *nodes[0].bounds();

        for node in nodes.iter().skip(1) {
            for (i, range) in node.bounds().iter().enumerate() {
                bounds[i] = bounds[i].combine(range);
            }
        }

        bounds
    }

    pub fn bounds(&self) -> &[ParameterRange; 7] {
        match self {
            TreeNode::Leaf { point, .. } => point,
            TreeNode::Branch { bounds, .. } => bounds,
        }
    }

    pub fn children(&self) -> &[TreeNode<T>] {
        match self {
            TreeNode::Leaf { .. } => &[],
            TreeNode::Branch { children, .. } => children,
        }
    }
}
