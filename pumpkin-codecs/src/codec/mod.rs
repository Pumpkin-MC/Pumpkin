mod either;
pub mod list;
pub mod map;
pub mod optional_field;
pub(crate) mod primitive;

use crate::codec::optional_field::OptionalFieldDecode;
use crate::map_like::MapLike;
use crate::struct_builder::StructBuilder;
use crate::{DataResult, DynamicOps};

/// A trait for something that can be encoded by a [`DynamicOps`] to its format.
pub trait Encode {
    /// Encodes this value to a value represented by the provided [`DynamicOps`]
    /// with the provided prefix.
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value>;

    /// Encodes this value to a value represented by the provided [`DynamicOps`] without a prefix.
    fn encode_start<O: DynamicOps>(&self, ops: &'static O) -> DataResult<O::Value> {
        self.encode(ops, ops.empty())
    }
}

/// A trait for something which can be added to a [`MapLike`] as a field with a provided name.
pub trait FieldEncode {
    /// Encodes this value to a map by adding a field, whose:
    /// - key is the field's `name`.
    /// - value is the encoded value represented by the provided [`DynamicOps`].
    fn encode_field<O: DynamicOps, B: StructBuilder<Value = O::Value>>(
        &self,
        name: &'static str,
        ops: &'static O,
        prefix: B,
    ) -> B;

    /// Encodes this value to a map by adding a defaulted field, whose:
    /// - key is the field's `name`.
    /// - value is the encoded value represented by the provided [`DynamicOps`].
    ///
    /// The field may not be encoded if `default` == `*self`.
    fn encode_defaulted_field<O: DynamicOps, B: StructBuilder<Value = O::Value>>(
        &self,
        name: &'static str,
        ops: &'static O,
        prefix: B,
        default: Self,
    ) -> B
    where
        Self: PartialEq;
}

impl<T: Encode> FieldEncode for T {
    fn encode_field<O: DynamicOps, B: StructBuilder<Value = O::Value>>(
        &self,
        name: &'static str,
        ops: &'static O,
        prefix: B,
    ) -> B {
        prefix.add_string_key_value_result(name, self.encode_start(ops))
    }

    fn encode_defaulted_field<O: DynamicOps, B: StructBuilder<Value = O::Value>>(
        &self,
        name: &'static str,
        ops: &'static O,
        prefix: B,
        default: Self,
    ) -> B
    where
        Self: PartialEq,
    {
        if default == *self {
            prefix.add_string_key_value_result(name, self.encode_start(ops))
        } else {
            prefix
        }
    }
}

/// A trait for something that can be decoded from the value represented by a [`DynamicOps`].
pub trait Decode: Sized {
    /// Decodes a value of this type from a value represented by the provided [`DynamicOps`],
    /// along with the remaining data.
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)>;

    /// Decodes a value of this type from a value represented by the provided [`DynamicOps`],
    /// without providing any other data.
    fn parse<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<Self> {
        Self::decode(input, ops).map(|(r, _)| r)
    }
}

/// A trait for something which can be decoded from a [`MapLike`] from a field with a provided name.
pub trait FieldDecode: Sized {
    /// Decodes a value of this type from a map by decoding one of its fields, whose:
    /// - key is the field's `name`.
    /// - value is the value represented by a [`DynamicsOps`] that is meant to be decoded.
    fn decode_field<O: DynamicOps>(
        name: &'static str,
        input: &impl MapLike<Value = O::Value>,
        ops: &'static impl DynamicOps<Value = O::Value>,
    ) -> DataResult<Self>;

    /// Decodes a value of this type from a map by decoding one of its defaulted fields, whose:
    /// - key is the field's `name`.
    /// - value is the value represented by a [`DynamicsOps`] that is meant to be decoded.
    ///
    /// If a value could not be decoded, the `default` value is returned.
    /// This method has an extra `lenient` parameter. If it is `true`, errors
    /// while trying to decode an explicit value, and the default value will be decoded instead.
    fn decode_defaulted_field<O: DynamicOps>(
        name: &'static str,
        input: &impl MapLike<Value = O::Value>,
        ops: &'static impl DynamicOps<Value = O::Value>,
        default: Self,
        lenient: bool,
    ) -> DataResult<Self>;
}

impl<T: Decode> FieldDecode for T {
    fn decode_field<O: DynamicOps>(
        name: &'static str,
        input: &impl MapLike<Value = O::Value>,
        ops: &'static impl DynamicOps<Value = O::Value>,
    ) -> DataResult<Self> {
        input.get_str(name).map_or_else(
            || DataResult::new_error(format!("No key {name} in map")),
            |v| Self::parse(v.clone(), ops),
        )
    }

    fn decode_defaulted_field<O: DynamicOps>(
        name: &'static str,
        input: &impl MapLike<Value = O::Value>,
        ops: &'static impl DynamicOps<Value = O::Value>,
        default: Self,
        lenient: bool,
    ) -> DataResult<Self> {
        let decoded_option = Option::decode_optional_field::<O>(name, input, ops, lenient);
        decoded_option.map(|o| o.unwrap_or(default))
    }
}

