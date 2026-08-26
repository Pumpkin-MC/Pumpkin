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
    BlockBehaviour, BrokenArgs, GetComparatorOutputArgs, NormalUseArgs, OnPlaceArgs, PlacedArgs,
    UseWithItemArgs,
};

#[pumpkin_block("minecraft:decorated_pot")]
pub struct DecoratedPotBlock;

impl BlockBehaviour for DecoratedPotBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props =
            DecoratedPotLikeProperties::from_state_id(args.block.default_state.id, args.block);
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = DecoratedPotBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        if args.item_stack.item_count == 0 {
            return self.normal_use(NormalUseArgs {
                server: args.server,
                world: args.world,
                block: args.block,
                position: args.position,
                player: args.player,
                hit: args.hit,
            });
        }

        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(pot_entity) = block_entity
                .as_any()
                .downcast_ref::<DecoratedPotBlockEntity>()
        {
            if pot_entity.try_insert_item(args.item_stack, 1) {
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
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        args.world.play_sound(
            Sound::BlockDecoratedPotInsertFail,
            SoundCategory::Blocks,
            &args.position.to_f64(),
        );
        BlockActionResult::Success
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(pot_entity) = block_entity
                .as_any()
                .downcast_ref::<DecoratedPotBlockEntity>()
            && let Some(contained) = pot_entity.take_item()
        {
            args.world.drop_stack(args.position, contained);
        }

        args.world.play_sound(
            Sound::BlockDecoratedPotShatter,
            SoundCategory::Blocks,
            &args.position.to_f64(),
        );
        args.world
            .drop_stack(args.position, ItemStack::new(4, &Item::BRICK));
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(pot_entity) = block_entity
                .as_any()
                .downcast_ref::<DecoratedPotBlockEntity>()
        {
            Some(pot_entity.get_comparator_output())
        } else {
            Some(0)
        }
    }
}
