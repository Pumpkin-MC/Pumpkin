use super::BlockProperty;
use async_trait::async_trait;
use pumpkin_macros::block_property;

#[block_property("lit")]
pub struct Lit(bool);

#[async_trait]
impl BlockProperty for Lit {}
