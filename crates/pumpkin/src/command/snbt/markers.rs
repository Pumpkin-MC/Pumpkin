use crate::command::{
    errors::error_types::CommandErrorType,
    snbt::rules::{EXPECTED_BINARY_NUMERAL, EXPECTED_DECIMAL_NUMERAL, EXPECTED_HEX_NUMERAL},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sign {
    Plus = 0,
    Minus = 1,
}

impl Sign {
    /// Returns the minimum number of characters required to express this sign to be parsed.
    ///
    /// For example, between `5.0` and `+5.0`, the former takes no space for the `+` symbol,
    /// while for `-5.0`, there must be a `-` symbol, so the minimum size there is `1` instead of `0`.
    #[must_use]
    #[inline]
    pub const fn minimum_size_parsable(self) -> usize {
        self as usize
    }

    /// Appends the slice containing the minimum characters required to express this sign to be parsed to a String
    /// referred by the given mutable reference.
    ///
    /// This is a no-op for [`Sign::Plus`], while for [`Sign::Minus`], a `-` is appended.
    #[inline]
    pub fn append_minimum_str_parsable(self, buffer: &mut String) {
        if self == Self::Minus {
            buffer.push('-');
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SignedPrefix {
    None,
    Unsigned,
    Signed,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypeSuffix {
    None,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
}

impl TypeSuffix {
    #[must_use]
    /// Returns the `default` suffix if this suffix is [`TypeSuffix::None`], otherwise
    /// it returns itself.
    pub fn or(self, default: Self) -> Self {
        if self == Self::None { default } else { self }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IntegerSuffix(pub SignedPrefix, pub TypeSuffix);

impl IntegerSuffix {
    pub const EMPTY: Self = Self(SignedPrefix::None, TypeSuffix::None);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Base {
    Binary,
    Decimal,
    Hexadecimal,
}

impl Base {
    #[must_use]
    pub const fn should_allow(self, c: char) -> bool {
        matches!(
            (self, c),
            (_, '_') |
            (Self::Binary, '0' | '1') |
            (Self::Decimal, '0'..='9') |
            (Self::Hexadecimal, '0'..='9' | 'A'..='F' | 'a'..='f')
        )
    }

    #[must_use]
    pub const fn no_value_error_type(self) -> &'static CommandErrorType<0> {
        match self {
            Self::Binary => &EXPECTED_BINARY_NUMERAL,
            Self::Decimal => &EXPECTED_DECIMAL_NUMERAL,
            Self::Hexadecimal => &EXPECTED_HEX_NUMERAL,
        }
    }

    #[must_use]
    pub const fn radix(self) -> u32 {
        match self {
            Self::Binary => 2,
            Self::Decimal => 10,
            Self::Hexadecimal => 16,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegerLiteral {
    pub sign: Sign,
    pub base: Base,
    pub digits: String,
    pub suffix: IntegerSuffix,
}

impl IntegerLiteral {
    pub const fn get_signed_prefix_or_default(&self) -> SignedPrefix {
        match (self.suffix.0, self.base) {
            (SignedPrefix::None, Base::Binary | Base::Hexadecimal) => SignedPrefix::Unsigned,
            (SignedPrefix::None, Base::Decimal) => SignedPrefix::Signed,
            (prefix, _) => prefix,
        }
    }
}

pub struct Signed<T> {
    pub sign: Sign,
    pub value: T,
}

/// Represents a explicit prefix set for an array.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ArrayPrefix {
    Byte,
    Long,
    Int,
}
