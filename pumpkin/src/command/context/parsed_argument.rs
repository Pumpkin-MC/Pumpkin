use std::any::Any;
use crate::command::context::parsed_argument::sealed::Sealed;
use crate::command::context::string_range::StringRange;

/// Represents a parsed argument of a particular type `T`.
pub struct TypedParsedArgument<T> {
    /// The range of this parsed argument.
    pub range: StringRange,

    /// The result of this parsed argument.
    pub result: T
}

// Prevent other crates from using this trait
// Thus, we can effectively 'seal' our trait meant
// only for `ParsedArgument`.
mod sealed {
    /// Private trait to ensure only `TypedParsedArgument` can implement `ParsedArgument`.
    pub trait Sealed {}
}

/// Represents a parsed argument of a value of any type.
pub trait ParsedArgument: Sealed {
    /// Returns the range of this parsed argument.
    fn range(&self) -> StringRange;

    /// Returns the result of this parsed argument.
    fn result<'a>(&'a self) -> &(dyn Any + 'a);
}

// Implement our private trait for all TypedParsedArgument types.
impl<T> Sealed for TypedParsedArgument<T> {}

impl<T> ParsedArgument for TypedParsedArgument<T> {
    fn range(&self) -> StringRange {
        self.range
    }

    fn result<'a>(&'a self) -> &(dyn Any + 'a) {
        &self.result
    }
}