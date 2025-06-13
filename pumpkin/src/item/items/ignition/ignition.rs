use crate::block::blocks::fire::FireBlockBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

pub struct Ignition {}

impl Ignition {
    pub async fn ignite_block<F>(
        ignite_logic: F,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        block: &Block,
        _server: &Server,
    ) where
        F: FnOnce(Arc<World>, BlockPos, &Option<Block>),
    {
        // TODO: check CampfireBlock, CandleBlock and CandleCakeBlock

        let world = player.world().await;
        let pos = location.offset(face.to_offset());

        let fire_block = FireBlockBase::get_fire_type(&world, &pos).await;

        let replacement_block = get_ignite_result(block).unwrap_or(fire_block.id);

        let result_block = &Block::from_id(replacement_block);

        if FireBlockBase::can_place_at(world.as_ref(), &pos).await {
            ignite_logic(world, pos, result_block);
        }
    }
}

fn get_ignite_result(block: &Block) -> Option<u16> {
    match &block.id {
        id if id == &Block::CAMPFIRE
            .is_tagged_with("#minecraft:extinguished")?
            .then_some(Block::CAMPFIRE)?.id => Some(Block::CAMPFIRE.id),

        _ => None,
    }
}