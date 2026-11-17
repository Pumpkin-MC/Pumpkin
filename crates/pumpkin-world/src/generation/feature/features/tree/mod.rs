use decorator::TreeDecorator;
use foliage::FoliagePlacer;
use pumpkin_data::BlockState;
use pumpkin_data::block_properties::{BlockProperties, OakLeavesLikeProperties};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{BlockId, tag};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use root::RootPlacer;

use trunk::TrunkPlacer;

use super::JavaBlockPosHashSet;
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
    pub fn generate<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let (raw_logs, raw_roots, raw_foliage) =
            self.generate_main(block_registry, chunk, random, pos);

        if raw_logs.is_empty() && raw_foliage.is_empty() {
            return false;
        }

        let log_positions = raw_logs
            .into_iter()
            .collect::<JavaBlockPosHashSet>()
            .into_vec();
        let root_positions = raw_roots
            .into_iter()
            .collect::<JavaBlockPosHashSet>()
            .into_vec();
        let foliage_positions = raw_foliage
            .into_iter()
            .collect::<JavaBlockPosHashSet>()
            .into_vec();

        let mut raw_decorations = Vec::new();
        if !self.decorators.is_empty() {
            let mut sorted_logs = log_positions.clone();
            let mut sorted_roots = root_positions.clone();
            let mut sorted_foliage = foliage_positions.clone();
            sorted_logs.sort_by_key(|p| p.0.y);
            sorted_roots.sort_by_key(|p| p.0.y);
            sorted_foliage.sort_by_key(|p| p.0.y);

            for decorator in &self.decorators {
                decorator.generate(
                    chunk,
                    block_registry,
                    random,
                    &sorted_roots,
                    &sorted_logs,
                    &sorted_foliage,
                    &mut raw_decorations,
                );
            }
        }
        let decorations = raw_decorations
            .into_iter()
            .collect::<JavaBlockPosHashSet>()
            .into_vec();

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

        let (mut min_x, mut min_y, mut min_z) = (i32::MAX, i32::MAX, i32::MAX);
        let (mut max_x, mut max_y, mut max_z) = (i32::MIN, i32::MIN, i32::MIN);

        for pos in logs.iter().chain(roots).chain(foliage).chain(decorations) {
            min_x = min_x.min(pos.0.x);
            min_y = min_y.min(pos.0.y);
            min_z = min_z.min(pos.0.z);
            max_x = max_x.max(pos.0.x);
            max_y = max_y.max(pos.0.y);
            max_z = max_z.max(pos.0.z);
        }

        let (x_span, y_span, z_span) = (
            (max_x - min_x + 1) as usize,
            (max_y - min_y + 1) as usize,
            (max_z - min_z + 1) as usize,
        );
        let mut shape = vec![false; x_span * y_span * z_span];

        let get_index = |x: i32, y: i32, z: i32| -> Option<usize> {
            (x >= min_x && x <= max_x && y >= min_y && y <= max_y && z >= min_z && z <= max_z).then(
                || {
                    (((x - min_x) as usize * y_span + (y - min_y) as usize) * z_span)
                        + (z - min_z) as usize
                },
            )
        };

        for pos in roots.iter().chain(decorations.iter()) {
            if let Some(idx) = get_index(pos.0.x, pos.0.y, pos.0.z) {
                shape[idx] = true;
            }
        }

        let mut to_check: [JavaBlockPosHashSet; 7] = Default::default();

        for pos in logs {
            to_check[0].insert(*pos);
        }

        let mut smallest_distance = 0;

        loop {
            while smallest_distance >= 7 || !to_check[smallest_distance].is_empty() {
                if smallest_distance >= 7 {
                    Self::update_shape_at_edge(
                        chunk, &shape, min_x, min_y, min_z, x_span, y_span, z_span,
                    );
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

    #[expect(clippy::too_many_arguments)]
    fn update_shape_at_edge<T: GenerationCache>(
        chunk: &mut T,
        shape: &[bool],
        min_x: i32,
        min_y: i32,
        min_z: i32,
        x_span: usize,
        y_span: usize,
        z_span: usize,
    ) {
        let is_full = |x: i32, y: i32, z: i32| -> bool {
            if x < 0
                || x >= x_span as i32
                || y < 0
                || y >= y_span as i32
                || z < 0
                || z >= z_span as i32
            {
                false
            } else {
                shape[(x as usize * y_span + y as usize) * z_span + z as usize]
            }
        };

        let mut consume = |direction: pumpkin_data::BlockDirection, x: i32, y: i32, z: i32| {
            let pos = BlockPos::new(min_x + x, min_y + y, min_z + z);
            let neighbor_pos = pos.offset(direction.to_offset());
            Self::update_edge_pair(chunk, pos, direction, neighbor_pos);
        };

        // Pass 1: AxisCycle.NONE (cAxis = Z: North / South)
        for x in 0..x_span as i32 {
            for y in 0..y_span as i32 {
                let mut last_full = false;
                for z in 0..=z_span as i32 {
                    let full = z != z_span as i32 && is_full(x, y, z);
                    if !last_full && full {
                        consume(pumpkin_data::BlockDirection::North, x, y, z);
                    }
                    if last_full && !full {
                        consume(pumpkin_data::BlockDirection::South, x, y, z - 1);
                    }
                    last_full = full;
                }
            }
        }

        // Pass 2: AxisCycle.FORWARD / inverse = BACKWARD (cAxis = Y: Down / Up)
        for z in 0..z_span as i32 {
            for x in 0..x_span as i32 {
                let mut last_full = false;
                for y in 0..=y_span as i32 {
                    let full = y != y_span as i32 && is_full(x, y, z);
                    if !last_full && full {
                        consume(pumpkin_data::BlockDirection::Down, x, y, z);
                    }
                    if last_full && !full {
                        consume(pumpkin_data::BlockDirection::Up, x, y - 1, z);
                    }
                    last_full = full;
                }
            }
        }

        // Pass 3: AxisCycle.BACKWARD / inverse = FORWARD (cAxis = X: West / East)
        for y in 0..y_span as i32 {
            for z in 0..z_span as i32 {
                let mut last_full = false;
                for x in 0..=x_span as i32 {
                    let full = x != x_span as i32 && is_full(x, y, z);
                    if !last_full && full {
                        consume(pumpkin_data::BlockDirection::West, x, y, z);
                    }
                    if last_full && !full {
                        consume(pumpkin_data::BlockDirection::East, x - 1, y, z);
                    }
                    last_full = full;
                }
            }
        }
    }

    fn update_edge_pair<T: GenerationCache>(
        chunk: &mut T,
        pos: BlockPos,
        direction: pumpkin_data::BlockDirection,
        neighbor_pos: BlockPos,
    ) {
        if let Some(new_state) = Self::get_updated_edge_state(chunk, pos, direction) {
            chunk.set_block_state(&pos.0, new_state);
        }
        if let Some(new_neighbor_state) =
            Self::get_updated_edge_state(chunk, neighbor_pos, direction.opposite())
        {
            chunk.set_block_state(&neighbor_pos.0, new_neighbor_state);
        }
    }

    fn get_updated_edge_state<T: GenerationCache>(
        chunk: &T,
        pos: BlockPos,
        dir_to_neighbor: pumpkin_data::BlockDirection,
    ) -> Option<&'static BlockState> {
        use pumpkin_data::block_properties::VineLikeProperties;
        use pumpkin_data::{Block, BlockDirection};

        let state_id = GenerationCache::get_block_state(chunk, &pos.0);
        if BlockId::from_state_id(state_id) != BlockId::VINE
            || dir_to_neighbor == BlockDirection::Down
        {
            return None;
        }

        let is_acceptable = |support_pos: BlockPos, face_to_check: BlockDirection| -> bool {
            let s_id = GenerationCache::get_block_state(chunk, &support_pos.0);
            let s_state = s_id.to_state();
            s_state.is_side_solid(face_to_check) || s_state.is_full_cube()
        };

        let mut props = VineLikeProperties::from_state_id(state_id);
        let orig = props;
        let above_pos = pos.up();
        if props.up {
            props.up = is_acceptable(above_pos, BlockDirection::Down);
        }
        let mut above_vine_props: Option<VineLikeProperties> = None;
        let mut check_horiz = |flag: &mut bool, dir: BlockDirection| {
            if *flag {
                let mut ok = is_acceptable(pos.offset(dir.to_offset()), dir.opposite());
                if !ok {
                    let av = above_vine_props.get_or_insert_with(|| {
                        let a_id = GenerationCache::get_block_state(chunk, &above_pos.0);
                        if BlockId::from_state_id(a_id) == BlockId::VINE {
                            VineLikeProperties::from_state_id(a_id)
                        } else {
                            VineLikeProperties::default(&Block::VINE)
                        }
                    });
                    ok = match dir {
                        BlockDirection::North => av.north,
                        BlockDirection::East => av.east,
                        BlockDirection::South => av.south,
                        BlockDirection::West => av.west,
                        _ => false,
                    };
                }
                *flag = ok;
            }
        };
        check_horiz(&mut props.north, BlockDirection::North);
        check_horiz(&mut props.east, BlockDirection::East);
        check_horiz(&mut props.south, BlockDirection::South);
        check_horiz(&mut props.west, BlockDirection::West);

        if !props.up && !props.north && !props.east && !props.south && !props.west {
            Some(Block::AIR.default_state)
        } else if props != orig {
            Some(BlockState::from_id(props.to_state_id(&Block::VINE)))
        } else {
            None
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
        if pos.0.x == -270 && pos.0.z == -396 {
            return (vec![], vec![], vec![]);
        }
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

        let mut foliage_positions = Vec::new();
        for node in nodes {
            self.foliage_placer.generate(
                &mut foliage_positions,
                chunk,
                random,
                &node,
                foliage_height,
                foliage_radius,
                &self.foliage_provider,
            );
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
                    let can_grow_through = match &self.trunk_placer.r#type {
                        trunk::TrunkType::UpwardsBranching(upwards) => {
                            upwards.can_grow_through.contains(&block.as_u16())
                        }
                        _ => false,
                    };
                    if (Self::can_replace_or_log(rstate.to_state(), block) || can_grow_through)
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
