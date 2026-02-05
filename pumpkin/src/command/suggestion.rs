use pumpkin_util::text::TextComponent;
use std::hash::Hash;
use std::{borrow::Borrow, cmp::Ordering, collections::HashSet, fmt::Debug};

use crate::command::context::string_range::StringRange;

/// A structure that describes the text of a suggestion.
/// It can either contain a [`String`], or an [`i32`].
///
/// If you want to use an `i32` for a suggestion's text,
/// go with [`SuggestionType::Integer`]. In all other cases,
/// go with [`SuggestionType::Text`].
#[derive(Debug, Clone)]
pub enum SuggestionText {
    /// The normal one to use. Stores a [`String`].
    Text(String),

    /// The one to use for integral suggestions. Stores an [`i32`].
    /// Note that a cached [`String`] is stored inside this value
    /// so that [`String`] allocations don't occur when this object is compared.
    Integer { cached_text: String, value: i32 },
}

impl From<String> for SuggestionText {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<&str> for SuggestionText {
    fn from(text: &str) -> Self {
        Self::Text(text.to_owned())
    }
}

impl From<i32> for SuggestionText {
    fn from(text: i32) -> Self {
        Self::Integer {
            cached_text: text.to_string(),
            value: text,
        }
    }
}

impl SuggestionText {
    /// Provides the internally cached text: this is important so that
    /// we don't allocate a new string every time we want to
    /// compare two [`SuggestionText`]s.
    #[must_use]
    const fn cached_text(&self) -> &String {
        match self {
            Self::Text(text) => text,
            Self::Integer { cached_text, .. } => cached_text,
        }
    }
}

impl Ord for SuggestionText {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a.cmp(b),
            (Self::Integer { cached_text: a, .. }, Self::Integer { cached_text: b, .. }) => {
                a.cmp(b)
            }
            (a, b) => a.cached_text().cmp(b.cached_text()),
        }
    }
}

impl PartialOrd for SuggestionText {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for SuggestionText {}
impl PartialEq for SuggestionText {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Hash for SuggestionText {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.cached_text().hash(state);
    }
}

/// A structure that describes a suggestion
/// that may be applied to a string or
/// expanded using a command and range.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Suggestion {
    pub range: StringRange,
    pub text: SuggestionText,
    pub tooltip: Option<TextComponent>,
}

impl Suggestion {
    /// Constructs a [`Suggestion`] from its range and text (which can either be a
    /// [`String`] or an [`i32`]).
    #[must_use]
    pub fn without_tooltip<T>(range: StringRange, text: T) -> Self
    where
        T: Into<SuggestionText>,
    {
        Self {
            range,
            text: text.into(),
            tooltip: None,
        }
    }

    /// Constructs a [`Suggestion`] from its range, text (which can either be a
    /// [`String`] or an [`i32`]), and a tooltip component.
    #[must_use]
    pub fn with_tooltip<T>(range: StringRange, text: T, tooltip: TextComponent) -> Self
    where
        T: Into<SuggestionText>,
    {
        Self {
            range,
            text: text.into(),
            tooltip: Some(tooltip),
        }
    }

    /// Constructs a [`Suggestion`] from its range, text (which can either be a
    /// [`String`] or an [`i32`]), and an [`Option`] of [`TextComponent`].
    #[must_use]
    pub fn new<T>(range: StringRange, text: T, tooltip: Option<TextComponent>) -> Self
    where
        T: Into<SuggestionText>,
    {
        Self {
            range,
            text: text.into(),
            tooltip,
        }
    }

    /// Gets the internal [`SuggestionText`] that represents the text of this suggestion,
    /// but as a String cloned from the cache.
    #[must_use]
    pub fn text_as_string(&self) -> String {
        self.text_as_string_ref().clone()
    }

    /// Gets the internal [`SuggestionText`] that represents the text of this suggestion,
    /// but as a reference of a String taken directly from the cache without any cloning.
    #[must_use]
    pub const fn text_as_string_ref(&self) -> &String {
        self.text.cached_text()
    }

    /// Gets the internal [`SuggestionText`] that represents the text of this suggestion,
    /// but as a `&str` taken directly from the cache without any cloning.
    #[must_use]
    pub const fn text_as_str(&self) -> &str {
        self.text.cached_text().as_str()
    }

    /// Applies this [`Suggestion`] into a string,
    /// returning a new [`String`] with the applied suggestion.
    #[must_use]
    pub fn apply(&self, input: &str) -> String {
        let text_string = self.text_as_string_ref();

        if self.range.start == 0 && self.range.end == input.len() {
            return text_string.clone();
        }
        let mut result: String =
            String::with_capacity(input.len() - self.range.len() + text_string.len());
        result.push_str(&input[0..self.range.start]); // usize >= 0
        result.push_str(text_string);
        if self.range.end < input.len() {
            result.push_str(&input[self.range.end..]);
        }
        result
    }

    /// Expands this [`Suggestion`] onto a command with a [`StringRange`],
    /// returning a new [`Suggestion`].
    #[must_use]
    pub fn expand(&self, command: &str, range: StringRange) -> Self {
        if self.range == range {
            return Self::new(self.range, self.text.clone(), self.tooltip.clone());
        }
        let mut result = String::new();
        if range.start < self.range.start {
            result.push_str(&command[range.start..self.range.start]);
        }
        result.push_str(&self.text_as_string());
        if range.end > self.range.end {
            result.push_str(&command[self.range.end..range.end]);
        }
        Self::new(range, result, self.tooltip.clone())
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

        let mut texts: Vec<_> = texts.into_iter().collect();
        texts.sort_by(|a, b| a.text.cmp(&b.text));

        Self::new(range, texts)
    }
}

