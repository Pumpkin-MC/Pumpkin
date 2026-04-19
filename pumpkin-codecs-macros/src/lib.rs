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

use proc_macro::TokenStream;
use proc_macro_error2::__export::proc_macro2;
use proc_macro_error2::__export::proc_macro2::Span;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Error, Field, Ident, LitStr, parse_macro_input, Token, LitBool, Data};

enum FieldData {
    /// Serialization occurs with the given field name.
    Present {
        name: String,
        lenient: bool,
        /// If `Some`, tells the specified default value of this field.
        default: Option<proc_macro2::TokenStream>,
        /// If this is true, tells that the `default` attribute was specified,
        /// but no specific default value was set.
        implicit_default: bool
    },
    /// Serialization of the field is ignored.
    Skipped {
        default: proc_macro2::TokenStream,
    },
}

/// Parses a single field to get its [`FieldData`].
fn generate_field_data(field: &Field) -> Result<FieldData, Error> {
    fn duplicate_error(ident: &Ident) -> Error {
        Error::new_spanned(ident, format!("A `{ident}` path was already defined"))
    }

    let mut field_name = None;
    let mut default = None;
    let mut implicit_default = false;
    let mut skipped = false;
    let mut lenient = false;

    for attr in &field.attrs {
        if attr.path().is_ident("field") {
            attr.parse_nested_meta(|meta| {
                let ident = meta.path.get_ident().expect("Ident should exist");
                // #[field(skip)]
                if meta.path.is_ident("skip") {
                    if skipped {
                        return Err(duplicate_error(ident));
                    }
                    skipped = true;
                    Ok(())
                }
                // #[field(default = ...)]
                else if meta.path.is_ident("default") {
                    if default.is_some() {
                        return Err(duplicate_error(ident));
                    }
                    if meta.input.peek(Token![=]) {
                        let _: Token![=] = meta.input.parse()?;
                        default = Some(meta.input.parse()?);
                    } else {
                        default = None;
                        implicit_default = true;
                    }
                    Ok(())
                }
                // #[field(lenient)]
                else if meta.path.is_ident("lenient") {
                    if lenient {
                        return Err(duplicate_error(ident));
                    }
                    lenient = true;
                    Ok(())
                }
                // #[field(name = "x")]
                else if meta.path.is_ident("name") {
                    if field_name.is_some() {
                        return Err(duplicate_error(ident));
                    }
                    let value = meta.value()?;
                    let lit = value.parse::<LitStr>()?;
                    field_name = Some(lit.value());
                    Ok(())
                } else {
                    Err(Error::new_spanned(
                        ident,
                        "Invalid attribute",
                    ))
                }
            })?;
        }
    }

    if skipped {
        if field_name.is_some() || lenient {
            return Err(Error::new_spanned(
                &field.ident,
                "Cannot specify `name` or `lenient` for a skipped field",
            ));
        }
        // Default to using the Default trait if no specific default value is given.
        Ok(FieldData::Skipped { default: default.unwrap_or_else(|| quote! { Default::default() }) })
    } else {
        let name = field_name.or_else(|| field.ident.as_ref().map(ToString::to_string));
        name.map_or_else(
            || Err(Error::new_spanned(
                &field.ident,
                "No field name could be inferred",
            )),
            |name| Ok(FieldData::Present {
                name, lenient, default, implicit_default
            })
        )
    }
}

/// Derives the `Encode` trait for a struct.
///
/// Check the module's documentation for every attribute you can use.
#[proc_macro_derive(Encode, attributes(field))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    match &input.data {
        Data::Struct(data) => {
            let mut builder_encodes = Vec::new();
            for field in &data.fields {
                match encode_field_tokens(field) {
                    Ok(EncodeFieldData { builder_encode }) => {
                        builder_encodes.push(builder_encode);
                    }
                    Err(e) => return e.to_compile_error().into(),
                }
            }
            quote! {
                impl crate::codec::Encode for #name {
                    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
                        let mut builder = ops.map_builder();
                        #(#builder_encodes)*
                        builder.build(prefix)
                    }
                }
            }.into()
        }
        _ => Error::new_spanned(&input, "Only structs are supported")
            .to_compile_error()
            .into(),
    }
}

struct EncodeFieldData {
    builder_encode: Option<proc_macro2::TokenStream>,
}

