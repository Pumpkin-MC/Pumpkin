use pumpkin_world::block::block_registry::Block;

use crate::plugin::{CancellableEvent, Event};

use super::{BlockBurnEvent, BlockEvent};

pub struct BlockBurnEventImpl {
    igniting_block: Block,
    block: Block,
    is_cancelled: bool,
}

impl BlockBurnEvent for BlockBurnEventImpl {
    fn get_igniting_block(&self) -> &Block {
        &self.igniting_block
    }
}

impl BlockEvent for BlockBurnEventImpl {
    fn get_block(&self) -> &Block {
        &self.block
    }
}

impl CancellableEvent for BlockBurnEventImpl {
    fn is_cancelled(&self) -> bool {
        self.is_cancelled
    }

    fn set_cancelled(&mut self, cancelled: bool) {
        self.is_cancelled = cancelled;
    }
}

impl Event for BlockBurnEventImpl {
    fn get_name_static() -> &'static str {
        "BlockBurnEvent"
    }

    fn get_name(&self) -> &'static str {
        "BlockBurnEvent"
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
