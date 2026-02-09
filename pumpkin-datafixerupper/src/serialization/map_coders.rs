use crate::serialization::HasValue;
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::key_compressor::KeyCompressor;
use crate::serialization::keyable::Keyable;
use crate::serialization::lifecycle::Lifecycle;
use crate::serialization::struct_builder::{
    ResultStructBuilder, StructBuilder, UniversalStructBuilder,
};
use crate::{impl_struct_builder, impl_universal_struct_builder};

/// A different encoder that encodes a value of type `Value` for a map.
pub trait MapEncoder: HasValue + Keyable {
    fn encode<T>(
        input: Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: impl StructBuilder<Value = T>,
    ) -> impl StructBuilder<Value = T>;

    /// Returns a [`KeyCompressor`] of this `MapEncoder` with the provided [`DynamicOps`].
    fn compressor<T, O: DynamicOps<Value = T>>(&self, ops: &'static O) -> KeyCompressor<T, O>;

    /// Returns a [`CompressedStructBuilder`] of this `MapEncoder` with the provided [`DynamicOps`].
    fn compressed_builder<T: Clone + 'static, O: DynamicOps<Value = T>>(
        &self,
        ops: &'static O,
    ) -> Box<dyn StructBuilder<Value = T>> {
        if ops.compress_maps() {
            Box::new(CompressedStructBuilder::new(ops, self.compressor(ops)))
        } else {
            Box::new(ops.map_builder())
        }
    }
}

/// A [`StructBuilder`] for compressed map data.
struct CompressedStructBuilder<T, O: DynamicOps<Value = T> + 'static> {
    builder: DataResult<Vec<T>>,
    ops: &'static O,
    compressor: KeyCompressor<T, O>,
}

impl<T: Clone, O: DynamicOps<Value = T> + 'static> CompressedStructBuilder<T, O> {
    pub(crate) const fn new(ops: &'static O, compressor: KeyCompressor<T, O>) -> Self {
        Self {
            builder: DataResult::success_with_lifecycle(vec![], Lifecycle::Stable),
            ops,
            compressor,
        }
    }
}

impl<T: Clone, O: DynamicOps<Value = T>> StructBuilder for CompressedStructBuilder<T, O> {
    type Value = T;

    impl_struct_builder!(builder, ops);
    impl_universal_struct_builder!(builder);
}

impl<T: Clone, O: DynamicOps<Value = T>> ResultStructBuilder for CompressedStructBuilder<T, O> {
    type Result = Vec<T>;

    fn build_with_builder(
        self,
        builder: Self::Result,
        prefix: Self::Value,
    ) -> DataResult<Self::Value> {
        self.ops.merge_values_into_list(prefix, builder)
    }
}

impl<T: Clone, O: DynamicOps<Value = T>> UniversalStructBuilder for CompressedStructBuilder<T, O> {
    fn append(
        &self,
        key: Self::Value,
        value: Self::Value,
        mut builder: Self::Result,
    ) -> Self::Result {
        if let Some(i) = self.compressor.compress_key(&key) {
            builder[i] = value;
        }
        builder
    }
}
