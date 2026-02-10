use crate::serialization::map_coders::{MapDecoder, MapEncoder};

/// A type of *codec* which encodes/decodes fields of a map.
/// `Value` is the type of field/value this is responsible for.
/// This is functionally different from [`Codec`].
trait MapCodec: MapEncoder + MapDecoder {}

// Any struct implementing MapEncoder<Value = A> and MapDecoder<Value = A> will also implement MapCodec<Value = A>.
impl<T> MapCodec for T where T: MapEncoder + MapDecoder {}
