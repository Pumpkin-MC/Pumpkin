use pumpkin_world::block::block_registry::Block;
use std::sync::Arc;

use crate::{
    entity::player::Player,
    plugin::{CancellableEvent, Event},
};

use super::{BlockCanBuildEvent, BlockEvent};

pub struct BlockCanBuildEventImpl {
    block_to_build: Block,
    buildable: bool,
    player: Arc<Player>,
    block: Block,
    is_cancelled: bool,
}

impl BlockCanBuildEvent for BlockCanBuildEventImpl {
    fn get_block_to_build(&self) -> &Block {
        &self.block_to_build
    }

    fn is_buildable(&self) -> bool {
        self.buildable
    }

    fn set_buildable(&mut self, buildable: bool) {
        self.buildable = buildable;
    }

    fn get_player(&self) -> Option<Arc<Player>> {
        Some(self.player.clone())
    }
}

impl BlockEvent for BlockCanBuildEventImpl {
    fn get_block(&self) -> &Block {
        &self.block
    }
}

impl CancellableEvent for BlockCanBuildEventImpl {
    fn is_cancelled(&self) -> bool {
        self.is_cancelled
    }

    fn set_cancelled(&mut self, cancelled: bool) {
        self.is_cancelled = cancelled;
    }
}

impl Event for BlockCanBuildEventImpl {
    fn get_name_static() -> &'static str {
        "BlockCanBuildEvent"
    }

    fn get_name(&self) -> &'static str {
        "BlockCanBuildEvent"
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
