use crate::serialization::lifecycle::Lifecycle;

/// Collects the partial value and message from a `DataResult` if it is an error.
/// Returns an [`Option`] of the provided `DataResult`.
/// - The partial value is stored into `$partial_name`.
/// - If a message is found, it is pushed to `$messages_vec`.
macro_rules! collect_partial_and_message {
    ($partial_name:ident, $result:ident, $messages_vec:ident) => {
        let $partial_name = match $result {
            DataResult::Success { result, .. } => Some(result),
            DataResult::Error {
                message,
                partial_result,
                ..
            } => {
                $messages_vec.push(message);
                partial_result
            }
        };
    };
}

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
    pub const fn error_with_lifecycle<T>(message: String, lifecycle: Lifecycle) -> DataResult<T> {
        DataResult::Error {
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
    ///
    /// The name of this function is equivalent to `and_then`.
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
                        DataResult::Success { result, .. } => {
                            DataResult::error_partial_with_lifecycle(message, result, new_lifecycle)
                        }
                        DataResult::Error {
                            partial_result,
                            message: second_message,
                            ..
                        } => DataResult::error_any_with_lifecycle(
                            Self::append_messages(&message, &second_message),
                            partial_result,
                            new_lifecycle,
                        ),
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
    pub fn apply<T>(self, function_result: DataResult<impl FnOnce(R) -> T>) -> DataResult<T> {
        let lifecycle = self.lifecycle().add(function_result.lifecycle());
        match (self, function_result) {
            (Self::Success { result, .. }, DataResult::Success { result: f, .. }) => {
                DataResult::success_with_lifecycle(f(result), lifecycle)
            }
            (
                Self::Success { result, .. },
                DataResult::Error {
                    partial_result,
                    message: func_message,
                    ..
                },
            ) => DataResult::error_any_with_lifecycle(
                func_message,
                partial_result.map(|f| f(result)),
                lifecycle,
            ),
            (
                Self::Error {
                    partial_result,
                    message,
                    ..
                },
                DataResult::Success { result: f, .. },
            ) => DataResult::error_any_with_lifecycle(message, partial_result.map(f), lifecycle),
            (
                Self::Error {
                    partial_result,
                    message,
                    ..
                },
                DataResult::Error {
                    partial_result: partial_func_result,
                    message: func_message,
                    ..
                },
            ) => DataResult::error_any_with_lifecycle(
                Self::append_messages(&message, &func_message),
                partial_result.and_then(|r| partial_func_result.map(|f| f(r))),
                lifecycle,
            ),
        }
    }

    /// Applies a function to each result of two `DataResult`s of different types.
    #[must_use]
    pub fn apply_2<A, T>(
        self,
        f: impl FnOnce(R, A) -> T,
        second_result: DataResult<A>,
    ) -> DataResult<T> {
        // TODO: make an Applicative trait and move this to be implemented to Applicative in some way.
        match (self, second_result) {
            // Both results are successful, just apply f.
            (Self::Success { result: r, .. }, DataResult::Success { result: a, .. }) => {
                DataResult::success(f(r, a))
            }

            // Both results are errors, append their messages and apply f if both have a partial result.
            (
                Self::Error {
                    partial_result: p1,
                    message: m1,
                    ..
                },
                DataResult::Error {
                    partial_result: p2,
                    message: m2,
                    ..
                },
            ) => DataResult::error_any_with_lifecycle(
                Self::append_messages(&m1, &m2),
                match (p1, p2) {
                    (Some(p1), Some(p2)) => Some(f(p1, p2)),
                    _ => None,
                },
                Lifecycle::Experimental,
            ),

            // Exactly one of both results is an error, just return its message without any partial value.
            (Self::Error { message: m1, .. }, _) => DataResult::error(m1),
            (_, DataResult::Error { message: m2, .. }) => DataResult::error(m2),
        }
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
    pub fn apply_3<A, B, T>(
        self,
        second_result: DataResult<A>,
        third_result: DataResult<B>,
        f: impl FnOnce(R, A, B) -> T,
    ) -> DataResult<T> {
        let r = self;
        let a = second_result;
        let b = third_result;

        let has_error = r.is_error() || a.is_error() || b.is_error();

        if !has_error {
            // All 3 results are successful.
            let Self::Success { result: r, .. } = r else {
                unreachable!()
            };
            let DataResult::Success { result: a, .. } = a else {
                unreachable!()
            };
            let DataResult::Success { result: b, .. } = b else {
                unreachable!()
            };
            return DataResult::success(f(r, a, b));
        }

        let mut messages: Vec<String> = vec![];

        // Collect any found errors.
        collect_partial_and_message!(r_partial, r, messages);
        collect_partial_and_message!(a_partial, a, messages);
        collect_partial_and_message!(b_partial, b, messages);

        DataResult::error_any_with_lifecycle(
            messages.join("; "),
            match (r_partial, a_partial, b_partial) {
                (Some(r), Some(a), Some(b)) => Some(f(r, a, b)),
                _ => None,
            },
            Lifecycle::Experimental,
        )
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

    /// Returns a `DataResult` with a new result/partial result, depending on the type of `DataResult` this is.
    /// - For a complete result, this returns another `DataResult` whose complete result is `value`.
    /// - For a partial result, this returns another `DataResult` whose partial result is `value`.
    /// - For no result, this returns itself.
    #[must_use]
    pub fn with_complete_or_partial<T>(self, value: T) -> DataResult<T> {
        match self {
            Self::Success { lifecycle, .. } => DataResult::success_with_lifecycle(value, lifecycle),
            Self::Error {
                message,
                lifecycle,
                partial_result: Some(_),
            } => DataResult::error_partial_with_lifecycle(message, value, lifecycle),
            Self::Error {
                message,
                lifecycle,
                partial_result: None,
            } => Self::error_with_lifecycle(message, lifecycle),
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

/// Asserts that the `$left` `DataResult` is a complete result (success) whose stored result is `$right`.
#[macro_export]
macro_rules! assert_success {
    ($left:expr, $right:expr $(,)?) => {{
        let result = $left;
        assert!(
            result.is_success(),
            "Expected a successful `DataResult`, got: {:?}",
            result
        );
        let value = result.unwrap();
        assert_eq!(
            value,
            $right,
            "DataResult was successful but the value doesn't match"
        );
    }};
}