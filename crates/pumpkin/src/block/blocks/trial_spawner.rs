use std::sync::Arc;

use pumpkin_data::block_properties::{
    BlockProperties, TrialSpawnerLikeProperties, TrialSpawnerState,
};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::entities::trial_spawner::TrialSpawnerBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, NormalUseArgs, PlacedArgs, UseWithItemArgs};

#[pumpkin_block("minecraft:trial_spawner")]
pub struct TrialSpawnerBlock;

impl BlockBehaviour for TrialSpawnerBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = TrialSpawnerBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = TrialSpawnerLikeProperties::from_state_id(state_id, args.block);

        match props.trial_spawner_state {
            TrialSpawnerState::Inactive | TrialSpawnerState::WaitingForPlayers => {
                props.trial_spawner_state = TrialSpawnerState::Active;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );

                args.world.play_sound(
                    Sound::BlockTrialSpawnerDetectPlayer,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
                args.world.play_sound(
                    Sound::BlockTrialSpawnerOpenShutter,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
            }
            TrialSpawnerState::Active => {
                // Eject trial rewards & key drop upon trial wave completion
                props.trial_spawner_state = TrialSpawnerState::WaitingForRewardEjection;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );

                args.world.play_sound(
                    Sound::BlockTrialSpawnerSpawnItemBegin,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
                args.world.play_sound(
                    Sound::BlockTrialSpawnerEjectItem,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );

                let key_stack = ItemStack::new(1, &Item::TRIAL_KEY);
                args.world.drop_stack(args.position, key_stack);

                props.trial_spawner_state = TrialSpawnerState::Cooldown;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );

                args.world.play_sound(
                    Sound::BlockTrialSpawnerCloseShutter,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
            }
            TrialSpawnerState::Cooldown => {
                args.world.play_sound(
                    Sound::BlockTrialSpawnerAmbient,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
            }
            TrialSpawnerState::WaitingForRewardEjection | TrialSpawnerState::EjectingReward => {
                props.trial_spawner_state = TrialSpawnerState::Cooldown;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
            }
        }

        BlockActionResult::Success
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        self.normal_use(NormalUseArgs {
            server: args.server,
            world: args.world,
            block: args.block,
            position: args.position,
            player: args.player,
            hit: args.hit,
        })
    }
}
