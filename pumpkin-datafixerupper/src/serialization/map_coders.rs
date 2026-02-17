use crate::serialization::HasValue;
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::key_compressor::KeyCompressor;
use crate::serialization::keyable::Keyable;
use crate::serialization::lifecycle::Lifecycle;
use crate::serialization::map_like::MapLike;
use crate::serialization::struct_builder::{
    MapBuilder, ResultStructBuilder, StructBuilder, UniversalStructBuilder,
};
use crate::{impl_struct_builder, impl_universal_struct_builder};
use std::fmt::Display;
use std::sync::Arc;

/// A [`StructBuilder`] for compressed map data.
pub struct CompressedStructBuilder<'a, T, O: DynamicOps<Value = T> + 'static> {
    builder: DataResult<Vec<T>>,
    ops: &'static O,
    compressor: &'a KeyCompressor,
}

impl<'a, T: Clone, O: DynamicOps<Value = T> + 'static> CompressedStructBuilder<'a, T, O> {
    #[expect(dead_code)]
    pub(crate) const fn new(ops: &'static O, compressor: &'a KeyCompressor) -> Self {
        Self {
            builder: DataResult::success_with_lifecycle(vec![], Lifecycle::Stable),
            ops,
            compressor,
        }
    }
}

impl<T: Clone, O: DynamicOps<Value = T>> StructBuilder for CompressedStructBuilder<'_, T, O> {
    type Value = T;

    impl_struct_builder!(builder);
    impl_universal_struct_builder!(builder, self.ops);
}

impl<T: Clone, O: DynamicOps<Value = T>> ResultStructBuilder for CompressedStructBuilder<'_, T, O> {
    type Result = Vec<T>;

    fn build_with_builder(
        self,
        builder: Self::Result,
        prefix: Self::Value,
    ) -> DataResult<Self::Value> {
        self.ops.merge_values_into_list(prefix, builder)
    }
}

impl<T: Clone, O: DynamicOps<Value = T>> UniversalStructBuilder
    for CompressedStructBuilder<'_, T, O>
{
    fn append(
        &self,
        key: Self::Value,
        value: Self::Value,
        mut builder: Self::Result,
    ) -> Self::Result {
        if let Some(i) = self.compressor.compress_key(&key, self.ops) {
            builder[i] = value;
        }
        builder
    }
}

/// A [`StructBuilder`] that could be compressed or uncompressed.
pub enum EncoderStructBuilder<T, O: DynamicOps<Value = T> + 'static> {
    Normal(O::StructBuilder),
    Compressed(MapBuilder<T, O>),
}

/// Outsources a function of [`EncoderStructBuilder`] to call the inner builder's method.
macro_rules! delegate_encoder_struct_builder_method {
    ($target:ident, $name:ident $(, $args:expr)*) => {
        match $target {
            Self::Normal(b) => Self::Normal(b.$name($($args),*)),
            Self::Compressed(b) => Self::Compressed(b.$name($($args),*)),
        }
    };
}

impl<T: Clone, O: DynamicOps<Value = T>> StructBuilder for EncoderStructBuilder<T, O> {
    type Value = T;

    fn add_key_value(self, key: Self::Value, value: Self::Value) -> Self {
        delegate_encoder_struct_builder_method!(self, add_key_value, key, value)
    }

    fn add_key_value_result(self, key: Self::Value, value: DataResult<Self::Value>) -> Self {
        delegate_encoder_struct_builder_method!(self, add_key_value_result, key, value)
    }

    fn add_key_result_value_result(
        self,
        key: DataResult<Self::Value>,
        value: DataResult<Self::Value>,
    ) -> Self {
        delegate_encoder_struct_builder_method!(self, add_key_result_value_result, key, value)
    }

    fn with_errors_from<U>(self, result: &DataResult<U>) -> Self {
        delegate_encoder_struct_builder_method!(self, with_errors_from, result)
    }

    fn add_string_key_value(self, key: &str, value: Self::Value) -> Self {
        delegate_encoder_struct_builder_method!(self, add_string_key_value, key, value)
    }

    fn add_string_key_value_result(self, key: &str, value: DataResult<Self::Value>) -> Self {
        delegate_encoder_struct_builder_method!(self, add_string_key_value_result, key, value)
    }

    fn set_lifecycle(self, lifecycle: Lifecycle) -> Self {
        delegate_encoder_struct_builder_method!(self, set_lifecycle, lifecycle)
    }

    fn map_error(self, f: Box<dyn FnOnce(String) -> String>) -> Self {
        delegate_encoder_struct_builder_method!(self, map_error, f)
    }