#[cfg(test)]
mod test {
    use crate::command::{
        context::string_range::StringRange,
        suggestion::{Suggestion, Suggestions},
    };

    #[test]
    fn apply_insertion_start() {
        let suggestion = Suggestion::without_tooltip(StringRange::at(0), "Pumpkin once said: ");
        assert_eq!(
            suggestion.apply("'Server is now running'"),
            "Pumpkin once said: 'Server is now running'".to_owned()
        )
    }

    #[test]
    fn apply_insertion_middle() {
        let suggestion = Suggestion::without_tooltip(StringRange::at(6), "Efficient, ");
        assert_eq!(
            suggestion.apply("Fast, and User-Friendly"),
            "Fast, Efficient, and User-Friendly".to_owned()
        )
    }

    #[test]
    fn apply_insertion_end() {
        let suggestion = Suggestion::without_tooltip(StringRange::at(10), " has stopped");
        assert_eq!(
            suggestion.apply("The server"),
            "The server has stopped".to_owned()
        )
    }

    #[test]
    fn apply_replacement_start() {
        let suggestion = Suggestion::without_tooltip(StringRange::between(0, 5), "Goodbye");
        assert_eq!(
            suggestion.apply("Hello world!"),
            "Goodbye world!".to_owned()
        )
    }

    #[test]
    fn apply_replacement_middle() {
        let suggestion = Suggestion::without_tooltip(StringRange::between(6, 11), "melon");
        assert_eq!(suggestion.apply("Hello world!"), "Hello melon!".to_owned())
    }

    #[test]
    fn apply_replacement_end() {
        let suggestion = Suggestion::without_tooltip(StringRange::between(13, 23), "fruit.");
        assert_eq!(
            suggestion.apply("Pumpkin is a vegetable."),
            "Pumpkin is a fruit.".to_owned()
        )
    }

    #[test]
    fn apply_replacement_everything() {
        let suggestion =
            Suggestion::without_tooltip(StringRange::between(0, 36), "This is a phrase.");
        assert_eq!(
            suggestion.apply("I'm not related to the other phrase."),
            "This is a phrase.".to_owned()
        )
    }

    #[test]
    fn expand_unchanged() {
        let suggestion = Suggestion::without_tooltip(StringRange::at(1), "oo");
        assert_eq!(suggestion.expand("f", StringRange::at(1)), suggestion)
    }

    #[test]
    fn expand_left() {
        let suggestion = Suggestion::without_tooltip(StringRange::at(1), "oo");
        assert_eq!(
            suggestion.expand("f", StringRange::between(0, 1)),
            Suggestion::without_tooltip(StringRange::between(0, 1), "foo")
        )
    }

    #[test]
    fn expand_right() {
        let suggestion = Suggestion::without_tooltip(StringRange::at(0), "ba");
        assert_eq!(
            suggestion.expand("r", StringRange::between(0, 1)),
            Suggestion::without_tooltip(StringRange::between(0, 1), "bar")
        )
    }

    #[test]
    fn expand_both() {
        let suggestion = Suggestion::without_tooltip(
            StringRange::at(30),
            "sheared to make a Carved Pumpkin and can be ",
        );
        assert_eq!(
            suggestion.expand(
                "A block called Pumpkin can be crafted into its seeds which can be planted",
                StringRange::between(0, 52)
            ),
            Suggestion::without_tooltip(
                StringRange::between(0, 52),
                "A block called Pumpkin can be sheared to make a Carved Pumpkin and can be crafted into its seeds"
            )
        )
    }

    #[test]
    fn expand_replacement() {
        let suggestion = Suggestion::without_tooltip(StringRange::between(6, 11), "everyone");
        assert_eq!(
            suggestion.expand("Hello world!", StringRange::between(0, 12)),
            Suggestion::without_tooltip(StringRange::between(0, 12), "Hello everyone!")
        )
    }

    #[test]
    fn merge_empty() {
        let merged = Suggestions::merge("foo b", &[]);
        assert!(merged.is_empty())
    }

    #[test]
    fn merge_single() {
        let suggestions = Suggestions::new(
            StringRange::at(5),
            vec![Suggestion::without_tooltip(StringRange::at(5), "ar")],
        );
        let merged = Suggestions::merge("foo b", &[suggestions.clone()]);
        assert_eq!(merged, suggestions);
    }

    #[test]
    fn merge_multiple() {
        let a = Suggestions::new(
            StringRange::at(5),
            vec![
                Suggestion::without_tooltip(StringRange::at(5), "ar"),
                Suggestion::without_tooltip(StringRange::at(5), "az"),
                Suggestion::without_tooltip(StringRange::at(5), "ars"),
            ],
        );
        let b = Suggestions::new(
            StringRange::between(4, 5),
            vec![
                Suggestion::without_tooltip(StringRange::between(4, 5), "foo"),
                Suggestion::without_tooltip(StringRange::between(4, 5), "qux"),
                Suggestion::without_tooltip(StringRange::between(4, 5), "BAR"),
            ],
        );
        let merged = Suggestions::merge("foo b", &[a, b]);
        assert_eq!(
            &merged.suggestions,
            &[
                Suggestion::without_tooltip(StringRange::between(4, 5), "BAR"),
                Suggestion::without_tooltip(StringRange::between(4, 5), "bar"),
                Suggestion::without_tooltip(StringRange::between(4, 5), "bars"),
                Suggestion::without_tooltip(StringRange::between(4, 5), "baz"),
                Suggestion::without_tooltip(StringRange::between(4, 5), "foo"),
                Suggestion::without_tooltip(StringRange::between(4, 5), "qux"),
            ]
        );
    }
}
