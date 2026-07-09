use crate::block::blocks::fire::FireBlockBase;
use crate::block::blocks::fire::fire::FireBlock;
use crate::entity::player::Player;
use crate::world::World;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{Block, BlockDirection, BlockStateId};
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
    ) -> bool
    where
        F: FnOnce(Arc<World>, BlockPos, BlockStateId) -> Fut,
        Fut: Future<Output = ()>,
    {
        let world = player.world();
        let pos = location.offset(face.to_offset());

        if world.get_fluid(&location).name != Fluid::EMPTY.name {
            return false;
        }
        let fire_block = FireBlockBase::get_fire_type(&world, &pos);

        let state_id = world.get_block_state_id(&location);

        if let Some(new_state_id) = can_be_lit(block, state_id) {
            ignite_logic(world.clone(), location, new_state_id).await;
            return true;
        }

        let state_id = FireBlock.get_state_for_position(&world, &fire_block, &pos);
        if FireBlockBase::can_place_at(&world, &pos) {
            ignite_logic(world.clone(), pos, state_id).await;
            return true;
        }

        false
    }
}

fn can_be_lit(block: &Block, state_id: BlockStateId) -> Option<BlockStateId> {
    let mut props = {
        let props = &block.properties(state_id)?;
        props.to_props()
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
