pub mod suggestions;

use pumpkin_util::text::TextComponent;
use std::fmt::Debug;
use std::hash::Hash;

use crate::command::context::string_range::StringRange;

/// A structure that describes the text of a suggestion.
/// It's actual value can either be a [`String`], or an [`i32`].
///
/// Use the [`new`] method to create new [`SuggestionType`]s.
///
/// If you want to use an `i32` for a suggestion's text,
/// go with [`SuggestionType::Integer`]. In all other cases,
/// go with [`SuggestionType::Text`].
///
/// # Invariant
/// A [`SuggestionText::Text`] **shall not exist** if it can instead
/// be fully expressed as a [`SuggestionText::Integer`] (no leading zeros).
/// This is important to establish proper ordering.
/// ```
/// use pumpkin::command::suggestion::SuggestionText;
///
/// let five_suggestion_1 = SuggestionText::new(5);
/// let five_suggestion_2 = SuggestionText::new("5");
/// let zero_five_suggestion = SuggestionText::new("05");
///
/// // `five_suggestion_1` and `five_suggestion_2`
/// // are both instances of `SuggestionText::Integer`,
/// // as guaranteed by the invariant, both having
/// // the same integer `5`.
/// assert_eq!(five_suggestion_1, five_suggestion_2);
///
/// // `zero_five_suggestion` contains a leading zero,
/// // and hence does not have an integer representing it
/// // fully, so it is an instance of `SuggestionText::Text`.
/// assert_ne!(five_suggestion_1, zero_five_suggestion);
/// ```
/// Violating this invariant is a logic error.
///
/// [`new`]: SuggestionText::new
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
        if let Ok(integer) = text.parse::<i32>()
            && integer.to_string() == text
        {
            Self::Integer {
                cached_text: text,
                value: integer,
            }
        } else {
            Self::Text(text)
        }
    }
}

