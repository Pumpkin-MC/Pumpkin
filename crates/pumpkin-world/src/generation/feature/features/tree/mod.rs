use decorator::TreeDecorator;
use foliage::FoliagePlacer;
use pumpkin_data::BlockState;
use pumpkin_data::block_properties::{BlockProperties, OakLeavesLikeProperties};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{BlockId, tag};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use root::RootPlacer;

use trunk::TrunkPlacer;

use crate::generation::proto_chunk::GenerationCache;
use crate::generation::{block_state_provider::BlockStateProvider, feature::size::FeatureSize};
use crate::world::WorldPortalExt;

pub mod decorator;
pub mod foliage;
pub mod root;
pub mod trunk;

pub struct TreeFeature {
    pub trunk_provider: BlockStateProvider,
    pub trunk_placer: TrunkPlacer,
    pub foliage_provider: BlockStateProvider,
    pub foliage_placer: FoliagePlacer,
    pub minimum_size: FeatureSize,
    pub ignore_vines: bool,
    pub decorators: Vec<TreeDecorator>,
    pub below_trunk_provider: BlockStateProvider,
    pub root_placer: Option<RootPlacer>,
}

pub struct TreeNode {
    center: BlockPos,
    foliage_radius: i32,
    giant_trunk: bool,
}

#[derive(Default, Clone, Debug)]
pub struct FifoSet {
    queue: std::collections::VecDeque<BlockPos>,
    set: std::collections::HashSet<BlockPos>,
}