    fn build(self, prefix: Self::Value) -> DataResult<Self::Value> {
        match self {
            Self::Normal(e) => e.build(prefix),
            Self::Compressed(e) => e.build(prefix),
        }
    }
}

/// A trait specifying that an object holds a [`KeyCompressor`].
pub trait CompressorHolder: Keyable {
    /// Returns the [`KeyCompressor`] of this object with the provided [`DynamicOps`].
    fn compressor(&self) -> Arc<KeyCompressor>;
}

/// A different encoder that encodes a value of type `Value` for a map.
pub trait MapEncoder: HasValue + Keyable + CompressorHolder {
    /// Encodes an input by working on a [`StructBuilder`].
    fn encode<T: Display + PartialEq + Clone, B: StructBuilder<Value = T>>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: B,
    ) -> B;

    /// Returns a [`StructBuilder`] of this `MapEncoder` with the provided [`DynamicOps`].
    fn builder<'a, T: Display + Clone + 'a, O: DynamicOps<Value = T> + 'static>(
        &'a self,
        ops: &'static O,
    ) -> EncoderStructBuilder<T, O> {
        if ops.compress_maps() {
            EncoderStructBuilder::Compressed(MapBuilder::new(ops))
        } else {
            EncoderStructBuilder::Normal(ops.map_builder())
        }
    }
}

/// A different decoder that decodes into something of type `Value` for a map.
pub trait MapDecoder: HasValue + Keyable + CompressorHolder {
    /// Decodes a map input.
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value>;

    fn compressed_decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value> {
        if ops.compress_maps() {
            // Since compressed maps are really just lists, we parse a list instead.
            return ops.get_iter(input).into_result().map_or_else(
                || DataResult::error("Input is not a list".to_string()),
                |iter| {
                    /// A [`MapLike`] for handling [`KeyCompressor`] methods.
                    struct CompressorMapLikeImpl<T, O: DynamicOps<Value = T> + 'static> {
                        list: Vec<T>,
                        compressor: Arc<KeyCompressor>,
                        ops: &'static O,
                    }

                    impl<T, O: DynamicOps<Value = T>> MapLike for CompressorMapLikeImpl<T, O> {
                        type Value = T;

                        fn get(&self, key: &Self::Value) -> Option<&Self::Value> {
                            self.compressor
                                .compress_key(key, self.ops)
                                .and_then(|i| self.list.get(i))
                        }

                        fn get_str(&self, key: &str) -> Option<&Self::Value> {
                            self.compressor
                                .compress_key_str(key)
                                .and_then(|i| self.list.get(i))
                        }

                        fn iter(&self) -> impl Iterator<Item = (Self::Value, &Self::Value)> + '_ {
                            self.list.iter().enumerate().filter_map(|(i, v)| {
                                self.compressor.decompress_key(i, self.ops).map(|k| (k, v))
                            })
                        }
                    }

                    self.decode(
                        &CompressorMapLikeImpl {
                            list: iter.collect(),
                            compressor: self.compressor(),
                            ops,
                        },
                        ops,
                    )
                },
            );
        }
        ops.get_map(&input)
            .with_lifecycle(Lifecycle::Stable)
            .flat_map(|map| self.decode(&map, ops))
    }
}

/// A helper macro for generating the [`CompressorHolder::compressor`] method
/// for structs implementing `CompressorHolder`.
///
/// This macro caches the [`KeyCompressor`] of this [`CompressorHolder`]
/// in a global map.
///
/// Implement this in an `impl` block for `CompressorHolder`.
#[macro_export]
macro_rules! impl_compressor {
    () => {
        fn compressor(&self) -> std::sync::Arc<KeyCompressor> {
            // We get the unique pointer of this holder.
            let key = std::ptr::from_ref::<Self>(self) as usize;
            // Then, we get the cache or store it.
            $crate::serialization::key_compressor::KEY_COMPRESSOR_CACHE
                .entry(key)
                .or_insert_with(|| {
                    let mut c = KeyCompressor::new();
                    c.populate(self.keys());
                    std::sync::Arc::new(c)
                })
                .value()
                .clone()
        }
    };
}

// Transformer map encoders and decoders

macro_rules! impl_map_encoder_transformer {
    ($name:ident, $function_return:ty) => {
        pub struct $name<B, E: MapEncoder + 'static> {
            encoder: &'static E,
            function: fn(&B) -> $function_return,
        }

        impl<B, E: MapEncoder> HasValue for $name<B, E> {
            type Value = B;
        }

        impl<B, E: MapEncoder> Keyable for $name<B, E> {
            fn keys(&self) -> Vec<String> {
                self.encoder.keys()
            }
        }

        impl<B, E: MapEncoder> CompressorHolder for $name<B, E> {
            fn compressor(&self) -> Arc<KeyCompressor> {
                self.encoder.compressor()
            }
        }
    };
}

