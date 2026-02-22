/// A value that can store a value of one of two types.
#[must_use]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Either<L, R> {
    /// The `Either` value of the left type.
    Left(L),
    /// The `Either` value of the right type.
    Right(R),
}

impl<L, R> Either<L, R> {
    /// Returns the left value wrapped in a `Some` if this `Either` is a `Left` one, returning `None` if not.
    pub fn left(self) -> Option<L> {
        match self {
            Self::Left(l) => Some(l),
            Self::Right(_) => None,
        }
    }

    /// Returns the right value wrapped in a `Some` if this `Either` is a `Right` one, returning `None` if not.
    pub fn right(self) -> Option<R> {
        match self {
            Self::Left(_) => None,
            Self::Right(r) => Some(r),
        }
    }

    /// Maps this `Either`'s value to a common type, returning the returned value of that type by either provided function.
    pub fn map<T>(self, left: impl FnOnce(L) -> T, right: impl FnOnce(R) -> T) -> T {
        match self {
            Self::Left(l) => left(l),
            Self::Right(r) => right(r),
        }
    }

    /// Maps this `Either` depending on the type of `Either` this is, returning a new one:
    /// - If it is `Left`, this calls the `left` function, and its returned value will be part of the new `Either`.
    /// - If it is `Right`, this calls the `right` function, and its returned value will be part of the new `Either`.
    pub fn map_both<L2, R2>(
        self,
        left: impl FnOnce(L) -> L2,
        right: impl FnOnce(R) -> R2,
    ) -> Either<L2, R2> {
        match self {
            Self::Left(l) => Either::Left(left(l)),
            Self::Right(r) => Either::Right(right(r)),
        }
    }

    /// Consumes the left value of this `Either` if it is a `Left` by passing it in a provided consumer function.
    pub fn consume_if_left(self, consumer: impl FnOnce(L)) {
        if let Self::Left(l) = self {
            consumer(l);
        }
    }

    /// Consumes the right value of this `Either` if it is a `Right` by passing it in a provided consumer function.
    pub fn consume_if_right(self, consumer: impl FnOnce(R)) {
        if let Self::Right(r) = self {
            consumer(r);
        }
    }

    /// Unwraps the left type of this `Either`, returning it wrapped in a [`Some`], if any.
    /// If no such value exists, this function returns [`None`].
    pub fn into_left(self) -> Option<L> {
        if let Self::Left(l) = self {
            Some(l)
        } else {
            None
        }
    }

    /// Unwraps the right type of this `Either`, returning it wrapped in a [`Some`], if any.
    /// If no such value exists, this function returns [`None`].
    pub fn into_right(self) -> Option<R> {
        if let Self::Right(r) = self {
            Some(r)
        } else {
            None
        }
    }

    /// Unwraps the left type of this `Either`, if any. If no such value exists, this function panics.
    pub fn unwrap_left(self) -> L {
        self.expect_left("No left value found in Either")
    }

    /// Unwraps the right type of this `Either`, if any. If no such value exists, this function panics.
    pub fn unwrap_right(self) -> R {
        self.expect_right("No right value found in Either")
    }

    /// Unwraps the left type of this `Either`, if any.
    /// If no such value exists, this function panics with a custom message.
    pub fn expect_left(self, message: &str) -> L {
        self.into_left().unwrap_or_else(|| panic!("{}", message))
    }

    /// Unwraps the left type of this `Either`, if any.
    /// If no such value exists, this function panics with a custom message.
    pub fn expect_right(self, message: &str) -> R {
        self.into_right().unwrap_or_else(|| panic!("{}", message))
    }
}
