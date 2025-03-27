use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::{BlockFlags, World};
use async_trait::async_trait;
use pumpkin_data::block::{Block, BlockProperties, BlockState, Boolean, JukeboxLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_registry::SYNCED_REGISTRIES;
use pumpkin_util::math::position::BlockPos;

#[pumpkin_block("minecraft:jukebox")]
pub struct JukeboxBlock;

impl JukeboxBlock {
    async fn has_record(&self, block: &Block, location: BlockPos, world: &World) -> bool {
        let state_id = world
            .get_block_state(&location)
            .await
            .expect("`location` should be a jukebox")
            .id;
        JukeboxLikeProperties::from_state_id(state_id, block).has_record == Boolean::True
    }

    async fn set_record(
        &self,
        has_record: bool,
        block: &Block,
        location: BlockPos,
        world: &Arc<World>,
    ) {
        let new_state = JukeboxLikeProperties {
            has_record: if has_record {
                Boolean::True
            } else {
                Boolean::False
            },
        };
        world
            .set_block_state(&location, new_state.to_state_id(block), BlockFlags::empty())
            .await;
    }

    async fn stop_music(&self, block: &Block, location: BlockPos, world: &Arc<World>) {
        self.set_record(false, block, location, world).await;
        world.stop_record(location).await;
    }
}

#[async_trait]
impl PumpkinBlock for JukeboxBlock {
    async fn normal_use(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        _server: &Server,
        _world: &Arc<World>,
    ) {
        // For now just stop the music at this position
        let world = &player.living_entity.entity.world.read().await;
        self.stop_music(block, location, world).await;
    }

    async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        item: &Item,
        _server: &Server,
        _world: &Arc<World>,
    ) -> BlockActionResult {
        let world = &player.living_entity.entity.world.read().await;

        // if the jukebox already has a record, stop playing
        if self.has_record(block, location, world).await {
            self.stop_music(block, location, world).await;
            return BlockActionResult::Consume;
        }

        let Some(jukebox_playable) = &item.components.jukebox_playable else {
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

        self.set_record(true, block, location, world).await;
        world.play_record(jukebox_song as i32, location).await;

        BlockActionResult::Consume
    }

    async fn broken(
        &self,
        _block: &Block,
        _player: &Player,
        location: BlockPos,
        _server: &Server,
        world: Arc<World>,
        _state: BlockState,
    ) {
        // For now just stop the music at this position
        world.stop_record(location).await;
    }
}
