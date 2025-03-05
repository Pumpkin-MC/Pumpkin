use crate::block::properties::Direction;
use crate::entity::player::Player;
use crate::{block::registry::BlockActionResult, world::World};
use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_data::screen::WindowType;
use pumpkin_inventory::Furnace;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::registry::Block;

use crate::{block::pumpkin_block::PumpkinBlock, server::Server};

#[pumpkin_block("minecraft:furnace")]
pub struct FurnaceBlock;

#[async_trait]
impl PumpkinBlock for FurnaceBlock {
    async fn on_place(
        &self,
        server: &Server,
        world: &World,
        block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        use_item_on: &SUseItemOn,
        player_direction: &Direction,
        other: bool,
    ) -> u16 {
        let player_direction = player_direction.opposite();

        server
            .block_properties_manager
            .on_place_state(
                world,
                block,
                face,
                block_pos,
                use_item_on,
                &player_direction,
                other,
            )
            .await
    }

    async fn normal_use(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        server: &Server,
    ) {
        self.open_furnace_screen(block, player, _location, server)
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
        self.open_furnace_screen(block, player, _location, server)
            .await;
        BlockActionResult::Consume
    }

    async fn broken(&self, block: &Block, player: &Player, location: BlockPos, server: &Server) {
        super::standard_on_broken_with_container(block, player, location, server).await;
    }
}

impl FurnaceBlock {
    pub async fn open_furnace_screen(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
    ) {
        super::standard_open_container::<Furnace>(
            block,
            player,
            location,
            server,
            WindowType::Furnace,
        )
        .await;
    }
}
