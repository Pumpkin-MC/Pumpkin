pub mod bamboo;
pub mod basalt_columns;
pub mod basalt_pillar;
pub mod block_column;
pub mod block_pile;
pub mod blue_ice;
pub mod bonus_chest;
pub mod chorus_plant;
pub mod coral;
pub mod delta_feature;
pub mod desert_well;
pub mod disk;
pub mod drip_stone;
pub mod end_gateway;
pub mod end_island;
pub mod end_platform;
pub mod end_podium;
pub mod end_spike;
pub mod fallen_tree;
pub mod fill_layer;
pub mod forest_rock;
pub mod fossil;
pub mod freeze_top_layer;
pub mod geode;
pub mod glowstone_blob;
pub mod huge_brown_mushroom;
pub mod huge_fungus;
pub mod huge_red_mushroom;
pub mod ice_spike;
pub mod iceberg;
pub mod kelp;
pub mod lake;
pub mod monster_room;
pub mod multiface_growth;
pub mod nether_forest_vegetation;
pub mod netherrack_replace_blobs;
pub mod ore;
pub mod overlay;
pub mod projected_random_patchy_square;
pub mod random_boolean_selector;
pub mod random_neighbor_spread;
pub mod random_patch;
pub mod random_selector;
pub mod replace_single_block;
pub mod root_system;
pub mod scattered_ore;
pub mod sculk;
pub mod sculk_patch;
pub mod sea_pickle;
pub mod seagrass;
pub mod simple_block;
pub mod simple_random_selector;
pub mod single_block_pillar;
pub mod spring_feature;
pub mod stepped_column_cluster;
pub mod tree;
pub mod twisting_vines;
pub mod underwater_magma;
pub mod vegetation_patch;
pub mod vines;
pub mod void_start_platform;
pub mod waterlogged_vegetation_patch;
pub mod weeping_vines;
pub mod weighted_random_selector;

use pumpkin_util::math::position::BlockPos;

/// A hash set of [`BlockPos`] that preserves `OpenJDK` `java.util.HashSet<BlockPos>` iteration order.
#[derive(Clone, Debug)]
pub struct JavaBlockPosHashSet {
    buckets: Vec<Vec<(i32, BlockPos)>>,
    len: usize,
    threshold: usize,
}

impl Default for JavaBlockPosHashSet {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaBlockPosHashSet {
    #[must_use]
    pub fn new() -> Self {
        Self {
            buckets: vec![Vec::new(); 16],
            len: 0,
            threshold: 12,
        }
    }

    const fn hash(pos: BlockPos) -> i32 {
        let h = pos
            .0
            .y
            .wrapping_add(pos.0.z.wrapping_mul(31))
            .wrapping_mul(31)
            .wrapping_add(pos.0.x);
        h ^ ((h as u32 >> 16) as i32)
    }

    pub fn insert(&mut self, pos: BlockPos) -> bool {
        let hash = Self::hash(pos);
        let cap = self.buckets.len();
        let idx = (hash as usize) & (cap - 1);
        if self.buckets[idx].iter().any(|&(_, p)| p == pos) {
            return false;
        }
        self.buckets[idx].push((hash, pos));
        self.len += 1;
        if self.len > self.threshold {
            self.resize();
        }
        true
    }

    #[must_use]
    pub fn contains(&self, pos: &BlockPos) -> bool {
        let hash = Self::hash(*pos);
        let idx = (hash as usize) & (self.buckets.len() - 1);
        self.buckets[idx].iter().any(|(_, p)| p == pos)
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn pop_first(&mut self) -> Option<BlockPos> {
        for bucket in &mut self.buckets {
            if !bucket.is_empty() {
                self.len -= 1;
                return Some(bucket.remove(0).1);
            }
        }
        None
    }

    pub fn iter(&self) -> impl Iterator<Item = &BlockPos> {
        self.buckets
            .iter()
            .flat_map(|bucket| bucket.iter().map(|(_, pos)| pos))
    }

    #[must_use]
    pub fn into_vec(self) -> Vec<BlockPos> {
        let mut out = Vec::with_capacity(self.len);
        for bucket in self.buckets {
            for (_, pos) in bucket {
                out.push(pos);
            }
        }
        out
    }

    fn resize(&mut self) {
        let old_cap = self.buckets.len();
        let new_cap = old_cap << 1;
        self.threshold <<= 1;
        let mut new_buckets = vec![Vec::new(); new_cap];
        for (j, bucket) in self.buckets.drain(..).enumerate() {
            for (hash, pos) in bucket {
                if (hash as usize & old_cap) == 0 {
                    new_buckets[j].push((hash, pos));
                } else {
                    new_buckets[j + old_cap].push((hash, pos));
                }
            }
        }
        self.buckets = new_buckets;
    }
}

impl FromIterator<BlockPos> for JavaBlockPosHashSet {
    fn from_iter<T: IntoIterator<Item = BlockPos>>(iter: T) -> Self {
        let mut set = Self::new();
        for pos in iter {
            set.insert(pos);
        }
        set
    }
}

type BucketEntryMap<'a> =
    std::iter::Map<std::slice::Iter<'a, (i32, BlockPos)>, fn(&(i32, BlockPos)) -> &BlockPos>;

impl<'a> IntoIterator for &'a JavaBlockPosHashSet {
    type Item = &'a BlockPos;
    type IntoIter = std::iter::FlatMap<
        std::slice::Iter<'a, Vec<(i32, BlockPos)>>,
        BucketEntryMap<'a>,
        fn(&'a Vec<(i32, BlockPos)>) -> BucketEntryMap<'a>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        const fn map_entry((_, pos): &(i32, BlockPos)) -> &BlockPos {
            pos
        }
        #[expect(
            clippy::ptr_arg,
            reason = "slice::Iter<Vec<...>> passes &Vec to fn pointer"
        )]
        fn map_bucket(bucket: &Vec<(i32, BlockPos)>) -> BucketEntryMap<'_> {
            bucket.iter().map(map_entry)
        }
        self.buckets.iter().flat_map(map_bucket)
    }
}

impl IntoIterator for JavaBlockPosHashSet {
    type Item = BlockPos;
    type IntoIter = std::vec::IntoIter<BlockPos>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}
