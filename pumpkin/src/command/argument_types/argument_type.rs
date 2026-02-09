use std::any::Any;
use std::pin::Pin;
use crate::command::{
    errors::command_syntax_error::CommandSyntaxError, string_reader::StringReader,
};
use crate::command::argument_types::argument_type::sealed::Sealed;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::suggestion::Suggestions;
use crate::command::suggestion::suggestions::SuggestionsBuilder;

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
    fn parse_with_source(
        &self,
        reader: &mut StringReader,
        _source: &CommandSource,
    ) -> Result<T, CommandSyntaxError> {
        self.parse(reader)
    }

    /// Provides a list of suggestions from this argument type.
    #[must_use]
    fn list_suggestions(
        &self,
        _context: &CommandContext,
        _suggestions_builder: &mut SuggestionsBuilder
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send>> {
        Box::pin(
            async move {
                Suggestions::empty()
            }
        )
    }

    /// Gets a selected list of examples which are considered
    /// valid when parsed into type `T`.
    ///
    /// Used for conflicts.
    #[must_use]
    fn examples(&self) -> &'static [&'static str] {
        &[]
    }
}

// Prevent other crates from using this trait
// Thus, we can effectively 'seal' our trait meant
// only for `AnyArgumentType`.
mod sealed {
    /// Private trait to ensure only types implementing `ArgumentType` can implement `AnyArgumentType`.
    pub trait Sealed {}
}

/// Represents an argument type with any parsable type.
pub trait AnyArgumentType: Sealed {
    /// Parses a value by using a [`StringReader`]. Call this only if you have no source.
    ///
    /// Errors should be propagated using the `?` operator, which will
    /// replicate Brigadier's behavior of exceptions.
    fn parse(&self, reader: &mut StringReader) -> Result<Box<dyn Any>, CommandSyntaxError>;

    /// Parses a value by using a [`StringReader`]. Call this only if you have no source.
    ///
    /// Errors should be propagated using the `?` operator, which will
    /// replicate Brigadier's behavior of exceptions.
    fn parse_with_source(&self, reader: &mut StringReader, source: &CommandSource) -> Result<Box<dyn Any>, CommandSyntaxError>;

    /// Provides a list of suggestions from this argument type.
    #[must_use]
    fn list_suggestions(
        &self,
        context: &CommandContext,
        suggestions_builder: &mut SuggestionsBuilder
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send>>;

    /// Gets a selected list of examples which are considered
    /// valid when parsed into type `T`.
    ///
    /// Used for conflicts.
    #[must_use]
    fn examples(&self) -> &'static [&'static str] {
        &[]
    }
}

// Implement our private trait for all argument types.
impl<T> Sealed for dyn ArgumentType<T> {}

impl<T: 'static> AnyArgumentType for dyn ArgumentType<T> {
    fn parse(&self, reader: &mut StringReader) -> Result<Box<dyn Any>, CommandSyntaxError> {
        match self.parse(reader) {
            Ok(value) => Ok(Box::new(value)),
            Err(error) => Err(error)
        }
    }

    fn parse_with_source(&self, reader: &mut StringReader, source: &CommandSource) -> Result<Box<dyn Any>, CommandSyntaxError> {
        match self.parse_with_source(reader, source) {
            Ok(value) => Ok(Box::new(value)),
            Err(error) => Err(error)
        }
    }

    fn list_suggestions(&self, context: &CommandContext, suggestions_builder: &mut SuggestionsBuilder) -> Pin<Box<dyn Future<Output = Suggestions> + Send>> {
        self.list_suggestions(context, suggestions_builder)
    }

    fn examples(&self) -> &'static [&'static str] {
        self.examples()
    }
}

