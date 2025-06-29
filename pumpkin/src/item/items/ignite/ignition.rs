use crate::block::blocks::fire::FireBlockBase;
use crate::block::blocks::fire::fire::FireBlock;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

pub struct Ignition;

impl Ignition {
    pub async fn ignite_block<F, Fut>(
        ignite_logic: F,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        block: &Block,
        _server: &Server,
    ) where
        F: FnOnce(Arc<World>, BlockPos, u16) -> Fut,
        Fut: Future<Output = ()>,
    {
        let world = player.world().await;
        let pos = location.offset(face.to_offset());

        if world.get_fluid(&location).await.name != Fluid::EMPTY.name {
            // if block is in fluid, do not ignite
            return;
        }
        let fire_block = FireBlockBase::get_fire_type(&world, &pos).await;

        let result_block_id = get_ignite_result(block, &world, &location)
            .await
            .unwrap_or(fire_block.default_state.id);

        let Some(result_block) = Block::from_state_id(result_block_id) else {
            return;
        };

        // checking by item_id because it always is similar
        let result_is_fire = fire_block.item_id == result_block.item_id;

        // TODO: create state direction for fire_block
        if result_is_fire {
            // calling if result is fire block.
            // will be contained fire direction logic
            let state_id = FireBlock.get_state_for_position(&world, block, &pos).await;
            if FireBlockBase::can_place_at(&world, &pos).await {
                ignite_logic(world, pos, state_id).await;
            }
            return;
        }

        // ignite candles, campfire
        ignite_logic(world, location, result_block_id).await;
    }
}

async fn get_ignite_result(block: &Block, world: &Arc<World>, location: &BlockPos) -> Option<u16> {
    let state_id = world.get_block_state_id(location).await;

    let original_props = match &block.properties(state_id) {
        Some(props) => props.to_props(),
        None => return None,
    };

    let props = original_props
        .iter()
        .map(|(key, value)| match (key.as_str(), value.as_str()) {
            ("extinguished", "true") => ("extinguished", "false"),
            ("lit", "false") => ("lit", "true"),
            _ => (key.as_str(), value.as_str()),
        })
        .collect();

    let new_state_id = block.from_properties(props).unwrap().to_state_id(block);

    (new_state_id != state_id).then_some(new_state_id)
}
