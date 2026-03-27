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

macro_rules! impl_square_cached_bounds {
    ($ty:ty, $normal_ty:ty, $squared_ty:ty) => {
        /// Returns a pair of bounds with the provided minimum and maximum values.
        #[must_use]
        pub fn new(min: $normal_ty, max: $normal_ty) -> Self {
            Self::new_with_bounds(Bounds::<$normal_ty>::new(Some(min), Some(max)))
        }

        /// Returns a pair of bounds with the provided minimum value.
        #[must_use]
        pub fn new_at_least(min: $normal_ty) -> Self {
            Self::new_with_bounds(Bounds::<$normal_ty>::new(Some(min), None))
        }

        /// Returns a pair of bounds with the provided maximum value.
        #[must_use]
        pub fn new_at_most(max: $normal_ty) -> Self {
            Self::new_with_bounds(Bounds::<$normal_ty>::new(None, Some(max)))
        }

        /// Returns whether a number satisfies these bounds.
        #[must_use]
        pub fn matches(&self, number: $normal_ty) -> bool {
            self.bounds.min.is_none_or(|min| min <= number)
                && self.bounds.max.is_none_or(|max| max >= number)
        }

        /// Returns whether a number satisfies these bounds' squared form.
        #[must_use]
        pub fn matches_square(&self, number: $squared_ty) -> bool {
            self.squared_bounds.min.is_none_or(|min| min <= number)
                && self.squared_bounds.max.is_none_or(|max| max >= number)
        }

        #[doc = concat!("Returns the maximum bound of this [`", stringify!($ty), "`].")]
        pub fn min(&self) -> Option<$normal_ty> {
            self.bounds.min
        }

        /// Returns the maximum bound of this [`IntBounds`].
        pub fn max(&self) -> Option<$normal_ty> {
            self.bounds.max
        }
    };
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

    impl_square_cached_bounds!(IntBounds, i32, i64);
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

    impl_square_cached_bounds!(DoubleBounds, f64, f64);
}

/// Represents a range of degrees, stored as `f32`s.
/// This range only stores the minimum and maximum degree values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatDegreeBounds {
    bounds: Bounds<f32>
}

impl FloatDegreeBounds {
    /// Returns the minimum degree amount of this [`FloatDegreeBounds`].
    pub fn min(&self) -> Option<f32> {
        self.bounds.min
    }

    /// Returns the maximum degree amount of this [`FloatDegreeBounds`].
    pub fn max(&self) -> Option<f32> {
        self.bounds.max
    }
}
