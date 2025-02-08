use pumpkin_macros::block_property;

use super::BlockProperty;

#[block_property("powered")]
pub enum Powered {
    True,
    False,
}

impl BlockProperty for Powered {}
