use pumpkin_world::block::block_registry::Block;
use std::sync::Arc;

use crate::{
    entity::player::Player,
    plugin::{CancellableEvent, Event},
};

use super::{BlockEvent, BlockPlaceEvent};

pub struct BlockPlaceEventImpl {
    player: Arc<Player>,
    block_placed: Block,
    block_placed_against: Block,
    can_build: bool,
    is_cancelled: bool,
}

impl BlockPlaceEvent for BlockPlaceEventImpl {
    fn get_player(&self) -> Option<Arc<Player>> {
        Some(self.player.clone())
    }

    fn can_build(&self) -> bool {
        self.can_build
    }

    fn set_build(&mut self, build: bool) {
        self.can_build = build;
    }

    fn get_block_placed_against(&self) -> &Block {
        &self.block_placed_against
    }

    fn get_block_placed(&self) -> &Block {
        &self.block_placed
    }
}

impl BlockEvent for BlockPlaceEventImpl {
    fn get_block(&self) -> &Block {
        &self.block_placed
    }
}

impl CancellableEvent for BlockPlaceEventImpl {
    fn is_cancelled(&self) -> bool {
        self.is_cancelled
    }

    fn set_cancelled(&mut self, cancelled: bool) {
        self.is_cancelled = cancelled;
    }
}

impl Event for BlockPlaceEventImpl {
    fn get_name_static() -> &'static str {
        "BlockPlaceEvent"
    }

    fn get_name(&self) -> &'static str {
        "BlockPlaceEvent"
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
