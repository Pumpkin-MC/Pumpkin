use pumpkin_data::{
    Block, BlockState, BlockStateId,
    block_properties::{
        BlockProperties, OakDoorLikeProperties, OakFenceGateLikeProperties, SnowLikeProperties,
    },
    fluid::Fluid,
    tag::{self, Taggable},
};
use pumpkin_util::math::{boundingbox::BoundingBox, vector3::Vector3};

use crate::{
    entity::ai::pathfinder::{
        node::{Coordinate, PathType},
        path_type_cache::PathTypeCache,
    },
    world::World,
};

use rustc_hash::FxHashMap;
use std::sync::Arc;

pub struct PathfindingContext {
    path_type_cache: Option<PathTypeCache>,
    mob_position: Vector3<i32>,
    world: Arc<World>,
    /// Vanilla `WalkNodeEvaluator.collisionCache` is an `Object2BooleanMap<AABB>`
    /// (`WalkNodeEvaluator.java:53`); we key on the AABB corner coordinate bits.
    collision_cache: FxHashMap<[u64; 6], bool>,
}

impl PathfindingContext {
    pub fn new(mob_position: Vector3<i32>, world: Arc<World>) -> Self {
        Self {
            path_type_cache: Some(PathTypeCache::new()),
            mob_position,
            world,
            collision_cache: FxHashMap::default(),
        }
    }

    pub fn with_cache(mob_position: Vector3<i32>, world: Arc<World>, cache: PathTypeCache) -> Self {
        Self {
            path_type_cache: Some(cache),
            mob_position,
            world,
            collision_cache: FxHashMap::default(),
        }
    }

    #[must_use]
    pub const fn mob_position(&self) -> Vector3<i32> {
        self.mob_position
    }

    pub fn get_path_type_from_state(&mut self, pos: Vector3<i32>) -> PathType {
        if let Some(ref cache) = self.path_type_cache
            && let Some(pt) = cache.get(pos)
        {
            return pt;
        }

        let pt = self.compute_path_type_from_state(pos);

        if let Some(ref mut cache) = self.path_type_cache {
            cache.insert(pos, pt);
        }

        pt
    }

    /// Vanilla `WalkNodeEvaluator.getFloorLevel` for land navigation.
    #[must_use]
    pub fn get_floor_level(&self, pos: Vector3<i32>) -> f64 {
        let below = pos.add_raw(0, -1, 0).as_blockpos();
        let state = self.world.get_block_state(&below);
        let collision_height = state
            .get_block_collision_shapes()
            .map(|shape| shape.max.y)
            .fold(0.0, f64::max);

        f64::from(below.0.y) + collision_height
    }

    /// Classifies a block position into a `PathType` for pathfinding.
    #[must_use]
    pub fn compute_path_type_from_state(&self, pos: Vector3<i32>) -> PathType {
        let block_pos = pos.as_blockpos();

        // Single async chunk lookup, then derive block & state from static arrays
        let state_id = self.world.get_block_state_id(&block_pos);
        let block = Block::from_state_id(state_id);
        let state = BlockState::from_id(state_id);

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
            // Vanilla WalkNodeEvaluator.java:473-479: read the door's OPEN
            // blockstate property (collision shapes are never empty for doors —
            // an open door still has its side panel collision).
            if OakDoorLikeProperties::from_state_id(state.id, block).open {
                return PathType::DoorOpen;
            }

            // Vanilla WalkNodeEvaluator.java:478: `door.type().canOpenByHand()`.
            // Iron is the only vanilla `BlockSetType` with `canOpenByHand=false`;
            // wooden and copper doors are all hand-openable.
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

        // Vanilla WalkNodeEvaluator.java:486: `block instanceof FenceGateBlock
        // && !blockState.getValue(FenceGateBlock.OPEN)`.
        if block.has_tag(&tag::Block::MINECRAFT_FENCE_GATES)
            && !OakFenceGateLikeProperties::from_state_id(state.id, block).open
        {
            return PathType::Fence;
        }

        // Vanilla WalkNodeEvaluator.java:489-491:
        // `!blockState.isPathfindable(PathComputationType.LAND)` -> BLOCKED.
        if !Self::is_land_pathfindable(block, state) {
            return PathType::Blocked;
        }

        if fluid.is_some_and(|f| f.has_tag(&tag::Fluid::MINECRAFT_WATER)) {
            return PathType::Water;
        }

        PathType::Open
    }

