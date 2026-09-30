use pumpkin_data::chunk::{
    Parameter, ParameterPoint, TargetPoint, quantize_coord, unquantize_coord,
};
use pumpkin_util::math::position::BlockPos;

use crate::generation::biome_coords;
use crate::generation::noise::router::multi_noise_sampler::MultiNoiseSampler;

pub struct Climate;

impl Climate {
    pub const QUANTIZATION_FACTOR: f32 = 10000.0;
    pub const PARAMETER_COUNT: usize = 7;

    #[inline]
    #[must_use]
    pub const fn target(
        temperature: f32,
        humidity: f32,
        continentalness: f32,
        erosion: f32,
        depth: f32,
        weirdness: f32,
    ) -> TargetPoint {
        TargetPoint::new(
            quantize_coord(temperature),
            quantize_coord(humidity),
            quantize_coord(continentalness),
            quantize_coord(erosion),
            quantize_coord(depth),
            quantize_coord(weirdness),
        )
    }

    #[inline]
    #[must_use]
    pub const fn parameters(
        temperature: f32,
        humidity: f32,
        continentalness: f32,
        erosion: f32,
        depth: f32,
        weirdness: f32,
        offset: f32,
    ) -> ParameterPoint {
        ParameterPoint::new(
            Parameter::point(temperature),
            Parameter::point(humidity),
            Parameter::point(continentalness),
            Parameter::point(erosion),
            Parameter::point(depth),
            Parameter::point(weirdness),
            quantize_coord(offset),
        )
    }

    #[inline]
    #[must_use]
    pub const fn quantize_coord(coord: f32) -> i64 {
        quantize_coord(coord)
    }

    #[inline]
    #[must_use]
    pub const fn unquantize_coord(coord: i64) -> f32 {
        unquantize_coord(coord)
    }

    #[must_use]
    pub fn find_spawn_position(
        target_climates: &[ParameterPoint],
        sampler: &mut impl ClimateSampler,
    ) -> BlockPos {
        SpawnFinder::find_spawn_position(target_climates, sampler)
    }
}

pub trait ClimateSampler {
    fn sample(&mut self, quart_x: i32, quart_y: i32, quart_z: i32) -> TargetPoint;
}

impl<F> ClimateSampler for F
where
    F: FnMut(i32, i32, i32) -> TargetPoint,
{
    fn sample(&mut self, quart_x: i32, quart_y: i32, quart_z: i32) -> TargetPoint {
        self(quart_x, quart_y, quart_z)
    }
}

