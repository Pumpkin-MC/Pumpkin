//! This module provides `derive` proc macros for `Encode` and `Decode`.

mod decode;
mod encode;
mod field;

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro_error2::__export::proc_macro2;
use proc_macro_error2::__export::proc_macro2::{Ident, Span};
use quote::{ToTokens, quote};
use syn::{DeriveInput, Type, parse_macro_input};

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
/// This trait also derives `MapEncode` (except for enums whose variants are all units and unit structs),
/// though this trait may only be useful directly for certain cases,
/// which is then used to derive `Encode`.
///
/// Check the [module's documentation](crate) for every attribute you can use.
#[proc_macro_derive(Encode)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    encode::derive_encode(&crate_token(), &input).unwrap_or_else(|e| e.to_compile_error().into())
}

/// Derives the `Decode` trait for a struct.
///
/// This trait also derives `MapDecode` (except for enums whose variants are all units and unit structs),
/// though this trait may only be useful directly for certain cases,
/// which is then used to derive `Decode`.
///
/// Check the [module's documentation](crate) for every attribute you can use.
#[proc_macro_derive(Decode)]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    decode::derive_decode(&crate_token(), &input).unwrap_or_else(|e| e.to_compile_error().into())
}

struct EnumDispatchData {
    tag_key: String,
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
