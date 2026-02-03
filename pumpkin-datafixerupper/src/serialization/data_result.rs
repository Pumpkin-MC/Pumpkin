use crate::serialization::lifecycle::Lifecycle;

// TODO: maybe use pub(crate) for certain functions? (certain functions here are not used outside the library)

/// A result that can either represent a successful result, or a
/// *partial* or no result with an error.
/// 
/// `R` is the type of result stored.
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
        message: Box<dyn Fn() -> String>,
    },
}

impl<R> DataResult<R> {

    /// Returns this `DataResult`'s lifecycle.
    pub const fn lifecycle(&self) -> Lifecycle {
        match self {
            DataResult::Success { lifecycle, .. } => *lifecycle,
            DataResult::Error { lifecycle, .. } => *lifecycle
        }
    }

    /// Sets this `DataResult`'s lifecycle.
    pub const fn set_lifecycle(&mut self, new_lifecycle: Lifecycle) {
        match self {
            DataResult::Success { lifecycle, .. } => {
                *lifecycle = new_lifecycle;
            }
            DataResult::Error { lifecycle, .. } => {
                *lifecycle = new_lifecycle;
            }
        }
    }

    /// Adds another `Lifecycle` to this this `DataResult`'s lifecycle.
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
        Self::Success {
            result,
            lifecycle
        }
    }

    /// Returns an *errored* `DataResult` with no result and an experimental lifecycle.
    #[inline]
    #[must_use]
    pub const fn error(error: Box<dyn Fn() -> String>) -> Self {
        Self::error_with_lifecycle(error, Lifecycle::Experimental)
    }

    /// Returns an *errored* `DataResult` with a partial result and an experimental lifecycle.
    #[inline]
    #[must_use]
    pub const fn error_partial(error: Box<dyn Fn() -> String>, partial_result: R) -> Self {
        Self::error_partial_with_lifecycle(error, partial_result, Lifecycle::Experimental)
    }

    /// Returns an *errored* `DataResult` with no result and a given lifecycle.
    #[inline]
    #[must_use]
    pub const fn error_with_lifecycle(error: Box<dyn Fn() -> String>, lifecycle: Lifecycle) -> Self {
        Self::Error {
            partial_result: None, lifecycle, message: error
        }
    }

    /// Returns an *errored* `DataResult` with a partial result and a given lifecycle.
    #[inline]
    pub const fn error_partial_with_lifecycle(error: Box<dyn Fn() -> String>, partial_result: R, lifecycle: Lifecycle) -> Self {
        Self::Error {
            partial_result: Some(partial_result), lifecycle, message: error
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

    /// Appends two messages to form a bigger one.
    /// This is useful for stacking message for data results with more than 1 error.
    pub fn append_messages(first: String, second: String) -> Box<dyn Fn() -> String> {
        return Box::new(move || { format!("{first}; {second}") });
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
            DataResult::Error { partial_result, lifecycle, message: error } => DataResult::Error {
                message: error,
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

    /// Chains a `DataResult` with another function taking a `DataResult`:
    /// - If there is a complete or partial result, `f` is called with that result, and the value returned by `f` is returned.
    /// - Otherwise, if there is an error with no result, this returns 
    pub fn and_then<T>(self, f: impl FnOnce(R) -> DataResult<T>) -> DataResult<T> {
        /*
             if (partialValue.isEmpty()) {
                return (Error<R2>) this;
            }
            final DataResult<R2> second = function.apply(partialValue.get());
            final Lifecycle combinedLifecycle = lifecycle.add(second.lifecycle());
            if (second instanceof final Success<R2> secondSuccess) {
                return new Error<>(messageSupplier, Optional.of(secondSuccess.value), combinedLifecycle);
            } else if (second instanceof final Error<R2> secondError) {
                return new Error<>(() -> appendMessages(messageSupplier.get(), secondError.messageSupplier.get()), secondError.partialValue, combinedLifecycle);
            } else {
                // TODO: Replace with record pattern matching in Java 21
                throw new UnsupportedOperationException();
            }
         */
        match self {
            DataResult::Success { result, lifecycle } => {
                let mut new_data_result = f(result);
                // Add this DataResult's lifecycle to the new DataResult.
                new_data_result.add_lifecycle(lifecycle);
                new_data_result
            },
            DataResult::Error { partial_result, lifecycle, message } => {
                if let Some(result) = partial_result {
                    // Try mapping the internal partial value.
                    let second_result = f(result);
                    let new_lifecycle = second_result.lifecycle().add(lifecycle);
                    match second_result {
                        DataResult::Success { result, .. } => {
                            DataResult::Error { partial_result: Some(result), lifecycle: new_lifecycle, message }
                        }
                        DataResult::Error { partial_result, message: second_message, .. } => {
                            DataResult::Error {
                                partial_result,
                                lifecycle: new_lifecycle,
                                message: Self::append_messages(message(), second_message())
                            }
                        }
                    }
                } else {
                    // Return this same Error.
                    DataResult::Error { partial_result: None, lifecycle, message }
                }
            }
        }
    }
}