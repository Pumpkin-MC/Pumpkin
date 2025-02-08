use super::BlockProperty;
use async_trait::async_trait;
use pumpkin_macros::block_property;

#[block_property("open")]
pub enum Open {
    True,
    False,
}

#[async_trait]
impl BlockProperty for Open {}
