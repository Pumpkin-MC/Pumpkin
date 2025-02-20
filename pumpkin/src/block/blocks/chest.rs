use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_data::{
    screen::WindowType,
    sound::{Sound, SoundCategory},
};
use pumpkin_inventory::{Chest, OpenContainer};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::{client::play::CBlockAction, codec::var_int::VarInt};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::{Block, get_block};

use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    entity::player::Player,
    server::Server,
};

#[derive(PartialEq)]
pub enum ChestState {
    IsOpened,
    IsClosed,
}

#[pumpkin_block("minecraft:chest")]
pub struct ChestBlock;

#[async_trait]
impl PumpkinBlock for ChestBlock {
    async fn normal_use(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        server: &Server,
    ) {
        self.open_chest_block(block, player, _location, server)
            .await;
    }

    async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        _item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        self.open_chest_block(block, player, _location, server)
            .await;
        BlockActionResult::Consume
    }

    async fn broken(&self, block: &Block, player: &Player, location: BlockPos, server: &Server) {
        super::standard_on_broken_with_container(block, player, location, server).await;
    }

    async fn close(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
        container: &mut OpenContainer,
    ) {
        container.remove_player(player.entity_id());

        self.play_chest_action(container, player, location, server, ChestState::IsClosed)
            .await;
    }
}

impl ChestBlock {
    pub async fn open_chest_block(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
    ) {
        // TODO: shouldn't Chest and window type be constrained together to avoid errors?
        super::standard_open_container::<Chest>(
            block,
            player,
            location,
            server,
            WindowType::Generic9x3,
        )
        .await;

        if let Some(container_id) = server.get_container_id(location, block.clone()).await {
            let open_containers = server.open_containers.read().await;
            if let Some(container) = open_containers.get(&u64::from(container_id)) {
                self.play_chest_action(container, player, location, server, ChestState::IsOpened)
                    .await;
            }
        }
    }

    pub async fn play_chest_action(
        &self,
        container: &OpenContainer,
        player: &Player,
        location: BlockPos,
        server: &Server,
        state: ChestState,
    ) {
        let num_players = container.get_number_of_players() as u8;
        if state == ChestState::IsClosed && num_players == 0 {
            player
                .world()
                .await
                .play_block_sound(Sound::BlockChestClose, SoundCategory::Blocks, location)
                .await;
        } else if state == ChestState::IsOpened && num_players == 1 {
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
                    1,
                    num_players,
                    VarInt(e.id.into()),
                ))
                .await;
        }
    }
}
