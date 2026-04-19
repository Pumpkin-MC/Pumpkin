use crate::field::{FieldData, ParsedField};
use crate::{option_type, parse_enum_dispatch_attributes, parse_enum_dispatch_variant_attributes};
use proc_macro::TokenStream;
use proc_macro_error2::__export::proc_macro2;
use proc_macro_error2::__export::proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Error, Fields, LitStr};

pub fn derive_encode(codecs_crate: &Ident, input: &DeriveInput) -> Result<TokenStream, Error> {
    let name = input.ident.clone();

    match &input.data {
        Data::Struct(data) => {
            // Add a special case for unit structs.
            if matches!(&data.fields, Fields::Unit) {
                return Ok(
                    quote! {
                        impl #codecs_crate::codec::Encode for #name {
                            fn encode<O: #codecs_crate::DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> #codecs_crate::DataResult<O::Value> {
                                let mut builder = ops.map_builder();
                                builder.build(prefix)
                            }
                        }
                    }.into()
                );
            }
            let variant_encode = derive_single_variant_encode(codecs_crate, &data.fields);
            Ok(
                quote! {
                    impl #codecs_crate::codec::Encode for #name {
                        fn encode<O: #codecs_crate::DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> #codecs_crate::DataResult<O::Value> {
                            #variant_encode
                        }
                    }
                }.into()
            )
        }
        Data::Enum(data) => {
            let dispatch_data = parse_enum_dispatch_attributes(&name, &input.attrs)?;
            let tag_key_lit = LitStr::new(&dispatch_data.tag_key, Span::call_site());

            let mut match_arms = Vec::new();
            for variant in &data.variants {
                let ty = parse_enum_dispatch_variant_attributes(&variant.ident, &variant.attrs)?;
                let ty_lit = LitStr::new(&ty, Span::call_site());

                let fields: Vec<_> = variant.fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| f.ident.as_ref().map_or_else(|| format_ident!("a{i}"), Clone::clone))
                    .collect();
                let ident = &variant.ident;
                let mat = match variant.fields {
                    Fields::Named(_) => Some(quote! { { #( #fields ),* } }),
                    Fields::Unnamed(_) => Some(quote! { ( #( #fields ),* ) }),
                    Fields::Unit => None
                };
                let variant_encode = derive_single_variant_builder_encode(codecs_crate, &variant.fields, |f| {
                    if matches!(&variant.fields, Fields::Unnamed(_)) {
                        let ident = format_ident!("a{}", f.index().unwrap());
                        quote! { #ident }
                    } else {
                        let ident = f.named_ident().unwrap();
                        quote! { #ident }
                    }
                });
                match_arms.push(quote! {
                    Self::#ident #mat => {
                        builder = builder.add_string_key_value(#tag_key_lit, ops.create_string(#ty_lit));
                        #variant_encode
                    }
                });
            };

            Ok(
                quote! {
                    impl #codecs_crate::codec::Encode for #name {
                        fn encode<O: #codecs_crate::DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> #codecs_crate::DataResult<O::Value> {
                            let mut builder = ops.map_builder();
                            match self {
                                #( #match_arms ),*
                            }
                            builder.build(prefix)
                        }
                    }
                }.into()
            )
        }
        Data::Union(_) => Err(Error::new_spanned(input, "Only structs and enums are supported"))
    }
}

/// Creates a single variant's encoding in tokens.
fn derive_single_variant_encode(codecs_crate: &Ident, fields: &Fields) -> proc_macro2::TokenStream {
    let builder_encodes = derive_single_variant_builder_encode(codecs_crate, fields, |f| {
        let access = f.access();
        quote! { &self. #access }
    });
    quote! {
        let mut builder = ops.map_builder();
        #builder_encodes
        builder.build(prefix)
    }
}

/// Creates a single variant's encoding in tokens.
fn derive_single_variant_builder_encode(codecs_crate: &Ident, fields: &Fields, access_fn: impl Fn(&ParsedField) -> proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let mut builder_encodes = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let field = ParsedField::from_field(field, index);
        match encode_field_tokens(codecs_crate, field, &access_fn) {
            Ok(EncodeFieldData { builder_encode }) => {
                builder_encodes.push(builder_encode);
            }
            Err(e) => return e.to_compile_error(),
        }
    }
    quote! { #(#builder_encodes)* }
}

struct EncodeFieldData {
    builder_encode: Option<proc_macro2::TokenStream>,
}

fn encode_field_tokens(codecs_crate: &Ident, field: ParsedField, access_fn: impl Fn(&ParsedField) -> proc_macro2::TokenStream) -> Result<EncodeFieldData, Error> {
    match field.generate_field_data()? {
        FieldData::Present {
            name,
            default,
            implicit_default,
            ..
        } => {
            let access = access_fn(&field);
            let encoded_name_lit = LitStr::new(&name, Span::call_site());
            let builder_encode = if option_type(field.ty()).is_some() {
                quote! {
                    builder = #codecs_crate::codec::optional_field::OptionalFieldEncode::encode_optional_field(#access, #encoded_name_lit, ops, builder);
                }
            } else if default.is_some() || implicit_default {
                let default_tokens = default.unwrap_or_else(|| quote! {Default::default()});
                quote! {
                    builder = #codecs_crate::codec::FieldEncode::encode_defaulted_field(#access, #encoded_name_lit, ops, builder, #default_tokens);
                }
            } else {
                quote! {
                    builder = #codecs_crate::codec::FieldEncode::encode_field(#access, #encoded_name_lit, ops, builder);
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
