//! This module provides `derive` proc macros for `Encode` and `Decode`.
//!
//! # Attributes
//!
//! Each field in a struct can have a `field` attribute, and it can have the following sub-attributes:
//!
//! - `skip`: Skips serializing the field entirely, and instead uses the value provided by `default`, if any.
//!   In this case, `default` is *optional*, and if it is not specified, this falls back to `Default::default()`.
//! - `name = "x"`: Sets this field to be encoded with the key `"x"`. If not specified, the field's name
//!   defaults to the Rust field's name.
//! - `default` or `default = ...`: Sets a default value for a field. If no value is specified, it defaults to `Default::default()`.
//!   This is used for skipped fields and encoding *defaulted* fields.
//! - `lenient`: Only for encoding `Option`s and defaulted fields. If a value is present and cannot be successfully decoded,
//!   the value is ignored and a `None`/the default value is decoded instead.

mod decode;
mod encode;
mod field;

use proc_macro::TokenStream;
use syn::Type;

/// Derives the `Encode` trait for a struct.
///
/// Check the module's documentation for every attribute you can use.
#[proc_macro_derive(Encode, attributes(field))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    encode::derive_encode(input)
}

#[proc_macro_derive(Decode, attributes(field))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    decode::derive_decode(input)
}

/// Expects an `Option` type, and if it is an `Option`, returns the type of the `Option` in a `Some`.
fn option_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && type_path.qself.is_none()
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
    {
        let args = match &segment.arguments {
            syn::PathArguments::AngleBracketed(args) => &args.args,
            _ => return None,
        };

        match args.first()? {
            syn::GenericArgument::Type(inner_ty) => Some(inner_ty),
            _ => None,
        }
    } else {
        None
    }
}
