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
            EncoderStructBuilder::Normal(b) => b.$name($($args),*),
            EncoderStructBuilder::Compressed(b) => b.$name($($args),*),
        }
    };
}

impl<T: Clone, O: DynamicOps<Value = T>> StructBuilder for EncoderStructBuilder<T, O> {
    type Value = T;

    fn add_key_value(&mut self, key: Self::Value, value: Self::Value) {
        delegate_encoder_struct_builder_method!(self, add_key_value, key, value);
    }

    fn add_key_value_result(&mut self, key: Self::Value, value: DataResult<Self::Value>) {
        delegate_encoder_struct_builder_method!(self, add_key_value_result, key, value);
    }

    fn add_key_result_value_result(
        &mut self,
        key: DataResult<Self::Value>,
        value: DataResult<Self::Value>,
    ) {
        delegate_encoder_struct_builder_method!(self, add_key_result_value_result, key, value);
    }

    fn add_errors_from(&mut self, result: DataResult<()>) {
        delegate_encoder_struct_builder_method!(self, add_errors_from, result);
    }

    fn add_string_key_value(&mut self, key: &str, value: Self::Value) {
        delegate_encoder_struct_builder_method!(self, add_string_key_value, key, value);
    }

    fn add_string_key_value_result(&mut self, key: &str, value: DataResult<Self::Value>) {
        delegate_encoder_struct_builder_method!(self, add_string_key_value_result, key, value);
    }

    fn set_lifecycle(&mut self, lifecycle: Lifecycle) {
        delegate_encoder_struct_builder_method!(self, set_lifecycle, lifecycle);
    }

    fn map_error(&mut self, f: Box<dyn FnOnce(String) -> String>) {
        delegate_encoder_struct_builder_method!(self, map_error, f);
    }

    fn build(self, prefix: Self::Value) -> DataResult<Self::Value> {
        delegate_encoder_struct_builder_method!(self, build, prefix)
    }
}

/// A trait specifying that an object holds a [`KeyCompressor`].
pub trait CompressorHolder: Keyable {
    /// Returns the [`KeyCompressor`] of this object with the provided [`DynamicOps`].
    fn compressor(&self) -> &KeyCompressor;
}

/// A different encoder that encodes a value of type `Value` for a map.
pub trait MapEncoder: HasValue + Keyable + CompressorHolder {
    /// Encodes an input by working on a [`StructBuilder`].
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: impl StructBuilder<Value = T>,
    ) -> impl StructBuilder<Value = T>;

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
            return ops.get_iter(&input).into_result().map_or_else(
                || DataResult::error("Input is not a list".to_string()),
                |iter| {
                    /// A [`MapLike`] for handling [`KeyCompressor`] methods.
                    struct CompressorMapLikeImpl<'a, T, O: DynamicOps<Value = T> + 'static> {
                        list: Vec<T>,
                        compressor: &'a KeyCompressor,
                        ops: &'static O,
                    }

                    impl<T, O: DynamicOps<Value = T>> MapLike for CompressorMapLikeImpl<'_, T, O> {
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
                            list: iter.map(Clone::clone).collect(),
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
/// for structs implementing one or both of them.
///
/// `$compressor` is where the [`OnceLock<KeyCompressor>`] will be stored.
/// Implement this in an `impl` block for [`CompressorHolder`].
#[macro_export]
macro_rules! impl_compressor {
    ($compressor:ident) => {
        fn compressor(&self) -> &KeyCompressor {
            &self.$compressor.get_or_init(|| {
                let mut c = KeyCompressor::new();
                c.populate(self.keys());
                c
            })
        }
    };
}
