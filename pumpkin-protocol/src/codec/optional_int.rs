use serde::{Serialize, Serializer};

use crate::codec::var_int::VarInt;

#[derive(Debug, Clone, Copy)]
pub struct OptionalInt(pub Option<i32>);

impl Serialize for OptionalInt {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Vanilla logic: empty is 0, otherwise value + 1
        let val = match self.0 {
            None => 0,
            Some(id) => id + 1,
        };
        VarInt(val).serialize(serializer)
    }
}
