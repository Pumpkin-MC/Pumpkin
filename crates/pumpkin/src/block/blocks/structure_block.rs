use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, NormalUseArgs, OnPlaceArgs};

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{BlockProperties, StructureBlockLikeProperties};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::PermissionLvl;

#[pumpkin_block("minecraft:structure_block")]
pub struct StructureBlock;

impl BlockBehaviour for StructureBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let props = StructureBlockLikeProperties::default(args.block);
            props.to_state_id(args.block)
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if args.player.permission_lvl.load() < PermissionLvl::Two {
                return BlockActionResult::Pass;
            }
            let Some(block_entity) = args.world.get_block_entity(args.position) else {
                return BlockActionResult::Pass;
            };
            args.world.update_block_entity(&block_entity);

            BlockActionResult::Success
        })
    }
}
