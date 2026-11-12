use std::sync::Arc;

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{LadderLikeProperties, VaultLikeProperties, VaultState};
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::entities::vault::VaultBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, OnPlaceArgs, PlacedArgs, UseWithItemArgs};
use crate::entity::EntityBase;
use crate::world::loot::{LootContextParameters, generate_loot_from_handle};

#[pumpkin_block("minecraft:vault")]
pub struct VaultBlock;

impl BlockBehaviour for VaultBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = LadderLikeProperties::default(args.block);
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = VaultBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        if args.item_stack.is_empty() {
            return BlockActionResult::PassToDefaultBlockAction;
        }
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = VaultLikeProperties::from_state_id(state_id);
        if props.vault_state != VaultState::Active {
            return BlockActionResult::PassToDefaultBlockAction;
        }

        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return BlockActionResult::PassToDefaultBlockAction;
        };
        let Some(vault_entity) = block_entity.as_any().downcast_ref::<VaultBlockEntity>() else {
            return BlockActionResult::PassToDefaultBlockAction;
        };

        let item_id = args.item_stack.item.id;
        let is_valid_key = if props.ominous {
            item_id == Item::OMINOUS_TRIAL_KEY.id
        } else {
            item_id == Item::TRIAL_KEY.id
        };

        if !is_valid_key {
            args.world.play_sound(
                Sound::BlockVaultInsertItemFail,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );
            return BlockActionResult::SuccessServer;
        }

        let player_uuid = args.player.gameprofile.id;

        if vault_entity.has_rewarded(&player_uuid) {
            args.world.play_sound(
                Sound::BlockVaultRejectRewardedPlayer,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );
            return BlockActionResult::SuccessServer;
        }

        vault_entity.mark_rewarded(player_uuid);

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

        props.vault_state = VaultState::Ejecting;
        args.world.set_block_state(
            args.position,
            props.to_state_id(args.block),
            BlockFlags::NOTIFY_ALL,
        );

        args.world.play_sound(
            Sound::BlockVaultEjectItem,
            SoundCategory::Blocks,
            &args.position.to_f64(),
        );

        let custom_loot_table = vault_entity
            .config
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(|cfg| cfg.get_string("loot_table").map(str::to_owned));
        let default_loot_table = if props.ominous {
            "minecraft:chests/trial_chambers/reward_ominous"
        } else {
            "minecraft:chests/trial_chambers/reward"
        };
        let loot_table_key = custom_loot_table.as_deref().unwrap_or(default_loot_table);
        let loot_stacks = args
            .world
            .get_loot_table(loot_table_key)
            .map(|handle| {
                let params = LootContextParameters {
                    world: Some(args.world),
                    position: Some(args.position.to_centered_f64()),
                    this_entity: Some(args.player.as_ref() as &dyn EntityBase),
                    ..Default::default()
                };
                generate_loot_from_handle(&handle, rand::random(), &params)
            })
            .unwrap_or_default();

        for stack in loot_stacks {
            args.world.drop_stack(args.position, stack);
        }

        props.vault_state = VaultState::Active;
        args.world.set_block_state(
            args.position,
            props.to_state_id(args.block),
            BlockFlags::NOTIFY_ALL,
        );

        BlockActionResult::SuccessServer
    }
}