fn encode_field_tokens(field: &Field) -> Result<EncodeFieldData, Error> {
    match generate_field_data(field)? {
        FieldData::Present { name, default, implicit_default, .. } => {
            let ident = field.ident.as_ref().unwrap();
            let encoded_name_lit = LitStr::new(&name, Span::call_site());
            let builder_encode = if option_type(&field.ty).is_some() {
                quote! {
                    builder = crate::codec::optional_field::OptionalFieldEncode::encode_optional_field(&self.#ident, #encoded_name_lit, ops, builder);
                }
            } else if default.is_some() || implicit_default {
                let default_tokens = default.unwrap_or_else(|| quote! {Default::default()});
                quote! {
                    builder = crate::codec::FieldEncode::encode_defaulted_field(&self.#ident, #encoded_name_lit, ops, builder, #default_tokens);
                }
            } else {
                quote! {
                    builder = crate::codec::FieldEncode::encode_field(&self.#ident, #encoded_name_lit, ops, builder);
                }
            };
            Ok(EncodeFieldData {
                builder_encode: Some(builder_encode),
            })
        }
        FieldData::Skipped { .. } => Ok(EncodeFieldData {
            builder_encode: None,
        }),
    }
}

#[proc_macro_derive(Decode, attributes(field))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    match &input.data {
        Data::Struct(data) => {
            let mut builder_decodes = Vec::new();
            let mut i = 0;
            let mut field_inputs = Vec::new();
            let mut field_outputs = Vec::new();
            for field in &data.fields {
                match decode_field_tokens(field, &mut i) {
                    Ok(DecodeFieldData {
                        builder_decode,
                        field_input,
                        field_output,
                    }) => {
                        builder_decodes.push(builder_decode);
                        if let Some(input) = field_input {
                            field_inputs.push(input);
                        }
                        field_outputs.push(field_output);
                    }
                    Err(e) => return e.to_compile_error().into(),
                }
            }
            if i < 1 {
                // TODO
                return Error::new_spanned(&input, "At least 1 field must be decoded")
                    .to_compile_error()
                    .into()
            } else if i > 16 {
                return Error::new_spanned(&input, "No more than 16 fields may be decoded")
                    .to_compile_error()
                    .into()
            }
            let constructor_tokens = quote! {
                |#( #field_inputs ),*| Self {#( #field_outputs ),*}
            };
            let apply_fn = if i == 1 { format_ident!("map") } else { format_ident!("apply_{}", i) };
            let other_apply_params = (1..i).map(|i| format_ident!("a{i}"));
            quote! {
                impl crate::codec::Decode for #name {
                    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
                        ops.get_map(&input)
                            .with_lifecycle(Lifecycle::Stable)
                            .flat_map(|map| {
                                #(#builder_decodes)*
                                a0.#apply_fn(#constructor_tokens, #( #other_apply_params ),*)
                                    .map(|r| (r, input.clone()))
                            })
                    }
                }
            }.into()
        }
        _ => Error::new_spanned(&input, "Only structs are supported")
            .to_compile_error()
            .into(),
    }
}

struct DecodeFieldData {
    /// The statement to decode a value from a map.
    builder_decode: Option<proc_macro2::TokenStream>,
    /// A constructor input in the `apply_n` or `map` function.
    field_input: Option<proc_macro2::TokenStream>,
    /// A value used to initialize the struct in the `apply_n` or `map` function.
    field_output: proc_macro2::TokenStream,
}

/// Expects an `Option` type, and if it is an `Option`, returns the type of the `Option` in a `Some`.
fn option_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty &&
        type_path.qself.is_none() && let Some(segment) = type_path.path.segments.last() && segment.ident == "Option" {
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

fn decode_field_tokens(field: &Field, counter: &mut usize) -> Result<DecodeFieldData, Error> {
    let ident = field.ident.as_ref().unwrap();
    match generate_field_data(field)? {
        FieldData::Present { name, lenient, default, implicit_default } => {
            let encoded_name_lit = LitStr::new(&name, Span::call_site());
            let decoded_ident = format_ident!("a{counter}");
            *counter += 1;
            let builder_decode = {
                if let Some(ty) = option_type(&field.ty) {
                    // For an Option, it can be lenient.
                    let lenient_token = LitBool::new(lenient, Span::call_site());
                    quote! {
                        let #decoded_ident: DataResult<Option<#ty>> = crate::codec::optional_field::OptionalFieldDecode::decode_optional_field::<O>(#encoded_name_lit, &map, ops, #lenient_token);
                    }
                } else if default.is_some() || implicit_default {
                        let lenient_token = LitBool::new(lenient, Span::call_site());
                        let default_tokens = default.unwrap_or_else(|| quote! {Default::default()});
                        let ty = &field.ty;
                        quote! {
                            let #decoded_ident: DataResult<#ty> = crate::codec::FieldDecode::decode_defaulted_field::<O>(#encoded_name_lit, &map, ops, #default_tokens, #lenient_token);
                        }
                } else {
                    if lenient {
                        return Err(Error::new_spanned(&field.ty, "Invalid use of `lenient`"));
                    }
                    quote! {
                        let #decoded_ident = crate::codec::FieldDecode::decode_field::<O>(#encoded_name_lit, &map, ops);
                    }
                }
            };
            Ok(DecodeFieldData {
                builder_decode: Some(builder_decode),
                field_input: Some(ident.into_token_stream()),
                field_output: ident.into_token_stream(),
            })
        }
        FieldData::Skipped { default } => {
            let default_tokens = quote! { #ident: #default };
            Ok(DecodeFieldData {
                builder_decode: None,
                field_input: None,
                field_output: default_tokens,
            })
        }
    }
}
