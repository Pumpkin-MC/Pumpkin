use crate::serialization::HasValue;
use crate::serialization::codecs::lazy::LazyCodec;
use crate::serialization::codecs::list::ListCodec;
use crate::serialization::codecs::primitive::{
    BoolCodec, ByteBufferCodec, ByteCodec, DoubleCodec, FloatCodec, IntCodec, IntStreamCodec,
    LongCodec, LongStreamCodec, ShortCodec, StringCodec,
};
use crate::serialization::coders::{
    ComappedEncoderImpl, Decoder, Encoder, FlatComappedEncoderImpl, FlatMappedDecoderImpl,
    MappedDecoderImpl, comap, flat_comap, flat_map, map,
};
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use std::fmt::Display;
use std::sync::LazyLock;

/// A type of *codec* describing the way to **encode from and decode into** something of a type `Value`  (`Value` -> `?` and `?` -> `Value`).
pub trait Codec: Encoder + Decoder {}

// Any struct implementing Encoder<Value = A> and Decoder<Value = A> will also implement Codec<Value = A>.
impl<T> Codec for T where T: Encoder + Decoder {}

/// A base codec allowing an arbitrary encoder and decoder.
pub struct BaseCodec<A, E: Encoder<Value = A> + 'static, D: Decoder<Value = A> + 'static> {
    encoder: E,
    decoder: D,
}

impl<A, E: Encoder<Value = A>, D: Decoder<Value = A>> HasValue for BaseCodec<A, E, D> {
    type Value = A;
}

impl<A, E: Encoder<Value = A>, D: Decoder<Value = A>> Encoder for BaseCodec<A, E, D> {
    fn encode<T: PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        self.encoder.encode(input, ops, prefix)
    }
}

impl<A, E: Encoder<Value = A>, D: Decoder<Value = A>> Decoder for BaseCodec<A, E, D> {
    fn decode<T: PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.decoder.decode(input, ops)
    }
}

/// A helper function to check whether a number is between the range `[min, max]` (both inclusive).
fn check_range<T: PartialOrd + Display + Clone>(input: &T, min: &T, max: &T) -> DataResult<T> {
    if input >= min && input <= max {
        DataResult::success(input.clone())
    } else {
        DataResult::error(format!("Value {input} is outside range [{min}, {max}]"))
    }
}

/// A codec for a specific number range.
pub struct RangeCodec<A: PartialOrd + Display, C: Codec<Value = A>> {
    codec: C,
    min: A,
    max: A,
}

impl<A: PartialOrd + Display + Clone, C: Codec<Value = A>> HasValue for RangeCodec<A, C> {
    type Value = A;
}

impl<A: PartialOrd + Display + Clone, C: Codec<Value = A>> Encoder for RangeCodec<A, C> {
    fn encode<T: PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        check_range(input, &self.min, &self.max).flat_map(|t| self.codec.encode(&t, ops, prefix))
    }
}

impl<A: PartialOrd + Display + Clone, C: Codec<Value = A>> Decoder for RangeCodec<A, C> {
    fn decode<T: PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.codec
            .decode(input, ops)
            .flat_map(|(i, t)| check_range(&i, &self.min, &self.max).map(|n| (n, t)))
    }
}

// Primitive codecs

/// A primitive codec for Java's `boolean` (`bool` in Rust).
pub const BOOL_CODEC: BoolCodec = BoolCodec;

/// A primitive codec for Java's `byte` (or `i8` in Rust).
pub const BYTE_CODEC: ByteCodec = ByteCodec;
/// A primitive codec for Java's `short` (or `i16` in Rust).
pub const SHORT_CODEC: ShortCodec = ShortCodec;
/// A primitive codec for Java's `int` (or `i32` in Rust).
pub const INT_CODEC: IntCodec = IntCodec;
/// A primitive codec for Java's `long` (or `i64` in Rust).
pub const LONG_CODEC: LongCodec = LongCodec;
/// A primitive codec for Java's `float` (or `f32` in Rust).
pub const FLOAT_CODEC: FloatCodec = FloatCodec;
/// A primitive codec for Java's `double` (or `f64` in Rust).
pub const DOUBLE_CODEC: DoubleCodec = DoubleCodec;

/// A primitive codec for Java's `String` (also `String` in Rust).
pub const STRING_CODEC: StringCodec = StringCodec;

/// A primitive codec for Java's `ByteBuffer`.
/// Here, this actually stores a [`Vec<i8>`].
pub const BYTE_BUFFER_CODEC: ByteBufferCodec = ByteBufferCodec;
/// A primitive codec for Java's `IntStream`.
/// Here, this actually stores a [`Vec<i32>`].
pub const INT_STREAM_CODEC: IntStreamCodec = IntStreamCodec;
/// A primitive codec for Java's `LongStream`.
/// Here, this actually stores a [`Vec<i64>`].
pub const LONG_STREAM_CODEC: LongStreamCodec = LongStreamCodec;

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
        type $short_type<A, S, C> = BaseCodec<S, $encoder_type<A, S, C>, $decoder_type<A, S, C>>;

        #[doc = "Transforms a [`Codec`] of type `A` to another [`Codec`] of type `S`. Use this if:"]
        #[doc = concat!("`A` is **", $a_equivalency, "** to `S`.")]
        #[doc = concat!("`S` is **", $s_equivalency, "** to `A`.")]
        #[doc = ""]
        #[doc = "A type `A` is *equivalent* to `B` if *A can always successfully be converted to B*."]
        pub const fn $name<A, C: Codec<Value = A>, S>(codec: &'static C, to: fn(&A) -> $to_func_result, from: fn(&S) -> $from_func_result) -> $short_type<A, S, C> {
            BaseCodec {
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
            RangeCodec {
                codec: $singleton_codec,
                min,
                max
            }
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
