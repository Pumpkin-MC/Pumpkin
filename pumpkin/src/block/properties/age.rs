use async_trait::async_trait;
use pumpkin_macros::block_property;

use super::BlockProperty;

// Those which requires custom names to values can be defined like this
#[block_property("age", [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])]
pub enum Age {
    Age0,
    Age1,
    Age2,
    Age3,
    Age4,
    Age5,
    Age6,
    Age7,
    Age8,
    Age9,
    Age10,
    Age11,
    Age12,
    Age13,
    Age14,
    Age15,
}

#[async_trait]
impl BlockProperty for Age {}
