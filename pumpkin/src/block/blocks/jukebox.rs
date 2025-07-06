use std::sync::Arc;

use crate::block::pumpkin_block::{BrokenArgs, NormalUseArgs, PumpkinBlock, UseWithItemArgs};
use crate::block::registry::BlockActionResult;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, JukeboxLikeProperties},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_registry::SYNCED_REGISTRIES;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block("minecraft:jukebox")]
pub struct JukeboxBlock;

impl JukeboxBlock {
    async fn has_record(&self, block: &Block, location: &BlockPos, world: &World) -> bool {
        let state_id = world.get_block_state(location).await.id;
        JukeboxLikeProperties::from_state_id(state_id, block).has_record
    }

    async fn set_record(
        &self,
        has_record: bool,
        block: &Block,
        location: &BlockPos,
        world: &Arc<World>,
    ) {
        let new_state = JukeboxLikeProperties { has_record };
        world
            .set_block_state(location, new_state.to_state_id(block), BlockFlags::empty())
            .await;
    }

    async fn stop_music(&self, block: &Block, position: &BlockPos, world: &Arc<World>) {
        self.set_record(false, block, position, world).await;
        world
            .sync_world_event(WorldEvent::JukeboxStopsPlaying, *position, 0)
            .await;
    }
}

#[async_trait]
impl PumpkinBlock for JukeboxBlock {
    async fn normal_use(&self, args: NormalUseArgs<'_>) {
        // For now just stop the music at this position
        let world = &args.player.living_entity.entity.world.read().await;
        self.stop_music(args.block, args.location, world).await;
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let world = &args.player.living_entity.entity.world.read().await;

        // if the jukebox already has a record, stop playing
        if self.has_record(args.block, args.location, world).await {
            self.stop_music(args.block, args.location, world).await;
            return BlockActionResult::Consume;
        }

        let Some(jukebox_playable) = &args
            .item_stack
            .lock()
            .await
            .item
            .components
            .jukebox_playable
        else {
            return BlockActionResult::Continue;
        };

        let Some(song) = jukebox_playable.split(':').nth(1) else {
            return BlockActionResult::Continue;
        };

        let Some(jukebox_song) = SYNCED_REGISTRIES.jukebox_song.get_index_of(song) else {
            log::error!("Jukebox playable song not registered!");
            return BlockActionResult::Continue;
        };

        //TODO: Update block nbt

        self.set_record(true, args.block, args.location, world)
            .await;
        world
            .sync_world_event(
                WorldEvent::JukeboxStartsPlaying,
                *args.location,
                jukebox_song as i32,
            )
            .await;

        BlockActionResult::Consume
    }

    async fn broken(&self, args: BrokenArgs<'_>) {
        // For now just stop the music at this position
        args.world
            .sync_world_event(WorldEvent::JukeboxStopsPlaying, *args.location, 0)
            .await;
    }
}
