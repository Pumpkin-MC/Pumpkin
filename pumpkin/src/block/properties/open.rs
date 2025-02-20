use super::BlockProperty;
use async_trait::async_trait;
use pumpkin_macros::block_property;

#[block_property("open")]
pub struct Open(bool);

#[async_trait]
impl BlockProperty for Open {}
