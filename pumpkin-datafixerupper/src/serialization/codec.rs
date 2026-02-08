use crate::serialization::codecs::list::ListCodec;
use crate::serialization::coders::{Decoder, Encoder};

/// A trait describing the way to **encode from and decode into** something of a type `Value`  (`Value -> ?` and `?` -> `Value`).
pub trait Codec: Encoder + Decoder {
    fn list_of(&'static self, min_size: usize, max_size: usize) -> ListCodec<Self> {
        ListCodec {
            element_codec: self,
            min_size,
            max_size,
        }
    }
}

// Any struct implementing Encoder<A> and Decoder<A> will also implement Codec<A>.
impl<T> Codec for T where T: Encoder + Decoder {}
