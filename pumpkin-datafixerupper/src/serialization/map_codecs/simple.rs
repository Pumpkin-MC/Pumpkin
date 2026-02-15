use crate::serialization::Display;
use crate::serialization::codec::Codec;
use crate::serialization::key_compressor::KeyCompressor;
use crate::serialization::keyable::Keyable;
use crate::serialization::map_codecs::base::BaseMapCodec;
use crate::serialization::map_coders::CompressorHolder;

use crate::impl_compressor;

use std::hash::Hash;
use std::sync::OnceLock;

/// A simple [`MapCodec`] implementation of [`BaseMapCodec`].
/// This codec has a fixed set of keys.
pub struct SimpleMapCodec<KC: Codec + 'static, VC: Codec + 'static>
where
    KC::Value: Display + Eq + Hash,
{
    pub(crate) key_codec: &'static KC,
    pub(crate) element_codec: &'static VC,

    pub(crate) keyable: Box<dyn Keyable>,
    pub(crate) compressor: OnceLock<KeyCompressor>,
}
impl<KC: Codec, VC: Codec> Keyable for SimpleMapCodec<KC, VC>
where
    KC::Value: Display + Eq + Hash,
{
    fn keys(&self) -> Vec<String> {
        self.keyable.keys()
    }
}

impl<KC: Codec, VC: Codec> CompressorHolder for SimpleMapCodec<KC, VC>
where
    KC::Value: Display + Eq + Hash,
{
    impl_compressor!(compressor);
}

impl<KC: Codec, VC: Codec> BaseMapCodec for SimpleMapCodec<KC, VC>
where
    KC::Value: Display + Eq + Hash,
{
    type Key = KC::Value;
    type KeyCodec = KC;
    type Element = VC::Value;
    type ElementCodec = VC;

    fn key_codec(&self) -> &'static Self::KeyCodec {
        self.key_codec
    }

    fn element_codec(&self) -> &'static Self::ElementCodec {
        self.element_codec
    }
}
