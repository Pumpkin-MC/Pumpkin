use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_data::BlockStateRef;
use pumpkin_data::block_properties::COLLISION_SHAPES;
use pumpkin_util::math::collision_shape::CollisionShape;

pub fn get_block(registry_id: &str) -> Option<Block> {
    let key = registry_id.replace("minecraft:", "");
    Block::from_registry_key(key.as_str())
}

pub fn get_block_by_id(id: u16) -> Option<Block> {
    Block::from_id(id)
}

pub fn get_state_by_id(id: u16) -> Option<BlockState> {
    if let Some(block) = Block::from_state_id(id) {
        let state: &BlockStateRef = block.states.iter().find(|state| state.id == id)?;
        Some(state.get_state())
    } else {
        None
    }
}

pub fn get_block_by_state_id(id: u16) -> Option<Block> {
    Block::from_state_id(id)
}

pub fn get_block_and_state_by_state_id(id: u16) -> Option<(Block, BlockState)> {
    if let Some(block) = Block::from_state_id(id) {
        let state: &BlockStateRef = block.states.iter().find(|state| state.id == id)?;
        Some((block, state.get_state()))
    } else {
        None
    }
}

pub fn get_block_by_item(item_id: u16) -> Option<Block> {
    Block::from_item_id(item_id)
}

pub fn get_block_collision_shapes(state_id: u16) -> Option<Vec<CollisionShape>> {
    let state = get_state_by_id(state_id)?;
    let mut shapes: Vec<CollisionShape> = Vec::with_capacity(state.collision_shapes.len());
    for i in 0..state.collision_shapes.len() {
        let shape = &COLLISION_SHAPES[state.collision_shapes[i] as usize];
        shapes.push(*shape);
    }
    Some(shapes)
}