    /// Vanilla `BlockBehaviour.isPathfindable(PathComputationType.LAND)`.
    ///
    /// The vanilla default (`BlockBehaviour.java:149-156`) is
    /// `!state.isCollisionShapeFullBlock(...)`, but essentially every
    /// partial-collision block a mob cannot stand inside overrides it to
    /// `false`: stairs (`StairBlock.java:230-232`), slabs
    /// (`SlabBlock.java:145-152`), chests (`ChestBlock.java:382`), panes,
    /// bars and glass walls (`CrossCollisionBlock.java:89-91`), walls
    /// (`WallBlock.java:104-106`), farmland (`FarmlandBlock.java:143-145`),
    /// dirt paths (`DirtPathBlock.java:79-81`), soul sand
    /// (`SoulSandBlock.java:49-51`), mud (`MudBlock.java:49-51`), beds,
    /// anvils, hoppers, cauldrons, lecterns, etc. Doors, trapdoors and fence
    /// gates also override it, but those are classified earlier in this chain
    /// exactly like vanilla `WalkNodeEvaluator.getPathTypeFromState`.
    ///
    /// Pumpkin has no per-block `isPathfindable` hook, so the closest faithful
    /// equivalent is: a block with any collision shape blocks land pathing.
    /// Known deviation: a handful of no-override partial-collision blocks
    /// (e.g. sea pickles, amethyst clusters, turtle eggs) keep the vanilla
    /// default of pathfindable but are treated as BLOCKED here — erring on
    /// the conservative side. Snow layers are special-cased below.
    #[must_use]
    fn is_land_pathfindable(block: &Block, state: &BlockState) -> bool {
        // Vanilla SnowLayerBlock.java:51-56: LAND is pathfindable while
        // `layers < 5`, even though layers 2..=4 already have collision.
        if block.id == Block::SNOW.id {
            return SnowLikeProperties::from_state_id(state.id, block).layers < 5;
        }

        state.collision_shapes.is_empty()
    }

    /// Vanilla `Level.getMinY`, the lower world bound used by the start-node
    /// scan and the water/fall column walks.
    #[must_use]
    pub fn min_y(&self) -> i32 {
        self.world.min_y
    }

    /// Water-column test used by vanilla `WalkNodeEvaluator.getStart`
    /// (`WalkNodeEvaluator.java:81`): `state.is(Blocks.WATER) ||
    /// state.getFluidState() == Fluids.WATER.getSource(false)`. The water
    /// block covers both source and flowing states; waterlogged states always
    /// hold source water.
    #[must_use]
    pub fn is_water_at(&self, pos: Vector3<i32>) -> bool {
        let state_id = self.world.get_block_state_id(&pos.as_blockpos());
        let block = Block::from_state_id(state_id);
        block.id == Block::WATER.id || BlockState::from_id(state_id).is_waterlogged()
    }

    /// Vanilla `BlockState.getFluidState().is(FluidTags.WATER)`, the per-cell
    /// water test of `SwimNodeEvaluator.getPathTypeOfMob`
    /// (`SwimNodeEvaluator.java:121,126`). Beyond the water block itself and
    /// waterlogged states, vanilla `getFluidState` overrides return a water
    /// source for kelp (`KelpBlock.java:87-89`), kelp plants
    /// (`KelpPlantBlock.java:47-49`), seagrass (`SeagrassBlock.java:94-96`),
    /// tall seagrass (`TallSeagrassBlock.java:87-89`) and bubble columns
    /// (`BubbleColumnBlock.java:85-87`).
    #[must_use]
    pub fn is_water_fluid_at(&self, pos: Vector3<i32>) -> bool {
        let state_id = self.world.get_block_state_id(&pos.as_blockpos());
        Self::is_water_fluid_state(state_id)
    }

    fn is_water_fluid_state(state_id: BlockStateId) -> bool {
        let block = Block::from_state_id(state_id);
        block.id == Block::WATER.id
            || block.id == Block::KELP.id
            || block.id == Block::KELP_PLANT.id
            || block.id == Block::SEAGRASS.id
            || block.id == Block::TALL_SEAGRASS.id
            || block.id == Block::BUBBLE_COLUMN.id
            || BlockState::from_id(state_id).is_waterlogged()
    }

    /// Vanilla `BlockState.getFluidState().isEmpty()`, used by the swim
    /// evaluator's breach test (`SwimNodeEvaluator.java:123`) and its
    /// out-of-fluid node penalty (`SwimNodeEvaluator.java:98`): no water and no
    /// lava fluid occupies the cell.
    #[must_use]
    pub fn is_fluid_empty_at(&self, pos: Vector3<i32>) -> bool {
        let state_id = self.world.get_block_state_id(&pos.as_blockpos());
        if Self::is_water_fluid_state(state_id) {
            return false;
        }
        !Fluid::from_state_id(state_id).is_some_and(|f| f.has_tag(&tag::Fluid::MINECRAFT_LAVA))
    }

