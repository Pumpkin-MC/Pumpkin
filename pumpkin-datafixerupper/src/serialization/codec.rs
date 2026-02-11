use crate::serialization::HasValue;
use crate::serialization::codecs::lazy::LazyCodec;
use crate::serialization::codecs::list::ListCodec;
use crate::serialization::codecs::primitive::{
    BoolCodec, ByteBufferCodec, ByteCodec, DoubleCodec, FloatCodec, IntCodec, IntStreamCodec,
    LongCodec, LongStreamCodec, ShortCodec, StringCodec,
};
use crate::serialization::codecs::range::RangeCodec;
use crate::serialization::codecs::range::new_range_codec;
use crate::serialization::coders::{
    ComappedEncoderImpl, Decoder, Encoder, FlatComappedEncoderImpl, FlatMappedDecoderImpl,
    MappedDecoderImpl, comap, decoder_field_of, encoder_field_of, flat_comap, flat_map, map,
};
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::map_codec::ComposedMapCodec;
use crate::serialization::map_codecs::field_coders::{FieldDecoder, FieldEncoder};
use std::sync::{LazyLock, OnceLock};

/// A type of *codec* describing the way to **encode from and decode into** something of a type `Value`  (`Value` -> `?` and `?` -> `Value`).
pub trait Codec: Encoder + Decoder {}

// Any struct implementing Encoder<Value = A> and Decoder<Value = A> will also implement Codec<Value = A>.
impl<T> Codec for T where T: Encoder + Decoder {}

/// A codec allowing an arbitrary encoder and decoder.
pub struct ComposedCodec<A, E: Encoder<Value = A> + 'static, D: Decoder<Value = A> + 'static> {
    encoder: E,
    decoder: D,
}

impl<A, E: Encoder<Value = A>, D: Decoder<Value = A>> HasValue for ComposedCodec<A, E, D> {
    type Value = A;
}

impl<A, E: Encoder<Value = A>, D: Decoder<Value = A>> Encoder for ComposedCodec<A, E, D> {
    fn encode<T: PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        self.encoder.encode(input, ops, prefix)
    }
}

impl<A, E: Encoder<Value = A>, D: Decoder<Value = A>> Decoder for ComposedCodec<A, E, D> {
    fn decode<T: PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.decoder.decode(input, ops)
    }
}

// Primitive codecs

macro_rules! define_const_codec {
    ($name:ident, $codec_ty:ident, $ty:ident, $java_ty:ident) => {
        #[doc = concat!("A primitive codec for Java's `", stringify!($java_ty), "` (`", stringify!($ty), "` in Rust).")]
        pub const $name: $codec_ty = $codec_ty;
    };
    (vec $name:ident, $codec_ty:ident, $vec_ty:ident, $java_ty:ident) => {
        #[doc = concat!("A primitive codec for Java's `", stringify!($java_ty), "`.")]
        #[doc = concat!("Here, this actually stores a [`Vec<", stringify!($vec_ty), ">`].")]
        pub const $name: $codec_ty = $codec_ty;
    };
}

define_const_codec!(BOOL_CODEC, BoolCodec, bool, boolean);

define_const_codec!(BYTE_CODEC, ByteCodec, i8, byte);
define_const_codec!(SHORT_CODEC, ShortCodec, i16, short);
define_const_codec!(INT_CODEC, IntCodec, i32, int);
define_const_codec!(LONG_CODEC, LongCodec, i64, long);
define_const_codec!(FLOAT_CODEC, FloatCodec, f32, float);
define_const_codec!(DOUBLE_CODEC, DoubleCodec, f64, double);

define_const_codec!(STRING_CODEC, StringCodec, String, String);

define_const_codec!(vec BYTE_BUFFER_CODEC, ByteBufferCodec, i8, ByteBuffer);
define_const_codec!(vec INT_STREAM_CODEC, IntStreamCodec, i16, IntStream);
define_const_codec!(vec LONG_STREAM_CODEC, LongStreamCodec, i32, LongStream);

// - Modifier methods -

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

