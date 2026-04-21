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
//!
//! For enum dispatch, you can also use the following:
//! - `tag_key("x")` on the `enum`: Tells the key for storing the enum's type. This is used to differentiate the variant
//!   during decoding. If omitted, this defaults to `"type"`.
//! - `tag("x")` on the `enum` variant: Tells the value for storing the enum's type. This is used to differentiate the variant
//!   during decoding.

mod decode;
mod encode;
mod field;

use proc_macro::TokenStream;
use proc_macro_error2::__export::proc_macro2::Ident;
use quote::format_ident;
use syn::{Attribute, DeriveInput, Error, LitStr, Type, parse_macro_input};

/// Derives the `Encode` trait for a struct.
///
/// This trait also derives `MapEncode`, though this trait may only be useful for certain cases,
/// which is then used to derive `Encode`.
///
/// Check the module's documentation for every attribute you can use.
#[proc_macro_derive(Encode, attributes(field, tag_key, tag))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let ident = format_ident!("crate");
    let input = parse_macro_input!(input as DeriveInput);
    encode::derive_encode(&ident, &input).unwrap_or_else(|e| e.to_compile_error().into())
}

/// Derives the `Decode` trait for a struct.
///
/// This trait also derives `MapDecode`, though this trait may only be useful for certain cases,
/// which is then used to derive `Decode`.
///
/// Check the module's documentation for every attribute you can use.
#[proc_macro_derive(Decode, attributes(field, tag_key, tag))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let ident = format_ident!("crate");
    let input = parse_macro_input!(input as DeriveInput);
    decode::derive_decode(&ident, &input).unwrap_or_else(|e| e.to_compile_error().into())
}

struct EnumDispatchData {
    tag_key: String,
}

fn duplicate_attribute_error(ident: &Ident) -> Error {
    Error::new_spanned(
        ident,
        format!("The `{ident}` attribute was already defined"),
    )
}

fn parse_enum_dispatch_attributes(
    ident: &Ident,
    attributes: &[Attribute],
) -> Result<EnumDispatchData, Error> {
    let mut tag_key = None;
    for attr in attributes {
        if attr.path().is_ident("tag_key") {
            if tag_key.is_some() {
                return Err(duplicate_attribute_error(ident));
            }
            let attr: LitStr = attr.parse_args()?;
            tag_key = Some(attr.value());
        }
    }
    let tag_key = tag_key.unwrap_or("type".to_string());
    Ok(EnumDispatchData { tag_key })
}

fn parse_enum_dispatch_variant_attributes(
    ident: &Ident,
    attributes: &[Attribute],
) -> Result<String, Error> {
    let mut ty = None;
    for attr in attributes {
        if attr.path().is_ident("tag") {
            if ty.is_some() {
                return Err(duplicate_attribute_error(ident));
            }
            let attr: LitStr = attr.parse_args()?;
            ty = Some(attr.value());
        }
    }
    ty.map_or_else(
        || {
            Err(Error::new_spanned(
                ident,
                "The `tag` attribute was not found",
            ))
        },
        Ok,
    )
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
