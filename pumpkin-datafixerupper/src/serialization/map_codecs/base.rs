use crate::serialization::codec::Codec;
use crate::serialization::coders::{Decoder, Encoder};
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::lifecycle::Lifecycle;
use crate::serialization::map_like::MapLike;
use crate::serialization::struct_builder::StructBuilder;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

/// A trait to provide basic functionality for an implementation of [`MapCodec`].
pub trait BaseMapCodec {
    /// The key type of this map codec.
    type Key: Display + Eq + Hash;
    type KeyCodec: Codec<Value = Self::Key> + 'static;

    /// The value (element) type of this map codec.
    type Element;
    type ElementCodec: Codec<Value = Self::Element> + 'static;

    fn key_codec(&self) -> &'static Self::KeyCodec;
    fn element_codec(&self) -> &'static Self::ElementCodec;

    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &HashMap<Self::Key, Self::Element>,
        ops: &'static impl DynamicOps<Value = T>,
        mut prefix: impl StructBuilder<Value = T>,
    ) -> impl StructBuilder<Value = T> {
        for (key, element) in input {
            prefix.add_key_result_value_result(
                self.key_codec().encode_start(key, ops),
                self.element_codec().encode_start(element, ops),
            );
        }
        prefix
    }

    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<HashMap<Self::Key, Self::Element>> {
        let mut read_map: HashMap<Self::Key, Self::Element> = HashMap::new();
        let mut failed: Vec<(T, T)> = vec![];

        let result = input.iter().fold(
            DataResult::success_with_lifecycle((), Lifecycle::Stable),
            |r, (k, e)| {
                // First, we try to parse the key and value.
                let key_result = self.key_codec().parse(k.clone(), ops);
                let element_result = self.element_codec().parse(e.clone(), ops);

                let entry_result =
                    key_result.apply_2_and_make_stable(|kr, er| (kr, er), element_result);
                let accumulated = r.add_message(&entry_result);
                let entry = entry_result.into_result_or_partial();

                if let Some((key, element)) = entry {
                    // If this parses successfully, we try adding it to our map.
                    if read_map.contains_key(&key) {
                        // There was already a value for this key.
                        failed.push((k, e.clone()));
                        return accumulated.add_message::<()>(&DataResult::error(format!(
                            "Duplicate entry for key: {key}"
                        )));
                    }
                    read_map.insert(key, element);
                } else {
                    // Could not parse.
                    failed.push((k, e.clone()));
                }

                accumulated
            },
        );

        let errors = ops.create_map(failed);

        result
            .with_complete_or_partial(read_map)
            .map_error(|e| format!("{e} (Missed inputs: {errors})"))
    }
}
