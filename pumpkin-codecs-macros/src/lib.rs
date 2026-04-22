//! This module provides `derive` proc macros for `Encode` and `Decode`.
//!
//! # Attributes
//!
//! Every attribute specified here is a sub-attribute; the root attribute must be  `#[codec(...)]`.
//!
//! ## Field Attributes
//! - `skip`: Skips serializing the field entirely, and instead uses the value provided by `default`, if any.
//!   In this case, `default` is *optional*, and if it is not specified, this falls back to `Default::default()`.
//! - `name = "x"`: Sets this field to be encoded with the key `"x"`. If not specified, the field's name
//!   defaults to the Rust field's name.
//! - `default` or `default = ...`: Sets a default value for a field. If no value is specified, it defaults to `Default::default()`.
//!   This is used for skipped fields and encoding *defaulted* fields.
//! - `lenient`: Only for encoding `Option`s and defaulted fields. If a value is present and cannot be successfully decoded,
//!   the value is ignored and a `None`/the default value is decoded instead.
//!
//! ## Struct/Enum Body Attributes
//! - `tag_key = "x"` on `enum`s: Tells the key for storing the enum's type. This is used to differentiate the variant
//!   during decoding. If omitted, this defaults to `"type"`.
//!
//! ## Enum Variant Attributes
//! - `tag = "x"`: Tells the value for storing the enum's type. This is used to differentiate the variant
//!   during decoding.

mod attribute;
mod decode;
mod encode;
mod field;

use crate::attribute::{ParsedAttribute, add_attribute_branch};
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro_error2::__export::proc_macro2;
use proc_macro_error2::__export::proc_macro2::{Ident, Span};
use quote::{ToTokens, quote};
use syn::{Attribute, DeriveInput, Error, LitStr, Path, Type, parse_macro_input};

/// Returns the tokens corresponding to the `pumpkin_codecs` crate.
fn crate_token() -> proc_macro2::TokenStream {
    match crate_name("pumpkin-codecs") {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(name)) => Ident::new(&name, Span::call_site()).into_token_stream(),
        Err(_) => Ident::new("pumpkin_codecs", Span::call_site()).into_token_stream(),
    }
}

/// Derives the `Encode` trait for a struct.
///
/// This trait also derives `MapEncode` (except for enums whose variants are all units),
/// though this trait may only be useful directly for certain cases,
/// which is then used to derive `Encode`.
///
/// Check the [module's documentation](crate) for every attribute you can use.
#[proc_macro_derive(Encode, attributes(codec))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    encode::derive_encode(&crate_token(), &input).unwrap_or_else(|e| e.to_compile_error().into())
}

/// Derives the `Decode` trait for a struct.
///
/// This trait also derives `MapDecode` (except for enums whose variants are all units),
/// though this trait may only be useful directly for certain cases,
/// which is then used to derive `Decode`.
///
/// Check the [module's documentation](crate) for every attribute you can use.
#[proc_macro_derive(Decode, attributes(codec))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    decode::derive_decode(&crate_token(), &input).unwrap_or_else(|e| e.to_compile_error().into())
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

fn parse_enum_dispatch_attributes(attributes: &[Attribute]) -> Result<EnumDispatchData, Error> {
    enum EnumDispatchAttribute {
        TagKey,
    }

    impl ParsedAttribute for EnumDispatchAttribute {
        fn from_path(path: &Path) -> Option<Self> {
            add_attribute_branch!(path, "tag_key", TagKey);
            None
        }
    }

    let mut tag_key = None;
    EnumDispatchAttribute::parse_attributes(attributes, |attribute, meta, ident| {
        match attribute {
            // tag_key = "x"
            EnumDispatchAttribute::TagKey => {
                if tag_key.is_some() {
                    return Err(duplicate_attribute_error(ident));
                }
                let value = meta.value()?;
                let lit = value.parse::<LitStr>()?;
                tag_key = Some(lit.value());
            }
        }
        Ok(())
    })?;
    let tag_key = tag_key.unwrap_or("type".to_string());
    Ok(EnumDispatchData { tag_key })
}

fn parse_enum_variant_attributes(ident: &Ident, attributes: &[Attribute]) -> Result<String, Error> {
    enum EnumVariantAttribute {
        Tag,
    }

    impl ParsedAttribute for EnumVariantAttribute {
        fn from_path(path: &Path) -> Option<Self> {
            add_attribute_branch!(path, "tag", Tag);
            None
        }
    }

    let mut ty = None;
    EnumVariantAttribute::parse_attributes(attributes, |attribute, meta, ident| {
        match attribute {
            // tag = "x"
            EnumVariantAttribute::Tag => {
                if ty.is_some() {
                    return Err(duplicate_attribute_error(ident));
                }
                let value = meta.value()?;
                let lit = value.parse::<LitStr>()?;
                ty = Some(lit.value());
            }
        }
        Ok(())
    })?;
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
