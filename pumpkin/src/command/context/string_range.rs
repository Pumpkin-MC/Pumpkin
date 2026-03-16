use std::ops::Range;

use crate::command::string_reader::StringReader;

/// Indicates a range that is effectively
/// a substring of a string from its `start`
/// and `end` byte-indices.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StringRange {
    pub start: usize,
    pub end: usize,
}

impl StringRange {
    /// Constructs a new substring range, with indices
    /// inclusive to `start`, but exclusive to `end`.
    #[must_use]
    pub const fn between(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Constructs an empty range on the left side of
    /// one character with index `pos`.
    #[must_use]
    pub const fn at(pos: usize) -> Self {
        Self::between(pos, pos)
    }

    /// Constructs a new substring range encompassing two
    /// [`StringRange`]s, returning a new one that covers
    /// both the required ranges.
    #[must_use]
    pub fn encompass(a: Self, b: Self) -> Self {
        Self::between(a.start.min(b.start), a.end.max(b.end))
    }

    /// Gets a [`str`] substring slice bound
    /// to a [`StringReader`] from this range.
    #[must_use]
    pub fn slice_from_reader<'a>(&self, reader: &'a StringReader) -> &'a str {
        &reader.string()[self.start..self.end]
    }

    /// Gets a [`str`] substring slice bound
    /// to a [`String`] from this range.
    #[must_use]
    pub fn substring_slice<'a>(&self, string: &'a str) -> &'a str {
        &string[self.start..self.end]
    }

    /// Returns if the length of this range is zero.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Returns the length of this range.
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.end - self.start
    }
}

impl From<Range<usize>> for StringRange {
    fn from(value: Range<usize>) -> Self {
        Self::between(value.start, value.end)
    }
}

impl From<StringRange> for Range<usize> {
    fn from(value: StringRange) -> Self {
        value.start..value.end
    }
}
