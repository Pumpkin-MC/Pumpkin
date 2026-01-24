use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{BlockProperties, HorizontalAxis, NetherPortalLikeProperties},
    tag,
    tag::Taggable,
};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
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
    pub fn get_teleport_position(&self) -> Vector3<f64> {
        let x = self.lower_corner.0.x as f64;
        let y = self.lower_corner.0.y as f64;
        let z = self.lower_corner.0.z as f64;

        match self.axis {
            HorizontalAxis::X => Vector3::new(x + (self.width as f64) / 2.0, y, z + 0.5),
            HorizontalAxis::Z => Vector3::new(x + 0.5, y, z + (self.width as f64) / 2.0),
        }
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
        for pos in blocks {
            world
                .set_block_state(
                    &pos,
                    state,
                    BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE,
                )
                .await;
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

    /// Search for existing portal. Radius: 16 blocks (Nether), 128 blocks (Overworld).
    pub async fn search_for_portal(
        world: &Arc<World>,
        target_pos: BlockPos,
    ) -> Option<PortalSearchResult> {
        let min_y = world.min_y;
        let max_y = min_y + world.dimension.height - 1;

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

        for pos in BlockPos::iterate_outwards(target_pos, search_radius, search_radius, search_radius) {
            if pos.0.y < min_y || pos.0.y > search_max_y {
                continue;
            }

            if world.get_block(&pos).await != &Block::NETHER_PORTAL {
                continue;
            }

            for axis in [HorizontalAxis::X, HorizontalAxis::Z] {
                if let Some(portal) = Self::get_on_axis(world, &pos, axis).await {
                    if portal.was_already_valid() {
                        return Some(PortalSearchResult {
                            lower_corner: portal.lower_conor,
                            axis: portal.axis,
                            width: portal.width,
                            height: portal.height,
                        });
                    }
                }
            }
        }

        None
    }

    /// Find safe location for new portal. Searches top-down using heightmap.
    /// Prefers ideal (3-wide clearance) over acceptable (center only) positions.
    /// Returns (position, axis, is_fallback) - fallback means no valid surface found.
    pub async fn find_safe_location(
        world: &Arc<World>,
        target_pos: BlockPos,
    ) -> Option<(BlockPos, HorizontalAxis, bool)> {
        let min_y = world.min_y;
        let max_y = min_y + world.dimension.height - 1;

        let top_y_limit = if world.dimension.has_ceiling {
            (min_y + world.dimension.logical_height - 1).min(max_y)
        } else {
            max_y
        };

        let mut ideal_pos: Option<(BlockPos, HorizontalAxis, f64)> = None;
        let mut acceptable_pos: Option<(BlockPos, HorizontalAxis, f64)> = None;

        for offset_x in -16..=16 {
            for offset_z in -16..=16 {
                let check_x = target_pos.0.x + offset_x;
                let check_z = target_pos.0.z + offset_z;

                // Use heightmap to start from actual surface
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

                            for axis in [HorizontalAxis::X, HorizontalAxis::Z] {
                                if Self::is_valid_portal_pos(world, floor_pos, axis, 0).await {
                                    let dist = target_pos.0.squared_distance_to(floor_pos.0.x, floor_pos.0.y, floor_pos.0.z) as f64;

                                    let is_ideal = Self::is_valid_portal_pos(world, floor_pos, axis, -1).await
                                        && Self::is_valid_portal_pos(world, floor_pos, axis, 1).await;

                                    if is_ideal {
                                        if ideal_pos.is_none() || dist < ideal_pos.as_ref().unwrap().2 {
                                            ideal_pos = Some((floor_pos, axis, dist));
                                        }
                                    } else if ideal_pos.is_none() {
                                        if acceptable_pos.is_none() || dist < acceptable_pos.as_ref().unwrap().2 {
                                            acceptable_pos = Some((floor_pos, axis, dist));
                                        }
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

        if let Some((pos, axis, _)) = ideal_pos {
            return Some((pos, axis, false));
        }
        if let Some((pos, axis, _)) = acceptable_pos {
            return Some((pos, axis, false));
        }

        // Fallback: no valid surface found, will need to build platform
        let fallback_y = target_pos.0.y.clamp(70.max(min_y + 1), top_y_limit - 9);
        Some((BlockPos(Vector3::new(target_pos.0.x, fallback_y, target_pos.0.z)), HorizontalAxis::X, true))
    }

    fn is_valid_portal_air(state: &BlockState) -> bool {
        state.replaceable() && !state.is_liquid()
    }

    /// Validates portal position: solid floor at y-1, clear space at y 0-3.
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

    /// Builds obsidian frame. Platform only built in fallback case.
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

        // Only build platform in fallback case (no valid surface found)
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

        // Build frame edges
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

        // Fill portal interior
        let mut props = NetherPortalLikeProperties::default(&Block::NETHER_PORTAL);
        props.axis = axis;
        let portal_state = props.to_state_id(&Block::NETHER_PORTAL);

        for x in 0..2 {
            for y in 0..3 {
                let pos = lower_corner
                    .offset_dir(direction.to_offset(), x)
                    .offset_dir(BlockDirection::Up.to_offset(), y);
                world
                    .set_block_state(&pos, portal_state, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        }
    }
}
