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

/// A macro to generate a function to apply a function to each result of `n` `DataResult`s.
macro_rules! impl_apply {
    (@internal_method $self:ident $f:ident $($result:ident),+) => {
        let result_1 = $self;
        if !(result_1.is_error() $(|| $result.is_error())+) {
            // All n results are successful.
            return DataResult::success($f(
                result_1.into_result().unwrap()
                $( , $result.into_result().unwrap() )+
            ));
        }
        let mut messages: Vec<String> = vec![];

        // Collect any found errors.
        collect_partial_and_message!(partial_1, result_1, messages);
        $( collect_partial_and_message!($result, $result, messages); )+

        return DataResult::error_any_with_lifecycle(
            messages.join("; "),
            match (partial_1, $($result, )+) {
                (Some(result_1), $(Some($result), )+) => Some($f(result_1 $(, $result )+)),
                _ => None,
            },
            Lifecycle::Experimental,
        );
    };

    ($name:ident, $n:literal, $($ty:ident, $result:ident),+) => {
        #[doc = concat!("Applies a function to each result of ", stringify!($n), " `DataResult`s of different types.")]
        ///
        /// - If any of the given results is a non-result, the returned result will also be a non-result.
        /// - Any errors found in error results (non-result or partial result) will be added to the returned result.
        /// - If all results are at least partial, `f` is called, which should return the final item to be wrapped in the returned result.
        ///
        /// The returned result is a *success* **if and only if** all provided results are successes as well.
        #[must_use]
        pub fn $name<$($ty,)+ T>(
            self,
            f: impl FnOnce(R $(,$ty)+) -> T
            $(, $result: DataResult<$ty>)+
        ) -> DataResult<T> {
            impl_apply!(@internal_method self f $($result),+);
        }
    };
    (expect $name:ident, $n:literal, $($ty:ident, $result:ident),+) => {
        #[doc = concat!("Applies a function to each result of ", stringify!($n), " `DataResult`s of different types.")]
        ///
        /// - If any of the given results is a non-result, the returned result will also be a non-result.
        /// - Any errors found in error results (non-result or partial result) will be added to the returned result.
        /// - If all results are at least partial, `f` is called, which should return the final item to be wrapped in the returned result.
        ///
        /// The returned result is a *success* **if and only if** all provided results are successes as well.
        #[must_use]
        #[expect(clippy::too_many_arguments)]
        pub fn $name<$($ty,)+ T>(
            self,
            f: impl FnOnce(R $(,$ty)+) -> T
            $(, $result: DataResult<$ty>)+
        ) -> DataResult<T> {
            impl_apply!(@internal_method self f $($result),+);
        }
    };
}

// TODO: maybe use pub(crate) for certain functions? (certain functions here are not used outside the library)

