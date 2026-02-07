use pumpkin_data::{
    Block,
    fluid::Fluid,
    tag::{self, Taggable},
};
use pumpkin_util::math::vector3::Vector3;

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
    pub const fn mob_position(&self) -> Vector3<i32> {
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

    /// Classifies a block position into a `PathType` for pathfinding.
    pub async fn compute_path_type_from_state(&self, pos: Vector3<i32>) -> PathType {
        let block_pos = pos.as_blockpos();

        let block = self.world.get_block(&block_pos).await;
        let state_id = self.world.get_block_state_id(&block_pos).await;
        let state = self.world.get_block_state(&block_pos).await;

        if block.id == Block::AIR.id
            || block.id == Block::VOID_AIR.id
            || block.id == Block::CAVE_AIR.id
        {
            return PathType::Open;
        }

        if block.has_tag(&tag::Block::MINECRAFT_TRAPDOORS)
            || block.id == Block::LILY_PAD.id
            || block.id == Block::BIG_DRIPLEAF.id
        {
            return PathType::Trapdoor;
        }

        if block.id == Block::POWDER_SNOW.id {
            return PathType::PowderSnow;
        }

        if block.id == Block::CACTUS.id || block.id == Block::SWEET_BERRY_BUSH.id {
            return PathType::DamageOther;
        }

        if block.id == Block::HONEY_BLOCK.id {
            return PathType::StickyHoney;
        }

        if block.id == Block::COCOA.id {
            return PathType::Cocoa;
        }

        if block.id == Block::WITHER_ROSE.id || block.id == Block::POINTED_DRIPSTONE.id {
            return PathType::DamageCautious;
        }

        let fluid = Fluid::from_state_id(state_id);
        if fluid.is_some_and(|f| f.has_tag(&tag::Fluid::MINECRAFT_LAVA)) {
            return PathType::Lava;
        }

        if block.id == Block::FIRE.id
            || block.id == Block::SOUL_FIRE.id
            || block.id == Block::MAGMA_BLOCK.id
            || block.id == Block::CAMPFIRE.id
            || block.id == Block::SOUL_CAMPFIRE.id
            || block.id == Block::LAVA_CAULDRON.id
        {
            return PathType::DamageFire;
        }

        if block.has_tag(&tag::Block::MINECRAFT_DOORS) {
            if state.collision_shapes.is_empty() {
                return PathType::DoorOpen;
            }

            return if block.id == Block::IRON_DOOR.id {
                PathType::DoorIronClosed
            } else {
                PathType::DoorWoodClosed
            };
        }

        if block.has_tag(&tag::Block::MINECRAFT_RAILS) {
            return PathType::Rail;
        }

        if block.has_tag(&tag::Block::MINECRAFT_LEAVES) {
            return PathType::Leaves;
        }

        if block.has_tag(&tag::Block::MINECRAFT_FENCES)
            || block.has_tag(&tag::Block::MINECRAFT_WALLS)
        {
            return PathType::Fence;
        }

        if block.has_tag(&tag::Block::MINECRAFT_FENCE_GATES) && !state.collision_shapes.is_empty() {
            return PathType::Fence;
        }

        if state.is_full_cube() {
            return PathType::Blocked;
        }

        if fluid.is_some_and(|f| f.has_tag(&tag::Fluid::MINECRAFT_WATER)) {
            return PathType::Water;
        }

        PathType::Open
    }

    /// Wraps the raw block type with below-check and neighbor danger scanning for OPEN nodes.
    pub async fn get_land_node_type(&mut self, pos: Vector3<i32>) -> PathType {
        let raw_type = self.get_path_type_from_state(pos).await;

        if raw_type == PathType::Open {
            let below_type = self
                .get_path_type_from_state(Vector3::new(pos.x, pos.y - 1, pos.z))
                .await;
            return match below_type {
                PathType::Open | PathType::Water | PathType::Lava | PathType::Walkable => {
                    PathType::Open
                }
                PathType::DamageFire => PathType::DamageFire,
                PathType::DamageOther => PathType::DamageOther,
                PathType::StickyHoney => PathType::StickyHoney,
                PathType::PowderSnow => PathType::DangerPowderSnow,
                PathType::DamageCautious => PathType::DamageCautious,
                PathType::Trapdoor => PathType::DangerTrapdoor,
                _ => {
                    self.get_node_type_from_neighbors(pos, PathType::Walkable)
                        .await
                }
            };
        }

        raw_type
    }

    /// Scans a 3x3x3 neighborhood for danger blocks and returns the appropriate danger type.
    pub async fn get_node_type_from_neighbors(
        &mut self,
        pos: Vector3<i32>,
        fallback: PathType,
    ) -> PathType {
        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                for dz in -1..=1i32 {
                    if dx == 0 && dz == 0 {
                        continue;
                    }

                    let neighbor_type = self
                        .get_path_type_from_state(Vector3::new(pos.x + dx, pos.y + dy, pos.z + dz))
                        .await;

                    if neighbor_type == PathType::DamageOther {
                        return PathType::DangerOther;
                    }
                    if neighbor_type == PathType::DamageFire || neighbor_type == PathType::Lava {
                        return PathType::DangerFire;
                    }
                    if neighbor_type == PathType::Water {
                        return PathType::WaterBorder;
                    }
                    if neighbor_type == PathType::DamageCautious {
                        return PathType::DamageCautious;
                    }
                }
            }
        }

        fallback
    }

    pub async fn has_collisions(&mut self, pos: Vector3<i32>) -> bool {
        if let Some(&cached) = self.collision_cache.get(&pos) {
            return cached;
        }

        let block_pos = pos.as_blockpos();
        let state = self.world.get_block_state(&block_pos).await;
        let has_collision = state.is_full_cube();

        self.collision_cache.insert(pos, has_collision);
        has_collision
    }

    pub fn clear_caches(&mut self) {
        if let Some(ref mut cache) = self.path_type_cache {
            cache.clear();
        }
        self.collision_cache.clear();
    }
}
