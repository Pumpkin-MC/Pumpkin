use crate::command::context::string_range::StringRange;
use crate::command::suggestion::{Suggestion, SuggestionText, Suggestions};
use pumpkin_util::text::TextComponent;

/// Represents a builder of [`Suggestion`]s.
pub struct SuggestionsBuilder {
    /// Represents the starting position of the [`SuggestionsBuilder`]
    /// from the start of the input string.
    pub start: usize,

    /// Represents the input of the [`SuggestionsBuilder`].
    pub input: String,

    /// Represents the lowercase version of the input of the [`SuggestionsBuilder`].
    pub input_lowercase: String,

    /// The eventual result of this [`SuggestionsBuilder`].
    pub result: Vec<Suggestion>,
}

impl SuggestionsBuilder {
    /// Constructs a new [`SuggestionsBuilder`] from the given
    /// input string and a starting position relative to it.
    #[must_use]
    pub fn new(input: String, start: usize) -> Self {
        Self {
            input: input.clone(),
            input_lowercase: input.to_lowercase(),
            start,
            result: Vec::new(),
        }
    }

    /// Gets the remaining substring of the underlying input string.
    #[must_use]
    pub fn remaining(&self) -> &str {
        &self.input[self.start..]
    }

    /// Gets the remaining substring of the underlying lowercased input string.
    #[must_use]
    pub fn remaining_lowercase(&self) -> &str {
        &self.input_lowercase[self.start..]
    }

    /// Builds the [`Suggestions`] object, consuming itself in the process.
    #[must_use]
    pub fn build(self) -> Suggestions {
        Suggestions::create(&self.input, self.result)
    }

    /// Adds a suggestion without a tooltip to this builder.
    #[must_use]
    pub fn suggest<T>(mut self, text: T) -> Self
    where
        T: Into<SuggestionText>,
    {
        let text = text.into();
        if text.cached_text() != self.remaining() {
            self.result.push(Suggestion::without_tooltip(
                StringRange::between(self.start, self.input.len()),
                text,
            ));
        }
        self
    }

    /// Adds a suggestion with a tooltip to this builder.
    #[must_use]
    pub fn suggest_with_tooltip<T>(mut self, text: T, tooltip: TextComponent) -> Self
    where
        T: Into<SuggestionText>,
    {
        let text = text.into();
        if text.cached_text() != self.remaining() {
            self.result.push(Suggestion::with_tooltip(
                StringRange::between(self.start, self.input.len()),
                text,
                tooltip,
            ));
        }
        self
    }

    /// Adds all suggestions from another [`SuggestionsBuilder`] to this one.
    #[must_use]
    pub fn add(mut self, other: &Self) -> Self {
        for suggestion in &other.result {
            self.result.push(suggestion.clone());
        }
        self
    }

    /// Creates another [`SuggestionsBuilder`] from this one
    /// by copying the input and taking the starting position.
    #[must_use]
    pub fn create_offset(&self, start: usize) -> Self {
        Self {
            input: self.input.clone(),
            input_lowercase: self.input_lowercase.clone(),
            start,
            result: Vec::new(),
        }
    }
}
