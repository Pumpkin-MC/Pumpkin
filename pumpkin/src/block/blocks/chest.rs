use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_inventory::{Chest, OpenContainer};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::{client::play::CBlockAction, codec::var_int::VarInt};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::{Block, get_block};

use crate::block::container::ContainerBlock;
use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    entity::player::Player,
    server::Server,
};
#[pumpkin_block("minecraft:chest")]
pub struct ChestBlock;

#[async_trait]
impl PumpkinBlock for ChestBlock {
    async fn normal_use(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
    ) {
        log::info!("hello opening");
        self.open(block, player, location, server).await;
        self.play_chest_action(player, location, server).await;
    }

    async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        _item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        log::info!("hello opening with use");
        self.open(block, player, location, server).await;
        BlockActionResult::Consume
    }

    async fn broken(&self, _block: &Block, player: &Player, location: BlockPos, server: &Server) {
        self.destroy(location, server, player).await;
    }

    async fn close(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
        _container: &mut OpenContainer,
    ) {
        self.play_chest_action(player, location, server).await;
        ContainerBlock::close(self, location, server, player).await;
    }
}

impl ChestBlock {
    pub async fn play_chest_action(&self, player: &Player, location: BlockPos, server: &Server) {
        let num_players = server
            .open_containers
            .read()
            .await
            .get_by_location(&location)
            .map(|container| container.all_player_ids().len())
            .unwrap_or_default();
        if num_players == 0 {
            player
                .world()
                .await
                .play_block_sound(Sound::BlockChestClose, SoundCategory::Blocks, location)
                .await;
        } else if num_players == 1 {
            player
                .world()
                .await
                .play_block_sound(Sound::BlockChestOpen, SoundCategory::Blocks, location)
                .await;
        }

        if let Some(e) = get_block("minecraft:chest").cloned() {
            server
                .broadcast_packet_all(&CBlockAction::new(
                    &location,
                    // TODO: BLOCK CLOSE ENUM HERE
                    1,
                    num_players as u8,
                    VarInt(e.id.into()),
                ))
                .await;
        }
    }
}

impl ContainerBlock<Chest> for ChestBlock {
    const UNIQUE: bool = false;
}
