use std::sync::Arc;

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{LadderLikeProperties, VaultLikeProperties, VaultState};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::entities::vault::VaultBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, NormalUseArgs, OnPlaceArgs, PlacedArgs, UseWithItemArgs};

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
        {
            let entity = VaultBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        }
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = VaultLikeProperties::from_state_id(state_id);

        if props.vault_state == VaultState::Ejecting || props.vault_state == VaultState::Unlocking {
            return BlockActionResult::Pass;
        }

        let item_id = args.item_stack.item.id;
        // Parity with vanilla 26.3:
        // An ominous vault requires an ominous trial key; a regular vault requires a regular trial key.
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
            return BlockActionResult::Success;
        }

        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(vault_entity) = block_entity.as_any().downcast_ref::<VaultBlockEntity>()
        {
            let player_uuid = args.player.gameprofile.id;

            if vault_entity.has_rewarded(&player_uuid) {
                args.world.play_sound(
                    Sound::BlockVaultRejectRewardedPlayer,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
                return BlockActionResult::Success;
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

            args.player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Used,
                item_id as i32,
                1,
            );
            if props.ominous {
                args.player.trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::Revaulting,
                );
            } else {
                args.player.trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::UnderLockAndKey,
                );
            }

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

            // Generate loot from Loot Table according to vanilla parity
            let loot_table_key = vault_entity.get_loot_table_key(props.ominous);
            let seed = (args.position.0.x as i64).wrapping_mul(3129871)
                ^ (args.position.0.z as i64).wrapping_mul(618791)
                ^ (args.position.0.y as i64)
                ^ rand::random::<i64>();

            let mut loot_stacks = Vec::new();
            if let Some(table_handle) = args.world.get_loot_table(&loot_table_key) {
                let mut params = crate::world::loot::LootContextParameters::default();
                params.position = Some(args.position.to_centered_f64());
                params.tool = Some(args.item_stack.clone());
                loot_stacks =
                    crate::world::loot::generate_loot_from_handle(&table_handle, seed, &params);
            }

            if loot_stacks.is_empty() {
                // Fallback trial vault loot
                loot_stacks = if props.ominous {
                    vec![
                        ItemStack::new(5, &Item::EMERALD),
                        ItemStack::new(2, &Item::DIAMOND),
                        ItemStack::new(3, &Item::IRON_INGOT),
                        ItemStack::new(8, &Item::WIND_CHARGE),
                    ]
                } else {
                    vec![
                        ItemStack::new(4, &Item::EMERALD),
                        ItemStack::new(1, &Item::DIAMOND),
                        ItemStack::new(2, &Item::IRON_INGOT),
                    ]
                };
            }

            for stack in loot_stacks {
                args.world.drop_stack(args.position, stack);
            }

            args.world.play_sound(
                Sound::BlockVaultCloseShutter,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );

            props.vault_state = VaultState::Active;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );

            return BlockActionResult::Success;
        }

        BlockActionResult::Pass
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        args.world.play_sound(
            Sound::BlockVaultInsertItemFail,
            SoundCategory::Blocks,
            &args.position.to_f64(),
        );
        BlockActionResult::Success
    }
}
