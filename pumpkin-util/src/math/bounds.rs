/// Represents a single range bound of some type `T`, whose bounds may be optional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Bounds<T: PartialOrd> {
    min: Option<T>,
    max: Option<T>,
}

impl<T: PartialOrd> Bounds<T> {
    const fn new<U: PartialOrd>(min: Option<U>, max: Option<U>) -> Bounds<U> {
        Bounds { min, max }
    }
}

/// Represents a range of integers.
/// This range stores both the bounds of the range and the squares of the bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntBounds {
    bounds: Bounds<i32>,
    squared_bounds: Bounds<i64>,
}

impl IntBounds {
    #[must_use]
    fn new_with_bounds(bounds: Bounds<i32>) -> Self {
        Self {
            bounds,
            squared_bounds: Bounds {
                min: bounds.min.map(|m| (m as i64) * (m as i64)),
                max: bounds.max.map(|m| (m as i64) * (m as i64)),
            },
        }
    }

    /// Returns a range with the provided minimum and maximum values.
    #[must_use]
    pub fn new(min: i32, max: i32) -> Self {
        Self::new_with_bounds(Bounds::<i32>::new(Some(min), Some(max)))
    }

    /// Returns a range with the provided minimum value.
    #[must_use]
    pub fn new_at_least(min: i32) -> Self {
        Self::new_with_bounds(Bounds::<i32>::new(Some(min), None))
    }

    /// Returns a range with the provided maximum value.
    #[must_use]
    pub fn new_at_most(max: i32) -> Self {
        Self::new_with_bounds(Bounds::<i32>::new(None, Some(max)))
    }

    /// Returns whether a number satisfies this range.
    #[must_use]
    pub fn matches(&self, number: i32) -> bool {
        self.bounds.min.is_none_or(|min| min <= number)
            && self.bounds.max.is_none_or(|max| max >= number)
    }

    /// Returns whether a number satisfies this range's squared form.
    #[must_use]
    pub fn matches_square(&self, number: i64) -> bool {
        self.squared_bounds.min.is_none_or(|min| min <= number)
            && self.squared_bounds.max.is_none_or(|max| max >= number)
    }
}

/// Represents a range of `f64`s.
/// This range stores both the bounds of the range and the squares of the bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoubleBounds {
    bounds: Bounds<f64>,
    squared_bounds: Bounds<f64>,
}

impl DoubleBounds {
    #[must_use]
    fn new_with_bounds(bounds: Bounds<f64>) -> Self {
        Self {
            bounds,
            squared_bounds: Bounds {
                min: bounds.min.map(|m| m * m),
                max: bounds.max.map(|m| m * m),
            },
        }
    }

    /// Returns a range with the provided minimum and maximum values.
    #[must_use]
    pub fn new(min: f64, max: f64) -> Self {
        Self::new_with_bounds(Bounds::<f64>::new(Some(min), Some(max)))
    }

    /// Returns a range with the provided minimum value.
    #[must_use]
    pub fn new_at_least(min: f64) -> Self {
        Self::new_with_bounds(Bounds::<f64>::new(Some(min), None))
    }

    /// Returns a range with the provided maximum value.
    #[must_use]
    pub fn new_at_most(max: f64) -> Self {
        Self::new_with_bounds(Bounds::<f64>::new(None, Some(max)))
    }

    /// Returns whether a number satisfies this range.
    #[must_use]
    pub fn matches(&self, number: f64) -> bool {
        self.bounds.min.is_none_or(|min| min <= number)
            && self.bounds.max.is_none_or(|max| max >= number)
    }

    /// Returns whether a number satisfies this range's squared form.
    #[must_use]
    pub fn matches_square(&self, number: f64) -> bool {
        self.squared_bounds.min.is_none_or(|min| min <= number)
            && self.squared_bounds.max.is_none_or(|max| max >= number)
    }
}

/// Represents a range of degrees, stored as `f32`s.
/// This range only stores the minimum and maximum degree values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatDegreeBounds {
    bounds: Bounds<f64>,
}