impl ClimateSampler for MultiNoiseSampler<'_> {
    fn sample(&mut self, quart_x: i32, quart_y: i32, quart_z: i32) -> TargetPoint {
        self.sample(quart_x, quart_y, quart_z)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnFinderResult {
    pub location: BlockPos,
    pub fitness: i64,
}

pub type FittestPositionFinderResult = SpawnFinderResult;
pub type FittestPositionFinder = SpawnFinder;

pub struct SpawnFinder {
    pub result: SpawnFinderResult,
}

impl SpawnFinder {
    pub const MAX_RADIUS: i64 = 2048;

    #[must_use]
    pub fn find_spawn_position(
        target_climates: &[ParameterPoint],
        sampler: &mut impl ClimateSampler,
    ) -> BlockPos {
        let finder = Self::new(target_climates, sampler);
        finder.result.location
    }

    #[must_use]
    pub fn new(target_climates: &[ParameterPoint], sampler: &mut impl ClimateSampler) -> Self {
        let mut finder = Self {
            result: Self::get_spawn_position_and_fitness(target_climates, sampler, 0, 0),
        };
        finder.radial_search(target_climates, sampler, 2048.0, 512.0);
        finder.radial_search(target_climates, sampler, 512.0, 32.0);
        finder
    }

    fn radial_search(
        &mut self,
        target_climates: &[ParameterPoint],
        sampler: &mut impl ClimateSampler,
        max_radius: f32,
        radius_increment: f32,
    ) {
        let mut angle = 0.0f32;
        let mut radius = radius_increment;
        let search_origin = self.result.location;

        while radius <= max_radius {
            let x = search_origin.0.x + (angle.sin() * radius) as i32;
            let z = search_origin.0.z + (angle.cos() * radius) as i32;
            let candidate = Self::get_spawn_position_and_fitness(target_climates, sampler, x, z);
            if candidate.fitness < self.result.fitness {
                self.result = candidate;
            }

            angle += radius_increment / radius;
            if angle > std::f32::consts::PI * 2.0 {
                angle = 0.0;
                radius += radius_increment;
            }
        }
    }

    #[must_use]
    pub fn get_spawn_position_and_fitness(
        target_climates: &[ParameterPoint],
        sampler: &mut impl ClimateSampler,
        block_x: i32,
        block_z: i32,
    ) -> SpawnFinderResult {
        let quart_x = biome_coords::from_block(block_x);
        let quart_z = biome_coords::from_block(block_z);
        let target_point = sampler.sample(quart_x, 0, quart_z);
        let zero_depth_target_point = TargetPoint::new(
            target_point.temperature,
            target_point.humidity,
            target_point.continentalness,
            target_point.erosion,
            0,
            target_point.weirdness,
        );

        let mut min_fitness = i64::MAX;
        for point in target_climates {
            min_fitness = min_fitness.min(point.fitness(&zero_depth_target_point));
        }

        let distance_bias_to_world_origin =
            (block_x as i64) * (block_x as i64) + (block_z as i64) * (block_z as i64);
        let fitness_with_distance = min_fitness * (2048 * 2048) + distance_bias_to_world_origin;

        SpawnFinderResult {
            location: BlockPos::new(block_x, 0, block_z),
            fitness: fitness_with_distance,
        }
    }
}

pub trait DistanceMetric<T> {
    fn distance(&self, node: &RTreeNode<T>, target: &[i64; 7]) -> i64;
}

impl<F, T> DistanceMetric<T> for F
where
    F: Fn(&RTreeNode<T>, &[i64; 7]) -> i64,
{
    fn distance(&self, node: &RTreeNode<T>, target: &[i64; 7]) -> i64 {
        self(node, target)
    }
}

pub fn default_distance_metric<T>(node: &RTreeNode<T>, target: &[i64; 7]) -> i64 {
    node.distance(target)
}

pub enum RTreeNode<T> {
    Leaf(RTreeLeaf<T>),
    SubTree(RTreeSubTree<T>),
}

pub struct RTreeLeaf<T> {
    pub parameter_space: [Parameter; 7],
    pub value: T,
}

pub struct RTreeSubTree<T> {
    pub parameter_space: [Parameter; 7],
    pub children: Vec<RTreeNode<T>>,
}

impl<T> RTreeNode<T> {
    #[must_use]
    pub const fn parameter_space(&self) -> &[Parameter; 7] {
        match self {
            Self::Leaf(leaf) => &leaf.parameter_space,
            Self::SubTree(subtree) => &subtree.parameter_space,
        }
    }

    #[must_use]
    pub fn distance(&self, target: &[i64; 7]) -> i64 {
        let space = self.parameter_space();
        let mut distance = 0i64;
        for i in 0..7 {
            let d = space[i].distance(target[i]);
            distance += d * d;
        }
        distance
    }
}

impl<T: Clone> Clone for RTreeNode<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Leaf(leaf) => Self::Leaf(RTreeLeaf {
                parameter_space: leaf.parameter_space,
                value: leaf.value.clone(),
            }),
            Self::SubTree(subtree) => Self::SubTree(RTreeSubTree {
                parameter_space: subtree.parameter_space,
                children: subtree.children.clone(),
            }),
        }
    }
}

pub struct RTree<T> {
    root: RTreeNode<T>,
}

impl<T: Clone> RTree<T> {
    #[must_use]
    pub fn create(values: Vec<(ParameterPoint, T)>) -> Self {
        assert!(
            !values.is_empty(),
            "Need at least one value to build the search tree."
        );
        let leaves: Vec<RTreeNode<T>> = values
            .into_iter()
            .map(|(p, val)| {
                RTreeNode::Leaf(RTreeLeaf {
                    parameter_space: p.parameter_space(),
                    value: val,
                })
            })
            .collect();
        Self {
            root: Self::build(7, leaves),
        }
    }

    fn build(dimensions: usize, mut children: Vec<RTreeNode<T>>) -> RTreeNode<T> {
        assert!(
            !children.is_empty(),
            "Need at least one child to build a node"
        );
        if children.len() == 1 {
            return children.remove(0);
        }

        if children.len() <= 6 {
            children.sort_by_key(|leaf| {
                let mut total_magnitude = 0i64;
                for dx in 0..dimensions {
                    let parameter = leaf.parameter_space()[dx];
                    total_magnitude += i64::midpoint(parameter.min, parameter.max).abs();
                }
                total_magnitude
            });
            let parameter_space = Self::build_parameter_space(&children);
            return RTreeNode::SubTree(RTreeSubTree {
                parameter_space,
                children,
            });
        }

        let mut min_cost = i64::MAX;
        let mut min_dimension = 0;
        let mut min_buckets: Vec<Vec<RTreeNode<T>>> = Vec::new();

        for d in 0..dimensions {
            Self::sort_nodes(&mut children, dimensions, d, false);
            let buckets = Self::bucketize(&children);
            let mut total_cost = 0i64;
            for bucket in &buckets {
                let param_space = Self::build_parameter_space(bucket);
                total_cost += Self::cost(&param_space);
            }

            if min_cost > total_cost {
                min_cost = total_cost;
                min_dimension = d;
                min_buckets = buckets;
            }
        }

        min_buckets.sort_by_key(|bucket| {
            let space = Self::build_parameter_space(bucket);
            let param = space[min_dimension];
            i64::midpoint(param.min, param.max).abs()
        });

        let sub_children: Vec<RTreeNode<T>> = min_buckets
            .into_iter()
            .map(|b| Self::build(dimensions, b))
            .collect();
        let parameter_space = Self::build_parameter_space(&sub_children);
        RTreeNode::SubTree(RTreeSubTree {
            parameter_space,
            children: sub_children,
        })
    }

