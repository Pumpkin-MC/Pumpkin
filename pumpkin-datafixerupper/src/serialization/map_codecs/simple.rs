use crate::serialization::Display;
use crate::serialization::codec::Codec;
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::key_compressor::KeyCompressor;
use crate::serialization::keyable::Keyable;
use crate::serialization::lifecycle::Lifecycle;
use crate::serialization::map_codecs::base::BaseMapCodec;
use crate::serialization::map_coders::{CompressorHolder, MapDecoder, MapEncoder};
use crate::serialization::map_like::MapLike;
use crate::serialization::struct_builder::StructBuilder;
use crate::{impl_base_map_codec_decode, impl_base_map_codec_encode, impl_compressor};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::OnceLock;

/// A simple implementation of [`BaseMapCodec`].
pub struct SimpleMapCodec<
    K: Display + Eq + Hash,
    V,
    KC: Codec<Value = K> + 'static,
    VC: Codec<Value = V> + 'static,
> {
    pub(crate) key_codec: &'static KC,
    pub(crate) element_codec: &'static VC,

    pub(crate) keyable: Box<dyn Keyable>,
    pub(crate) compressor: OnceLock<KeyCompressor>,
}
impl<K: Display + Eq + Hash, V, KC: Codec<Value = K>, VC: Codec<Value = V>> Keyable
    for SimpleMapCodec<K, V, KC, VC>
{
    fn keys(&self) -> Vec<String> {
        self.keyable.keys()
    }
}

impl<K: Display + Eq + Hash, V, KC: Codec<Value = K>, VC: Codec<Value = V>> CompressorHolder
    for SimpleMapCodec<K, V, KC, VC>
{
    impl_compressor!(compressor);
}

impl<K: Display + Eq + Hash, V, KC: Codec<Value = K>, VC: Codec<Value = V>> MapEncoder
    for SimpleMapCodec<K, V, KC, VC>
{
    impl_base_map_codec_encode!();
}

impl<K: Display + Eq + Hash, V, KC: Codec<Value = K>, VC: Codec<Value = V>> BaseMapCodec
    for SimpleMapCodec<K, V, KC, VC>
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

impl<K: Display + Eq + Hash, V, KC: Codec<Value = K>, VC: Codec<Value = V>> MapDecoder
    for SimpleMapCodec<K, V, KC, VC>
{
    impl_base_map_codec_decode!(K, V);
}
