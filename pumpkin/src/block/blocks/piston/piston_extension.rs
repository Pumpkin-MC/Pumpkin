use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::{Block, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::pumpkin_block::BrokenArgs;
use crate::block::pumpkin_block::PumpkinBlock;

use super::piston::PistonProps;

pub(crate) type MovingPistonProps = pumpkin_data::block_properties::MovingPistonLikeProperties;

#[pumpkin_block("minecraft:moving_piston")]
pub struct PistonExtensionBlock;

#[async_trait]
impl PumpkinBlock for PistonExtensionBlock {
    async fn broken(&self, args: BrokenArgs<'_>) {
        let props = MovingPistonProps::from_state_id(args.state.id, &Block::MOVING_PISTON);
        let pos = args
            .position
            .offset(props.facing.opposite().to_block_direction().to_offset());
        let (new_block, new_state) = args.world.get_block_and_block_state(&pos).await;
        if &Block::PISTON == new_block || &Block::STICKY_PISTON == new_block {
            let props = PistonProps::from_state_id(new_state.id, new_block);
            if props.extended {
                // TODO: use player
                args.world
                    .break_block(&pos, None, BlockFlags::SKIP_DROPS)
                    .await;
            }
        }
    }
}