/// A result that can either represent a successful result, or a
/// *partial* or non-result with an error.
///
/// `R` is the type of result stored.
#[derive(Clone, Debug)]
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

    /// Sets this `DataResult`'s lifecycle and returns a new result.
    #[must_use]
    pub fn with_lifecycle(self, new_lifecycle: Lifecycle) -> Self {
        match self {
            Self::Success { result, .. } => Self::Success {
                result,
                lifecycle: new_lifecycle,
            },
            Self::Error {
                partial_result,
                message,
                ..
            } => Self::Error {
                partial_result,
                message,
                lifecycle: new_lifecycle,
            },
        }
    }

    /// Adds another `Lifecycle` to this `DataResult`'s lifecycle and returns the new result.
    #[must_use]
    pub fn add_lifecycle(self, added_lifecycle: Lifecycle) -> Self {
        let new_lifecycle = self.lifecycle().add(added_lifecycle);
        self.with_lifecycle(new_lifecycle)
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
    pub const fn has_result_or_partial(&self) -> bool {
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
                // Add this DataResult's lifecycle to the new DataResult.
                f(result).add_lifecycle(lifecycle)
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

    /// Applies a function to each result of 2 `DataResult`s of different types.
    ///
    /// - If any of the given results is a non-result, the returned result will also be a non-result.
    /// - Any errors found in error results (non-result or partial result) will be added to the returned result.
    /// - If both results are at least partial, `f` is called, which should return the final item to be wrapped in the returned result.
    ///
    /// The returned result is a *success* **if and only if** both results are successes as well.
    #[must_use]
    pub fn apply_2<R2, T>(
        self,
        f: impl FnOnce(R, R2) -> T,
        second_result: DataResult<R2>,
    ) -> DataResult<T> {
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

    /// Similar to [`apply_2`], but this also marks the returned `DataResult` as [`Lifecycle::Stable`].
    #[must_use]
    pub fn apply_2_and_make_stable<R2, T>(
        self,
        f: impl FnOnce(R, R2) -> T,
        second_result: DataResult<R2>,
    ) -> DataResult<T> {
        self.apply_2(f, second_result)
            .with_lifecycle(Lifecycle::Stable)
    }

    impl_apply!(apply_3, 3, R2, result_2, R3, result_3);
    impl_apply!(apply_4, 4, R2, result_2, R3, result_3, R4, result_4);
    impl_apply!(
        apply_5, 5, R2, result_2, R3, result_3, R4, result_4, R5, result_5
    );
    impl_apply!(
        apply_6, 6, R2, result_2, R3, result_3, R4, result_4, R5, result_5, R6, result_6
    );
    impl_apply!(expect apply_7, 7, R2, result_2, R3, result_3, R4, result_4, R5, result_5, R6, result_6, R7, result_7);
    impl_apply!(expect apply_8, 8, R2, result_2, R3, result_3, R4, result_4, R5, result_5, R6, result_6, R7, result_7, R8, result_8);
    impl_apply!(expect apply_9, 9, R2, result_2, R3, result_3, R4, result_4, R5, result_5, R6, result_6, R7, result_7, R8, result_8, R9, result_9);
    impl_apply!(expect apply_10, 10, R2, result_2, R3, result_3, R4, result_4, R5, result_5, R6, result_6, R7, result_7, R8, result_8, R9, result_9, R10, result_10);
    impl_apply!(expect apply_11, 11, R2, result_2, R3, result_3, R4, result_4, R5, result_5, R6, result_6, R7, result_7, R8, result_8, R9, result_9, R10, result_10, R11, result_11);
    impl_apply!(expect apply_12, 12, R2, result_2, R3, result_3, R4, result_4, R5, result_5, R6, result_6, R7, result_7, R8, result_8, R9, result_9, R10, result_10, R11, result_11, R12, result_12);
    impl_apply!(expect apply_13, 13, R2, result_2, R3, result_3, R4, result_4, R5, result_5, R6, result_6, R7, result_7, R8, result_8, R9, result_9, R10, result_10, R11, result_11, R12, result_12, R13, result_13);
    impl_apply!(expect apply_14, 14, R2, result_2, R3, result_3, R4, result_4, R5, result_5, R6, result_6, R7, result_7, R8, result_8, R9, result_9, R10, result_10, R11, result_11, R12, result_12, R13, result_13, R14, result_14);
    impl_apply!(expect apply_15, 15, R2, result_2, R3, result_3, R4, result_4, R5, result_5, R6, result_6, R7, result_7, R8, result_8, R9, result_9, R10, result_10, R11, result_11, R12, result_12, R13, result_13, R14, result_14, R15, result_15);
    impl_apply!(expect apply_16, 16, R2, result_2, R3, result_3, R4, result_4, R5, result_5, R6, result_6, R7, result_7, R8, result_8, R9, result_9, R10, result_10, R11, result_11, R12, result_12, R13, result_13, R14, result_14, R15, result_15, R16, result_16);

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

    /// Applies a boxed `dyn` function to `DataResult` errors, leaving successes untouched.
    /// This can be used to provide additional context to an error.
    #[must_use]
    pub fn map_error_dyn(self, f: Box<dyn FnOnce(String) -> String>) -> Self {
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
                message, lifecycle, ..
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

    /// Convenience method to add a message of another `DataResult` (`other_result`) to this `DataResult`.
    /// This is useful for *unit tuple* `DataResult`s used simply for final results of complex objects.
    /// - If `other_result` is a complete result, nothing happens.
    /// - If both results are partial, the returned result is also partial. Otherwise, it is a non-result.
    /// - Messages found in any `DataResult` error are concatenated and used in the returned result.
    ///
    /// This always returns a *stable* result.
    #[must_use]
    pub fn add_message<T>(self, other_result: &DataResult<T>) -> Self {
        match (self, other_result) {
            // Both results are successful.
            (Self::Success { result: r, .. }, DataResult::Success { .. }) => {
                Self::success_with_lifecycle(r, Lifecycle::Stable)
            }

            // Both results are errors.
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
            ) => Self::error_any_with_lifecycle(
                Self::append_messages(&m1, m2),
                if matches!((&p1, &p2), (Some(_), Some(_))) {
                    p1
                } else {
                    None
                },
                Lifecycle::Stable,
            ),

            // Exactly one of both results is an error.
            (Self::Error { message: m1, .. }, _) => {
                Self::error_with_lifecycle(m1, Lifecycle::Stable)
            }
            (_, DataResult::Error { message: m2, .. }) => {
                Self::error_with_lifecycle(m2.clone(), Lifecycle::Stable)
            }
        }
    }
}

// Assertion functions

/// Asserts that the `$left` `DataResult` is a complete result (success) whose stored result is `$right`.
#[macro_export]
macro_rules! assert_success {
    ($left:expr, $right:expr $(,)?) => {{
        let result = $left;
        assert!(
            result.is_success(),
            "Expected a `DataResult` success, got: {:?}",
            result
        );
        let value = result.clone().unwrap();
        assert_eq!(
            value, $right,
            "`DataResult` was successful but the value doesn't match"
        );
    }};
}
