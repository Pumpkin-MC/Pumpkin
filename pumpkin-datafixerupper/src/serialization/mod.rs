pub mod data_result;
pub mod dynamic_ops;
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
