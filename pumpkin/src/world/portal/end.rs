use std::sync::Arc;

use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection, BlockId, block_properties::BlockProperties};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::world::BlockFlags;

use crate::world::World;

type EndPortalFrameProperties = pumpkin_data::block_properties::EndPortalFrameLikeProperties;

pub struct EndPortal;

impl EndPortal {
    const FRAME_BLOCK: Block = Block::END_PORTAL_FRAME;
    const FRAME_BLOCK_ID: BlockId = Self::FRAME_BLOCK.id;

    pub async fn get_new_portal(world: &Arc<World>, pos: BlockPos) {
        for mid_pos in Self::candidate_centers(world, pos) {
            if Self::is_valid_portal(world, mid_pos) {
                Self::create_portal(world, mid_pos).await;
                return;
            }
        }
    }

    /// Possible portal centers for a frame block: 2 steps in its facing direction,
    /// with lateral offsets covering the three frames on that side.
    fn candidate_centers(world: &World, pos: BlockPos) -> Vec<BlockPos> {
        let (block, state) = world.get_block_and_state_id(&pos);
        if block != &Self::FRAME_BLOCK {
            return Vec::new();
        }

        let properties = EndPortalFrameProperties::from_state_id(state, block);
        let facing_dir = properties.facing;
        let mut candidates = Vec::with_capacity(3);
        for lateral in -1..=1 {
            candidates.push(
                pos.offset_dir(facing_dir.to_offset(), 2)
                    .offset_dir(facing_dir.rotate_counter_clockwise().to_offset(), lateral),
            );
        }
        candidates
    }

    fn is_valid_portal(world: &World, pos: BlockPos) -> bool {
        for dir in BlockDirection::horizontal() {
            let mid_pos = pos.offset_dir(dir.to_offset(), 2);
            let left_pos = mid_pos.offset_dir(dir.rotate_clockwise().to_offset(), 1);
            let right_pos = mid_pos.offset_dir(dir.rotate_counter_clockwise().to_offset(), 1);

            let (mid_block, mid_state) = world.get_block_and_state_id(&mid_pos);
            let (left_block, left_state) = world.get_block_and_state_id(&left_pos);
            let (right_block, right_state) = world.get_block_and_state_id(&right_pos);

            if left_block.id != Self::FRAME_BLOCK_ID
                || mid_block.id != Self::FRAME_BLOCK_ID
                || right_block.id != Self::FRAME_BLOCK_ID
            {
                return false;
            }

            let mid_properties = EndPortalFrameProperties::from_state_id(mid_state, mid_block);
            let left_properties = EndPortalFrameProperties::from_state_id(left_state, left_block);
            let right_properties =
                EndPortalFrameProperties::from_state_id(right_state, right_block);

            // Frames on this side face toward the center (opposite of outward dir).
            if left_properties.facing != dir.opposite()
                || mid_properties.facing != dir.opposite()
                || right_properties.facing != dir.opposite()
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
        let portal_state = Block::END_PORTAL.default_state.id;
        let mut positions = Vec::with_capacity(9);
        for x in -1..=1 {
            for z in -1..=1 {
                positions.push(pos.offset(Vector3::new(x, 0, z)));
            }
        }

        // Place one-by-one so EndPortalBlock::placed attaches block entities and
        // broadcasts CBlockEntityData (requires chunk_data_nbt on the BE).
        for position in &positions {
            world
                .set_block_state(position, portal_state, BlockFlags::NOTIFY_LISTENERS)
                .await;
        }

        world.sync_world_event(WorldEvent::SoundEndPortalSpawn, pos, 0);
    }
}
