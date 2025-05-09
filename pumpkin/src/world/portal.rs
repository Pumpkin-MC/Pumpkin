use std::sync::Arc;

use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, HorizontalAxis, NetherPortalLikeProperties},
    tag::Tagable,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use super::{BlockFlags, World};

pub struct NetherPortal {
    axis: HorizontalAxis,
    found_portal_blocks: u32,
    negative_direction: BlockDirection,
    lower_conor: BlockPos,
    width: u32,
    height: u32,
}

impl NetherPortal {
    /// This is Vanilla
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
        // We remove 1 block because of the frame border
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
        // We check both axis here X and Z
        if let Some(portal) = Self::get_on_axis(world, pos, first_axis).await {
            if portal.is_valid() && portal.found_portal_blocks == 0 {
                return Some(portal);
            }
        }
        let next_axis = if first_axis == HorizontalAxis::X {
            HorizontalAxis::Z
        } else {
            HorizontalAxis::X
        };
        if let Some(portal) = Self::get_on_axis(world, pos, next_axis).await {
            if portal.is_valid() && portal.found_portal_blocks == 0 {
                return Some(portal);
            }
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
        // TODO: check max getBottomY
        let limit_y = pos.0.y - Self::MAX_HEIGHT as i32;
        let mut pos = *pos;
        while pos.0.y > limit_y {
            let (block, state) = world.get_block_and_block_state(&pos.down()).await.unwrap();
            if !Self::valid_state_inside_portal(&block, &state) {
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
            let (block, block_state) = world
                .get_block_and_block_state(&lower_corner)
                .await
                .unwrap();
            if !Self::valid_state_inside_portal(&block, &block_state) {
                if Self::FRAME_BLOCK != block {
                    break;
                }
                return i;
            }
            let block = world.get_block(&lower_corner.down()).await.unwrap();
            if Self::FRAME_BLOCK != block {
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
            if world.get_block(&pos).await.unwrap() != Self::FRAME_BLOCK {
                return i as u32;
            }

            pos = lower_corner
                .offset_dir(BlockDirection::Up.to_offset(), i)
                .offset_dir(negative_dir.to_offset(), width as i32);
            if world.get_block(&pos).await.unwrap() != Self::FRAME_BLOCK {
                return i as u32;
            }

            for j in 0..width {
                pos = lower_corner
                    .offset_dir(BlockDirection::Up.to_offset(), i)
                    .offset_dir(negative_dir.to_offset(), j as i32);
                let (block, block_state) = world.get_block_and_block_state(&pos).await.unwrap();
                if !Self::valid_state_inside_portal(&block, &block_state) {
                    return i as u32;
                }
                if block == Block::NETHER_PORTAL {
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
            if Self::FRAME_BLOCK != world.get_block(&pos).await.unwrap() {
                return false;
            }
        }
        true
    }

    /// What is allowed to be inside the Portal frame
    fn valid_state_inside_portal(block: &Block, state: &BlockState) -> bool {
        state.is_air()
            || block.is_tagged_with("minecraft:fire").unwrap()
            || block == &Block::NETHER_PORTAL
    }
}
