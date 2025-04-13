//pub mod tag;

pub mod item {
    include!(concat!(env!("OUT_DIR"), "/item.rs"));
}

pub mod packet {
    include!(concat!(env!("OUT_DIR"), "/packet.rs"));
}

pub mod screen {
    include!(concat!(env!("OUT_DIR"), "/screen.rs"));
}

pub mod particle {
    include!(concat!(env!("OUT_DIR"), "/particle.rs"));
}

pub mod sound {
    include!(concat!(env!("OUT_DIR"), "/sound.rs"));
    include!(concat!(env!("OUT_DIR"), "/sound_category.rs"));
}

pub mod chunk {
    include!(concat!(env!("OUT_DIR"), "/biome.rs"));
    include!(concat!(env!("OUT_DIR"), "/noise_parameter.rs"));
    include!(concat!(env!("OUT_DIR"), "/chunk_status.rs"));
}

pub mod game_event {
    include!(concat!(env!("OUT_DIR"), "/game_event.rs"));
}

pub mod entity {
    include!(concat!(env!("OUT_DIR"), "/entity_status.rs"));
    include!(concat!(env!("OUT_DIR"), "/status_effect.rs"));
    include!(concat!(env!("OUT_DIR"), "/spawn_egg.rs"));
    include!(concat!(env!("OUT_DIR"), "/entity_type.rs"));
    include!(concat!(env!("OUT_DIR"), "/entity_pose.rs"));
}

pub mod world {
    include!(concat!(env!("OUT_DIR"), "/world_event.rs"));
    include!(concat!(env!("OUT_DIR"), "/message_type.rs"));
}

pub mod scoreboard {
    include!(concat!(env!("OUT_DIR"), "/scoreboard_slot.rs"));
}

pub mod damage {
    include!(concat!(env!("OUT_DIR"), "/damage_type.rs"));
}

pub mod fluid {
    include!(concat!(env!("OUT_DIR"), "/fluid.rs"));
}

pub mod block {
    include!(concat!(env!("OUT_DIR"), "/block.rs"));

    pub fn get_block(registry_id: &str) -> Option<Block> {
        let key = registry_id.replace("minecraft:", "");
        Block::from_registry_key(key.as_str())
    }

    pub fn get_block_by_id(id: u16) -> Option<Block> {
        Block::from_id(id)
    }

    pub fn get_state_by_state_id(id: u16) -> Option<BlockState> {
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
        let state = get_state_by_state_id(state_id)?;
        let mut shapes: Vec<CollisionShape> = vec![];
        for i in 0..state.collision_shapes.len() {
            let shape = &COLLISION_SHAPES[state.collision_shapes[i] as usize];
            shapes.push(*shape);
        }
        Some(shapes)
    }

    pub fn blocks_movement(block_state: &BlockState) -> bool {
        if block_state.is_solid {
            if let Some(block) = get_block_by_state_id(block_state.id) {
                return block != Block::COBWEB && block != Block::BAMBOO_SAPLING;
            }
        }
        false
    }
}

pub mod tag {
    include!(concat!(env!("OUT_DIR"), "/tag.rs"));
}

pub mod noise_router {
    include!(concat!(env!("OUT_DIR"), "/noise_router.rs"));
}
