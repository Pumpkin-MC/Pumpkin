use crate::impl_compressor;
use crate::serialization::HasValue;
use crate::serialization::codec::Codec;
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::key_compressor::KeyCompressor;
use crate::serialization::keyable::Keyable;
use crate::serialization::map_codec::MapCodec;
use crate::serialization::map_coders::{CompressorHolder, MapDecoder, MapEncoder};
use crate::serialization::map_like::MapLike;
use crate::serialization::struct_builder::StructBuilder;
use std::fmt::Display;
use std::sync::Arc;

/// A [`MapCodec`] that describes an optional field.
pub struct OptionalFieldMapCodec<C: Codec + 'static> {
    element_codec: &'static C,
    name: &'static str,
    /// Whether this field should give a complete result for an
    /// error result (partial or no result) of the underlying codec.
    lenient: bool,
}

impl<C: Codec> HasValue for OptionalFieldMapCodec<C> {
    // The type of this `MapCodec` should be an `Option`.
    type Value = Option<C::Value>;
}

impl<C: Codec> Keyable for OptionalFieldMapCodec<C> {
    fn keys(&self) -> Vec<String> {
        vec![self.name.to_string()]
    }
}

impl<C: Codec> CompressorHolder for OptionalFieldMapCodec<C> {
    impl_compressor!();
}

impl<C: Codec> MapEncoder for OptionalFieldMapCodec<C> {
    fn encode<T: Display + PartialEq + Clone, B: StructBuilder<Value = T>>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: B,
    ) -> B {
        if let Some(input) = input.as_ref() {
            prefix
                .add_string_key_value_result(self.name, self.element_codec.encode_start(input, ops))
        } else {
            prefix
        }
    }
}

impl<C: Codec> MapDecoder for OptionalFieldMapCodec<C> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value> {
        input.get_str(self.name).map_or_else(
            || DataResult::success(None),
            |value| {
                let result = self.element_codec.parse(value.clone(), ops);
                if result.is_error() && self.lenient {
                    DataResult::success(None)
                } else {
                    result.map(Some)
                }
            },
        )
    }
}

/// A wrapper around a [`MapCodec`] returning an [`Option`] type that
/// can provide a default value to transform the `MapCodec` type into its non-`Option` type.
pub struct DefaultValueProviderMapCodec<
    T: PartialEq + Clone,
    C: MapCodec<Value = Option<T>> + 'static,
> {
    codec: C,
    default: fn() -> T,
}

impl<T: PartialEq + Clone, C: MapCodec<Value = Option<T>>> HasValue
    for DefaultValueProviderMapCodec<T, C>
{
    type Value = T;
}

impl<T: PartialEq + Clone, C: MapCodec<Value = Option<T>>> Keyable
    for DefaultValueProviderMapCodec<T, C>
{
    fn keys(&self) -> Vec<String> {
        self.codec.keys()
    }
}

impl<T: PartialEq + Clone, C: MapCodec<Value = Option<T>>> CompressorHolder
    for DefaultValueProviderMapCodec<T, C>
{
    fn compressor(&self) -> Arc<KeyCompressor> {
        self.codec.compressor()
    }
}

impl<T: PartialEq + Clone, C: MapCodec<Value = Option<T>>> MapEncoder
    for DefaultValueProviderMapCodec<T, C>
{
    fn encode<U: Display + PartialEq + Clone, B: StructBuilder<Value = U>>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = U>,
        prefix: B,
    ) -> B {
        let clone = Some(input.clone());
        self.codec.encode(
            if *input == (self.default)() {
                &None
            } else {
                &clone
            },
            ops,
            prefix,
        )
    }
}

impl<T: PartialEq + Clone, C: MapCodec<Value = Option<T>>> MapDecoder
    for DefaultValueProviderMapCodec<T, C>
{
    fn decode<U: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = U>,
        ops: &'static impl DynamicOps<Value = U>,
    ) -> DataResult<Self::Value> {
        self.codec
            .decode(input, ops)
            .map(|value| value.unwrap_or_else(self.default))
    }
}

/// Returns a new [`DefaultValueProviderMapCodec`] with the provided [`Option`] [`MapCodec`] and a default value factory.
pub(crate) const fn new_default_value_provider_map_codec<
    T: PartialEq + Clone,
    C: MapCodec<Value = Option<T>>,
>(
    map_codec: C,
    default: fn() -> T,
) -> DefaultValueProviderMapCodec<T, C> {
    DefaultValueProviderMapCodec {
        codec: map_codec,
        default,
    }
}

/// Returns a new [`OptionalFieldMapCodec`].
pub(crate) const fn new_optional_field_map_codec<C: Codec>(
    element_codec: &'static C,
    name: &'static str,
    lenient: bool,
) -> OptionalFieldMapCodec<C> {
    OptionalFieldMapCodec {
        element_codec,
        name,
        lenient,
    }
}
