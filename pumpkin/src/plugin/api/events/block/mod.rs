use std::sync::Arc;

use pumpkin_world::block::block_registry::Block;

use crate::entity::player::Player;

use super::CancellableEvent;

pub mod r#break;
pub mod burn;
pub mod can_build;
pub mod place;

pub trait BlockEvent: CancellableEvent {
    fn get_block(&self) -> &Block;
}

pub trait BlockExpEvent: BlockEvent {
    fn get_exp_to_drop(&self) -> u32;
    fn set_exp_to_drop(&mut self, exp: u32);
}

pub trait BlockBreakEvent: BlockExpEvent {
    fn get_player(&self) -> Option<Arc<Player>>;
    fn will_drop(&self) -> bool;
    fn set_drop(&mut self, drop: bool);
}

pub trait BlockBurnEvent: BlockEvent {
    fn get_igniting_block(&self) -> &Block;
}

pub trait BlockCanBuildEvent: BlockEvent {
    fn get_block_to_build(&self) -> &Block;
    fn is_buildable(&self) -> bool;
    fn set_buildable(&mut self, buildable: bool);
    fn get_player(&self) -> Option<Arc<Player>>;
}

pub trait BlockPlaceEvent: BlockEvent {
    fn get_player(&self) -> Option<Arc<Player>>;
    fn can_build(&self) -> bool;
    fn set_build(&mut self, build: bool);
    fn get_block_placed_against(&self) -> &Block;
    fn get_block_placed(&self) -> &Block;
}
