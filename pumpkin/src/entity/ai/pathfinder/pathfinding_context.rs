use pumpkin_data::{
    Block,
    fluid::Fluid,
    tag::{self, Taggable},
};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::{
    entity::ai::pathfinder::{
        node::{Coordinate, PathType},
        path_type_cache::PathTypeCache,
    },
    world::World,
};

use std::{collections::HashMap, sync::Arc};

pub struct PathfindingContext {
    path_type_cache: Option<PathTypeCache>,
    mob_position: Vector3<i32>,
    world: Arc<World>,
    collision_cache: HashMap<Vector3<i32>, bool>,
}

impl PathfindingContext {
    pub fn new(mob_position: Vector3<i32>, world: Arc<World>) -> Self {
        Self {
            path_type_cache: Some(PathTypeCache::new()),
            mob_position,
            world,
            collision_cache: HashMap::new(),
        }
    }

    pub fn with_cache(mob_position: Vector3<i32>, world: Arc<World>, cache: PathTypeCache) -> Self {
        Self {
            path_type_cache: Some(cache),
            mob_position,
            world,
            collision_cache: HashMap::new(),
        }
    }

    #[must_use]
    pub fn mob_position(&self) -> Vector3<i32> {
        self.mob_position
    }

    pub async fn get_path_type_from_state(&mut self, pos: Vector3<i32>) -> PathType {
        if let Some(ref cache) = self.path_type_cache
            && let Some(pt) = cache.get(pos)
        {
            return pt;
        }

        let pt = self.compute_path_type_from_state(pos).await;

        if let Some(ref mut cache) = self.path_type_cache {
            cache.insert(pos, pt);
        }

        pt
    }

    pub async fn compute_path_type_from_state(&self, pos: Vector3<i32>) -> PathType {
        let pos = pos.as_blockpos();

        let block = self.world.get_block(&pos).await;
        let state_id = self.world.get_block_state_id(&pos).await;

        if block.id == Block::AIR.id {
            // Check if there's solid ground directly below for walking
            let below_pos = BlockPos::new(pos.0.x, pos.0.y - 1, pos.0.z);
            let below_block = self.world.get_block(&below_pos).await;

            if Self::is_solid_walkable_surface(below_block) {
                PathType::Walkable
            } else {
                PathType::Open
            }
        }
        // Handle special walkable blocks (partial blocks that can be walked on)
        else if Self::is_partial_walkable_block(block) {
            PathType::Walkable
        }
        // Handle dangerous/special blocks
        else if block.has_tag(&tag::Block::MINECRAFT_TRAPDOORS)
            || block.id == Block::LILY_PAD.id
            || block.id == Block::BIG_DRIPLEAF.id
        {
            PathType::Trapdoor
        } else if block.id == Block::POWDER_SNOW.id {
            PathType::PowderSnow
        } else if block.id == Block::CACTUS.id || block.id == Block::SWEET_BERRY_BUSH.id {
            PathType::DamageOther
        } else if block.id == Block::HONEY_BLOCK.id {
            PathType::StickyHoney
        } else if block.id == Block::COCOA.id {
            PathType::Cocoa
        } else if block.id == Block::WITHER_ROSE.id || block.id == Block::POINTED_DRIPSTONE.id {
            PathType::DamageCautious
        } else {
            let fluid = Fluid::from_state_id(state_id);
            if fluid.is_some_and(|f| f.has_tag(&tag::Fluid::MINECRAFT_LAVA)) {
                PathType::Lava
            } else if block.id == Block::FIRE.id
                || block.id == Block::SOUL_FIRE.id
                || block.id == Block::MAGMA_BLOCK.id
            {
                PathType::DamageFire
            } else if block.has_tag(&tag::Block::MINECRAFT_DOORS) {
                // TODO: Properly implement door states
                PathType::DoorIronClosed
            } else if block.has_tag(&tag::Block::MINECRAFT_RAILS) {
                PathType::Rail
            } else if block.has_tag(&tag::Block::MINECRAFT_LEAVES) {
                PathType::Leaves
            } else if block.has_tag(&tag::Block::MINECRAFT_FENCES)
                || block.has_tag(&tag::Block::MINECRAFT_WALLS)
            {
                PathType::Fence
            } else if fluid.is_some_and(|f| f.has_tag(&tag::Fluid::MINECRAFT_WATER)) {
                PathType::Water
            } else {
                // Most solid blocks should be treated as blocked for pathfinding
                // since mobs can't walk through them
                PathType::Blocked
            }
        }
    }

    fn is_solid_walkable_surface(block: &Block) -> bool {
        match block.id {
            id if id == Block::AIR.id => false,
            id if id == Block::WATER.id => false,
            id if id == Block::LAVA.id => false,
            id if id == Block::VOID_AIR.id => false,
            id if id == Block::CAVE_AIR.id => false,

            id if id == Block::CACTUS.id => true,
            id if id == Block::MAGMA_BLOCK.id => true,
            id if id == Block::WITHER_ROSE.id => true,

            _ => {
                !block.has_tag(&tag::Block::MINECRAFT_FLOWERS)
                    && !block.has_tag(&tag::Block::MINECRAFT_SAPLINGS)
                    && block.id != Block::TALL_GRASS.id
                    && block.id != Block::FERN.id
                    && block.id != Block::DEAD_BUSH.id
            }
        }
    }

    fn is_partial_walkable_block(block: &Block) -> bool {
        block.has_tag(&tag::Block::MINECRAFT_STAIRS)
            || block.has_tag(&tag::Block::MINECRAFT_SLABS)
            || block.id == Block::FARMLAND.id
            || block.id == Block::SOUL_SAND.id
            || block.id == Block::SNOW.id
    }

    pub fn has_collisions(&mut self, _pos: Vector3<i32>) -> bool {
        /*
        * TODO: Find how this is implemented on vanilla (might need to modify extractor)
        *
        if let Some(&cached) = self.collision_cache.get(&pos) {
            return cached;
        }

        let block = self.world.get_block(&pos).await;
        let has_collision = ?;

        self.collision_cache.insert(pos, has_collision);
        has_collision
        */
        false
    }

    pub fn clear_caches(&mut self) {
        if let Some(ref mut cache) = self.path_type_cache {
            cache.clear();
        }
        self.collision_cache.clear();
    }
}
