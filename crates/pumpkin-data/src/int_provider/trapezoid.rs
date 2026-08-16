use std::any::Any;

use pumpkin_codecs::{
    DataResult, Decode, DynamicOps, Encode,
    codec::{FieldDecode, FieldEncode},
    struct_builder::StructBuilder as _,
};
use pumpkin_util::{
    identifier::Identifier,
    random::{RandomGenerator, RandomImpl},
};

use super::{IntProvider, Pipe};

#[derive(Clone, Debug)]
pub struct TrapezoidIntProvider {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
    pub plateau: i32,
}

impl TrapezoidIntProvider {
    #[must_use]
    pub const fn new(min_inclusive: i32, max_inclusive: i32, plateau: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
            plateau,
        }
    }
}

impl IntProvider for TrapezoidIntProvider {
    fn get_min(&self) -> i32 {
        self.min_inclusive
    }
    fn get(&self, random: &mut RandomGenerator) -> i32 {
        if self.plateau == 0 && self.max_inclusive == -self.min_inclusive {
            return random.next_bounded_i32(self.max_inclusive + 1)
                - random.next_bounded_i32(self.max_inclusive + 1);
        }
        let range = self.max_inclusive - self.min_inclusive;
        if self.plateau == range {
            return random.next_bounded_i32(range + 1) + self.min_inclusive;
        }
        let plateau_start = (range - self.plateau) / 2;
        let plateau_end = range - plateau_start;
        self.min_inclusive
            + random.next_bounded_i32(plateau_end + 1)
            + random.next_bounded_i32(plateau_start + 1)
    }
    fn get_max(&self) -> i32 {
        self.max_inclusive
    }
    fn provider_type(&self) -> Identifier {
        Identifier::vanilla_static("trapezoid")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Encode for TrapezoidIntProvider {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.min_inclusive
            .encode_field("min", ops, ops.map_builder())
            .pipe(|builder| self.max_inclusive.encode_field("max", ops, builder))
            .pipe(|builder| self.plateau.encode_field("plateau", ops, builder))
            .build(prefix)
    }
}

impl Decode for TrapezoidIntProvider {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            i32::decode_field::<O>("min", &map, ops).apply_3(
                |min_inclusive, max_inclusive, plateau| {
                    (
                        Self {
                            min_inclusive,
                            max_inclusive,
                            plateau,
                        },
                        ops.empty(),
                    )
                },
                i32::decode_field::<O>("max", &map, ops),
                i32::decode_field::<O>("plateau", &map, ops),
            )
        })
    }
}
