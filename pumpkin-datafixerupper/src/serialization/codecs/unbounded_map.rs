use crate::serialization::HasValue;
use crate::serialization::codec::Codec;
use crate::serialization::coders::{Decoder, Encoder};
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::lifecycle::Lifecycle;
use crate::serialization::map_codecs::base::BaseMapCodec;
use crate::serialization::struct_builder::StructBuilder;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

/// A type of [`Codec`] for a map with no known list of keys.
pub struct UnboundedMapCodec<
    K: Display + Eq + Hash,
    V,
    KC: Codec<Value = K> + 'static,
    VC: Codec<Value = V> + 'static,
> {
    pub(crate) key_codec: &'static KC,
    pub(crate) element_codec: &'static VC,
}

impl<K: Display + Eq + Hash, V, KC: Codec<Value = K>, VC: Codec<Value = V>> BaseMapCodec
    for UnboundedMapCodec<K, V, KC, VC>
{
    type Key = K;
    type KeyCodec = KC;
    type Element = V;
    type ElementCodec = VC;

    fn key_codec(&self) -> &'static Self::KeyCodec {
        self.key_codec
    }

    fn element_codec(&self) -> &'static Self::ElementCodec {
        self.element_codec
    }
}

impl<K: Display + Eq + Hash, V, KC: Codec<Value = K>, VC: Codec<Value = V>> HasValue
    for UnboundedMapCodec<K, V, KC, VC>
{
    type Value = HashMap<K, V>;
}

impl<K: Display + Eq + Hash, V, KC: Codec<Value = K>, VC: Codec<Value = V>> Encoder
    for UnboundedMapCodec<K, V, KC, VC>
{
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        BaseMapCodec::encode(self, input, ops, ops.map_builder()).build(prefix)
    }
}

impl<K: Display + Eq + Hash, V, KC: Codec<Value = K>, VC: Codec<Value = V>> Decoder
    for UnboundedMapCodec<K, V, KC, VC>
{
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        ops.get_map(&input)
            .with_lifecycle(Lifecycle::Stable)
            .flat_map(|map| BaseMapCodec::decode(self, &map, ops))
            .map(|r| (r, input))
    }
}
