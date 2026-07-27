use std::sync::Arc;

use super::border::BorderSnapshot;
use super::chunk_view::ChunkView;
use super::poi;
use super::rectangle::{RectAxis, get_largest_rectangle_around};
use super::spiral::SpiralAround;
use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{BlockProperties, HorizontalAxis, NetherPortalLikeProperties},
    dimension::Dimension,
    tag,
    tag::Taggable,
};
use pumpkin_util::math::{boundingbox::EntityDimensions, position::BlockPos, vector3::Vector3};
use pumpkin_world::{chunk::ChunkHeightmapType, world::BlockFlags};

use crate::world::World;

/// `PortalForcer.NETHER_PORTAL_RADIUS` (`PortalForcer.java:26`).
const SEARCH_RADIUS_NETHER: i32 = 16;
/// `PortalForcer.OVERWORLD_PORTAL_RADIUS` (`PortalForcer.java:27`).
const SEARCH_RADIUS_OVERWORLD: i32 = 128;
/// Both limits passed to `BlockUtil.getLargestRectangleAround` when a portal is
/// measured (`NetherPortalBlock.java:149`, `NetherPortalBlock.java:170`).
const PORTAL_RECTANGLE_LIMIT: i32 = 21;
/// `PortalForcer.createPortal` spirals `BlockPos.spiralAround(origin, 16, ..)`
/// (`PortalForcer.java:61`).
const CREATE_PORTAL_SPIRAL_RADIUS: i32 = 16;

#[derive(Debug, Clone)]
pub struct PortalSearchResult {
    pub lower_corner: BlockPos,
    pub axis: HorizontalAxis,
    pub width: u32,
    pub height: u32,
}

impl PortalSearchResult {
    #[must_use]
    pub fn get_teleport_position(&self) -> Vector3<f64> {
        let x = f64::from(self.lower_corner.0.x);
        let y = f64::from(self.lower_corner.0.y);
        let z = f64::from(self.lower_corner.0.z);

        match self.axis {
            HorizontalAxis::X => Vector3::new(x + f64::from(self.width) / 2.0, y, z + 0.5),
            HorizontalAxis::Z => Vector3::new(x + 0.5, y, z + f64::from(self.width) / 2.0),
        }
    }

    /// Calculates the yaw adjustment when teleporting between portals with different axes.
    /// Returns the new yaw value for the entity.
    #[must_use]
    pub fn calculate_teleport_yaw(
        &self,
        current_yaw: f32,
        source_axis: Option<HorizontalAxis>,
    ) -> f32 {
        let Some(src_axis) = source_axis else {
            return current_yaw;
        };

        if src_axis == self.axis {
            return current_yaw;
        }

        // Axis changed, rotate yaw by 90 degrees
        // X axis portal faces East/West, Z axis portal faces North/South
        match (src_axis, self.axis) {
            (HorizontalAxis::X, HorizontalAxis::Z) => current_yaw + 90.0,
            (HorizontalAxis::Z, HorizontalAxis::X) => current_yaw - 90.0,
            _ => current_yaw,
        }
    }