    /// Vanilla `BlockState.isAir()` (`SwimNodeEvaluator.java:123`).
    #[must_use]
    pub fn is_air_at(&self, pos: Vector3<i32>) -> bool {
        let state_id = self.world.get_block_state_id(&pos.as_blockpos());
        BlockState::from_id(state_id).is_air()
    }

    /// Vanilla `BlockState.isPathfindable(PathComputationType.WATER)`. The
    /// default (`BlockBehaviour.java:157`) is
    /// `state.getFluidState().is(FluidTags.WATER)`; every override — doors
    /// (`DoorBlock.java:125-130`), fences (`FenceBlock.java:63-65`), walls,
    /// panes, sea pickles (`SeaPickleBlock.java:170-172`), stairs, slabs,
    /// chests, cauldrons, … — returns `false` for WATER, and all of those
    /// carry a collision shape. Closest faithful equivalent without a
    /// per-block hook: water fluid present and no collision shape.
    #[must_use]
    pub fn is_water_pathfindable(&self, pos: Vector3<i32>) -> bool {
        let state_id = self.world.get_block_state_id(&pos.as_blockpos());
        Self::is_water_fluid_state(state_id)
            && BlockState::from_id(state_id).collision_shapes.is_empty()
    }

    /// Vanilla `Level.getSeaLevel`, read by the shallow-swimming penalty of
    /// `AmphibiousNodeEvaluator.getNeighbors` (`AmphibiousNodeEvaluator.java:78`).
    #[must_use]
    pub fn sea_level(&self) -> i32 {
        self.world.sea_level
    }

    /// Predicate of the airborne start-node downward scan, vanilla
    /// `WalkNodeEvaluator.getStart` (`WalkNodeEvaluator.java:93`):
    /// `state.isAir() || state.isPathfindable(PathComputationType.LAND)`.
    #[must_use]
    pub fn is_air_or_land_pathfindable(&self, pos: Vector3<i32>) -> bool {
        let state_id = self.world.get_block_state_id(&pos.as_blockpos());
        let block = Block::from_state_id(state_id);
        let state = BlockState::from_id(state_id);
        state.is_air() || Self::is_land_pathfindable(block, state)
    }

    /// Wraps the raw block type with below-check and neighbor danger scanning for OPEN nodes.
    pub fn get_land_node_type(&mut self, pos: Vector3<i32>) -> PathType {
        let raw_type = self.get_path_type_from_state(pos);

        if raw_type == PathType::Open {
            let below_type = self.get_path_type_from_state(Vector3::new(pos.x, pos.y - 1, pos.z));
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
                _ => self.get_node_type_from_neighbors(pos, PathType::Walkable),
            };
        }

        raw_type
    }

    /// Scans a 3x3x3 neighborhood for danger blocks and returns the appropriate danger type.
    pub fn get_node_type_from_neighbors(
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

                    let neighbor_type = self.get_path_type_from_state(Vector3::new(
                        pos.x + dx,
                        pos.y + dy,
                        pos.z + dz,
                    ));

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

    /// Vanilla `WalkNodeEvaluator.hasCollisions` (`WalkNodeEvaluator.java:314-316`):
    /// `!this.currentContext.level().noCollision(this.mob, aabb)`, cached per AABB.
    ///
    /// Gap: vanilla `noCollision` also honors the world border and hard entity
    /// colliders (boats, shulkers); Pumpkin's `is_space_empty` checks block
    /// collision shapes only.
    pub fn has_collisions(&mut self, aabb: &BoundingBox) -> bool {
        let key = [
            aabb.min.x.to_bits(),
            aabb.min.y.to_bits(),
            aabb.min.z.to_bits(),
            aabb.max.x.to_bits(),
            aabb.max.y.to_bits(),
            aabb.max.z.to_bits(),
        ];
        if let Some(&cached) = self.collision_cache.get(&key) {
            return cached;
        }

        let has_collision = !self.world.is_space_empty(*aabb);

        self.collision_cache.insert(key, has_collision);
        has_collision
    }

    pub fn clear_caches(&mut self) {
        if let Some(ref mut cache) = self.path_type_cache {
            cache.clear();
        }
        self.collision_cache.clear();
    }
}
