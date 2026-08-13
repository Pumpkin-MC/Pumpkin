use std::sync::Arc;

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{BlockProperties, DecoratedPotLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_macros::pumpkin_block;

use crate::block::entities::decorated_pot::DecoratedPotBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, GetComparatorOutputArgs, NormalUseArgs, OnPlaceArgs,
    PlacedArgs, UseWithItemArgs,
};

#[pumpkin_block("minecraft:decorated_pot")]
pub struct DecoratedPotBlock;

impl BlockBehaviour for DecoratedPotBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props =
                DecoratedPotLikeProperties::from_state_id(args.block.default_state.id, args.block);
            props.facing = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();
            props.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let entity = DecoratedPotBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        })
    }

    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if args.item_stack.item_count == 0 {
                return self
                    .normal_use(NormalUseArgs {
                        server: args.server,
                        world: args.world,
                        block: args.block,
                        position: args.position,
                        player: args.player,
                        hit: args.hit,
                    })
                    .await;
            }

            if let Some(block_entity) = args.world.get_block_entity(args.position)
                && let Some(pot_entity) = block_entity
                    .as_any()
                    .downcast_ref::<DecoratedPotBlockEntity>()
            {
                if pot_entity.try_insert_item(args.item_stack, 1).await {
                    args.world.play_sound(
                        Sound::BlockDecoratedPotInsert,
                        SoundCategory::Blocks,
                        &args.position.to_f64(),
                    );
                } else {
                    args.world.play_sound(
                        Sound::BlockDecoratedPotInsertFail,
                        SoundCategory::Blocks,
                        &args.position.to_f64(),
                    );
                }
                return BlockActionResult::Success;
            }

            BlockActionResult::Pass
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            args.world.play_sound(
                Sound::BlockDecoratedPotInsertFail,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );
            BlockActionResult::Success
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position)
                && let Some(pot_entity) = block_entity
                    .as_any()
                    .downcast_ref::<DecoratedPotBlockEntity>()
                && let Some(contained) = pot_entity.take_item().await
            {
                args.world.drop_stack(args.position, contained).await;
            }

            args.world.play_sound(
                Sound::BlockDecoratedPotShatter,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );
            args.world
                .drop_stack(args.position, ItemStack::new(4, &Item::BRICK))
                .await;
        })
    }

    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position)
                && let Some(pot_entity) = block_entity
                    .as_any()
                    .downcast_ref::<DecoratedPotBlockEntity>()
            {
                Some(pot_entity.get_comparator_output().await)
            } else {
                Some(0)
            }
        })
    }
}
