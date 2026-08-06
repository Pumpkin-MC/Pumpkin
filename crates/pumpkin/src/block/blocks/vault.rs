use std::sync::Arc;

use pumpkin_data::block_properties::{BlockProperties, VaultLikeProperties, VaultState};
use pumpkin_macros::pumpkin_block;

use crate::block::entities::vault::VaultBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, PlacedArgs, UseWithItemArgs};

#[pumpkin_block("minecraft:vault")]
pub struct VaultBlock;

impl BlockBehaviour for VaultBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let block_entity = VaultBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(block_entity));
        })
    }

    // VaultBlock.java:45-68
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let props = VaultLikeProperties::from_state_id(
                args.world.get_block_state_id(args.position),
                args.block,
            );
            if props.vault_state != VaultState::Active {
                return BlockActionResult::Pass;
            }
            let Some(block_entity) = args.world.get_block_entity(args.position) else {
                return BlockActionResult::Pass;
            };
            let Some(vault) = block_entity.as_any().downcast_ref::<VaultBlockEntity>() else {
                return BlockActionResult::Pass;
            };
            let mut item_stack = args.item_stack.lock().await;
            if item_stack.is_empty() {
                return BlockActionResult::Pass;
            }
            if vault
                .try_insert_key(args.world, args.player, &mut item_stack)
                .await
            {
                BlockActionResult::SuccessServer
            } else {
                BlockActionResult::Consume
            }
        })
    }
}
