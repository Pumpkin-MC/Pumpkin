use crate::command::context::string_range::StringRange;
use crate::command::suggestion::{Suggestion, SuggestionText};
use pumpkin_util::text::TextComponent;
use std::borrow::Borrow;
use std::cmp::Ordering;

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
        &self.input[self.start.min(self.input.len())..]
    }

    /// Gets the remaining substring of the underlying lowercased input string.
    #[must_use]
    pub fn remaining_lowercase(&self) -> &str {
        &self.input_lowercase[self.start.min(self.input_lowercase.len())..]
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
    pub fn append(mut self, other: Self) -> Self {
        for suggestion in other.result {
            self.result.push(suggestion);
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

    /// Takes only the values that satisfy the current builder prefix, and
    /// suggests them. For this function to work currently, **all values provided
    /// must be in lowercase**.
    ///
    /// Example:
    /// - If the builder has `b` and the values are `acacia_boat`, `blue`, and `stick`, only the first two will get counted,
    ///   as `boat` and `blue` start with the letter `b`.
    /// - If the builder has `bl` instead, only `blue` will get counted.
    #[must_use]
    pub fn filter_and_suggest_lowercase(mut self, values: Vec<String>) -> Self {
        for value in values {
            if Self::matches_substr(self.remaining_lowercase(), &value) {
                self = self.suggest(value);
            }
        }
        self
    }

    /// Takes only the values that satisfy the current builder prefix, and
    /// suggests them.
    ///
    /// Example:
    /// - If the builder has `b` and the values are `acacia_boat`, `blue`, and `stick`, only the first two will get counted,
    ///   as `boat` and `blue` start with the letter `b`.
    /// - If the builder has `bl` instead, only `blue` will get counted.
    #[must_use]
    pub fn filter_and_suggest(mut self, values: &[&str]) -> Self {
        for value in values {
            if Self::matches_substr(self.remaining_lowercase(), &value.to_lowercase()) {
                self = self.suggest(value.to_string());
            }
        }
        self
    }

    /// Takes the value only if it satisfies the current builder prefix, and
    /// suggests them.
    #[must_use]
    pub fn filter_and_suggest_one(mut self, value: impl Into<SuggestionText>) -> Self {
        let value = value.into();
        if Self::matches_substr(
            self.remaining_lowercase(),
            &value.cached_text().to_lowercase(),
        ) {
            self = self.suggest(value);
        }
        self
    }

    /// Takes only the values that satisfy the current builder prefix, and
    /// suggests them.
    ///
    /// Example:
    /// - If the builder has `b` and the values are `acacia_boat`, `blue`, and `stick`, only the first two will get counted,
    ///   as `boat` and `blue` start with the letter `b`.
    /// - If the builder has `bl` instead, only `blue` will get counted.
    #[must_use]
    pub fn filter_and_suggest_iter(
        mut self,
        values: impl IntoIterator<Item = impl Into<SuggestionText>>,
    ) -> Self {
        for value in values {
            let value = value.into();
            if Self::matches_substr(
                self.remaining_lowercase(),
                &value.cached_text().to_lowercase(),
            ) {
                self = self.suggest(value);
            }
        }
        self
    }

    fn matches_substr(pattern: &str, input: &str) -> bool {
        let mut current_str = input;
        while !current_str.starts_with(pattern) {
            match current_str.find(['.', '_', '/']) {
                Some(pos) => current_str = &current_str[(pos + 1)..],
                None => return false,
            }
        }
        true
    }

    /// A helper method to suggest coordinate-related text.
    pub fn suggest_3d_coordinates(
        mut self,
        suggestions: TextCoordinates,
        validator: impl Fn(&str) -> bool,
    ) -> Suggestions {
        let input = self.remaining();
        let coordinate = suggestions.get_coordinate();

        if input.is_empty() {
            let full = format!("{coordinate} {coordinate} {coordinate}");
            if validator(&full) {
                self = self.filter_and_suggest_one(coordinate.to_string());
                self = self.filter_and_suggest_one(format!("{coordinate} {coordinate}"));
                self = self.filter_and_suggest_one(full);
            }
        } else {
            let mut split = input.split(' ');

            match (split.next(), split.next(), split.next()) {
                (Some(part1), None, None) => {
                    let full = format!("{part1} {coordinate} {coordinate}");
                    if validator(&full) {
                        let partial = format!("{part1} {coordinate}");
                        self = self.filter_and_suggest_one(partial);
                        self = self.filter_and_suggest_one(full);
                    }
                }
                (Some(part1), Some(part2), None) => {
                    let full = format!("{part1} {part2} {coordinate}");
                    if validator(&full) {
                        self = self.filter_and_suggest_one(full);
                    }
                }
                _ => {}
            }
        }

        self.build()
    }

    /// A helper method to suggest coordinate-related text.
    pub fn suggest_2d_coordinates(
        mut self,
        suggestions: TextCoordinates,
        validator: impl Fn(&str) -> bool,
    ) -> Suggestions {
        let input = self.remaining();
        let coordinate = suggestions.get_coordinate();

        if input.is_empty() {
            let full = format!("{coordinate} {coordinate}");
            if validator(&full) {
                self = self.filter_and_suggest_one(coordinate.to_string());
                self = self.filter_and_suggest_one(full);
            }
        } else {
            let mut split = input.split(' ');

            if let Some(part) = split.next()
                && split.next().is_none()
            {
                let full = format!("{part} {coordinate}");
                if validator(&full) {
                    self = self.filter_and_suggest_one(full);
                }
            }
        }

        self.build()
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

        let mut texts = Vec::new();

        for suggestions in &input {
            for suggestion in &suggestions.borrow().suggestions {
                if !texts.contains(&suggestion) {
                    texts.push(suggestion);
                }
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
            .expect("Suggestions list is not empty, so range should exist");

        let mut texts = Vec::new();
        for suggestion in &suggestions {
            let suggestion = suggestion.borrow().expand(command, range);
            if !texts.contains(&suggestion) {
                texts.push(suggestion);
            }
        }

        Self::new(range, Self::sort(texts))
    }

    /// Sorts a set of [`Suggestion`]s, in the following precedence:
    ///
    /// 1. If both suggestions are integers, their integral value is compared.
    /// 2. Otherwise, compare their text lexicographically.
    fn sort(suggestions: Vec<Suggestion>) -> Vec<Suggestion> {
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
                    let text_lowercase = text.to_lowercase();
                    text_suggestions.push((
                        text,
                        suggestion.tooltip,
                        suggestion.range,
                        text_lowercase,
                    ));
                }
                SuggestionText::Integer { cached_text, value } => integer_suggestions.push((
                    cached_text,
                    value,
                    suggestion.tooltip,
                    suggestion.range,
                )),
            }
        }

        text_suggestions.sort_by(|a, b| a.3.cmp(&b.3));
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
                    Ordering::Equal => {
                        tracing::error!("Duplicate suggestion found during merge");
                        PushSide::Text
                    }
                },
                (Some(_), None) => PushSide::Text,
                (None, Some(_)) => PushSide::Integer,
                (None, None) => PushSide::Break,
            };

            match side {
                PushSide::Text => {
                    let text = text_iter.next().expect(
                        "text_iter should have a next value because side is PushSide::Text",
                    );
                    suggestions.push(Suggestion {
                        text: SuggestionText::Text(text.0),
                        tooltip: text.1,
                        range: text.2,
                    });
                }
                PushSide::Integer => {
                    let text = integer_iter.next().expect(
                        "integer_iter should have a next value because side is PushSide::Integer",
                    );
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

/// Represents server-side only coordinate suggestions.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TextCoordinates {
    /// Represents `^ ^ ^`.
    Local,

    /// Represents `~ ~ ~`.
    Global,
}

impl TextCoordinates {
    /// Get the symbol of a coordinate of this suggestion set.
    #[must_use]
    pub const fn get_coordinate(self) -> &'static str {
        match self {
            Self::Local => "^",
            Self::Global => "~",
        }
    }
}