    #[must_use]
    pub fn entity_pos_in_portal(
        &self,
        entity_pos: Vector3<f64>,
        dimensions: &EntityDimensions,
    ) -> Vector3<f64> {
        let portal_width = f64::from(self.width) - f64::from(dimensions.width);
        let portal_height = f64::from(self.height) - f64::from(dimensions.height);
        let lower = self.lower_corner.0;

        let axis_progress = if portal_width > 0.0 {
            let axis_coord = match self.axis {
                HorizontalAxis::X => entity_pos.x,
                HorizontalAxis::Z => entity_pos.z,
            };
            let lower_axis = match self.axis {
                HorizontalAxis::X => f64::from(lower.x),
                HorizontalAxis::Z => f64::from(lower.z),
            };
            let offset = axis_coord - (lower_axis + f64::from(dimensions.width) / 2.0);
            (offset / portal_width).clamp(0.0, 1.0)
        } else {
            0.5
        };

        let y_progress = if portal_height > 0.0 {
            let offset = entity_pos.y - f64::from(lower.y);
            (offset / portal_height).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let perp_offset = match self.axis {
            HorizontalAxis::X => entity_pos.z - (f64::from(lower.z) + 0.5),
            HorizontalAxis::Z => entity_pos.x - (f64::from(lower.x) + 0.5),
        };
        // Clamp perpendicular offset to keep exit position within portal bounds
        // (prevents spawning inside solid blocks next to the portal)
        let perp_offset = perp_offset.clamp(-0.5, 0.5);

        Vector3::new(axis_progress, y_progress, perp_offset)
    }

    #[must_use]
    pub fn calculate_exit_position(
        &self,
        relative_pos: Vector3<f64>,
        dimensions: &EntityDimensions,
    ) -> Vector3<f64> {
        let portal_width = f64::from(self.width) - f64::from(dimensions.width);
        let portal_height = f64::from(self.height) - f64::from(dimensions.height);
        let lower = self.lower_corner.0;

        let axis_offset = if portal_width > 0.0 {
            relative_pos
                .x
                .mul_add(portal_width, f64::from(dimensions.width) / 2.0)
        } else {
            f64::from(self.width) / 2.0
        };

        let y_offset = if portal_height > 0.0 {
            relative_pos.y * portal_height
        } else {
            0.0
        };

        match self.axis {
            HorizontalAxis::X => Vector3::new(
                f64::from(lower.x) + axis_offset,
                f64::from(lower.y) + y_offset,
                f64::from(lower.z) + 0.5 + relative_pos.z,
            ),
            HorizontalAxis::Z => Vector3::new(
                f64::from(lower.x) + 0.5 + relative_pos.z,
                f64::from(lower.y) + y_offset,
                f64::from(lower.z) + axis_offset,
            ),
        }
    }

    pub fn find_open_position(
        &self,
        world: &Arc<World>,
        fallback: Vector3<f64>,
        dimensions: &EntityDimensions,
    ) -> Vector3<f64> {
        if dimensions.width > 4.0 || dimensions.height > 4.0 {
            return fallback;
        }

        let half_height = f64::from(dimensions.height) / 2.0;
        let check_pos = Vector3::new(fallback.x, fallback.y + half_height, fallback.z);
        let search_radius = 1.0;
        let step = 0.5;

        let mut best_pos = fallback;
        let mut best_dist = f64::MAX;

        let mut dx = -search_radius;
        while dx <= search_radius {
            let mut dz = -search_radius;
            while dz <= search_radius {
                let test_pos = Vector3::new(check_pos.x + dx, check_pos.y, check_pos.z + dz);
                if Self::is_position_clear(world, test_pos, dimensions) {
                    let dist = dx * dx + dz * dz;
                    if dist < best_dist {
                        best_dist = dist;
                        best_pos = Vector3::new(test_pos.x, fallback.y, test_pos.z);
                    }
                }
                dz += step;
            }
            dx += step;
        }

        best_pos
    }

    fn is_position_clear(
        world: &Arc<World>,
        center: Vector3<f64>,
        dimensions: &EntityDimensions,
    ) -> bool {
        let half_width = f64::from(dimensions.width) / 2.0;
        let height = f64::from(dimensions.height);

        // Calculate the bounding box in block coordinates
        let min_x = (center.x - half_width).floor() as i32;
        let max_x = (center.x + half_width).floor() as i32;
        let min_y = (center.y - height / 2.0).floor() as i32;
        let max_y = (center.y + height / 2.0).floor() as i32;
        let min_z = (center.z - half_width).floor() as i32;
        let max_z = (center.z + half_width).floor() as i32;

        // Check ALL blocks that overlap with the entity bounding box
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let block_pos = BlockPos(Vector3::new(x, y, z));
                    let state = world.get_block_state(&block_pos);
                    if state.is_solid_block() {
                        return false;
                    }
                }
            }
        }
        true
    }
}

pub struct NetherPortal {
    axis: HorizontalAxis,
    found_portal_blocks: u32,
    negative_direction: BlockDirection,
    lower_conor: BlockPos,
    width: u32,
    height: u32,
}

