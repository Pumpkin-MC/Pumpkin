use crate::block::blocks::fire::FireBlockBase;
use crate::block::blocks::fire::fire::FireBlock;
use crate::entity::player::Player;
use crate::world::World;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

pub struct Ignition;

impl Ignition {
    pub async fn ignite_block<F, Fut>(
        ignite_logic: F,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        block: &Block,
    ) where
        F: FnOnce(Arc<World>, BlockPos, u16) -> Fut,
        Fut: Future<Output = ()>,
    {
        let world = player.world();
        let pos = location.offset(face.to_offset());

        if world.get_fluid(&location).await.name != Fluid::EMPTY.name {
            return;
        }
        let fire_block = FireBlockBase::get_fire_type(world, &pos).await;

        let state_id = world.get_block_state_id(&location).await;

        if let Some(new_state_id) = can_be_lit(block, state_id) {
            ignite_logic(world.clone(), location, new_state_id).await;
            return;
        }

        let state_id = FireBlock
            .get_state_for_position(world, &fire_block, &pos)
            .await;
        if FireBlockBase::can_place_at(world, &pos).await {
            ignite_logic(world.clone(), pos, state_id).await;
        }
    }
}

fn can_be_lit(block: &Block, state_id: u16) -> Option<u16> {
    let mut props = match &block.properties(state_id) {
        Some(props) => props.to_props(),
        None => return None,
    };

    if let Some((_, value)) = props.iter_mut().find(|(k, _)| *k == "extinguished") {
        *value = "false";
    } else if let Some((_, value)) = props.iter_mut().find(|(k, _)| *k == "lit") {
        *value = "true";
    } else {
        return None;
    }

    let new_state_id = block.from_properties(&props).to_state_id(block);

    (new_state_id != state_id).then_some(new_state_id)
}
