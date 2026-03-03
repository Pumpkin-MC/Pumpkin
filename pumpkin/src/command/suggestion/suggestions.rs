use crate::command::context::string_range::StringRange;
use crate::command::suggestion::{Suggestion, SuggestionText};
use pumpkin_util::text::TextComponent;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashSet;

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
    pub fn new(input: &str, start: usize) -> Self {
        Self {
            input: input.to_string(),
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
    pub fn append(mut self, other: &Self) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Suggestions {
    pub range: StringRange,
    pub suggestions: Vec<Suggestion>,
}

impl Suggestions {
    /// Constructs a new [`Suggestions`] structure from
    /// a range and [`Suggestion`]s.
    #[must_use]
    pub const fn new(range: StringRange, suggestions: Vec<Suggestion>) -> Self {
        Self { range, suggestions }
    }

    /// Constructs a new [`Suggestions`] of zero size and no range.
    #[must_use]
    pub const fn empty() -> Self {
        Self::new(StringRange::at(0), vec![])
    }

    /// Returns whether this [`Suggestions`] *is* of zero size.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.suggestions.is_empty()
    }

    /// Merges all [`Suggestions`] provided with a command into a single [`Suggestions`].
    #[must_use]
    pub fn merge<I, S>(command: &str, input: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Borrow<Self>,
    {
        let input: Vec<S> = input.into_iter().collect();

        if input.is_empty() {
            return Self::empty();
        } else if input.len() == 1 {
            return input[0].borrow().clone();
        }

        let mut texts = HashSet::new();

        for suggestions in &input {
            for suggestion in &suggestions.borrow().suggestions {
                texts.insert(suggestion);
            }
        }

        Self::create(command, texts)
    }

    /// Creates a single [`Suggestions`] structure from
    /// many [`Suggestion`]s and a command.
    #[must_use]
    pub fn create<I, S>(command: &str, suggestions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Borrow<Suggestion>,
    {
        let suggestions: Vec<S> = suggestions.into_iter().collect();

        if suggestions.is_empty() {
            return Self::empty();
        }

        // First, we figure out the range encompassing all suggestions provided.
        let range = suggestions
            .iter()
            .map(|s| s.borrow().range)
            .reduce(StringRange::encompass)
            .unwrap();

        let mut texts: HashSet<Suggestion> = HashSet::new();
        for suggestion in &suggestions {
            texts.insert(suggestion.borrow().expand(command, range));
        }

        Self::new(range, Self::sort(texts))
    }

    /// Sorts a set of [`Suggestion`]s, in the following precedence:
    ///
    /// 1. If both suggestions are integers, their integral value is compared.
    /// 2. Otherwise, compare their text lexicographically.
    fn sort(suggestions: HashSet<Suggestion>) -> Vec<Suggestion> {
        enum PushSide {
            Text,
            Integer,
            Break,
        }

        let mut text_suggestions = Vec::new();
        let mut integer_suggestions = Vec::new();

        let len = suggestions.len();

        for suggestion in suggestions {
            match suggestion.text {
                SuggestionText::Text(text) => {
                    text_suggestions.push((text, suggestion.tooltip, suggestion.range));
                }
                SuggestionText::Integer { cached_text, value } => integer_suggestions.push((
                    cached_text,
                    value,
                    suggestion.tooltip,
                    suggestion.range,
                )),
            }
        }

        // We need not preserve the original order as
        // there cannot be two or more equivalent suggestions in a set.
        text_suggestions.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        integer_suggestions.sort_unstable_by_key(|x| x.1);

        let mut text_iter = text_suggestions.into_iter().peekable();
        let mut integer_iter = integer_suggestions.into_iter().peekable();

        let mut suggestions = Vec::with_capacity(len);

        loop {
            let text = text_iter.peek();
            let integer = integer_iter.peek();

            let side = match (text, integer) {
                (Some(text), Some(integer)) => match text.0.cmp(&integer.0) {
                    Ordering::Less => PushSide::Text,
                    Ordering::Greater => PushSide::Integer,
                    Ordering::Equal => unreachable!(),
                },
                (Some(_), None) => PushSide::Text,
                (None, Some(_)) => PushSide::Integer,
                (None, None) => PushSide::Break,
            };

            match side {
                PushSide::Text => {
                    let text = text_iter.next().unwrap();
                    suggestions.push(Suggestion {
                        text: SuggestionText::Text(text.0),
                        tooltip: text.1,
                        range: text.2,
                    });
                }
                PushSide::Integer => {
                    let text = integer_iter.next().unwrap();
                    suggestions.push(Suggestion {
                        text: SuggestionText::Integer {
                            cached_text: text.0,
                            value: text.1,
                        },
                        tooltip: text.2,
                        range: text.3,
                    });
                }
                PushSide::Break => break,
            }
        }

        suggestions
    }
}
