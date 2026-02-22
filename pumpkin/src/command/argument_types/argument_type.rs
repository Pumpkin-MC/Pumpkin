use pumpkin_protocol::java::client::play::SuggestionProviders;

use crate::command::argument_types::argument_type::sealed::Sealed;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::suggestion::suggestions::Suggestions;
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::command::{
    errors::command_syntax_error::CommandSyntaxError, string_reader::StringReader,
};
use std::any::Any;
use std::pin::Pin;

pub type JavaClientArgumentType<'a> = pumpkin_protocol::java::client::play::ArgumentType<'a>;

/// Represents an argument type that parses a particular type `Item`.
pub trait ArgumentType: Send + Sync {
    /// The data type that this argument type parses.
    type Item: Send + Sync;

    /// Parses a `T` by using a [`StringReader`]. Call this only if you have no source.
    ///
    /// Errors should be propagated using the `?` operator, which will
    /// replicate Brigadier's behavior of exceptions.
    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError>;

    /// Parses a `T` by using a [`StringReader`],
    /// along with a particular source of type `S`.
    ///
    /// Errors should be propagated using the `?` operator, which will
    /// replicate Brigadier's behavior of exceptions.
    fn parse_with_source(
        &self,
        reader: &mut StringReader,
        _source: &CommandSource,
    ) -> Result<Self::Item, CommandSyntaxError> {
        self.parse(reader)
    }

    /// Provides a list of suggestions from this argument type.
    #[must_use]
    fn list_suggestions(
        &self,
        _context: &CommandContext,
        _suggestions_builder: &mut SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send>> {
        Box::pin(async move { Suggestions::empty() })
    }

    /// Returns the Java client-side parser used for this argument type.
    #[must_use]
    fn client_side_parser(&self) -> JavaClientArgumentType;

    /// Overrides the suggestion providers provided from this argument if a [`Some`] containing them
    /// is returned.
    #[must_use]
    fn override_suggestion_providers(&self) -> Option<SuggestionProviders> {
        None
    }

    /// Gets a selected list of examples which are considered
    /// valid when parsed into type `T`.
    ///
    /// Used for conflicts.
    #[must_use]
    fn examples(&self) -> Vec<String> {
        Vec::new()
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
pub trait AnyArgumentType: Sealed + Send + Sync {
    /// Parses a value by using a [`StringReader`]. Call this only if you have no source.
    ///
    /// Errors should be propagated using the `?` operator, which will
    /// replicate Brigadier's behavior of exceptions.
    fn parse(
        &self,
        reader: &mut StringReader,
    ) -> Result<Box<dyn Any + Send + Sync>, CommandSyntaxError>;

    /// Parses a value by using a [`StringReader`]. Call this only if you have no source.
    ///
    /// Errors should be propagated using the `?` operator, which will
    /// replicate Brigadier's behavior of exceptions.
    fn parse_with_source(
        &self,
        reader: &mut StringReader,
        source: &CommandSource,
    ) -> Result<Box<dyn Any + Send + Sync>, CommandSyntaxError>;

    /// Provides a list of suggestions from this argument type.
    #[must_use]
    fn list_suggestions(
        &self,
        context: &CommandContext,
        suggestions_builder: &mut SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send>>;

    /// Returns the Java client-side parser used for this argument type.
    #[must_use]
    fn client_side_parser(&self) -> JavaClientArgumentType;

    /// Overrides the suggestion providers provided from this argument if a [`Some`] containing them
    /// is returned.
    #[must_use]
    fn override_suggestion_providers(&self) -> Option<SuggestionProviders> {
        None
    }

    /// Gets a selected list of examples which are considered
    /// valid when parsed into type `T`.
    ///
    /// Used for conflicts.
    #[must_use]
    fn examples(&self) -> Vec<String> {
        Vec::new()
    }
}

// Implement our private trait for all argument types.
impl<U: ArgumentType<Item = T>, T: Send + Sync + 'static> Sealed for U {}

impl<U: ArgumentType<Item = T>, T: Send + Sync + 'static> AnyArgumentType for U {
    fn parse(
        &self,
        reader: &mut StringReader,
    ) -> Result<Box<dyn Any + Send + Sync>, CommandSyntaxError> {
        match self.parse(reader) {
            Ok(value) => Ok(Box::new(value)),
            Err(error) => Err(error),
        }
    }

    fn parse_with_source(
        &self,
        reader: &mut StringReader,
        source: &CommandSource,
    ) -> Result<Box<dyn Any + Send + Sync>, CommandSyntaxError> {
        match self.parse_with_source(reader, source) {
            Ok(value) => Ok(Box::new(value)),
            Err(error) => Err(error),
        }
    }

    fn list_suggestions(
        &self,
        context: &CommandContext,
        suggestions_builder: &mut SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send>> {
        self.list_suggestions(context, suggestions_builder)
    }

    fn examples(&self) -> Vec<String> {
        self.examples()
    }
    
    fn client_side_parser(&self) -> pumpkin_protocol::java::client::play::ArgumentType {
        self.client_side_parser()
    }

    fn override_suggestion_providers(&self) -> Option<SuggestionProviders> {
        self.override_suggestion_providers()
    }
}
