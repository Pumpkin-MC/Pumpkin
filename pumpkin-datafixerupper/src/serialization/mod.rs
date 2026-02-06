pub mod codec;
pub mod codecs;
pub mod data_result;
pub mod decoder;
pub mod dynamic_ops;
pub mod encoder;
pub mod lifecycle;
pub mod map_like;

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