impl NetherPortal {
    const MIN_WIDTH: u32 = 2;
    const MAX_WIDTH: u32 = 21;
    const MAX_HEIGHT: u32 = 21;
    const MIN_HEIGHT: u32 = 3;
    const FRAME_BLOCK: Block = Block::OBSIDIAN;

    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.width >= Self::MIN_WIDTH
            && self.width <= Self::MAX_WIDTH
            && self.height >= Self::MIN_HEIGHT
            && self.height <= Self::MAX_HEIGHT
    }

    #[must_use]
    pub const fn was_already_valid(&self) -> bool {
        self.is_valid() && self.found_portal_blocks == self.width * self.height
    }

    #[must_use]
    pub const fn lower_corner(&self) -> BlockPos {
        self.lower_conor
    }

    #[must_use]
    pub const fn axis(&self) -> HorizontalAxis {
        self.axis
    }

    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    pub async fn create(&self, world: &Arc<World>) {
        let mut props = NetherPortalLikeProperties::default(&Block::NETHER_PORTAL);
        props.axis = self.axis;
        let state = props.to_state_id(&Block::NETHER_PORTAL);
        let blocks = BlockPos::iterate(
            self.lower_conor,
            self.lower_conor
                .offset_dir(BlockDirection::Up.to_offset(), self.height as i32 - 1)
                .offset_dir(self.negative_direction.to_offset(), self.width as i32 - 1),
        );

        // The POI lock must not be held across `set_block_state`: that call can
        // reach `NetherPortalBlock::on_state_replaced`, which locks `portal_poi`
        // itself and would deadlock on this same mutex.
        let mut placed = Vec::new();
        for pos in blocks {
            world
                .set_block_state(
                    &pos,
                    state,
                    BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE,
                )
                .await;
            placed.push(pos);
        }

        let mut poi_storage = world.portal_poi.lock().await;
        for pos in placed {
            poi_storage.add_portal(pos);
        }
    }

    pub fn get_new_portal(
        world: &World,
        pos: &BlockPos,
        first_axis: HorizontalAxis,
    ) -> Option<Self> {
        if let Some(portal) = Self::get_on_axis(world, pos, first_axis)
            && portal.is_valid()
            && portal.found_portal_blocks == 0
        {
            return Some(portal);
        }
        let next_axis = if first_axis == HorizontalAxis::X {
            HorizontalAxis::Z
        } else {
            HorizontalAxis::X
        };
        if let Some(portal) = Self::get_on_axis(world, pos, next_axis)
            && portal.is_valid()
            && portal.found_portal_blocks == 0
        {
            return Some(portal);
        }
        None
    }

    pub fn get_on_axis(world: &World, pos: &BlockPos, axis: HorizontalAxis) -> Option<Self> {
        let direction = if axis == HorizontalAxis::X {
            BlockDirection::West
        } else {
            BlockDirection::South
        };
        let cornor = Self::get_lower_cornor(world, direction, pos)?;
        let width = Self::get_width(world, &cornor, direction);
        if !(Self::MIN_WIDTH..=Self::MAX_WIDTH).contains(&width) {
            return None;
        }
        let mut found_portal_blocks = 0;
        let height = Self::get_height(world, &cornor, direction, width, &mut found_portal_blocks)?;
        Some(Self {
            axis,
            found_portal_blocks,
            negative_direction: direction,
            lower_conor: cornor,
            width,
            height,
        })
    }

    fn get_lower_cornor(
        world: &World,
        direction: BlockDirection,
        pos: &BlockPos,
    ) -> Option<BlockPos> {
        let limit_y = pos.0.y - Self::MAX_HEIGHT as i32;
        let mut pos = *pos;
        while pos.0.y > limit_y {
            let (block, state) = world.get_block_and_state(&pos.down());
            if !Self::valid_state_inside_portal(block, state) {
                break;
            }
            pos = pos.down();
        }
        let neg_dir = direction.opposite();
        let width = (Self::get_width(world, &pos, neg_dir) as i32) - 1;
        if width < 0 {
            return None;
        }
        Some(pos.offset_dir(neg_dir.to_offset(), width))
    }

    fn get_width(
        world: &World,
        original_lower_corner: &BlockPos,
        negative_dir: BlockDirection,
    ) -> u32 {
        let mut lower_corner;
        for i in 0..=Self::MAX_WIDTH {
            lower_corner = original_lower_corner.offset_dir(negative_dir.to_offset(), i as i32);
            let (block, block_state) = world.get_block_and_state(&lower_corner);
            if !Self::valid_state_inside_portal(block, block_state) {
                if &Self::FRAME_BLOCK != block {
                    break;
                }
                return i;
            }
            let block = world.get_block(&lower_corner.down());
            if &Self::FRAME_BLOCK != block {
                break;
            }
        }
        0
    }

    fn get_height(
        world: &World,
        lower_corner: &BlockPos,
        negative_dir: BlockDirection,
        width: u32,
        found_portal_blocks: &mut u32,
    ) -> Option<u32> {
        let height = Self::get_potential_height(
            world,
            lower_corner,
            negative_dir,
            width,
            found_portal_blocks,
        );
        if !(Self::MIN_HEIGHT..=Self::MAX_HEIGHT).contains(&height)
            || !Self::is_horizontal_frame_valid(world, lower_corner, negative_dir, width, height)
        {
            return None;
        }
        Some(height)
    }

    fn get_potential_height(
        world: &World,
        lower_corner: &BlockPos,
        negative_dir: BlockDirection,
        width: u32,
        found_portal_blocks: &mut u32,
    ) -> u32 {
        for i in 0..Self::MAX_HEIGHT as i32 {
            let mut pos = lower_corner
                .offset_dir(BlockDirection::Up.to_offset(), i)
                .offset_dir(negative_dir.to_offset(), -1);
            if world.get_block(&pos) != &Self::FRAME_BLOCK {
                return i as u32;
            }

            pos = lower_corner
                .offset_dir(BlockDirection::Up.to_offset(), i)
                .offset_dir(negative_dir.to_offset(), width as i32);
            if world.get_block(&pos) != &Self::FRAME_BLOCK {
                return i as u32;
            }

            for j in 0..width {
                pos = lower_corner
                    .offset_dir(BlockDirection::Up.to_offset(), i)
                    .offset_dir(negative_dir.to_offset(), j as i32);
                let (block, block_state) = world.get_block_and_state(&pos);
                if !Self::valid_state_inside_portal(block, block_state) {
                    return i as u32;
                }
                if block == &Block::NETHER_PORTAL {
                    *found_portal_blocks += 1;
                }
            }
        }
        21
    }

    fn is_horizontal_frame_valid(
        world: &World,
        lower_corner: &BlockPos,
        dir: BlockDirection,
        width: u32,
        height: u32,
    ) -> bool {
        let mut pos;
        for i in 0..width {
            pos = lower_corner
                .offset_dir(BlockDirection::Up.to_offset(), height as i32)
                .offset_dir(dir.to_offset(), i as i32);
            if &Self::FRAME_BLOCK != world.get_block(&pos) {
                return false;
            }
        }
        true
    }

    fn valid_state_inside_portal(block: &Block, state: &BlockState) -> bool {
        state.is_air()
            || block.has_tag(&tag::Block::MINECRAFT_FIRE)
            || block == &Block::NETHER_PORTAL
    }

    /// Vanilla `PortalForcer.findClosestPortalPosition` (`PortalForcer.java:44-49`).
    ///
    /// Queries the portal-block index for candidates in a square of `radius`
    /// around the approximate exit position, drops anything outside the world
    /// border or no longer a portal block, and picks the closest one — ties
    /// broken by lowest Y, matching vanilla's
    /// `comparingDouble(distSqr).thenComparingInt(Vec3i::getY)`.
    ///
    /// Radius is 16 when the destination is the Nether and 128 otherwise
    /// (`PortalForcer.java:46`).
    ///
    /// # Known gap versus vanilla
    ///
    /// Vanilla calls `PoiManager.ensureLoadedAndValid` first
    /// (`PortalForcer.java:47`), which force-loads any chunk in the search square
    /// whose POI section has not been validated. It can afford that because it
    /// only needs `ChunkStatus.EMPTY` and because `PoiManager` re-indexes every
    /// chunk as it loads (`PoiManager.checkConsistencyWithBlocks`).
    ///
    /// Pumpkin has no chunk-load POI hook and no partial-status chunk fetch, so
    /// there is no cheap equivalent: force-loading a 128-block square would fully
    /// generate up to 289 chunks per transit. The index is instead written
    /// whenever Pumpkin itself lights or builds a portal
    /// (`NetherPortal::create`, `NetherPortal::build_portal_frame`) and persists
    /// across restarts, which covers every portal this server creates.
    ///
    /// Not covered: portal blocks that this server never placed — an imported
    /// world, an externally edited region file, or a world predating the index.
    /// Those stay invisible to the search, so a traversal near them still builds
    /// a fresh portal. Closing it needs a POI-indexing hook on chunk load, which
    /// lives outside this module.
    pub async fn search_for_portal(
        world: &Arc<World>,
        target_pos: BlockPos,
    ) -> Option<PortalSearchResult> {
        // `toNether` in vanilla is `newLevel.dimension() == Level.NETHER`
        // (`NetherPortalBlock.java:136`), i.e. dimension identity — not a
        // property like `has_ceiling`, which a custom dimension could also set.
        let to_nether = world.dimension == Dimension::THE_NETHER;
        let search_radius = if to_nether {
            SEARCH_RADIUS_NETHER
        } else {
            SEARCH_RADIUS_OVERWORLD
        };

        let candidates = Self::query_portal_index(world, target_pos, search_radius).await;
        if candidates.is_empty() {
            return None;
        }

        // `worldBorder::isWithinBounds` filter (PortalForcer.java:48). The border
        // is snapshotted so no lock is held across the block reads below.
        let border = BorderSnapshot::capture(world).await;
        let candidates: Vec<BlockPos> = candidates
            .into_iter()
            .filter(|pos| border.contains_block(pos.0.x, pos.0.z))
            .collect();

        let mut view = ChunkView::new(world);
        let mut best: Option<(BlockPos, f64)> = None;
        let mut stale: Vec<BlockPos> = Vec::new();

        for pos in candidates {
            // `hasProperty(HORIZONTAL_AXIS)` filter (PortalForcer.java:48):
            // a stale index entry whose block is gone must not be reused.
            if view.block(&pos).await != &Block::NETHER_PORTAL {
                stale.push(pos);
                continue;
            }

            let dist = f64::from(target_pos.0.squared_distance_to(pos.0.x, pos.0.y, pos.0.z));
            // min(comparingDouble(distSqr).thenComparingInt(getY)): keep the
            // first minimum, so a later equal-distance candidate only wins on a
            // strictly lower Y.
            let is_better = best.as_ref().is_none_or(|(best_pos, best_dist)| {
                dist < *best_dist || (dist == *best_dist && pos.0.y < best_pos.0.y)
            });
            if is_better {
                best = Some((pos, dist));
            }
        }

        // Prune index entries whose portal block is gone. `on_state_replaced`
        // already removes broken portals, but entries can still be stale after a
        // world edit or an unclean shutdown.
        if !stale.is_empty() {
            let mut poi_storage = world.portal_poi.lock().await;
            for pos in stale {
                poi_storage.remove(&pos);
            }
        }

        let (found, _) = best?;
        Some(Self::measure_portal_at(&mut view, found).await)
    }

    /// Measures the portal containing `pos` the way vanilla does at
    /// `NetherPortalBlock.java:149`: `BlockUtil.getLargestRectangleAround` over
    /// blocks whose state equals the found portal's state, limit 21 on both axes.
    async fn measure_portal_at(view: &mut ChunkView<'_>, pos: BlockPos) -> PortalSearchResult {
        let state_id = view.state_id(&pos).await;
        let axis = NetherPortalLikeProperties::from_state_id(state_id, &Block::NETHER_PORTAL).axis;
        let axis1 = match axis {
            HorizontalAxis::X => RectAxis::X,
            HorizontalAxis::Z => RectAxis::Z,
        };

        // `getLargestRectangleAround` takes a synchronous predicate, so the
        // blocks it may touch are prefetched into the view first. The scan never
        // leaves a 21-block radius of `pos` (limits at NetherPortalBlock.java:149).
        for d in -PORTAL_RECTANGLE_LIMIT..=PORTAL_RECTANGLE_LIMIT {
            for dy in -PORTAL_RECTANGLE_LIMIT..=PORTAL_RECTANGLE_LIMIT {
                let probe = axis1.relative(pos.add(0, dy, 0), d);
                let _ = view.state_id(&probe).await;
            }
        }

        let rect = get_largest_rectangle_around(
            pos,
            axis1,
            PORTAL_RECTANGLE_LIMIT,
            RectAxis::Y,
            PORTAL_RECTANGLE_LIMIT,
            &mut |probe| view.cached_state_id(&probe) == Some(state_id),
        );

        PortalSearchResult {
            lower_corner: rect.min_corner,
            axis,
            width: rect.axis1_size.max(1) as u32,
            height: rect.axis2_size.max(1) as u32,
        }
    }

    /// Vanilla `PoiManager.getInSquare` restricted to nether portals
    /// (`PoiManager.java:88-95`, called from `PortalForcer.java:48`).
    async fn query_portal_index(
        world: &Arc<World>,
        center: BlockPos,
        radius: i32,
    ) -> Vec<BlockPos> {
        let mut poi_storage = world.portal_poi.lock().await;
        poi_storage.get_in_square(center, radius, Some(poi::POI_TYPE_NETHER_PORTAL))
    }

    /// Positive direction along `axis`, vanilla
    /// `Direction.get(AxisDirection.POSITIVE, portalAxis)` (`PortalForcer.java:52`).
    const fn positive_direction(axis: HorizontalAxis) -> BlockDirection {
        match axis {
            HorizontalAxis::X => BlockDirection::East,
            HorizontalAxis::Z => BlockDirection::South,
        }
    }

    /// Vanilla `Direction.getClockWise()` for the two directions used here:
    /// EAST -> SOUTH, SOUTH -> WEST (`PortalForcer.java:98`, `PortalForcer.java:132`).
    const fn clockwise(direction: BlockDirection) -> BlockDirection {
        match direction {
            BlockDirection::East => BlockDirection::South,
            BlockDirection::South => BlockDirection::West,
            BlockDirection::West => BlockDirection::North,
            _ => BlockDirection::East,
        }
    }

    /// Highest Y a portal may occupy: vanilla
    /// `min(getMaxY(), getMinY() + getLogicalHeight() - 1)` (`PortalForcer.java:58`).
    ///
    /// This is unconditional in vanilla — it is not gated on `has_ceiling`.
    fn max_placeable_y(world: &World) -> i32 {
        let min_y = world.min_y;
        let max_y = min_y + world.dimension.height - 1;
        max_y.min(min_y + world.dimension.logical_height - 1)
    }

    /// Locates where to build a new portal, or reports that a forced one is needed.
    ///
    /// Faithful port of the search half of `PortalForcer.createPortal`
    /// (`PortalForcer.java:51-89`) plus the forced-placement fallback
    /// (`PortalForcer.java:90-97`). The returned bool is vanilla's
    /// "nothing found, blow out a pocket first" case.
    ///
    /// Vanilla keeps the source portal's axis throughout; the axis is never
    /// re-chosen per candidate, so it is returned unchanged.
    pub async fn find_safe_location(
        world: &Arc<World>,
        target_pos: BlockPos,
        axis: HorizontalAxis,
    ) -> Option<(BlockPos, HorizontalAxis, bool)> {
        let border = BorderSnapshot::capture(world).await;
        let mut view = ChunkView::new(world);

        if let Some(found) =
            Self::search_portal_site(world, &mut view, target_pos, axis, &border).await
        {
            return Some((found, axis, false));
        }

        // PortalForcer.java:90-97: no site at all, so a pocket is carved out at
        // the target. minStartY is `max(getMinY() - -1, 70)`.
        let max_placeable_y = Self::max_placeable_y(world);
        let max_start_y = max_placeable_y - 9;
        let min_start_y = (world.min_y + 1).max(70);
        if max_start_y < min_start_y {
            return None;
        }

        let direction = Self::positive_direction(axis).to_offset();
        let forced = BlockPos::new(
            target_pos.0.x - direction.x,
            target_pos.0.y.clamp(min_start_y, max_start_y),
            target_pos.0.z - direction.z,
        );
        Some((border.clamp_to_bounds(forced), axis, true))
    }

    /// The spiral scan of `PortalForcer.createPortal` (`PortalForcer.java:59-89`).
    async fn search_portal_site(
        world: &Arc<World>,
        view: &mut ChunkView<'_>,
        origin: BlockPos,
        axis: HorizontalAxis,
        border: &BorderSnapshot,
    ) -> Option<BlockPos> {
        let direction = Self::positive_direction(axis);
        let offset = direction.to_offset();
        let min_y = world.min_y;
        let max_placeable_y = Self::max_placeable_y(world);

        let mut closest_full: Option<(BlockPos, f64)> = None;
        let mut closest_partial: Option<(BlockPos, f64)> = None;

        // PortalForcer.java:61 spirals with EAST/SOUTH literally — the spiral
        // directions do not depend on the portal axis, only the frame tests do.
        for column in SpiralAround::new(
            origin,
            CREATE_PORTAL_SPIRAL_RADIUS,
            BlockDirection::East.to_offset(),
            BlockDirection::South.to_offset(),
        ) {
            // PortalForcer.java:63: both the column and the block one step along
            // `direction` must be inside the border.
            if !border.contains_block(column.0.x, column.0.z)
                || !border.contains_block(column.0.x + offset.x, column.0.z + offset.z)
            {
                continue;
            }

            let surface = view
                .heightmap_height(ChunkHeightmapType::MotionBlocking, column.0.x, column.0.z)
                .await;
            let mut y = max_placeable_y.min(surface);

            while y >= min_y {
                let pos = BlockPos::new(column.0.x, y, column.0.z);
                if !Self::can_portal_replace(view, &pos).await {
                    y -= 1;
                    continue;
                }

                // Walk to the bottom of this replaceable run (PortalForcer.java:70-72).
                let first_empty_y = y;
                while y > min_y
                    && Self::can_portal_replace(view, &BlockPos::new(column.0.x, y - 1, column.0.z))
                        .await
                {
                    y -= 1;
                }

                // PortalForcer.java:73. Note vanilla accepts delta_y == 0: a
                // single replaceable block is fine, the frame check decides.
                let delta_y = first_empty_y - y;
                if y + 4 > max_placeable_y || (delta_y > 0 && delta_y < 3) {
                    y -= 1;
                    continue;
                }

                let floor = BlockPos::new(column.0.x, y, column.0.z);
                if Self::can_host_frame(view, floor, direction, 0).await {
                    let distance = f64::from(origin.squared_distance(&floor));

                    // PortalForcer.java:77-80: a site with clear space on both
                    // sides is preferred over a bare one.
                    let full = Self::can_host_frame(view, floor, direction, -1).await
                        && Self::can_host_frame(view, floor, direction, 1).await;
                    if full && closest_full.as_ref().is_none_or(|(_, d)| *d > distance) {
                        closest_full = Some((floor, distance));
                    }

                    // PortalForcer.java:81: once any full site exists, partial
                    // sites stop being recorded.
                    if closest_full.is_none()
                        && closest_partial.as_ref().is_none_or(|(_, d)| *d > distance)
                    {
                        closest_partial = Some((floor, distance));
                    }
                }
                y -= 1;
            }
        }

        // PortalForcer.java:86-89: fall back to the partial site.
        closest_full.or(closest_partial).map(|(pos, _)| pos)
    }

    /// Vanilla `PortalForcer.canPortalReplaceBlock` (`PortalForcer.java:126-129`):
    /// `canBeReplaced() && getFluidState().isEmpty()`.
    async fn can_portal_replace(view: &mut ChunkView<'_>, pos: &BlockPos) -> bool {
        Self::is_valid_portal_air(view.state(pos).await)
    }

    const fn is_valid_portal_air(state: &BlockState) -> bool {
        state.replaceable() && !state.is_liquid()
    }

    /// Vanilla `PortalForcer.canHostFrame` (`PortalForcer.java:131-144`).
    ///
    /// `offset` shifts the tested slab sideways along `direction.getClockWise()`,
    /// which is how vanilla distinguishes a fully clear site from a bare one.
    /// Blocks below the floor must be solid in the legacy sense (`isSolid`, not
    /// `isSolidRender`), everything at or above it must be replaceable.
    async fn can_host_frame(
        view: &mut ChunkView<'_>,
        origin: BlockPos,
        direction: BlockDirection,
        offset: i32,
    ) -> bool {
        let step = direction.to_offset();
        let side = Self::clockwise(direction).to_offset();
        for width in -1..3 {
            for height in -1..4 {
                let pos = BlockPos::new(
                    origin.0.x + step.x * width + side.x * offset,
                    origin.0.y + height,
                    origin.0.z + step.z * width + side.z * offset,
                );
                let state = view.state(&pos).await;
                if height < 0 {
                    if !state.is_solid() {
                        return false;
                    }
                } else if !Self::is_valid_portal_air(state) {
                    return false;
                }
            }
        }
        true
    }

    pub async fn build_portal_frame(
        world: &Arc<World>,
        lower_corner: BlockPos,
        axis: HorizontalAxis,
        is_fallback: bool,
    ) {
        let direction = if axis == HorizontalAxis::X {
            BlockDirection::East
        } else {
            BlockDirection::South // Fixed: positive Z direction
        };
        let perpendicular = if axis == HorizontalAxis::X {
            BlockDirection::South // Fixed: East.rotateYClockwise()
        } else {
            BlockDirection::West // Fixed: South.rotateYClockwise()
        };

        let obsidian_state = Block::OBSIDIAN.default_state.id;
        let air_state = Block::AIR.default_state.id;

        if is_fallback {
            // Clear area around the portal matching vanilla exactly:
            // perpendicular: -1, 0, 1 (3 blocks)
            // portal_dir: 0, 1 (2 blocks - portal interior only)
            // height: -1, 0, 1, 2 (4 blocks)
            for perp in -1..2 {
                for portal_dir in 0..2 {
                    for height in -1..3 {
                        let pos = lower_corner
                            .offset_dir(direction.to_offset(), portal_dir)
                            .offset_dir(perpendicular.to_offset(), perp)
                            .offset_dir(BlockDirection::Up.to_offset(), height);

                        let state = if height < 0 {
                            obsidian_state
                        } else {
                            air_state
                        };
                        world
                            .set_block_state(&pos, state, BlockFlags::NOTIFY_ALL)
                            .await;
                    }
                }
            }
        }

        for portal_dir in -1..3 {
            for height in -1..4 {
                if portal_dir == -1 || portal_dir == 2 || height == -1 || height == 3 {
                    let pos = lower_corner
                        .offset_dir(direction.to_offset(), portal_dir)
                        .offset_dir(BlockDirection::Up.to_offset(), height);
                    world
                        .set_block_state(&pos, obsidian_state, BlockFlags::NOTIFY_ALL)
                        .await;
                }
            }
        }

        let mut props = NetherPortalLikeProperties::default(&Block::NETHER_PORTAL);
        props.axis = axis;
        let portal_state = props.to_state_id(&Block::NETHER_PORTAL);

        // Place every portal block first, then index them. Holding the POI lock
        // across `set_block_state` would deadlock: replacing a block runs
        // `NetherPortalBlock::on_state_replaced`, which locks `portal_poi` itself.
        let mut placed = Vec::with_capacity(6);
        for x in 0..2 {
            for y in 0..3 {
                let pos = lower_corner
                    .offset_dir(direction.to_offset(), x)
                    .offset_dir(BlockDirection::Up.to_offset(), y);
                world
                    .set_block_state(
                        &pos,
                        portal_state,
                        BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE,
                    )
                    .await;
                placed.push(pos);
            }
        }

        let mut poi_storage = world.portal_poi.lock().await;
        for pos in placed {
            poi_storage.add_portal(pos);
        }
    }
}