    fn sort_nodes(
        children: &mut [RTreeNode<T>],
        dimensions: usize,
        dimension: usize,
        absolute: bool,
    ) {
        children.sort_by(|a, b| {
            for d in 0..dimensions {
                let dim = (dimension + d) % dimensions;
                let param_a = a.parameter_space()[dim];
                let param_b = b.parameter_space()[dim];
                let center_a = i64::midpoint(param_a.min, param_a.max);
                let center_b = i64::midpoint(param_b.min, param_b.max);
                let val_a = if absolute { center_a.abs() } else { center_a };
                let val_b = if absolute { center_b.abs() } else { center_b };
                let ord = val_a.cmp(&val_b);
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            std::cmp::Ordering::Equal
        });
    }

    fn bucketize(nodes: &[RTreeNode<T>]) -> Vec<Vec<RTreeNode<T>>> {
        let mut buckets = Vec::new();
        let mut current_bucket = Vec::new();
        let expected_children_count =
            6.0f64.powf(((nodes.len() as f64 - 0.01).ln() / 6.0f64.ln()).floor()) as usize;

        for child in nodes {
            current_bucket.push(child.clone());
            if current_bucket.len() >= expected_children_count {
                buckets.push(std::mem::take(&mut current_bucket));
            }
        }
        if !current_bucket.is_empty() {
            buckets.push(current_bucket);
        }
        buckets
    }

    fn cost(parameter_space: &[Parameter; 7]) -> i64 {
        let mut result = 0i64;
        for param in parameter_space {
            result += (param.max - param.min).abs();
        }
        result
    }

    fn build_parameter_space(children: &[RTreeNode<T>]) -> [Parameter; 7] {
        assert!(!children.is_empty(), "SubTree needs at least one child");
        let mut bounds = *children[0].parameter_space();
        for child in &children[1..] {
            let space = child.parameter_space();
            for d in 0..7 {
                bounds[d] = bounds[d].span_with(Some(&space[d]));
            }
        }
        bounds
    }

    #[must_use]
    pub fn search<'a>(&'a self, target: &TargetPoint) -> &'a T {
        self.search_with_metric(target, &default_distance_metric)
    }

    pub fn search_with_metric<'a>(
        &'a self,
        target: &TargetPoint,
        distance_metric: &impl DistanceMetric<T>,
    ) -> &'a T {
        let target_array = target.to_parameter_array();
        let mut min_distance = i64::MAX;
        let mut closest_leaf: Option<&'a RTreeLeaf<T>> = None;
        Self::search_node(
            &self.root,
            &target_array,
            &mut min_distance,
            &mut closest_leaf,
            distance_metric,
        );
        closest_leaf.map_or_else(
            || unreachable!("Search must find a leaf"),
            |leaf| &leaf.value,
        )
    }

    fn search_node<'a>(
        node: &'a RTreeNode<T>,
        target: &[i64; 7],
        min_distance: &mut i64,
        closest_leaf: &mut Option<&'a RTreeLeaf<T>>,
        distance_metric: &impl DistanceMetric<T>,
    ) {
        match node {
            RTreeNode::Leaf(leaf) => {
                let dist = distance_metric.distance(node, target);
                if dist < *min_distance {
                    *min_distance = dist;
                    *closest_leaf = Some(leaf);
                }
            }
            RTreeNode::SubTree(subtree) => {
                for child in &subtree.children {
                    let child_dist = distance_metric.distance(child, target);
                    if child_dist < *min_distance {
                        Self::search_node(
                            child,
                            target,
                            min_distance,
                            closest_leaf,
                            distance_metric,
                        );
                    }
                }
            }
        }
    }
}

pub struct ParameterList<T> {
    values: Vec<(ParameterPoint, T)>,
    index: RTree<T>,
}

impl<T: Clone> ParameterList<T> {
    #[must_use]
    pub fn new(values: Vec<(ParameterPoint, T)>) -> Self {
        let index = RTree::create(values.clone());
        Self { values, index }
    }

    #[must_use]
    pub fn values(&self) -> &[(ParameterPoint, T)] {
        &self.values
    }

    #[must_use]
    pub fn find_value(&self, target: &TargetPoint) -> &T {
        self.find_value_index(target)
    }

    #[must_use]
    pub fn find_value_brute_force(&self, target: &TargetPoint) -> &T {
        let mut best_fitness = i64::MAX;
        let mut best_val = None;
        for (point, val) in &self.values {
            let fitness = point.fitness(target);
            if fitness < best_fitness {
                best_fitness = fitness;
                best_val = Some(val);
            }
        }
        best_val.unwrap_or_else(|| unreachable!("ParameterList must not be empty"))
    }

    #[must_use]
    pub fn find_value_index(&self, target: &TargetPoint) -> &T {
        self.index.search(target)
    }

    pub fn find_value_index_with_metric(
        &self,
        target: &TargetPoint,
        distance_metric: &impl DistanceMetric<T>,
    ) -> &T {
        self.index.search_with_metric(target, distance_metric)
    }
}
