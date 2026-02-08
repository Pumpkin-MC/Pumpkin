use crate::serialization::coders::{Decoder, Encoder};

/// A trait describing the way to **encode from and decode into** something of a type `Value`  (`Value -> ?` and `?` -> `Value`).
pub trait Codec: Encoder + Decoder {}

// Any struct implementing Encoder<A> and Decoder<A> will also implement Codec<A>.
impl<T> Codec for T where T: Encoder + Decoder {}
