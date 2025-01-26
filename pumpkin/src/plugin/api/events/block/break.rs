use std::sync::Arc;

use pumpkin_world::block::block_registry::Block;

use crate::{
    entity::player::Player,
    plugin::{CancellableEvent, Event},
};

use super::{BlockBreakEvent, BlockEvent, BlockExpEvent};

pub struct BlockBreakEventImpl {
    player: Option<Arc<Player>>,
    block: Block,
    exp: u32,
    drop: bool,
    is_cancelled: bool,
}

impl BlockBreakEventImpl {
    #[must_use]
    pub fn new(player: Option<Arc<Player>>, block: Block, exp: u32, drop: bool) -> Self {
        Self {
            player,
            block,
            exp,
            drop,
            is_cancelled: false,
        }
    }
}

impl BlockBreakEvent for BlockBreakEventImpl {
    fn get_player(&self) -> Option<Arc<Player>> {
        self.player.clone()
    }

    fn will_drop(&self) -> bool {
        self.drop
    }

    fn set_drop(&mut self, drop: bool) {
        self.drop = drop;
    }
}

impl BlockExpEvent for BlockBreakEventImpl {
    fn get_exp_to_drop(&self) -> u32 {
        self.exp
    }

    fn set_exp_to_drop(&mut self, exp: u32) {
        self.exp = exp;
    }
}

impl BlockEvent for BlockBreakEventImpl {
    fn get_block(&self) -> &Block {
        &self.block
    }
}

impl CancellableEvent for BlockBreakEventImpl {
    fn is_cancelled(&self) -> bool {
        self.is_cancelled
    }

    fn set_cancelled(&mut self, cancelled: bool) {
        self.is_cancelled = cancelled;
    }
}

impl Event for BlockBreakEventImpl {
    fn get_name_static() -> &'static str {
        "BlockBreakEvent"
    }

    fn get_name(&self) -> &'static str {
        "BlockBreakEvent"
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
