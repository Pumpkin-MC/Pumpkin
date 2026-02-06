use crate::command::{
    errors::command_syntax_error::CommandSyntaxError, string_reader::StringReader,
};

/// Represents an argument type that parses a particular type `T`.
pub trait ArgumentType<T> {
    /// Parses a `T` by using a [`StringReader`]. Call this only if you have no source.
    ///
    /// Errors should be propagated using the `?` operator, which will
    /// replicate Brigadier's behavior of exceptions.
    fn parse(&self, reader: &mut StringReader) -> Result<T, CommandSyntaxError>;

    /// Parses a `T` by using a [`StringReader`],
    /// along with a particular source of type `S`.
    ///
    /// Errors should be propagated using the `?` operator, which will
    /// replicate Brigadier's behavior of exceptions.
    fn parse_with_source<S>(
        &self,
        reader: &mut StringReader,
        _source: &S,
    ) -> Result<T, CommandSyntaxError> {
        self.parse(reader)
    }

    // TODO: Add suggestions here when command context is implemented.

    /// Gets a selected list of examples which are considered
    /// valid when parsed into type `T`.
    ///
    /// Used for conflicts.
    #[must_use]
    fn examples(&self) -> &'static [&'static str] {
        &[]
    }
}
