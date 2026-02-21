use crate::serialization::HasValue;
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::key_compressor::KeyCompressor;
use crate::serialization::keyable::Keyable;
use crate::serialization::lifecycle::Lifecycle;
use crate::serialization::map_codecs::validated::{ValidatedMapCodec, new_validated_map_codec};
use crate::serialization::map_coders::{
    ComappedMapEncoderImpl, CompressorHolder, FlatComappedMapEncoderImpl, FlatMappedMapDecoderImpl,
    MapDecoder, MapEncoder, MappedMapDecoderImpl, comap, flat_comap, flat_map, map,
};
use crate::serialization::map_like::MapLike;
use crate::serialization::struct_builder::StructBuilder;
use crate::serialization::struct_codecs::Field;
use std::fmt::Display;
use std::sync::Arc;

/// A type of *codec* which encodes/decodes fields of a map.
///
/// The number of keys a `MapCodec` can work with can be one or many keys.
///
/// **This is functionally different from [`Codec`].**
/// The main difference is that while a `Codec` works on encoding/decoding values, a `MapCodec`
/// works on a [`MapLike`].
///
/// # Using Map Codecs
/// They can be used in struct codecs as one part of a struct.
/// **Just like codecs, map codecs are also meant to be static instances, and they should not be created at runtime.
/// They are also immutable, which means they cannot be modified after they are created.**
///
/// # Creating Map Codecs
/// There are a few ways to create map codecs.
///
/// ## Field Map Codecs
/// These are the most commonly used map codecs. The `codec` module has methods for creating them with a `Codec` instance:
/// - [`field`]: For required fields.
/// - [`optional_field`] and [`lenient_optional_field`]: For optional fields encoding/decoding an [`Option`] type.
/// - [`optional_field_with_default`] and [`lenient_optional_field_with_default`]:
///   For optional fields which have a default value for when no value is found while decoding.
///
/// # Transformers
/// A map codec of a type `B` can be implemented by *transforming* another codec of type `A` to work with type `B`,
/// similar to a `Codec`.
/// The following methods can be used depending on the equivalence relation between the two types:
/// - [`xmap`]
/// - [`flat_xmap`]
///
/// # Validator Map Codecs
/// The [`validate`] function returns a codec wrapper that validates a value before encoding and after decoding.
/// A validated codec takes a function that can either return an [`Ok`] for a success,
/// or an [`Err`] with the provided message to place in a `DataResult`.
///
/// [`Codec`]: super::codec::Codec
/// [`field`]: super::codec::field
/// [`optional_field`]: super::codec::optional_field
/// [`lenient_optional_field`]: super::codec::lenient_optional_field
/// [`optional_field_with_default`]: super::codec::optional_field_with_default
/// [`lenient_optional_field_with_default`]: super::codec::lenient_optional_field_with_default
pub trait MapCodec: MapEncoder + MapDecoder {}

// Any struct implementing MapEncoder<Value = A> and MapDecoder<Value = A> will also implement MapCodec<Value = A>.
impl<T> MapCodec for T where T: MapEncoder + MapDecoder {}

/// A map codec allowing an arbitrary encoder and decoder.
pub struct ComposedMapCodec<E: MapEncoder + 'static, D: MapDecoder<Value = E::Value> + 'static> {
    pub(crate) encoder: E,
    pub(crate) decoder: D,
}

impl<E: MapEncoder, D: MapDecoder<Value = E::Value>> HasValue for ComposedMapCodec<E, D> {
    type Value = E::Value;
}

impl<E: MapEncoder, D: MapDecoder<Value = E::Value>> Keyable for ComposedMapCodec<E, D> {
    fn keys(&self) -> Vec<String> {
        let mut vec = self.encoder.keys();
        vec.extend(self.decoder.keys());
        vec
    }
}

impl<E: MapEncoder, D: MapDecoder<Value = E::Value>> CompressorHolder for ComposedMapCodec<E, D> {
    fn compressor(&self) -> Arc<KeyCompressor> {
        // This could return either the encoder or decoder's compressor, but we'll stick with the encoder's.
        self.encoder.compressor()
    }
}

impl<E: MapEncoder, D: MapDecoder<Value = E::Value>> MapEncoder for ComposedMapCodec<E, D> {
    fn encode<T: Display + PartialEq + Clone, B: StructBuilder<Value = T>>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: B,
    ) -> B {
        self.encoder.encode(input, ops, prefix)
    }
}

impl<E: MapEncoder, D: MapDecoder<Value = E::Value>> MapDecoder for ComposedMapCodec<E, D> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value> {
        self.decoder.decode(input, ops)
    }
}

/// Wraps a [`MapCodec`] to make its [`DataResult`]s stable.
pub struct StableMapCodec<C: MapCodec> {
    map_codec: C,
}

