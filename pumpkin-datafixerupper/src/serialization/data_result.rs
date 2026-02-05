use crate::serialization::lifecycle::Lifecycle;

// TODO: maybe use pub(crate) for certain functions? (certain functions here are not used outside the library)
// TODO: when codecs are implemented, add doc examples

/// A result that can either represent a successful result, or a
/// *partial* or no result with an error.
///
/// `R` is the type of result stored.
#[derive(Debug)]
pub enum DataResult<R> {
    /// Contains a complete result and has no error.
    Success { result: R, lifecycle: Lifecycle },
    /// Contains no or a partial result and has an error.
    /// The error is a *format string*.
    Error {
        partial_result: Option<R>,
        lifecycle: Lifecycle,
        message: String,
    },
}

impl<R> DataResult<R> {
    /// Returns this `DataResult`'s lifecycle.
    pub const fn lifecycle(&self) -> Lifecycle {
        match self {
            Self::Success { lifecycle, .. } | Self::Error { lifecycle, .. } => *lifecycle,
        }
    }

    /// Sets this `DataResult`'s lifecycle (mutates it).
    pub const fn set_lifecycle(&mut self, new_lifecycle: Lifecycle) {
        match self {
            Self::Success { lifecycle, .. } | Self::Error { lifecycle, .. } => {
                *lifecycle = new_lifecycle;
            }
        }
    }