impl_map_encoder_transformer!(ComappedMapEncoderImpl, E::Value);

impl<B, E: MapEncoder> MapEncoder for ComappedMapEncoderImpl<B, E> {
    fn encode<T: Display + PartialEq + Clone, S: StructBuilder<Value = T>>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: S,
    ) -> S {
        self.encoder.encode(&(self.function)(input), ops, prefix)
    }
}

/// Returns a *contramapped* (*comapped*) transformation of a provided [`MapEncoder`].
/// A *comapped* encoder transforms the input before encoding.
pub(crate) const fn comap<B, E: MapEncoder>(
    encoder: &'static E,
    f: fn(&B) -> E::Value,
) -> ComappedMapEncoderImpl<B, E> {
    ComappedMapEncoderImpl {
        encoder,
        function: f,
    }
}

impl_map_encoder_transformer!(FlatComappedMapEncoderImpl, DataResult<E::Value>);

impl<B, E: MapEncoder> MapEncoder for FlatComappedMapEncoderImpl<B, E> {
    fn encode<T: Display + PartialEq + Clone, S: StructBuilder<Value = T>>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: S,
    ) -> S {
        let result = (self.function)(input);
        let builder = prefix.with_errors_from(&result);
        // We want to encode either a complete or partial result if there is one.
        // Otherwise, we do nothing.
        match result {
            DataResult::Success { result: r, .. }
            | DataResult::Error {
                partial_result: Some(r),
                ..
            } => self.encoder.encode(&r, ops, builder),
            DataResult::Error {
                partial_result: None,
                ..
            } => builder,
        }
    }
}

/// Returns a *flat contramapped* (*flat-comapped*) transformation of a provided [`MapEncoder`].
/// A *flat comapped* encoder transforms the input before encoding, but the transformation can fail.
pub(crate) const fn flat_comap<B, E: MapEncoder>(
    encoder: &'static E,
    f: fn(&B) -> DataResult<E::Value>,
) -> FlatComappedMapEncoderImpl<B, E> {
    FlatComappedMapEncoderImpl {
        encoder,
        function: f,
    }
}

macro_rules! impl_map_decoder_transformer {
    ($name:ident, $function_return:ty) => {
        pub struct $name<B, D: MapDecoder + 'static> {
            decoder: &'static D,
            function: fn(D::Value) -> $function_return,
        }

        impl<B, D: MapDecoder> HasValue for $name<B, D> {
            type Value = B;
        }

        impl<B, D: MapDecoder> Keyable for $name<B, D> {
            fn keys(&self) -> Vec<String> {
                self.decoder.keys()
            }
        }

        impl<B, D: MapDecoder> CompressorHolder for $name<B, D> {
            fn compressor(&self) -> Arc<KeyCompressor> {
                self.decoder.compressor()
            }
        }
    };
}

impl_map_decoder_transformer!(MappedMapDecoderImpl, B);

impl<B, D: MapDecoder> MapDecoder for MappedMapDecoderImpl<B, D> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value> {
        self.decoder.decode(input, ops).map(|a| (self.function)(a))
    }
}

/// Returns a *covariant mapped* transformation of a provided [`MapDecoder`].
/// A *mapped* decoder transforms the output after decoding.
pub(crate) const fn map<B, D: MapDecoder>(
    decoder: &'static D,
    f: fn(D::Value) -> B,
) -> MappedMapDecoderImpl<B, D> {
    MappedMapDecoderImpl {
        decoder,
        function: f,
    }
}

impl_map_decoder_transformer!(FlatMappedMapDecoderImpl, DataResult<B>);

impl<B, D: MapDecoder> MapDecoder for FlatMappedMapDecoderImpl<B, D> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value> {
        self.decoder
            .decode(input, ops)
            .flat_map(|a| (self.function)(a))
    }
}

/// Returns a *covariant flat-mapped* transformation of a provided [`MapDecoder`].
/// A *flat-mapped* decoder transforms the output after decoding, but the transformation can fail.
pub(crate) const fn flat_map<B, D: MapDecoder>(
    decoder: &'static D,
    f: fn(D::Value) -> DataResult<B>,
) -> FlatMappedMapDecoderImpl<B, D> {
    FlatMappedMapDecoderImpl {
        decoder,
        function: f,
    }
}
