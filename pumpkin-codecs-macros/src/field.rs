use proc_macro_error2::__export::proc_macro2;
use proc_macro_error2::__export::proc_macro2::Ident;
use quote::ToTokens;
use syn::{Field, Index, Type};

/// A [`Field`] reference wrapper to easily tell if the field
/// is named or not.
#[derive(Copy, Clone)]
pub enum ParsedField<'a> {
    Named(&'a Field),
    Unnamed(&'a Field, usize),
}

impl<'a> ParsedField<'a> {
    /// Returns the name of this field as an `Ident`, as a reference, if any.
    pub const fn named_ident(self) -> Option<&'a Ident> {
        match self {
            ParsedField::Named(f) => Some(f.ident.as_ref().unwrap()),
            ParsedField::Unnamed(_, _) => None,
        }
    }

    /// Returns the `TokenStream` for accessing this field of a value.
    /// It can be an `Ident` or `Index`.
    pub fn access(self) -> proc_macro2::TokenStream {
        match self {
            ParsedField::Named(f) => f.ident.as_ref().unwrap().clone().into_token_stream(),
            ParsedField::Unnamed(_, i) => Index::from(i).into_token_stream(),
        }
    }

    /// Returns the `Type`, as a reference, of this field.
    pub const fn ty(self) -> &'a Type {
        match self {
            ParsedField::Named(f) | ParsedField::Unnamed(f, _) => &f.ty,
        }
    }

    /// Constructs a new `ParsedField` from a `Field`'s reference and the provided index,
    /// which may or may not be used.
    pub const fn from_field(value: &'a Field, index: usize) -> Self {
        if value.ident.is_some() {
            ParsedField::Named(value)
        } else {
            ParsedField::Unnamed(value, index)
        }
    }
}