impl FifoSet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, pos: BlockPos) -> bool {
        if self.set.insert(pos) {
            self.queue.push_back(pos);
            true
        } else {
            false
        }
    }

    pub fn pop_first(&mut self) -> Option<BlockPos> {
        if let Some(pos) = self.queue.pop_front() {
            self.set.remove(&pos);
            Some(pos)
        } else {
            None
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

impl TreeFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let (mut log_positions, mut root_positions, mut foliage_positions) =
            self.generate_main(block_registry, chunk, random, pos);

        if log_positions.is_empty() && foliage_positions.is_empty() {
            return false;
        }

        log_positions.sort_by_key(|p| p.0.y);
        root_positions.sort_by_key(|p| p.0.y);
        foliage_positions.sort_by_key(|p| p.0.y);

        let mut decorations = Vec::new();
        for decorator in &self.decorators {
            decorator.generate(
                chunk,
                block_registry,
                random,
                &root_positions,
                &log_positions,
                &foliage_positions,
                &mut decorations,
            );
        }

        Self::update_leaves(
            chunk,
            &log_positions,
            &root_positions,
            &foliage_positions,
            &decorations,
        );

        true
    }

    pub fn update_leaves<T: GenerationCache>(
        chunk: &mut T,
        logs: &[BlockPos],
        roots: &[BlockPos],
        foliage: &[BlockPos],
        decorations: &[BlockPos],
    ) {
        if logs.is_empty() && foliage.is_empty() {
            return;
        }

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut min_z = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        let mut max_z = i32::MIN;

        for pos in logs
            .iter()
            .chain(roots.iter())
            .chain(foliage.iter())
            .chain(decorations.iter())
        {
            min_x = min_x.min(pos.0.x);
            min_y = min_y.min(pos.0.y);
            min_z = min_z.min(pos.0.z);
            max_x = max_x.max(pos.0.x);
            max_y = max_y.max(pos.0.y);
            max_z = max_z.max(pos.0.z);
        }

        let x_span = (max_x - min_x + 1) as usize;
        let y_span = (max_y - min_y + 1) as usize;
        let z_span = (max_z - min_z + 1) as usize;

        let total_size = x_span * y_span * z_span;
        let mut shape = vec![false; total_size];

        let get_index = |x: i32, y: i32, z: i32| -> Option<usize> {
            if x < min_x || x > max_x || y < min_y || y > max_y || z < min_z || z > max_z {
                None
            } else {
                let x_idx = (x - min_x) as usize;
                let y_idx = (y - min_y) as usize;
                let z_idx = (z - min_z) as usize;
                Some((x_idx * y_span + y_idx) * z_span + z_idx)
            }
        };

        for pos in roots.iter().chain(decorations.iter()) {
            if let Some(idx) = get_index(pos.0.x, pos.0.y, pos.0.z) {
                shape[idx] = true;
            }
        }

        let mut to_check: [FifoSet; 7] = Default::default();

        for pos in logs {
            to_check[0].insert(*pos);
        }

        let mut smallest_distance = 0;

        loop {
            while smallest_distance >= 7 || !to_check[smallest_distance].is_empty() {
                if smallest_distance >= 7 {
                    return;
                }

                let Some(pos) = to_check[smallest_distance].pop_first() else {
                    break;
                };

                if let Some(idx) = get_index(pos.0.x, pos.0.y, pos.0.z) {
                    if smallest_distance != 0 {
                        let (block, state) = chunk.get_block_and_state(&pos);
                        if OakLeavesLikeProperties::handles_block_id(block.id) {
                            let mut props = OakLeavesLikeProperties::from_state_id(state.id);
                            props.distance = smallest_distance as u8;
                            let new_state = &block.states[props.to_index() as usize];
                            chunk.set_block_state(&pos.0, new_state);
                        }
                    }

                    shape[idx] = true;

                    for direction in pumpkin_data::BlockDirection::all() {
                        let offset = direction.to_offset();
                        let neighbor_pos = pos.offset(offset);
                        if let Some(n_idx) =
                            get_index(neighbor_pos.0.x, neighbor_pos.0.y, neighbor_pos.0.z)
                            && !shape[n_idx]
                        {
                            let (n_block, n_state) = chunk.get_block_and_state(&neighbor_pos);
                            let distance = if n_block
                                .has_tag(&tag::Block::MINECRAFT_PREVENTS_NEARBY_LEAF_DECAY)
                            {
                                Some(0)
                            } else if OakLeavesLikeProperties::handles_block_id(n_block.id) {
                                Some(
                                    OakLeavesLikeProperties::from_state_id(n_state.id).distance
                                        as usize,
                                )
                            } else {
                                None
                            };

                            if let Some(dist) = distance {
                                let new_distance = dist.min(smallest_distance + 1);
                                if new_distance < 7 {
                                    to_check[new_distance].insert(neighbor_pos);
                                    smallest_distance = smallest_distance.min(new_distance);
                                }
                            }
                        }
                    }
                }
            }

            smallest_distance += 1;
        }
    }

    #[must_use]
    pub fn can_replace_or_log(state: &BlockState, id: BlockId) -> bool {
        Self::can_replace(state, id) || id.has_tag(tag::Block::MINECRAFT_LOGS)
    }

    #[must_use]
    pub fn is_air_or_leaves(state: &BlockState, id: BlockId) -> bool {
        state.is_air() || id.has_tag(tag::Block::MINECRAFT_LEAVES)
    }

    #[must_use]
    pub fn can_replace(state: &BlockState, id: BlockId) -> bool {
        state.is_air() || id.has_tag(tag::Block::MINECRAFT_REPLACEABLE_BY_TREES)
    }

    fn generate_main<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> (Vec<BlockPos>, Vec<BlockPos>, Vec<BlockPos>) {
        let height = self.trunk_placer.get_height(random);
        let foliage_height = self
            .foliage_placer
            .r#type
            .get_random_height(random, height as i32);
        let trunk_height = height as i32 - foliage_height;
        let foliage_radius = self.foliage_placer.get_random_radius(random, trunk_height);

        let trunk_start = self
            .root_placer
            .as_ref()
            .map_or(pos, |placer| placer.trunk_offset(pos, random));

        let min_y = pos.0.y.min(trunk_start.0.y);
        let max_y = pos.0.y.max(trunk_start.0.y) + height as i32 + 1;
        if min_y < chunk.bottom_y() as i32 + 1 || max_y > chunk.top_y() as i32 {
            return (vec![], vec![], vec![]);
        }

        let clipped_height = self.minimum_size.min_clipped_height;
        let clipped_tree_height = self.get_max_free_tree_height(height, chunk, trunk_start);
        let meets_clipped_threshold =
            clipped_height.is_some_and(|min_clipped| clipped_tree_height >= min_clipped as i32);
        if clipped_tree_height < height as i32 && !meets_clipped_threshold {
            return (vec![], vec![], vec![]);
        }

        let root_positions = if let Some(placer) = &self.root_placer {
            match placer.generate(chunk, block_registry, random, pos, trunk_start) {
                Some(positions) => positions,
                None => return (vec![], vec![], vec![]),
            }
        } else {
            Vec::new()
        };

        let trunk_state = self.trunk_provider.get(random, pos, chunk, block_registry);

        let (nodes, logs) = self.trunk_placer.generate(
            block_registry,
            clipped_tree_height as u32,
            trunk_start,
            chunk,
            random,
            &self.below_trunk_provider,
            trunk_state,
        );

        let foliage_state = self
            .foliage_provider
            .get(random, pos, chunk, block_registry);
        let mut foliage_positions = Vec::new();
        for node in nodes {
            foliage_positions.extend(self.foliage_placer.generate(
                chunk,
                random,
                &node,
                foliage_height,
                foliage_radius,
                foliage_state,
            ));
        }
        (logs, root_positions, foliage_positions)
    }

    fn get_max_free_tree_height<T: GenerationCache>(
        &self,
        max_tree_height: u32,
        chunk: &T,
        tree_pos: BlockPos,
    ) -> i32 {
        for y in 0..=(max_tree_height as i32 + 1) {
            let r = self.minimum_size.r#type.get_radius(max_tree_height, y);
            for x in -r..=r {
                for z in -r..=r {
                    let pos = BlockPos(tree_pos.0.add_raw(x, y, z));
                    let rstate = GenerationCache::get_block_state(chunk, &pos.0);
                    let block = rstate.to_block_id();
                    if Self::can_replace_or_log(rstate.to_state(), block)
                        && (self.ignore_vines || block != BlockId::VINE)
                    {
                        continue;
                    }
                    return y - 2;
                }
            }
        }
        max_tree_height as i32
    }
}
