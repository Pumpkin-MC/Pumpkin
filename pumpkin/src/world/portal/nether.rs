use std::sync::Arc;

use super::poi;
use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{BlockProperties, HorizontalAxis, NetherPortalLikeProperties},
    tag,
    tag::Taggable,
};
use pumpkin_util::math::{boundingbox::EntityDimensions, position::BlockPos, vector3::Vector3};
use pumpkin_world::world::BlockFlags;

use crate::world::World;

const SEARCH_RADIUS_NETHER: i32 = 16;
const SEARCH_RADIUS_OVERWORLD: i32 = 128;

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
    pub fn calculate_teleport_yaw(&self, current_yaw: f32, source_axis: Option<HorizontalAxis>) -> f32 {
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
    pub fn entity_pos_in_portal(&self, entity_pos: Vector3<f64>, dimensions: &EntityDimensions) -> Vector3<f64> {
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

        Vector3::new(axis_progress, y_progress, perp_offset)
    }

    #[must_use]
    pub fn calculate_exit_position(&self, relative_pos: Vector3<f64>, dimensions: &EntityDimensions) -> Vector3<f64> {
        let portal_width = f64::from(self.width) - f64::from(dimensions.width);
        let portal_height = f64::from(self.height) - f64::from(dimensions.height);
        let lower = self.lower_corner.0;

        let axis_offset = if portal_width > 0.0 {
            relative_pos.x * portal_width + f64::from(dimensions.width) / 2.0
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

    pub async fn find_open_position(
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
                if self.is_position_clear(world, test_pos, dimensions).await {
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

    async fn is_position_clear(&self, world: &Arc<World>, center: Vector3<f64>, dimensions: &EntityDimensions) -> bool {
        let half_width = f64::from(dimensions.width) / 2.0;
        let height = f64::from(dimensions.height);

        let check_points = [
            Vector3::new(center.x - half_width, center.y - height / 2.0, center.z - half_width),
            Vector3::new(center.x + half_width, center.y - height / 2.0, center.z + half_width),
            Vector3::new(center.x - half_width, center.y + height / 2.0, center.z - half_width),
            Vector3::new(center.x + half_width, center.y + height / 2.0, center.z + half_width),
        ];

        for point in check_points {
            let block_pos = BlockPos::floored(point.x, point.y, point.z);
            let state = world.get_block_state(&block_pos).await;
            if state.is_solid_block() {
                return false;
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
    pub fn is_valid(&self) -> bool {
        self.width >= Self::MIN_WIDTH
            && self.width <= Self::MAX_WIDTH
            && self.height >= Self::MIN_WIDTH
            && self.height <= Self::MAX_HEIGHT
    }

    #[must_use]
    pub fn was_already_valid(&self) -> bool {
        self.is_valid() && self.found_portal_blocks == self.width * self.height
    }

    #[must_use]
    pub fn lower_corner(&self) -> BlockPos {
        self.lower_conor
    }

    #[must_use]
    pub fn axis(&self) -> HorizontalAxis {
        self.axis
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    pub async fn create(&self, world: &Arc<World>) {
        let mut props = NetherPortalLikeProperties::default(&Block::NETHER_PORTAL);
        props.axis = self.axis;
        let state = props.to_state_id(&Block::NETHER_PORTAL);
        let blocks: Vec<BlockPos> = BlockPos::iterate(
            self.lower_conor,
            self.lower_conor
                .offset_dir(BlockDirection::Up.to_offset(), self.height as i32 - 1)
                .offset_dir(self.negative_direction.to_offset(), self.width as i32 - 1),
        ).collect();

        let mut poi_storage = world.portal_poi.lock().await;
        for pos in blocks {
            world
                .set_block_state(
                    &pos,
                    state,
                    BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE,
                )
                .await;
            poi_storage.add_portal(pos);
        }
    }

    pub async fn get_new_portal(
        world: &World,
        pos: &BlockPos,
        first_axis: HorizontalAxis,
    ) -> Option<Self> {
        if let Some(portal) = Self::get_on_axis(world, pos, first_axis).await
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
        if let Some(portal) = Self::get_on_axis(world, pos, next_axis).await
            && portal.is_valid()
            && portal.found_portal_blocks == 0
        {
            return Some(portal);
        }
        None
    }

    pub async fn get_on_axis(world: &World, pos: &BlockPos, axis: HorizontalAxis) -> Option<Self> {
        let direction = if axis == HorizontalAxis::X {
            BlockDirection::West
        } else {
            BlockDirection::South
        };
        let cornor = Self::get_lower_cornor(world, direction, pos).await?;
        let width = Self::get_width(world, &cornor, &direction).await;
        if !(Self::MIN_WIDTH..=Self::MAX_WIDTH).contains(&width) {
            return None;
        }
        let mut found_portal_blocks = 0;
        let height =
            Self::get_height(world, &cornor, &direction, width, &mut found_portal_blocks).await?;
        Some(Self {
            axis,
            found_portal_blocks,
            negative_direction: direction,
            lower_conor: cornor,
            width,
            height,
        })
    }

    async fn get_lower_cornor(
        world: &World,
        direction: BlockDirection,
        pos: &BlockPos,
    ) -> Option<BlockPos> {
        let limit_y = pos.0.y - Self::MAX_HEIGHT as i32;
        let mut pos = *pos;
        while pos.0.y > limit_y {
            let (block, state) = world.get_block_and_state(&pos.down()).await;
            if !Self::valid_state_inside_portal(block, state) {
                break;
            }
            pos = pos.down();
        }
        let neg_dir = direction.opposite();
        let width = (Self::get_width(world, &pos, &neg_dir).await as i32) - 1;
        if width < 0 {
            return None;
        }
        Some(pos.offset_dir(neg_dir.to_offset(), width))
    }

    async fn get_width(
        world: &World,
        original_lower_corner: &BlockPos,
        negative_dir: &BlockDirection,
    ) -> u32 {
        let mut lower_corner;
        for i in 0..=Self::MAX_WIDTH {
            lower_corner = original_lower_corner.offset_dir(negative_dir.to_offset(), i as i32);
            let (block, block_state) = world.get_block_and_state(&lower_corner).await;
            if !Self::valid_state_inside_portal(block, block_state) {
                if &Self::FRAME_BLOCK != block {
                    break;
                }
                return i;
            }
            let block = world.get_block(&lower_corner.down()).await;
            if &Self::FRAME_BLOCK != block {
                break;
            }
        }
        0
    }

    async fn get_height(
        world: &World,
        lower_corner: &BlockPos,
        negative_dir: &BlockDirection,
        width: u32,
        found_portal_blocks: &mut u32,
    ) -> Option<u32> {
        let height = Self::get_potential_height(
            world,
            lower_corner,
            negative_dir,
            width,
            found_portal_blocks,
        )
        .await;
        if !(Self::MIN_HEIGHT..=Self::MAX_HEIGHT).contains(&height)
            || !Self::is_horizontal_frame_valid(world, lower_corner, negative_dir, width, height)
                .await
        {
            return None;
        }
        Some(height)
    }

    async fn get_potential_height(
        world: &World,
        lower_corner: &BlockPos,
        negative_dir: &BlockDirection,
        width: u32,
        found_portal_blocks: &mut u32,
    ) -> u32 {
        for i in 0..Self::MAX_HEIGHT as i32 {
            let mut pos = lower_corner
                .offset_dir(BlockDirection::Up.to_offset(), i)
                .offset_dir(negative_dir.to_offset(), -1);
            if world.get_block(&pos).await != &Self::FRAME_BLOCK {
                return i as u32;
            }

            pos = lower_corner
                .offset_dir(BlockDirection::Up.to_offset(), i)
                .offset_dir(negative_dir.to_offset(), width as i32);
            if world.get_block(&pos).await != &Self::FRAME_BLOCK {
                return i as u32;
            }

            for j in 0..width {
                pos = lower_corner
                    .offset_dir(BlockDirection::Up.to_offset(), i)
                    .offset_dir(negative_dir.to_offset(), j as i32);
                let (block, block_state) = world.get_block_and_state(&pos).await;
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

    async fn is_horizontal_frame_valid(
        world: &World,
        lower_corner: &BlockPos,
        dir: &BlockDirection,
        width: u32,
        height: u32,
    ) -> bool {
        let mut pos;
        for i in 0..width {
            pos = lower_corner
                .offset_dir(BlockDirection::Up.to_offset(), height as i32)
                .offset_dir(dir.to_offset(), i as i32);
            if &Self::FRAME_BLOCK != world.get_block(&pos).await {
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

    pub async fn search_for_portal(
        world: &Arc<World>,
        target_pos: BlockPos,
    ) -> Option<PortalSearchResult> {
        let min_y = world.min_y;
        let max_y = min_y + world.dimension.height - 1;
        let worldborder = world.worldborder.lock().await;

        let search_radius = if world.dimension.has_ceiling {
            SEARCH_RADIUS_NETHER
        } else {
            SEARCH_RADIUS_OVERWORLD
        };

        let search_max_y = if world.dimension.has_ceiling {
            (min_y + world.dimension.logical_height - 1).min(max_y)
        } else {
            max_y
        };

        let mut poi_storage = world.portal_poi.lock().await;
        let portal_positions = poi_storage.get_in_square(target_pos, search_radius, Some(poi::POI_TYPE_NETHER_PORTAL));
        drop(poi_storage);

        let mut best: Option<(PortalSearchResult, f64, i32)> = None;

        for pos in portal_positions {
            if pos.0.y < min_y || pos.0.y > search_max_y {
                continue;
            }

            if !worldborder.contains_block(pos.0.x, pos.0.z) {
                continue;
            }

            if world.get_block(&pos).await != &Block::NETHER_PORTAL {
                continue;
            }

            for axis in [HorizontalAxis::X, HorizontalAxis::Z] {
                if let Some(portal) = Self::get_on_axis(world, &pos, axis).await
                    && portal.was_already_valid()
                {
                    let dist = f64::from(target_pos.0.squared_distance_to(
                        portal.lower_conor.0.x,
                        portal.lower_conor.0.y,
                        portal.lower_conor.0.z,
                    ));
                    let y = portal.lower_conor.0.y;

                    let is_better = match &best {
                        None => true,
                        Some((_, best_dist, best_y)) => {
                            dist < *best_dist || ((dist - *best_dist).abs() < f64::EPSILON && y < *best_y)
                        }
                    };

                    if is_better {
                        best = Some((
                            PortalSearchResult {
                                lower_corner: portal.lower_conor,
                                axis: portal.axis,
                                width: portal.width,
                                height: portal.height,
                            },
                            dist,
                            y,
                        ));
                    }
                }
            }
        }

        best.map(|(result, _, _)| result)
    }

    pub async fn find_safe_location(
        world: &Arc<World>,
        target_pos: BlockPos,
        axis: HorizontalAxis,
    ) -> Option<(BlockPos, HorizontalAxis, bool)> {
        let min_y = world.min_y;
        let max_y = min_y + world.dimension.height - 1;
        let worldborder = world.worldborder.lock().await;

        let top_y_limit = if world.dimension.has_ceiling {
            (min_y + world.dimension.logical_height - 1).min(max_y)
        } else {
            max_y
        };

        let direction = if axis == HorizontalAxis::X {
            BlockDirection::East
        } else {
            BlockDirection::North
        };

        let mut ideal_pos: Option<(BlockPos, HorizontalAxis, f64)> = None;
        let mut acceptable_pos: Option<(BlockPos, HorizontalAxis, f64)> = None;

        for offset_x in -16..=16 {
            for offset_z in -16..=16 {
                let check_x = target_pos.0.x + offset_x;
                let check_z = target_pos.0.z + offset_z;

                if !worldborder.contains_block(check_x, check_z) {
                    continue;
                }

                let offset_pos = BlockPos(Vector3::new(check_x, 0, check_z))
                    .offset_dir(direction.to_offset(), 1);
                if !worldborder.contains_block(offset_pos.0.x, offset_pos.0.z) {
                    continue;
                }

                let heightmap_y = world.get_motion_blocking_height(check_x, check_z).await;
                let start_y = heightmap_y.min(top_y_limit);

                let mut y = start_y;
                while y >= min_y {
                    let pos = BlockPos(Vector3::new(check_x, y, check_z));
                    let state = world.get_block_state(&pos).await;

                    if Self::is_valid_portal_air(state) {
                        let mut bottom_y = y;
                        while bottom_y > min_y {
                            let below = BlockPos(Vector3::new(check_x, bottom_y - 1, check_z));
                            let below_state = world.get_block_state(&below).await;
                            if !Self::is_valid_portal_air(below_state) {
                                break;
                            }
                            bottom_y -= 1;
                        }

                        let air_height = y - bottom_y;
                        if air_height >= 3 && bottom_y + 4 <= top_y_limit {
                            let floor_pos = BlockPos(Vector3::new(check_x, bottom_y, check_z));

                            for check_axis in [HorizontalAxis::X, HorizontalAxis::Z] {
                                if Self::is_valid_portal_pos(world, floor_pos, check_axis, 0).await {
                                    let dist = f64::from(target_pos.0.squared_distance_to(floor_pos.0.x, floor_pos.0.y, floor_pos.0.z));

                                    let is_ideal = Self::is_valid_portal_pos(world, floor_pos, check_axis, -1).await
                                        && Self::is_valid_portal_pos(world, floor_pos, check_axis, 1).await;

                                    if is_ideal {
                                        if ideal_pos.is_none() || dist < ideal_pos.as_ref().unwrap().2 {
                                            ideal_pos = Some((floor_pos, check_axis, dist));
                                        }
                                    } else if ideal_pos.is_none()
                                        && (acceptable_pos.is_none() || dist < acceptable_pos.as_ref().unwrap().2)
                                    {
                                        acceptable_pos = Some((floor_pos, check_axis, dist));
                                    }
                                }
                            }
                        }
                        y = bottom_y - 1;
                    } else {
                        y -= 1;
                    }
                }
            }
        }

        if let Some((pos, result_axis, _)) = ideal_pos {
            return Some((pos, result_axis, false));
        }
        if let Some((pos, result_axis, _)) = acceptable_pos {
            return Some((pos, result_axis, false));
        }

        let fallback_y = target_pos.0.y.clamp(70.max(min_y + 1), top_y_limit - 9);
        let fallback_pos = BlockPos(Vector3::new(
            target_pos.0.x - direction.to_offset().x,
            fallback_y,
            target_pos.0.z - direction.to_offset().z,
        ));
        let clamped_pos = worldborder.clamp_block(fallback_pos.0.x, fallback_pos.0.z);
        Some((BlockPos(Vector3::new(clamped_pos.0, fallback_y, clamped_pos.1)), axis, true))
    }

    fn is_valid_portal_air(state: &BlockState) -> bool {
        state.replaceable() && !state.is_liquid()
    }

    async fn is_valid_portal_pos(
        world: &Arc<World>,
        floor_pos: BlockPos,
        axis: HorizontalAxis,
        perpendicular_offset: i32,
    ) -> bool {
        let direction = if axis == HorizontalAxis::X {
            BlockDirection::East
        } else {
            BlockDirection::North
        };
        let perpendicular = if axis == HorizontalAxis::X {
            BlockDirection::North
        } else {
            BlockDirection::East
        };

        for portal_dir in -1..3 {
            for height in -1..4 {
                let pos = floor_pos
                    .offset_dir(direction.to_offset(), portal_dir)
                    .offset_dir(perpendicular.to_offset(), perpendicular_offset)
                    .offset_dir(BlockDirection::Up.to_offset(), height);

                let state = world.get_block_state(&pos).await;

                if height < 0 {
                    if !state.is_solid_block() {
                        return false;
                    }
                } else if !Self::is_valid_portal_air(state) {
                    return false;
                }
            }
        }

        true
    }

    pub async fn build_portal_frame(world: &Arc<World>, lower_corner: BlockPos, axis: HorizontalAxis, is_fallback: bool) {
        let direction = if axis == HorizontalAxis::X {
            BlockDirection::East
        } else {
            BlockDirection::North
        };
        let perpendicular = if axis == HorizontalAxis::X {
            BlockDirection::North
        } else {
            BlockDirection::East
        };

        let obsidian_state = Block::OBSIDIAN.default_state.id;
        let air_state = Block::AIR.default_state.id;

        if is_fallback {
            for perp in -1..=1 {
                for portal_dir in 0..2 {
                    for height in -1..3 {
                        let pos = lower_corner
                            .offset_dir(direction.to_offset(), portal_dir)
                            .offset_dir(perpendicular.to_offset(), perp)
                            .offset_dir(BlockDirection::Up.to_offset(), height);

                        let state = if height < 0 { obsidian_state } else { air_state };
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

        let mut poi_storage = world.portal_poi.lock().await;
        for x in 0..2 {
            for y in 0..3 {
                let pos = lower_corner
                    .offset_dir(direction.to_offset(), x)
                    .offset_dir(BlockDirection::Up.to_offset(), y);
                world
                    .set_block_state(&pos, portal_state, BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE)
                    .await;
                poi_storage.add_portal(pos);
            }
        }
    }
}
