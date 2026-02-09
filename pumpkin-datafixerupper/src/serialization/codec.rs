use crate::serialization::codecs::lazy::LazyCodec;
use crate::serialization::codecs::list::ListCodec;
use crate::serialization::coders::{Decoder, Encoder};
use std::sync::LazyLock;

/// A trait describing the way to **encode from and decode into** something of a type `Value`  (`Value` -> `?` and `?` -> `Value`).
pub trait Codec: Encoder + Decoder {}

// Any struct implementing Encoder<A> and Decoder<A> will also implement Codec<A>.
impl<T> Codec for T where T: Encoder + Decoder {}

// Modifier methods

/// Creates a [`LazyCodec`] with a *function pointer* that returns a new [`Codec`], which will be called on first use.
pub const fn lazy<C: Codec>(f: fn() -> C) -> LazyCodec<C> {
    LazyCodec {
        codec: LazyLock::new(f),
    }
}

/// Creates a [`ListCodec`] of another [`Codec`].
pub const fn list_of<C: Codec>(
    codec: &'static C,
    min_size: usize,
    max_size: usize,
) -> ListCodec<C> {
    ListCodec {
        element_codec: codec,
        min_size,
        max_size,
    }
}
