use crate::context::command_context::CommandContext;
use crate::source::{CommandSource, DummySource};
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

/// The [`Suggestions`] given by a [`SuggestionProvider`].
pub type SuggestionProviderResult = Suggestions;

/// A trait allowing an object to provide suggestions using a
/// [`CommandContext`] and [`SuggestionsBuilder`].
pub trait SuggestionProvider<S: CommandSource = DummySource>: Send + Sync {
    /// Uses a [`CommandContext`] and [`SuggestionsBuilder`] to suggest.
    ///
    /// # Arguments
    /// - `context`: The context to use for building the suggestions.
    /// - `builder`: The builder to consume for the suggestions.
    ///
    /// # Returns
    /// The [`Suggestions`] representing the suggested items.
    fn suggest(
        &self,
        context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult;
}