impl From<&str> for SuggestionText {
    fn from(text: &str) -> Self {
        text.to_owned().into()
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
    pub const fn cached_text(&self) -> &String {
        match self {
            Self::Text(text) => text,
            Self::Integer { cached_text, .. } => cached_text,
        }
    }

    /// Creates a new [`SuggestionText`] from a usable value.
    /// This value can be a `&str`, a [`String`], or an `i32`.
    pub fn new(value: impl Into<Self>) -> Self {
        value.into()
    }
}

impl Eq for SuggestionText {}
impl PartialEq for SuggestionText {
    fn eq(&self, other: &Self) -> bool {
        self.cached_text() == other.cached_text()
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

#[cfg(test)]
mod test {
    use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
    use crate::command::{context::string_range::StringRange, suggestion::Suggestion};
    use std::slice;

    #[test]
    fn apply_insertion_start() {
        let suggestion = Suggestion::without_tooltip(StringRange::at(0), "Pumpkin once said: ");
        assert_eq!(
            suggestion.apply("'Server is now running'"),
            "Pumpkin once said: 'Server is now running'".to_owned()
        );
    }

    #[test]
    fn apply_insertion_middle() {
        let suggestion = Suggestion::without_tooltip(StringRange::at(6), "Efficient, ");
        assert_eq!(
            suggestion.apply("Fast, and User-Friendly"),
            "Fast, Efficient, and User-Friendly".to_owned()
        );
    }

    #[test]
    fn apply_insertion_end() {
        let suggestion = Suggestion::without_tooltip(StringRange::at(10), " has stopped");
        assert_eq!(
            suggestion.apply("The server"),
            "The server has stopped".to_owned()
        );
    }

    #[test]
    fn apply_replacement_start() {
        let suggestion = Suggestion::without_tooltip(StringRange::between(0, 5), "Goodbye");
        assert_eq!(
            suggestion.apply("Hello world!"),
            "Goodbye world!".to_owned()
        );
    }

    #[test]
    fn apply_replacement_middle() {
        let suggestion = Suggestion::without_tooltip(StringRange::between(6, 11), "melon");
        assert_eq!(suggestion.apply("Hello world!"), "Hello melon!".to_owned());
    }

    #[test]
    fn apply_replacement_end() {
        let suggestion = Suggestion::without_tooltip(StringRange::between(13, 23), "fruit.");
        assert_eq!(
            suggestion.apply("Pumpkin is a vegetable."),
            "Pumpkin is a fruit.".to_owned()
        );
    }

    #[test]
    fn apply_replacement_everything() {
        let suggestion =
            Suggestion::without_tooltip(StringRange::between(0, 36), "This is a phrase.");
        assert_eq!(
            suggestion.apply("I'm not related to the other phrase."),
            "This is a phrase.".to_owned()
        );
    }

    #[test]
    fn expand_unchanged() {
        let suggestion = Suggestion::without_tooltip(StringRange::at(1), "oo");
        assert_eq!(suggestion.expand("f", StringRange::at(1)), suggestion);
    }

    #[test]
    fn expand_left() {
        let suggestion = Suggestion::without_tooltip(StringRange::at(1), "oo");
        assert_eq!(
            suggestion.expand("f", StringRange::between(0, 1)),
            Suggestion::without_tooltip(StringRange::between(0, 1), "foo")
        );
    }

    #[test]
    fn expand_right() {
        let suggestion = Suggestion::without_tooltip(StringRange::at(0), "ba");
        assert_eq!(
            suggestion.expand("r", StringRange::between(0, 1)),
            Suggestion::without_tooltip(StringRange::between(0, 1), "bar")
        );
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
        );
    }

    #[test]
    fn expand_replacement() {
        let suggestion = Suggestion::without_tooltip(StringRange::between(6, 11), "everyone");
        assert_eq!(
            suggestion.expand("Hello world!", StringRange::between(0, 12)),
            Suggestion::without_tooltip(StringRange::between(0, 12), "Hello everyone!")
        );
    }

    #[test]
    fn merge_empty() {
        let merged = Suggestions::merge("foo b", &[]);
        assert!(merged.is_empty());
    }

    #[test]
    fn merge_single() {
        let suggestions = Suggestions::new(
            StringRange::at(5),
            vec![Suggestion::without_tooltip(StringRange::at(5), "ar")],
        );
        let merged = Suggestions::merge("foo b", slice::from_ref(&suggestions));
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

    #[test]
    fn suggest_append() {
        let suggestions = SuggestionsBuilder::new("Hello w", 6)
            .suggest("world!")
            .build();

        assert_eq!(
            suggestions.suggestions,
            vec![Suggestion::without_tooltip(
                StringRange::between(6, 7),
                "world!"
            )]
        );
        assert_eq!(suggestions.range, StringRange::between(6, 7));
    }

    #[test]
    fn suggest_replace() {
        let suggestions = SuggestionsBuilder::new("Hello w", 6)
            .suggest("everyone!")
            .build();

        assert_eq!(
            suggestions.suggestions,
            vec![Suggestion::without_tooltip(
                StringRange::between(6, 7),
                "everyone!"
            )]
        );
        assert_eq!(suggestions.range, StringRange::between(6, 7));
    }

    #[test]
    fn suggest_noop() {
        let suggestions = SuggestionsBuilder::new("hello", 6).build();

        assert!(suggestions.is_empty());
    }

    #[test]
    fn suggest_multiple() {
        let suggestions = SuggestionsBuilder::new("Cut a b", 6)
            .suggest("banana")
            .suggest("plum")
            .suggest("tomato")
            .build();

        assert_eq!(
            suggestions.suggestions,
            vec![
                Suggestion::without_tooltip(StringRange::between(6, 7), "banana"),
                Suggestion::without_tooltip(StringRange::between(6, 7), "plum"),
                Suggestion::without_tooltip(StringRange::between(6, 7), "tomato")
            ]
        );
        assert_eq!(suggestions.range, StringRange::between(6, 7));
    }

    #[test]
    fn sort() {
        let suggestions = SuggestionsBuilder::new("A random thing to say is foobar", 25)
            .suggest("1")
            .suggest(9)
            .suggest("4")
            .suggest(6)
            .suggest("05")
            .suggest(533)
            .suggest("x8")
            .suggest("a")
            .suggest("x")
            .suggest("6x")
            .build();

        let internal_sorted_repr: Vec<String> = suggestions
            .suggestions
            .into_iter()
            .map(|suggestion| suggestion.text_as_string())
            .collect();

        assert_eq!(
            internal_sorted_repr,
            vec!["05", "1", "4", "6", "6x", "9", "533", "a", "x", "x8"]
        );
    }
}