/// Helper macro to generate the shorthand types and functions of the transformer [`Codec`] methods.
macro_rules! make_codec_transformation_function {
    ($name:ident, $short_type:ident, $encoder_type:ident, $decoder_type:ident, $encoder_func:ident, $decoder_func:ident, $to_func_result:ty, $from_func_result:ty, $a_equivalency:literal, $s_equivalency:literal) => {
        type $short_type<A, S, C> = ComposedCodec<S, $encoder_type<A, S, C>, $decoder_type<A, S, C>>;

        #[doc = "Transforms a [`Codec`] of type `A` to another [`Codec`] of type `S`. Use this if:"]
        #[doc = concat!("- `A` is **", $a_equivalency, "** to `S`.")]
        #[doc = concat!("- `S` is **", $s_equivalency, "** to `A`.")]
        #[doc = ""]
        #[doc = "A type `A` is *equivalent* to `B` if *A can always successfully be converted to B*."]
        pub const fn $name<A, C: Codec<Value = A>, S>(codec: &'static C, to: fn(&A) -> $to_func_result, from: fn(&S) -> $from_func_result) -> $short_type<A, S, C> {
            ComposedCodec {
                encoder: $encoder_func(codec, from),
                decoder: $decoder_func(codec, to)
            }
        }
    };
}

// Transformer functions

make_codec_transformation_function!(
    xmap,
    XmapCodec,
    ComappedEncoderImpl,
    MappedDecoderImpl,
    comap,
    map,
    S,
    A,
    "equivalent",
    "equivalent"
);

make_codec_transformation_function!(
    comap_flat_map,
    ComapFlatMapCodec,
    ComappedEncoderImpl,
    FlatMappedDecoderImpl,
    comap,
    flat_map,
    DataResult<S>,
    A,
    "equivalent",
    "not equivalent"
);

make_codec_transformation_function!(
    flat_map_comap,
    FlatMapComapCodec,
    FlatComappedEncoderImpl,
    MappedDecoderImpl,
    flat_comap,
    map,
    S,
    DataResult<A>,
    "not equivalent",
    "equivalent"
);

make_codec_transformation_function!(
    flat_xmap,
    FlatXmapCodec,
    FlatComappedEncoderImpl,
    FlatMappedDecoderImpl,
    flat_comap,
    flat_map,
    DataResult<S>,
    DataResult<A>,
    "not equivalent",
    "not equivalent"
);

// Range codec functions

macro_rules! make_codec_range_function {
    ($func_name:ident, $shorthand_name:ident, $ty:ty, $codec:ident, $singleton_codec:ident, $java_type:ident) => {
        type $shorthand_name = RangeCodec<$ty, $codec>;

        #[doc = concat!("Returns a version of [`", stringify!($singleton_codec), "`] for `", stringify!($ty), "`s (or `", stringify!($java_type), "`s in Java) constrained to a minimum *(inclusive)* and maximum *(inclusive)* value.")]
        pub const fn $func_name(min: $ty, max: $ty) -> $shorthand_name {
            new_range_codec(&$singleton_codec, min, max)
        }
    };
}

make_codec_range_function!(int_range, IntRangeCodec, i32, IntCodec, INT_CODEC, int);
make_codec_range_function!(
    float_range,
    FloatRangeCodec,
    f32,
    FloatCodec,
    FLOAT_CODEC,
    float
);
make_codec_range_function!(
    double_range,
    DoubleRangeCodec,
    f64,
    DoubleCodec,
    DOUBLE_CODEC,
    double
);

type FieldMapCodec<A, C> = ComposedMapCodec<A, FieldEncoder<A, C>, FieldDecoder<A, C>>;

/// Creates a [`MapCodec`] for a field which relies on the provided [`Codec`] for serialization/deserialization.
pub(crate) const fn field_of<A, C: Codec<Value = A>>(
    name: &'static str,
    codec: &'static C,
) -> FieldMapCodec<A, C> {
    ComposedMapCodec {
        encoder: encoder_field_of(name, codec),
        decoder: decoder_field_of(name, codec),
        compressor: OnceLock::new(),
    }
}
