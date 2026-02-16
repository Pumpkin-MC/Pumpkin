use crate::serialization::base_map_codec::BaseMapCodec;
use crate::serialization::codec::Codec;
use crate::serialization::key_compressor::KeyCompressor;
use crate::serialization::keyable::Keyable;
use crate::serialization::map_coders::CompressorHolder;
use crate::serialization::{Display, HasValue};

use crate::impl_compressor;

use std::hash::Hash;
use std::sync::OnceLock;

/// A simple [`MapCodec`] implementation of [`BaseMapCodec`].
/// This codec has a fixed set of keys.
pub struct SimpleMapCodec<K: Codec + 'static, V: Codec + 'static, Key: Keyable>
where
    K::Value: Display + Eq + Hash,
{
    key_codec: &'static K,
    element_codec: &'static V,

    keyable: Key,
    compressor: OnceLock<KeyCompressor>,
}
impl<K: Codec, V: Codec, Key: Keyable> Keyable for SimpleMapCodec<K, V, Key>
where
    K::Value: Display + Eq + Hash,
{
    fn keys(&self) -> Vec<String> {
        self.keyable.keys()
    }
}

impl<K: Codec, V: Codec, Key: Keyable> CompressorHolder for SimpleMapCodec<K, V, Key>
where
    K::Value: Display + Eq + Hash,
{
    impl_compressor!(compressor);
}

impl<K: Codec, V: Codec, Key: Keyable> BaseMapCodec for SimpleMapCodec<K, V, Key>
where
    K::Value: Display + Eq + Hash,
{
    type Key = K::Value;
    type KeyCodec = K;
    type Element = V::Value;
    type ElementCodec = V;

    fn key_codec(&self) -> &'static Self::KeyCodec {
        self.key_codec
    }

    fn element_codec(&self) -> &'static Self::ElementCodec {
        self.element_codec
    }
}

pub(crate) const fn new_simple_map_codec<K: Codec, V: Codec, Key: Keyable>(
    key_codec: &'static K,
    element_codec: &'static V,
    keyable: Key,
) -> SimpleMapCodec<K, V, Key>
where
    <K as HasValue>::Value: Display + Eq + Hash,
{
    SimpleMapCodec {
        key_codec,
        element_codec,
        keyable,
        compressor: OnceLock::new(),
    }
}
