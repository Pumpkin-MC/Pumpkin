extern crate core;

use core::fmt;
use std::fmt::{Display, Formatter};

pub mod base_map_codec;
pub mod codec;
pub mod codecs;
pub mod coders;
pub mod data_result;
pub mod dynamic_ops;
pub mod json_ops;
pub mod key_compressor;
pub mod keyable;
pub mod lifecycle;
pub mod list_builder;
pub mod map_codec;
pub mod map_codecs;
pub mod map_coders;
pub mod map_like;
pub mod struct_builder;
pub mod struct_codecs;

/// A trait specifying a single type.
/// This is used to prevent type conflicts for `Codec`s and `MapCodec`s implementing an encoder and decoder.
pub trait HasValue {
    type Value;
}

/// Represents a generic number in Java.
pub enum Number {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl From<Number> for i64 {
    fn from(num: Number) -> Self {
        match num {
            Number::Byte(b) => b as Self,
            Number::Short(s) => s as Self,
            Number::Int(i) => i as Self,
            Number::Long(l) => l,
            Number::Float(f) => f as Self,
            Number::Double(d) => d as Self,
        }
    }
}

impl From<Number> for i32 {
    fn from(num: Number) -> Self {
        match num {
            Number::Byte(b) => b as Self,
            Number::Short(s) => s as Self,
            Number::Int(i) => i,
            Number::Long(l) => l as Self,
            Number::Float(f) => f as Self,
            Number::Double(d) => d as Self,
        }
    }
}

impl From<Number> for i16 {
    fn from(num: Number) -> Self {
        // Similar to Java, we will first convert the number to an `i16`, and then to an `i8`.
        i32::from(num) as Self
    }
}

impl From<Number> for i8 {
    fn from(num: Number) -> Self {
        // Similar to Java, we will first convert the number to an `i32`, and then to an `i8`.
        i32::from(num) as Self
    }
}

impl From<Number> for u8 {
    fn from(num: Number) -> Self {
        i32::from(num) as Self
    }
}

impl From<Number> for f32 {
    fn from(num: Number) -> Self {
        match num {
            Number::Byte(b) => b as Self,
            Number::Short(s) => s as Self,
            Number::Int(i) => i as Self,
            Number::Long(l) => l as Self,
            Number::Float(f) => f,
            Number::Double(d) => d as Self,
        }
    }
}

impl From<Number> for f64 {
    fn from(num: Number) -> Self {
        match num {
            Number::Byte(b) => b as Self,
            Number::Short(s) => s as Self,
            Number::Int(i) => i as Self,
            Number::Long(l) => l as Self,
            Number::Float(f) => f as Self,
            Number::Double(d) => d,
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Byte(v) => write!(f, "{v}"),
            Self::Short(v) => write!(f, "{v}"),
            Self::Int(v) => write!(f, "{v}"),
            Self::Long(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Double(v) => write!(f, "{v}"),
        }
    }
}

impl From<Number> for serde_json::Value {
    fn from(num: Number) -> Self {
        match num {
            Number::Byte(n) => n.into(),
            Number::Short(n) => n.into(),
            Number::Int(n) => n.into(),
            Number::Long(n) => n.into(),
            Number::Float(n) => n.into(),
            Number::Double(n) => n.into(),
        }
    }
}

/// An error struct returned for an invalid conversion to [`Number`] from a [`serde_json::Value`].
pub struct FromJsonValueError;

impl TryFrom<&serde_json::Value> for Number {
    type Error = FromJsonValueError;

    fn try_from(num: &serde_json::Value) -> Result<Self, Self::Error> {
        num.clone().try_into()
    }
}

impl TryFrom<serde_json::Value> for Number {
    type Error = FromJsonValueError;

    fn try_from(num: serde_json::Value) -> Result<Self, Self::Error> {
        match num {
            serde_json::Value::Number(n) => n.try_into(),
            _ => Err(FromJsonValueError),
        }
    }
}

impl TryFrom<serde_json::Number> for Number {
    type Error = FromJsonValueError;

    fn try_from(num: serde_json::Number) -> Result<Self, Self::Error> {
        // Try converting the number to an integer first.
        num.as_i64().map_or_else(
            // Try the float conversion.
            || {
                num.as_f64()
                    .map_or(Err(FromJsonValueError), |f| Ok(Self::Double(f)))
            },
            // Do the integer conversion.
            |n| Ok(Self::Long(n)),
        )
    }
}
