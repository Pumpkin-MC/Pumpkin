use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::block::{Block, BlockState};
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_registry::SYNCED_REGISTRIES;
use pumpkin_util::math::position::BlockPos;

#[pumpkin_block("minecraft:jukebox")]
pub struct JukeboxBlock;

#[async_trait]
impl PumpkinBlock for JukeboxBlock {
    async fn normal_use(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        _server: &Server,
        _world: &World,
    ) {
        // For now just stop the music at this position
        let world = &player.living_entity.entity.world.read().await;

        world.stop_record(location).await;
    }

    async fn use_with_item(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        item: &Item,
        _server: &Server,
        _world: &World,
    ) -> BlockActionResult {
        let world = &player.living_entity.entity.world.read().await;

        let Some(jukebox_playable) = &item.components.jukebox_playable else {
            return BlockActionResult::Continue;
        };

        let Some(song) = jukebox_playable.song.split(':').nth(1) else {
            return BlockActionResult::Continue;
        };

        let Some(jukebox_song) = SYNCED_REGISTRIES.jukebox_song.get_index_of(song) else {
            log::error!("Jukebox playable song not registered!");
            return BlockActionResult::Continue;
        };

        //TODO: Update block state and block nbt

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
