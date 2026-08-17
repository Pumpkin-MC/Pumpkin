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

use super::{IntProvider, IntProviderValue, Pipe};

#[derive(Clone, Debug)]
pub struct WeightedEntry {
    pub data: IntProviderValue,
    pub weight: i32,
}

impl Encode for WeightedEntry {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.data
            .encode_field("data", ops, ops.map_builder())
            .pipe(|builder| self.weight.encode_field("weight", ops, builder))
            .build(prefix)
    }
}

impl Decode for WeightedEntry {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            IntProviderValue::decode_field::<O>("data", &map, ops).apply_2(
                |data, weight| (Self { data, weight }, ops.empty()),
                i32::decode_field::<O>("weight", &map, ops),
            )
        })
    }
}

#[derive(Clone, Debug)]
pub struct WeightedListIntProvider {
    pub distribution: Vec<WeightedEntry>,
}

impl WeightedListIntProvider {
    #[must_use]
    pub const fn new(distribution: Vec<WeightedEntry>) -> Self {
        Self { distribution }
    }
}

impl IntProvider for WeightedListIntProvider {
    fn get_min(&self) -> i32 {
        self.distribution
            .iter()
            .map(|entry| entry.data.get_min())
            .min()
            .unwrap_or(0)
    }

    fn get(&self, random: &mut RandomGenerator) -> i32 {
        if self.distribution.is_empty() {
            return 0;
        }
        let total_weight: i32 = self.distribution.iter().map(|entry| entry.weight).sum();
        if total_weight <= 0 {
            return 0;
        }
        let chosen_weight = random.next_bounded_i32(total_weight);
        let mut current_weight = 0;
        for entry in &self.distribution {
            current_weight += entry.weight;
            if chosen_weight < current_weight {
                return entry.data.get(random);
            }
        }
        self.distribution
            .last()
            .map_or(0, |entry| entry.data.get(random))
    }

    fn get_max(&self) -> i32 {
        self.distribution
            .iter()
            .map(|entry| entry.data.get_max())
            .max()
            .unwrap_or(0)
    }

    fn provider_type(&self) -> Identifier {
        Identifier::vanilla_static("weighted_list")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Encode for WeightedListIntProvider {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.distribution
            .encode_field("distribution", ops, ops.map_builder())
            .build(prefix)
    }
}

impl Decode for WeightedListIntProvider {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            Vec::<WeightedEntry>::decode_field::<O>("distribution", &map, ops)
                .map(|distribution| (Self { distribution }, ops.empty()))
        })
    }
}
