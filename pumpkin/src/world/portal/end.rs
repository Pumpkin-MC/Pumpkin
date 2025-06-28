use std::sync::Arc;

use pumpkin_data::{Block, BlockDirection, block_properties::BlockProperties};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::world::BlockFlags;

use crate::world::World;

type EndPortalFrameProperties = pumpkin_data::block_properties::EndPortalFrameLikeProperties;

pub struct EndPortal;

impl EndPortal {
    const FRAME_BLOCK: Block = Block::END_PORTAL_FRAME;
    const FRAME_BLOCK_ID: u16 = Self::FRAME_BLOCK.id;

    pub async fn get_new_portal(world: &Arc<World>, pos: BlockPos) {
        let mid_pos = Self::get_mid_pos(world, pos);
        if let Some(mid_pos) = mid_pos.await {
            if Self::is_valid_portal(world, mid_pos).await {
                Self::create_portal(world, mid_pos).await;
            }
        }
    }

    async fn get_mid_pos(world: &World, pos: BlockPos) -> Option<BlockPos> {
        let (block, state) = world.get_block_and_block_state(&pos).await;
        if block != Self::FRAME_BLOCK {
            return None;
        }

        let properties = EndPortalFrameProperties::from_state_id(state.id, &block);
        let facing_dir = properties.facing;
        let left_pos = pos.offset_dir(facing_dir.rotate_clockwise().to_offset(), 1);
        let right_pos = pos.offset_dir(facing_dir.rotate_counter_clockwise().to_offset(), 1);

        let left_block = world.get_block(&left_pos).await;
        let right_block = world.get_block(&right_pos).await;

        let offset = match (left_block.id, right_block.id) {
            (Self::FRAME_BLOCK_ID, Self::FRAME_BLOCK_ID) => 0, // Middle block
            (Self::FRAME_BLOCK_ID, _) => -1,                   // Right block (offset left)
            (_, Self::FRAME_BLOCK_ID) => 1,                    // Left block (offset right)
            _ => return None,
        };

        Some(
            pos.offset_dir(facing_dir.to_offset(), 2)
                .offset_dir(facing_dir.rotate_counter_clockwise().to_offset(), offset),
        )
    }

    async fn is_valid_portal(world: &World, pos: BlockPos) -> bool {
        for dir in BlockDirection::horizontal() {
            let facing = dir.to_horizontal_facing().unwrap();
            let mid_pos = pos.offset_dir(dir.to_offset(), 2);
            let left_pos = mid_pos.offset_dir(facing.rotate_clockwise().to_offset(), 1);
            let right_pos = mid_pos.offset_dir(facing.rotate_counter_clockwise().to_offset(), 1);

            let (mid_block, mid_state) = world.get_block_and_block_state(&mid_pos).await;
            let (left_block, left_state) = world.get_block_and_block_state(&left_pos).await;
            let (right_block, right_state) = world.get_block_and_block_state(&right_pos).await;

            if left_block.id != Self::FRAME_BLOCK_ID
                || mid_block.id != Self::FRAME_BLOCK_ID
                || right_block.id != Self::FRAME_BLOCK_ID
            {
                return false;
            }

            let mid_properties = EndPortalFrameProperties::from_state_id(mid_state.id, &mid_block);
            let left_properties =
                EndPortalFrameProperties::from_state_id(left_state.id, &left_block);
            let right_properties =
                EndPortalFrameProperties::from_state_id(right_state.id, &right_block);

            if left_properties.facing != facing.opposite()
                || mid_properties.facing != facing.opposite()
                || right_properties.facing != facing.opposite()
            {
                return false;
            }

            if !left_properties.eye || !mid_properties.eye || !right_properties.eye {
                return false;
            }
        }
        true
    }

    async fn create_portal(world: &Arc<World>, pos: BlockPos) {
        for x in -1..=1 {
            for z in -1..=1 {
                world
                    .set_block_state(
                        &pos.offset(Vector3::new(x, 0, z)),
                        Block::END_PORTAL.default_state.id,
                        BlockFlags::empty(),
                    )
                    .await;
            }
        }
    }
}
