use std::sync::Arc;

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{
    BlockProperties, LadderLikeProperties, VaultLikeProperties, VaultState,
};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::entities::vault::VaultBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, NormalUseArgs, OnPlaceArgs, PlacedArgs, UseWithItemArgs,
};

#[pumpkin_block("minecraft:vault")]
pub struct VaultBlock;

impl BlockBehaviour for VaultBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = LadderLikeProperties::default(args.block);
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
            let entity = VaultBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        })
    }

    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let item_id = args.item_stack.item.id;
            let is_trial_key =
                item_id == Item::TRIAL_KEY.id || item_id == Item::OMINOUS_TRIAL_KEY.id;

            if !is_trial_key {
                args.world.play_sound(
                    Sound::BlockVaultInsertItemFail,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
                return BlockActionResult::Success;
            }

            if let Some(block_entity) = args.world.get_block_entity(args.position)
                && let Some(vault_entity) = block_entity.as_any().downcast_ref::<VaultBlockEntity>()
            {
                let player_uuid = args.player.gameprofile.id;

                if vault_entity.has_rewarded(&player_uuid).await {
                    args.world.play_sound(
                        Sound::BlockVaultRejectRewardedPlayer,
                        SoundCategory::Blocks,
                        &args.position.to_f64(),
                    );
                    return BlockActionResult::Success;
                }

                vault_entity.mark_rewarded(player_uuid).await;

                args.item_stack
                    .decrement_unless_creative(args.player.gamemode.load(), 1);

                args.world.play_sound(
                    Sound::BlockVaultInsertItem,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
                args.world.play_sound(
                    Sound::BlockVaultOpenShutter,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );

                let state_id = args.world.get_block_state_id(args.position);
                let mut props = VaultLikeProperties::from_state_id(state_id, args.block);
                props.vault_state = VaultState::Ejecting;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;

                args.world.play_sound(
                    Sound::BlockVaultEjectItem,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );

                // Spawn trial vault loot (emeralds, diamond, iron)
                let loot_stacks = vec![
                    ItemStack::new(4, &Item::EMERALD),
                    ItemStack::new(1, &Item::DIAMOND),
                    ItemStack::new(2, &Item::IRON_INGOT),
                ];

                for stack in loot_stacks {
                    args.world.drop_stack(args.position, stack).await;
                }

                props.vault_state = VaultState::Active;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;

                return BlockActionResult::Success;
            }

            BlockActionResult::Pass
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            args.world.play_sound(
                Sound::BlockVaultInsertItemFail,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );
            BlockActionResult::Success
        })
    }
}