impl<C: MapCodec> HasValue for StableMapCodec<C> {
    type Value = C::Value;
}

impl<C: MapCodec> Keyable for StableMapCodec<C> {
    fn keys(&self) -> Vec<String> {
        self.map_codec.keys()
    }
}

impl<C: MapCodec> CompressorHolder for StableMapCodec<C> {
    fn compressor(&self) -> Arc<KeyCompressor> {
        self.map_codec.compressor()
    }
}

impl<C: MapCodec> MapEncoder for StableMapCodec<C> {
    fn encode<T: Display + PartialEq + Clone, B: StructBuilder<Value = T>>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: B,
    ) -> B {
        self.map_codec
            .encode(input, ops, prefix)
            .set_lifecycle(Lifecycle::Stable)
    }
}

impl<C: MapCodec> MapDecoder for StableMapCodec<C> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value> {
        self.map_codec
            .decode(input, ops)
            .with_lifecycle(Lifecycle::Stable)
    }
}

/// Returns a [`Field`] with the provided owned [`MapCodec`] and a getter,
/// which tells the field how to get a part of a struct to serialize.
pub const fn for_getter<T, C: MapCodec + 'static>(
    map_codec: C,
    getter: fn(&T) -> &C::Value,
) -> Field<T, C> {
    Field::Owned(map_codec, getter)
}

/// Returns a [`Field`] with the provided [`MapCodec`] reference and a getter,
/// which tells the field how to get a part of a struct to serialize.
pub const fn for_getter_ref<T, C: MapCodec>(
    map_codec: &'static C,
    getter: fn(&T) -> &C::Value,
) -> Field<T, C> {
    Field::Borrowed(map_codec, getter)
}

/// Returns another [`MapCodec`] of a provided `MapCodec` which provides [`DataResult`]s of the wrapped `map_codec`,
/// but always sets their lifecycle to [`Lifecycle::Stable`].
pub const fn stable<C: MapCodec>(map_codec: C) -> StableMapCodec<C> {
    StableMapCodec { map_codec }
}

/// Helper macro to generate the shorthand types and functions of the transformer [`MapCodec`] methods.
macro_rules! make_map_codec_transformation_function {
    ($name:ident, $short_type:ident, $encoder_type:ident, $decoder_type:ident, $encoder_func:ident, $decoder_func:ident, $to_func_result:ty, $from_func_result:ty, $a_equivalency:literal, $s_equivalency:literal) => {
        pub type $short_type<S, C> = ComposedMapCodec<$encoder_type<S, C>, $decoder_type<S, C>>;

        #[doc = "Transforms a [`MapCodec`] of type `A` to another [`MapCodec`] of type `S`."]
        ///
        /// - `to` is the function called on `A` after decoding to convert it to `S`.
        /// - `from` is the function called on `S` before encoding to convert it to `A`.
        ///
        /// Use this if:
        #[doc = concat!("- `A` is **", $a_equivalency, "** to `S`.")]
        #[doc = concat!("- `S` is **", $s_equivalency, "** to `A`.")]
        #[doc = ""]
        #[doc = "A type `A` is *fully equivalent* to `B` if *A can always successfully be converted to B*."]
        pub const fn $name<A, C: MapCodec<Value = A>, S>(map_codec: &'static C, to: fn(A) -> $to_func_result, from: fn(&S) -> $from_func_result) -> $short_type<S, C> {
            ComposedMapCodec {
                encoder: $encoder_func(map_codec, from),
                decoder: $decoder_func(map_codec, to)
            }
        }
    };
}

make_map_codec_transformation_function!(
    xmap,
    XmapMapCodec,
    ComappedMapEncoderImpl,
    MappedMapDecoderImpl,
    comap,
    map,
    S,
    A,
    "equivalent",
    "equivalent"
);

make_map_codec_transformation_function!(
    flat_xmap,
    FlatXmapMapCodec,
    FlatComappedMapEncoderImpl,
    FlatMappedMapDecoderImpl,
    flat_comap,
    flat_map,
    DataResult<S>,
    DataResult<A>,
    "partially equivalent",
    "partially equivalent"
);

/// Returns a transformer map codec that validates a value before encoding and after decoding by calling a function,
/// which provides a [`DataResult`] depending on that value's validity.
///
/// `validator` is a function that takes the pointer of a value and returns a [`Result`].
/// - If the returned result is an [`Ok`], the codec works as normal.
/// - Otherwise, it always returns a non-result with the message [`String`].
pub const fn validate<C: MapCodec>(
    codec: &'static C,
    validator: fn(&C::Value) -> Result<(), String>,
) -> ValidatedMapCodec<C> {
    new_validated_map_codec(codec, validator)
}
