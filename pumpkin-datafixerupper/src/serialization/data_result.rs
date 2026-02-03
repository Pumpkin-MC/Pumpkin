use core::fmt;

use crate::serialization::lifecycle::{Lifecycle};

/// A result that can either represent a successful result, or a
/// *partial* or no result with an error.
/// 
/// `R` is the type of result stored.
#[derive(Debug)]
pub enum DataResult<R> {
    /// Contains a complete result and has no error.
    Success {
        result: R,
        lifecycle: Lifecycle
    },
    /// Contains no or a partial result and has an error.
    /// The error is a *format string*.
    Error {
        partial_result: Option<R>,
        lifecycle: Lifecycle,
        error: fmt::Arguments<'static>
    },
}

impl<R> DataResult<R> {
    /// Returns a *successful* `DataResult` with an experimental lifecycle.
    #[inline]
    #[must_use]
    pub const fn success(result: R) -> Self {
        Self::success_with_lifecycle(result, Lifecycle::Experimental)
    }

    /// Returns a *successful* `DataResult` with a given lifecycle.
    #[inline]
    #[must_use]
    pub const fn success_with_lifecycle(result: R, lifecycle: Lifecycle) -> Self {
        Self::Success {
            result,
            lifecycle
        }
    }

    /// Returns an *errored* `DataResult` with no result and an experimental lifecycle.
    #[inline]
    #[must_use]
    pub const fn error(error: fmt::Arguments<'static>) -> Self {
        Self::error_with_lifecycle(error, Lifecycle::Experimental)
    }

    /// Returns an *errored* `DataResult` with a partial result and an experimental lifecycle.
    #[inline]
    #[must_use]
    pub const fn error_partial(error: fmt::Arguments<'static>, partial_result: R) -> Self {
        Self::error_partial_with_lifecycle(error, partial_result, Lifecycle::Experimental)
    }

    /// Returns an *errored* `DataResult` with no result and a given lifecycle.
    #[inline]
    #[must_use]
    pub const fn error_with_lifecycle(error: fmt::Arguments<'static>, lifecycle: Lifecycle) -> Self {
        Self::Error {
            partial_result: None, lifecycle, error
        }
    }

    /// Returns an *errored* `DataResult` with a partial result and a given lifecycle.
    #[inline]
    pub const fn error_partial_with_lifecycle(error: fmt::Arguments<'static>, partial_result: R, lifecycle: Lifecycle) -> Self {
        Self::Error {
            partial_result: Some(partial_result), lifecycle, error
        }
    }

    /// Tries to get a complete result from this `DataResult`. If no such result exists, this returns [`None`] (even for partial results).
    /// 
    /// To allow partial results, use [`DataResult::into_result_or_partial`].\
    #[inline]
    pub fn into_result(self) -> Option<R> {
        if let Self::Success { result, .. } = self {
            Some(result)
        } else {
            None
        }
    }

    /// Tries to get a complete or partial result. If no such result exists, this returns [`None`].
    pub fn into_result_or_partial(self) -> Option<R> {
        match self {
            Self::Success { result, .. } => Some(result),
            Self::Error { partial_result, .. } => partial_result
        }
    }

    /// Tries to get a complete result from this `DataResult`. If no such result exists, this function panics.
    pub fn unwrap(self) -> R {
        self.into_result().unwrap_or_else(|| panic!("No partial or complete result found for DataResult"))
    }

    /// Tries to get a complete or partial result from this `DataResult`. If no such result exists, this function panics.
    pub fn unwrap_or_partial(self) -> R {
        self.into_result_or_partial().unwrap_or_else(|| panic!("No partial or complete result found for DataResult"))
    }

    /// Whether this `DataResult` has a complete or partial result.
    pub fn has_result_or_partial(self) -> bool {
        !matches!(self, Self::Error { partial_result: None, .. })
    }

    /// Maps a `DataResult` of a type `R` to a `DataResult` of a type `T` by applying a function, leaving errors untouched.
    /// 
    /// `f` is only applied to complete results, not partial ones.
    pub fn map<T>(self, op: impl FnOnce(R) -> T) -> DataResult<T> {
        match self {
            DataResult::Success { result, lifecycle } => DataResult::Success {
                result: op(result),
                lifecycle,
            },
            DataResult::Error { partial_result, lifecycle, error } => DataResult::Error {
                error,
                lifecycle,
                partial_result: partial_result.map(op),
            },
        }
    }

    /// Maps a `DataResult` of a type `R` to a type `T`.
    /// - If there is a complete result, `f` (the result function) is called with that result.
    /// - Otherwise, if there is an error, `default` (the error function) is called with the error as the parameter.
    pub fn map_or_else<T>(self, default: impl FnOnce(DataResult<R>) -> T, f: impl FnOnce(R) -> T) -> T {
        match self {
            DataResult::Success { result, .. } => f(result),
            DataResult::Error { .. } => default(self)
        }
    }


}