/// Provides a fallible/infallible encode conversion from a first type to a second type via `backward`.
///
/// You probably do not need to use this directly.
#[macro_export]
macro_rules! encode_impl {
    (infallible $second_type:ty, $backward:path) => {
        impl $crate::Encode for $second_type {
            fn encode<O: $crate::DynamicOps>(
                &self,
                ops: &'static O,
                prefix: O::Value,
            ) -> $crate::DataResult<O::Value> {
                $backward(self).encode(ops, prefix)
            }
        }
    };

    (fallible $backward:path) => {
        impl $crate::Encode for $second_type {
            fn encode<O: $crate::DynamicOps>(
                &self,
                ops: &'static O,
                prefix: O::Value,
            ) -> $crate::DataResult<O::Value> {
                $backward(self).flat_map(|m| m.encode(ops, prefix))
            }
        }
    };
}

/// Provides a fallible/infallible decode conversion from a first type to a second type via `forward`.
///
/// You probably do not need to use this directly.
#[macro_export]
macro_rules! decode_impl {
    (infallible $first_type:ty, $second_type:ty, $forward:path) => {
        impl $crate::Decode for $second_type {
            fn decode<O: $crate::DynamicOps>(
                input: O::Value,
                ops: &'static O,
            ) -> $crate::DataResult<(Self, O::Value)> {
                <$first_type>::decode(input, ops).flat_map(|(s, p)| ($forward(s), p))
            }
        }
    };

    (fallible $first_type:ty, $second_type:ty, $forward:path) => {
        impl $crate::Decode for $second_type {
            fn decode<O: $crate::DynamicOps>(
                input: O::Value,
                ops: &'static O,
            ) -> $crate::DataResult<(Self, O::Value)> {
                <$first_type>::decode(input, ops)
                    .flat_map(|(s, p)| $forward(s).map(|ident| (ident, p)))
            }
        }
    };
}

/// Provides an easy `xmap`-like implementation of a *second type*
/// by using the already-existing implementation of a *first type*.
///
/// The macro is written as `xmap_codec_impl!(first => second, forward, backward)`,
/// where:
/// - `forward` is the infallible conversion `fn(first_type) -> second_type` (for decoding).
/// - `backward` is the infallible conversion `fn(&second_type) -> first_type` (for encoding).
#[macro_export]
macro_rules! xmap_codec_impl {
    ($first_type:ty => $second_type:ty, $forward:path, $backward:path) => {
        $crate::encode_impl!(infallible $second_type, $backward);
        $crate::decode_impl!(infallible $first_type, $second_type, $forward);
    };
}

/// Provides an easy `comapFlatMap`-like implementation of a *second type*
/// by using the already-existing implementation of a *first type*.
///
/// The macro is written as `comap_flat_map_codec_impl!(first => second, forward, backward)`,
/// where:
/// - `forward` is the fallible conversion `fn(first_type) -> DataResult<second_type>` (for decoding).
/// - `backward` is the infallible conversion `fn(&second_type) -> first_type` (for encoding).
#[macro_export]
macro_rules! comap_flat_map_codec_impl {
    ($first_type:ty => $second_type:ty, $forward:path, $backward:path) => {
        $crate::encode_impl!(infallible $second_type, $backward);
        $crate::decode_impl!(fallible $first_type, $second_type, $forward);
    };
}

/// Provides an easy `flatComapMap`-like implementation of a *second type*
/// by using the already-existing implementation of a *first type*.
///
/// The macro is written as `flat_comap_map_codec_impl!(first => second, forward, backward)`,
/// where:
/// - `forward` is the infallible conversion `fn(first_type) -> second_type` (for decoding).
/// - `backward` is the fallible conversion `fn(&second_type) -> DataResult<first_type>` (for encoding).
#[macro_export]
macro_rules! flat_comap_map_codec_impl {
    ($first_type:ty => $second_type:ty, $forward:path, $backward:path) => {
        $crate::encode_impl!(fallible $second_type, $backward);
        $crate::decode_impl!(infallible $first_type, $second_type, $forward);
    };
}

/// Provides an easy `flatXmap`-like implementation of a *second type*
/// by using the already-existing implementation of a *first type*.
///
/// The macro is written as `flat_xmap_codec_impl!(first => second, forward, backward)`,
/// where:
/// - `forward` is the fallible conversion `fn(first_type) -> DataResult<second_type>` (for decoding).
/// - `backward` is the fallible conversion `fn(&second_type) -> DataResult<first_type>` (for encoding).
#[macro_export]
macro_rules! flat_xmap_codec_impl {
    ($first_type:ty => $second_type:ty, $forward:path, $backward:path) => {
        $crate::encode_impl!(fallible $second_type, $backward);
        $crate::decode_impl!(fallible $first_type, $second_type, $forward);
    };
}
