use crate::block::blocks::fire::FireBlockBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::item::Item;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

pub struct Ignition {}

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

        let fire_block = FireBlockBase::get_fire_type(&world, &pos).await;

        let result_block_id = get_ignite_result(block, &world, &location)
            .await
            .unwrap_or(fire_block.default_state_id);

        let result_block = match Block::from_state_id(result_block_id) {
            Some(block) => block,
            _ => return,
        };

        // checking by item_id because it always is similar
        let result_is_fire = fire_block.item_id == result_block.item_id;

        // TODO: create state direction for fire_block
        if result_is_fire {
            // calling if result is fire block.
            // will be contained fire direction logic
            if FireBlockBase::can_place_at(world.as_ref(), &pos).await {
                ignite_logic(world, pos, result_block_id).await;
            }
            return;
        }

        // ignite candles, campfire
        ignite_logic(world, location, result_block_id).await;
    }

    pub async fn run_fire_distribute(_world: Arc<World>, _start_pos: &BlockPos) {
        tokio::spawn(async move {
            // todo
        });
    }
}

async fn get_ignite_result(block: &Block, world: &Arc<World>, location: &BlockPos) -> Option<u16> {
    let state_id = world.get_block_state_id(location).await;

    let original_props = match &block.properties(state_id) {
        Some(props) => props.to_props(),
        None => return None,
    };

    let mut props_vec: Vec<(&str, &str)> = Vec::with_capacity(original_props.len());
    for (key, _value) in &original_props {
        if key == "extinguished" {
            // campfire
            props_vec.push((key.as_str(), "true"));
        } else if key == "lit" {
            // candles
            props_vec.push((key.as_str(), "true"));
        }
    }

    let new_state_id = block.from_properties(props_vec).unwrap().to_state_id(block);

    if new_state_id != state_id {
        Some(new_state_id)
    } else {
        None
    }
}
