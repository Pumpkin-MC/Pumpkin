use crate::duplicate_attribute_error;
use proc_macro_error2::__export::proc_macro2;
use proc_macro_error2::__export::proc_macro2::Ident;
use quote::{ToTokens, quote};
use syn::{Attribute, Error, Field, Index, LitStr, Path, Token, Type};

/// Data from parsing a single field.
pub enum FieldData {
    /// Serialization occurs with the given field name.
    Present {
        name: String,
        lenient: bool,
        /// If `Some`, tells the specified default value of this field.
        default: Option<proc_macro2::TokenStream>,
        /// If this is true, tells that the `default` attribute was specified,
        /// but no specific default value was set.
        implicit_default: bool,
    },
    /// Serialization of the field is ignored.
    Skipped { default: proc_macro2::TokenStream },
}

/// A [`Field`] reference wrapper to easily tell if the field
/// is named or not.
#[derive(Debug, Copy, Clone)]
pub enum ParsedField<'a> {
    Named(&'a Field),
    Unnamed(&'a Field, usize),
}

/// A valid field attribute for the Encode and Decode trait derives.
pub enum ParsedFieldAttribute {
    Default,
    Lenient,
    Name,
    Skip,
}

macro_rules! add_attribute_branch {
    ($path:ident, $ident:literal, $var:ident) => {
        if $path.is_ident($ident) {
            return Some(Self::$var);
        }
    };
}

impl ParsedFieldAttribute {
    fn from_path(path: &Path) -> Option<Self> {
        add_attribute_branch!(path, "default", Default);
        add_attribute_branch!(path, "lenient", Lenient);
        add_attribute_branch!(path, "name", Name);
        add_attribute_branch!(path, "skip", Skip);
        None
    }
}

impl<'a> ParsedField<'a> {
    /// Returns the name of this field as an `Ident`, as a reference, if any.
    pub const fn named_ident(self) -> Option<&'a Ident> {
        match self {
            ParsedField::Named(f) => Some(f.ident.as_ref().unwrap()),
            ParsedField::Unnamed(_, _) => None,
        }
    }

    /// Returns the index of this field, if any.
    pub const fn index(&self) -> Option<usize> {
        match self {
            ParsedField::Named(_) => None,
            ParsedField::Unnamed(_, i) => Some(*i),
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

    /// Returns a slice of the list of `Attribute`s of this field.
    pub fn attrs(self) -> &'a [Attribute] {
        match self {
            ParsedField::Named(f) | ParsedField::Unnamed(f, _) => &f.attrs,
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

    /// Parses this field to get its [`FieldData`].
    pub fn generate_field_data(self) -> Result<FieldData, Error> {
        let mut field_name = None;
        let mut default = None;
        let mut implicit_default = false;
        let mut skipped = false;
        let mut lenient = false;

        for attr in self.attrs() {
            if attr.path().is_ident("field") {
                attr.parse_nested_meta(|meta| {
                    let ident = meta.path.get_ident().expect("Ident should exist");
                    if let Some(attribute) = ParsedFieldAttribute::from_path(&meta.path) {
                        match attribute {
                            // default or default = ..
                            ParsedFieldAttribute::Default => {
                                if default.is_some() {
                                    return Err(duplicate_attribute_error(ident));
                                }
                                if meta.input.peek(Token![=]) {
                                    let _: Token![=] = meta.input.parse()?;
                                    default = Some(meta.input.parse()?);
                                } else {
                                    default = None;
                                    implicit_default = true;
                                }
                            }
                            // lenient
                            ParsedFieldAttribute::Lenient => {
                                if lenient {
                                    return Err(duplicate_attribute_error(ident));
                                }
                                lenient = true;
                            }
                            // name = "x"
                            ParsedFieldAttribute::Name => {
                                if field_name.is_some() {
                                    return Err(duplicate_attribute_error(ident));
                                }
                                let value = meta.value()?;
                                let lit = value.parse::<LitStr>()?;
                                field_name = Some(lit.value());
                            }
                            // skip
                            ParsedFieldAttribute::Skip => {
                                if skipped {
                                    return Err(duplicate_attribute_error(ident));
                                }
                                skipped = true;
                            }
                        }
                        Ok(())
                    } else {
                        Err(Error::new_spanned(ident, "Invalid attribute"))
                    }
                })?;
            }
        }

        if skipped {
            if field_name.is_some() || lenient {
                return Err(Error::new_spanned(
                    self.access(),
                    "Cannot specify `name` or `lenient` for a skipped field",
                ));
            }
            // Default to using the Default trait if no specific default value is given.
            Ok(FieldData::Skipped {
                default: default.unwrap_or_else(|| quote! { Default::default() }),
            })
        } else {
            let name = field_name.or_else(|| self.named_ident().map(ToString::to_string));
            name.map_or_else(
                || {
                    Err(Error::new_spanned(
                        self.access(),
                        "No field name could be inferred",
                    ))
                },
                |name| {
                    Ok(FieldData::Present {
                        name,
                        lenient,
                        default,
                        implicit_default,
                    })
                },
            )
        }
    }
}
