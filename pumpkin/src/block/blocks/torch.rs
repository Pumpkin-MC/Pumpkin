use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry;
use pumpkin_world::block::{BlockDirection, registry::Block};

use crate::block::blocks::vertical_attachment::VerticalAttachment;
use crate::{
    block::{properties::Direction, pumpkin_block::PumpkinBlock},
    server::Server,
    world::World,
};

#[pumpkin_block("minecraft:wall_torch")]
pub struct TorchBlock;

#[async_trait]
impl PumpkinBlock for TorchBlock {
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
        VerticalAttachment::on_place(
            self,
            server,
            world,
            block,
            face,
            block_pos,
            use_item_on,
            player_direction,
            other,
        )
        .await
    }
}

impl VerticalAttachment for TorchBlock {
    fn get_standing_block(&self) -> &'static Block {
        registry::get_block("minecraft:torch").unwrap()
    }
}

#[pumpkin_block("minecraft:redstone_wall_torch")]
pub struct RedstoneTorchBlock;

#[async_trait]
impl PumpkinBlock for RedstoneTorchBlock {
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
        VerticalAttachment::on_place(
            self,
            server,
            world,
            block,
            face,
            block_pos,
            use_item_on,
            player_direction,
            other,
        )
        .await
    }
}

impl VerticalAttachment for RedstoneTorchBlock {
    fn get_standing_block(&self) -> &'static Block {
        registry::get_block("minecraft:redstone_torch").unwrap()
    }
}

#[pumpkin_block("minecraft:soul_wall_torch")]
pub struct SoulTorchBlock;

#[async_trait]
impl PumpkinBlock for SoulTorchBlock {
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
        VerticalAttachment::on_place(
            self,
            server,
            world,
            block,
            face,
            block_pos,
            use_item_on,
            player_direction,
            other,
        )
        .await
    }
}

impl VerticalAttachment for SoulTorchBlock {
    fn get_standing_block(&self) -> &'static Block {
        registry::get_block("minecraft:soul_torch").unwrap()
    }
}
