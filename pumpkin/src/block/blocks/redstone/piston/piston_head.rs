use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block::{Block, BlockProperties, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{block::FacingExt, world::BlockFlags};

use crate::entity::player::Player;
use crate::{
    block::pumpkin_block::{BlockMetadata, PumpkinBlock},
    world::World,
};

use super::piston::{PistonBlock, PistonProps};

type MovingPistonProps = pumpkin_data::block::MovingPistonLikeProperties;

use crate::server::Server;
#[pumpkin_block("minecraft:piston_head")]
pub struct PistonHeadBlock;

#[async_trait]
impl PumpkinBlock for PistonHeadBlock {}
