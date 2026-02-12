use crate::serialization::HasValue;
use crate::serialization::codec::Codec;
use crate::serialization::map_codec::MapCodec;
use std::collections::HashMap;
use std::hash::Hash;

/// A trait to provide basic functionality for an implementation of [`MapCodec`].
pub trait BaseMapCodec: MapCodec {
    /// The key type of this map codec.
    type Key: Eq + Hash;
    type KeyCodec: Codec<Value = Self::Key>;

    /// The value (element) type of this map codec.
    type Element;
    type ElementCodec: Codec<Value = Self::Element>;

    fn key_codec(&self) -> &'static Self::KeyCodec;
    fn element_codec(&self) -> &'static Self::ElementCodec;
}

impl<T> HasValue for T
where
    T: BaseMapCodec,
{
    type Value = HashMap<T::Key, T::Element>;
}

/// Implements the [`MapCodec::encode`] function for [`BaseMapCodec`] implementations.
#[macro_export]
macro_rules! impl_base_map_codec_encode {
    () => {
        fn encode<T: PartialEq + Clone>(
            &self,
            input: &Self::Value,
            ops: &'static impl DynamicOps<Value = T>,
            mut prefix: impl StructBuilder<Value = T>,
        ) -> impl StructBuilder<Value = T> {
            for (key, element) in input {
                prefix.add_key_result_value_result(
                    self.key_codec().encode_start(key, ops),
                    self.element_codec().encode_start(element, ops),
                )
            }
            prefix
        }
    };
}

/// Implements the [`MapCodec::decode`] function for [`BaseMapCodec`] implementations.
#[macro_export]
macro_rules! impl_base_map_codec_decode {
    ($key_type:ty, $element_type:ty) => {
        fn decode<T: Display + PartialEq + Clone>(
            &self,
            input: &impl MapLike<Value = T>,
            ops: &'static impl DynamicOps<Value = T>,
        ) -> DataResult<HashMap<$key_type, $element_type>> {
            let mut read_map: HashMap<$key_type, $element_type> = HashMap::new();
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
                        if let Some(_) = read_map.get(&key) {
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
    };
}
