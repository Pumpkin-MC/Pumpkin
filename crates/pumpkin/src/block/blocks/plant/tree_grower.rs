use std::sync::Arc;

use pumpkin_data::{
    Block, BlockStateId, configured_feature::ConfiguredFeature as FeatureKey, tag, tag::Taggable,
};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};
use pumpkin_world::{
    generation::feature::configured_features::{CONFIGURED_FEATURES, ConfiguredFeature},
    world::BlockFlags,
};

use crate::world::{World, generation_cache::WorldGenerationCache};

struct TwoByTwoSaplingPos {
    offset_x: i32,
    offset_z: i32,
    saplings: Vec<(BlockPos, BlockStateId)>,
}

pub struct TreeGrower {
    trees: &'static [(FeatureKey, i32)],
    mega_trees: &'static [(FeatureKey, i32)],
    flower_trees: &'static [(FeatureKey, i32)],
    shortest: Option<FeatureKey>,
}

impl TreeGrower {
    pub const OAK: Self = Self {
        trees: &[(FeatureKey::Oak, 9), (FeatureKey::FancyOak, 1)],
        mega_trees: &[],
        flower_trees: &[
            (FeatureKey::OakBees005, 9),
            (FeatureKey::FancyOakBees005, 1),
        ],
        shortest: Some(FeatureKey::Oak),
    };
    pub const SPRUCE: Self = Self {
        trees: &[(FeatureKey::Spruce, 1)],
        mega_trees: &[(FeatureKey::MegaSpruce, 1), (FeatureKey::MegaPine, 1)],
        flower_trees: &[],
        shortest: Some(FeatureKey::Spruce),
    };
    pub const MANGROVE: Self = Self {
        trees: &[(FeatureKey::Mangrove, 15), (FeatureKey::TallMangrove, 85)],
        mega_trees: &[],
        flower_trees: &[],
        shortest: Some(FeatureKey::Mangrove),
    };
    pub const AZALEA: Self = Self {
        trees: &[(FeatureKey::AzaleaTree, 1)],
        mega_trees: &[],
        flower_trees: &[],
        shortest: Some(FeatureKey::AzaleaTree),
    };
    pub const BIRCH: Self = Self {
        trees: &[(FeatureKey::Birch, 1)],
        mega_trees: &[],
        flower_trees: &[(FeatureKey::BirchBees005, 1)],
        shortest: Some(FeatureKey::Birch),
    };
    pub const JUNGLE: Self = Self {
        trees: &[(FeatureKey::JungleTreeNoVine, 1)],
        mega_trees: &[(FeatureKey::MegaJungleTree, 1)],
        flower_trees: &[],
        shortest: Some(FeatureKey::JungleTreeNoVine),
    };
    pub const ACACIA: Self = Self {
        trees: &[(FeatureKey::Acacia, 1)],
        mega_trees: &[],
        flower_trees: &[],
        shortest: Some(FeatureKey::Acacia),
    };
    pub const CHERRY: Self = Self {
        trees: &[(FeatureKey::Cherry, 1)],
        mega_trees: &[],
        flower_trees: &[(FeatureKey::CherryBees005, 1)],
        shortest: Some(FeatureKey::Cherry),
    };
    pub const DARK_OAK: Self = Self {
        trees: &[],
        mega_trees: &[(FeatureKey::DarkOak, 1)],
        flower_trees: &[],
        shortest: None,
    };
    pub const PALE_OAK: Self = Self {
        trees: &[],
        mega_trees: &[(FeatureKey::PaleOakBonemeal, 1)],
        flower_trees: &[],
        shortest: None,
    };