    /// Adds another `Lifecycle` to this this `DataResult`'s lifecycle (mutates it).
    pub const fn add_lifecycle(&mut self, added_lifecycle: Lifecycle) {
        self.set_lifecycle(self.lifecycle().add(added_lifecycle));
    }

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
        Self::Success { result, lifecycle }
    }

    /// Returns an *errored* `DataResult` with no result and an experimental lifecycle.
    #[inline]
    #[must_use]
    pub const fn error(error: String) -> Self {
        Self::error_with_lifecycle(error, Lifecycle::Experimental)
    }

    /// Returns an *errored* `DataResult` with a partial result and an experimental lifecycle.
    #[inline]
    #[must_use]
    pub const fn error_partial(error: String, partial_result: R) -> Self {
        Self::error_partial_with_lifecycle(error, partial_result, Lifecycle::Experimental)
    }

    /// Returns an *errored* `DataResult` with no result and a given lifecycle.
    #[inline]
    #[must_use]
    pub const fn error_with_lifecycle(message: String, lifecycle: Lifecycle) -> Self {
        Self::Error {
            partial_result: None,
            lifecycle,
            message,
        }
    }

    /// Returns an *errored* `DataResult` with a partial result and a given lifecycle.
    #[inline]
    pub const fn error_partial_with_lifecycle(
        message: String,
        partial_result: R,
        lifecycle: Lifecycle,
    ) -> Self {
        Self::Error {
            partial_result: Some(partial_result),
            lifecycle,
            message,
        }
    }

    /// Returns an *errored* `DataResult` with result [`Option<R>`] and a given lifecycle.
    #[inline]
    const fn error_any_with_lifecycle(
        message: String,
        partial_result: Option<R>,
        lifecycle: Lifecycle,
    ) -> Self {
        Self::Error {
            partial_result,
            lifecycle,
            message,
        }
    }

    /// Tries to get a complete result from this `DataResult`. If no such result exists, this returns [`None`] (even for partial results).
    ///
    /// To allow partial results, use [`DataResult::into_result_or_partial`].
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
            Self::Error { partial_result, .. } => partial_result,
        }
    }

    /// Tries to get a complete result from this `DataResult`. If no such result exists, this function panics.
    pub fn unwrap(self) -> R {
        self.into_result()
            .unwrap_or_else(|| panic!("No partial or complete result found for DataResult"))
    }

    /// Tries to get a complete or partial result from this `DataResult`. If no such result exists, this function panics.
    pub fn unwrap_or_partial(self) -> R {
        self.into_result_or_partial()
            .unwrap_or_else(|| panic!("No partial or complete result found for DataResult"))
    }

    /// Whether this `DataResult` has a complete or partial result.
    pub fn has_result_or_partial(self) -> bool {
        !matches!(
            self,
            Self::Error {
                partial_result: None,
                ..
            }
        )
    }

    /// Appends two messages to form a bigger one.
    /// This is useful for stacking message for data results with more than 1 error.
    #[must_use]
    pub fn append_messages(first: &str, second: &str) -> String {
        format!("{first}; {second}")
    }

    /// Maps a `DataResult` of a type `R` to a `DataResult` of a type `T` by applying a function, leaving errors untouched.
    ///
    /// `f` is only applied to complete results, not partial ones.
    #[must_use]
    pub fn map<T>(self, op: impl FnOnce(R) -> T) -> DataResult<T> {
        match self {
            Self::Success { result, lifecycle } => {
                DataResult::success_with_lifecycle(op(result), lifecycle)
            }
            Self::Error {
                partial_result,
                lifecycle,
                message,
            } => DataResult::error_any_with_lifecycle(message, partial_result.map(op), lifecycle),
        }
    }

    /// Maps a `DataResult` of a type `R` to a type `T`.
    /// - If there is a complete result, `f` (the result function) is called with that result.
    /// - Otherwise, if there is an error, `default` (the error function) is called with the error as the parameter.
    #[must_use]
    pub fn map_or_else<T>(self, default: impl FnOnce(Self) -> T, f: impl Fn(R) -> T) -> T {
        match self {
            Self::Success { result, .. } => f(result),
            Self::Error { .. } => default(self),
        }
    }

    /// Chains a `DataResult` with another function taking a `DataResult`.
    /// - If there is a complete or partial result, `f` is called with that result, and the value returned by `f` is returned.
    ///   For a partial result, new messages are propagated via concatenation.
    /// - Otherwise, if there is an error with no result, this propagates this error `DataResult`.
    ///
    /// In other words, `f` will process the complete or partial result of this `DataResult` (if any), appending errors if necessary.
    #[must_use]
    pub fn flat_map<T>(self, f: impl FnOnce(R) -> DataResult<T>) -> DataResult<T> {
        match self {
            Self::Success { result, lifecycle } => {
                let mut new_data_result = f(result);
                // Add this DataResult's lifecycle to the new DataResult.
                new_data_result.add_lifecycle(lifecycle);
                new_data_result
            }
            Self::Error {
                partial_result,
                lifecycle,
                message,
            } => {
                if let Some(result) = partial_result {
                    // Try mapping the internal partial value.
                    let second_result = f(result);
                    let new_lifecycle = second_result.lifecycle().add(lifecycle);
                    match second_result {
                        DataResult::Success { result, .. } => DataResult::Error {
                            partial_result: Some(result),
                            lifecycle: new_lifecycle,
                            message,
                        },
                        DataResult::Error {
                            partial_result,
                            message: second_message,
                            ..
                        } => DataResult::Error {
                            partial_result,
                            lifecycle: new_lifecycle,
                            message: Self::append_messages(&message, &second_message),
                        },
                    }
                } else {
                    // Return this same Error.
                    DataResult::Error {
                        partial_result: None,
                        lifecycle,
                        message,
                    }
                }
            }
        }
    }

    /// Applies a function wrapped in a `DataResult` to the value wrapped in this `DataResult`.
    #[must_use]
    pub fn apply<R2>(self, function_result: DataResult<impl FnOnce(R) -> R2>) -> DataResult<R2> {
        function_result.flat_map(|func| self.map(func))
    }

    /// Applies a function to each result of two `DataResult`s of different types.
    #[must_use]
    pub fn apply_2<R2, T>(
        self,
        f: impl FnOnce(R, R2) -> T,
        second_result: DataResult<R2>,
    ) -> DataResult<T> {
        self.flat_map(|r1| second_result.map(|r2| f(r1, r2)))
    }

    /// Applies a function to each result of two `DataResult`s of different types, marking the resulting `DataResult` as [`Lifecycle::Stable`].
    #[must_use]
    pub fn apply_2_and_make_stable<R2, T>(
        self,
        f: impl FnOnce(R, R2) -> T,
        second_result: DataResult<R2>,
    ) -> DataResult<T> {
        let mut result = self.apply_2(f, second_result);
        result.set_lifecycle(Lifecycle::Stable);
        result
    }

    /// Applies a function to each result of three `DataResult`s of different types.
    #[must_use]
    pub fn apply_3<R2, R3, T>(
        self,
        f: impl FnOnce(R, R2, R3) -> T,
        second_result: DataResult<R2>,
        third_result: DataResult<R3>,
    ) -> DataResult<T> {
        self.flat_map(|r1| second_result.flat_map(|r2| third_result.map(|r3| f(r1, r2, r3))))
    }

    /// Applies a function to `DataResult` errors, leaving successes untouched.
    /// This can be used to provide additional context to an error.
    #[must_use]
    pub fn map_error(self, f: impl FnOnce(String) -> String) -> Self {
        match self {
            Self::Success { .. } => self,
            Self::Error {
                message,
                lifecycle,
                partial_result,
            } => Self::error_any_with_lifecycle(f(message), partial_result, lifecycle),
        }
    }

    /// Promotes a `DataResult` containing a partial result to a success `DataResult`, providing
    /// the error message to a function `f` and removing it from the new `DataResult`.
    /// `DataResult`s with no result or a complete result are left untouched.
    #[must_use]
    pub fn promote_partial(self, f: impl FnOnce(String)) -> Self {
        match self {
            Self::Success { .. } => self,
            Self::Error {
                message,
                lifecycle,
                partial_result,
            } => {
                f(message.clone());
                partial_result.map_or_else(
                    || Self::error_with_lifecycle(message, lifecycle),
                    |result| Self::success_with_lifecycle(result, lifecycle),
                )
            }
        }
    }

    /// Returns a `DataResult` with a new partial value, leaving `DataResult`s with no result or a complete result untouched.
    #[must_use]
    pub fn with_partial(self, partial_value: R) -> Self {
        match self {
            Self::Success { .. } => self,
            Self::Error {
                message, lifecycle, ..
            } => Self::error_partial_with_lifecycle(message, partial_value, lifecycle),
        }
    }

    /// Returns whether this `DataResult` was a success.
    pub const fn is_success(&self) -> bool {
        matches!(self, &Self::Success { .. })
    }

    /// Returns whether this `DataResult` was an error (including partial result errors).
    pub const fn is_error(&self) -> bool {
        !self.is_success()
    }
}
