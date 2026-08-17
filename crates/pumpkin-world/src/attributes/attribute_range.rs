use pumpkin_codecs::DataResult;
use std::fmt::Debug;

/// Validates and sanitizes values for an environment attribute.
///
/// The range is independent from the value type itself: an `f32` attribute can, for
/// example, accept all values, only non-negative values, or only values in `[0, 1]`.
pub trait AttributeRange<T>: Debug + Send + Sync {
    fn validate(&self, value: T) -> DataResult<T>;
    fn sanitize(&self, value: T) -> T;
}

/// An attribute range which accepts every value unchanged.
#[derive(Debug, Clone, Copy, Default)]
pub struct AnyRange;

impl<T> AttributeRange<T> for AnyRange {
    fn validate(&self, value: T) -> DataResult<T> {
        DataResult::new_success(value)
    }

    fn sanitize(&self, value: T) -> T {
        value
    }
}

pub static ANY_RANGE: AnyRange = AnyRange;

/// Inclusive range for floating-point environment attributes.
#[derive(Debug, Clone, Copy)]
pub struct FloatRange {
    min: f32,
    max: f32,
}

impl FloatRange {
    #[must_use]
    pub const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub const UNIT: Self = Self::new(0.0, 1.0);
    pub const NON_NEGATIVE: Self = Self::new(0.0, f32::MAX);
}

impl AttributeRange<f32> for FloatRange {
    fn validate(&self, value: f32) -> DataResult<f32> {
        if value.is_nan() || value < self.min || value > self.max {
            DataResult::new_error(format!(
                "Environment attribute value {value} is outside [{}, {}]",
                self.min, self.max
            ))
        } else {
            DataResult::new_success(value)
        }
    }

    fn sanitize(&self, value: f32) -> f32 {
        if value.is_nan() {
            self.min
        } else {
            value.clamp(self.min, self.max)
        }
    }
}
