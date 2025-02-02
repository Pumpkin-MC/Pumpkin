use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;
use pumpkin_registry::SYNCED_REGISTRIES;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;
use pumpkin_world::item::registry::Item;

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
    ) {
        // For now just stop the music at this position
        let world = &player.living_entity.entity.world;

        world.stop_record(location).await;
    }

    async fn use_with_item(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        item: &Item,
        _server: &Server,
    ) -> BlockActionResult {
        let world = &player.living_entity.entity.world;

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

    async fn broken(&self, _block: &Block, player: &Player, location: BlockPos, _server: &Server) {
        // For now just stop the music at this position
        let world = &player.living_entity.entity.world;

        world.stop_record(location).await;
    }
}