    #[must_use]
    pub fn for_block(block: &Block) -> Option<&'static Self> {
        match block.name {
            "oak_sapling" => Some(&Self::OAK),
            "spruce_sapling" => Some(&Self::SPRUCE),
            "birch_sapling" => Some(&Self::BIRCH),
            "jungle_sapling" => Some(&Self::JUNGLE),
            "acacia_sapling" => Some(&Self::ACACIA),
            "dark_oak_sapling" => Some(&Self::DARK_OAK),
            "pale_oak_sapling" => Some(&Self::PALE_OAK),
            "cherry_sapling" => Some(&Self::CHERRY),
            "azalea" | "flowering_azalea" => Some(&Self::AZALEA),
            "mangrove_propagule" => Some(&Self::MANGROVE),
            _ => None,
        }
    }

    fn pick(list: &[(FeatureKey, i32)], random: &mut RandomGenerator) -> Option<FeatureKey> {
        let total: i32 = list.iter().map(|(_, weight)| *weight).sum();
        if total <= 0 {
            return None;
        }
        let mut roll = random.next_bounded_i32(total);
        for (key, weight) in list {
            roll -= weight;
            if roll < 0 {
                return Some(*key);
            }
        }
        None
    }

    fn has_flowers(world: &World, pos: &BlockPos) -> bool {
        for x in -2..=2 {
            for y in -1..=1 {
                for z in -2..=2 {
                    let block = world.get_block(&pos.offset(Vector3::new(x, y, z)));
                    if block.has_tag(&tag::Block::MINECRAFT_FLOWERS) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn surrounding(
        world: &World,
        pos: &BlockPos,
        dx: i32,
        dz: i32,
    ) -> Vec<(BlockPos, BlockStateId)> {
        [(dx, dz), (dx + 1, dz), (dx, dz + 1), (dx + 1, dz + 1)]
            .into_iter()
            .map(|(x, z)| {
                let position = pos.offset(Vector3::new(x, 0, z));
                (position, world.get_block_state_id(&position))
            })
            .collect()
    }

    fn find_two_by_two(world: &World, block: &Block, pos: &BlockPos) -> Option<TwoByTwoSaplingPos> {
        for dx in [0, -1] {
            for dz in [0, -1] {
                let saplings = Self::surrounding(world, pos, dx, dz);
                if saplings
                    .iter()
                    .all(|(_, state_id)| Block::from_state_id(*state_id) == block)
                {
                    return Some(TwoByTwoSaplingPos {
                        offset_x: dx,
                        offset_z: dz,
                        saplings,
                    });
                }
            }
        }
        None
    }

    fn remove_saplings(world: &Arc<World>, saplings: &[(BlockPos, BlockStateId)]) {
        for (pos, _) in saplings {
            world.set_block_state(
                pos,
                Block::AIR.default_state.id,
                BlockFlags::NOTIFY_LISTENERS,
            );
        }
    }

    fn reset_saplings(world: &Arc<World>, saplings: &[(BlockPos, BlockStateId)]) {
        for (pos, state_id) in saplings {
            world.set_block_state(pos, *state_id, BlockFlags::NOTIFY_LISTENERS);
        }
    }

    fn place(
        world: &Arc<World>,
        key: FeatureKey,
        pos: BlockPos,
        random: &mut RandomGenerator,
    ) -> bool {
        let Some(ConfiguredFeature::Tree(tree)) = CONFIGURED_FEATURES.get(&key) else {
            return false;
        };
        let portal = world.level.world_portal.load_full();
        let Some(portal) = portal.as_ref() else {
            return false;
        };
        let mut cache = WorldGenerationCache::new(world.clone(), &pos);
        if !tree.generate(&**portal, &mut cache, random, pos) {
            return false;
        }
        cache.apply();
        true
    }

    #[must_use]
    pub fn min_height(&self) -> i32 {
        self.shortest
            .and_then(|key| match CONFIGURED_FEATURES.get(&key) {
                Some(ConfiguredFeature::Tree(tree)) => {
                    Some(i32::from(tree.trunk_placer.base_height))
                }
                _ => None,
            })
            .unwrap_or(0)
    }

    #[must_use]
    pub fn can_grow(
        &self,
        world: &World,
        pos: &BlockPos,
        block: &Block,
        random: &mut RandomGenerator,
    ) -> bool {
        let has_flowers = Self::has_flowers(world, pos);
        let list = if has_flowers && !self.flower_trees.is_empty() {
            self.flower_trees
        } else {
            self.trees
        };
        let single = Self::pick(list, random);
        let mega = Self::pick(self.mega_trees, random);
        if single.is_none() && mega.is_some() {
            Self::find_two_by_two(world, block, pos).is_some()
        } else {
            true
        }
    }

    pub fn grow_tree(
        &self,
        world: &Arc<World>,
        pos: &BlockPos,
        block: &Block,
        state_id: BlockStateId,
        random: &mut RandomGenerator,
    ) -> bool {
        if let Some(mega) = Self::pick(self.mega_trees, random)
            && let Some(two_by_two) = Self::find_two_by_two(world, block, pos)
        {
            Self::remove_saplings(world, &two_by_two.saplings);
            let origin = pos.offset(Vector3::new(two_by_two.offset_x, 0, two_by_two.offset_z));
            if Self::place(world, mega, origin, random) {
                return true;
            }
            Self::reset_saplings(world, &two_by_two.saplings);
            return false;
        }

        let has_flowers = Self::has_flowers(world, pos);
        let list = if has_flowers && !self.flower_trees.is_empty() {
            self.flower_trees
        } else {
            self.trees
        };
        let Some(key) = Self::pick(list, random) else {
            return false;
        };
        let sapling = [(*pos, state_id)];
        Self::remove_saplings(world, &sapling);
        if Self::place(world, key, *pos, random) {
            return true;
        }
        Self::reset_saplings(world, &sapling);
        false
    }
}
