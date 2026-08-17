use pumpkin_codecs::{
    DataResult, Decode, DynamicOps, Encode, MapLike, struct_builder::StructBuilder as _,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorldClock;

impl Encode for WorldClock {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        ops.map_builder().build(prefix)
    }
}

impl Decode for WorldClock {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            if map.iter().next().is_some() {
                DataResult::new_error("world clock definition must be an empty object")
            } else {
                DataResult::new_success((Self, ops.empty()))
            }
        })
    }
}
