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

impl TreeFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        min_y: i8,
        height: u16,
        feature_name: pumpkin_data::placed_feature::PlacedFeature, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let (log_positions, root_positions, foliage_positions) = self.generate_main(
            block_registry,
            chunk,
            min_y,
            height,
            feature_name,
            random,
            pos,
        );

        if log_positions.is_empty() && foliage_positions.is_empty() {
            return false;
        }

        for decorator in &self.decorators {
            decorator.generate(
                chunk,
                block_registry,
                random,
                &root_positions,
                &log_positions,
                &foliage_positions,
            );
        }

        Self::update_leaves(chunk, &log_positions, &root_positions, &foliage_positions);

        true
    }

    pub fn update_leaves<T: GenerationCache>(
        chunk: &mut T,
        logs: &[BlockPos],
        roots: &[BlockPos],
        foliage: &[BlockPos],
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

        for pos in logs.iter().chain(roots.iter()).chain(foliage.iter()) {
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
        let mut visited = vec![false; total_size];

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

        for pos in roots {
            if let Some(idx) = get_index(pos.0.x, pos.0.y, pos.0.z) {
                visited[idx] = true;
            }
        }

        let mut to_check: [std::collections::HashSet<BlockPos>; 7] = Default::default();
        for pos in logs {
            to_check[0].insert(*pos);
        }

        let mut smallest_distance = 0;

        while smallest_distance < 7 {
            while smallest_distance < 7 && !to_check[smallest_distance].is_empty() {
                let Some(pos) = to_check[smallest_distance].iter().next().copied() else {
                    break;
                };
                to_check[smallest_distance].remove(&pos);

                let Some(idx) = get_index(pos.0.x, pos.0.y, pos.0.z) else {
                    continue;
                };

                if smallest_distance != 0 {
                    let (block, state) = chunk.get_block_and_state(&pos);
                    if OakLeavesLikeProperties::handles_block_id(block.id) {
                        let mut props = OakLeavesLikeProperties::from_state_id(state.id, block);
                        props.distance = smallest_distance as u8;
                        let new_state = &block.states[props.to_index() as usize];
                        chunk.set_block_state(&pos.0, new_state);
                    }
                }

                visited[idx] = true;

                for direction in pumpkin_data::BlockDirection::all() {
                    let offset = direction.to_offset();
                    let neighbor_pos = pos.offset(offset);
                    if let Some(n_idx) =
                        get_index(neighbor_pos.0.x, neighbor_pos.0.y, neighbor_pos.0.z)
                        && !visited[n_idx]
                    {
                        let (n_block, n_state) = chunk.get_block_and_state(&neighbor_pos);
                        let distance =
                            if n_block.has_tag(&tag::Block::MINECRAFT_PREVENTS_NEARBY_LEAF_DECAY) {
                                Some(0)
                            } else if OakLeavesLikeProperties::handles_block_id(n_block.id) {
                                Some(
                                    OakLeavesLikeProperties::from_state_id(n_state.id, n_block)
                                        .distance as usize,
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

            smallest_distance += 1;
        }
    }

    pub fn can_replace_or_log(state: &BlockState, id: BlockId) -> bool {
        Self::can_replace(state, id) || id.has_tag(tag::Block::MINECRAFT_LOGS)
    }

    pub fn is_air_or_leaves(state: &BlockState, id: BlockId) -> bool {
        state.is_air() || id.has_tag(tag::Block::MINECRAFT_LEAVES)
    }

    pub fn can_replace(state: &BlockState, id: BlockId) -> bool {
        state.is_air() || id.has_tag(tag::Block::MINECRAFT_REPLACEABLE_BY_TREES)
    }

    #[expect(clippy::too_many_arguments)]
    fn generate_main<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature_name: pumpkin_data::placed_feature::PlacedFeature, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> (Vec<BlockPos>, Vec<BlockPos>, Vec<BlockPos>) {
        let height = self.trunk_placer.get_height(random);

        let trunk_start = self
            .root_placer
            .as_ref()
            .map_or(pos, |placer| placer.trunk_offset(pos, random));

        let clipped_height = self.minimum_size.min_clipped_height;
        let top = self.get_top(height, chunk, trunk_start);
        if top < height && top < clipped_height.map_or(u32::MAX, |h| h as u32) {
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
            top,
            trunk_start,
            chunk,
            random,
            &self.below_trunk_provider,
            trunk_state,
        );

        let foliage_height = self
            .foliage_placer
            .r#type
            .get_random_height(random, height as i32);
        let base_height = height as i32 - foliage_height;
        let foliage_radius = self.foliage_placer.get_random_radius(random, base_height);
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

    fn get_top<T: GenerationCache>(&self, height: u32, chunk: &T, init_pos: BlockPos) -> u32 {
        for y in 0..=height + 1 {
            let j = self.minimum_size.r#type.get_radius(height, y as i32);
            for x in -j..=j {
                for z in -j..=j {
                    let pos = BlockPos(init_pos.0.add_raw(x, y as i32, z));
                    let rstate = GenerationCache::get_block_state(chunk, &pos.0);
                    let block = rstate.to_block_id();
                    if Self::can_replace_or_log(rstate.to_state(), block)
                        && (self.ignore_vines || block != BlockId::VINE)
                    {
                        continue;
                    }
                    return y.saturating_sub(2);
                }
            }
        }
        height
    }
}
