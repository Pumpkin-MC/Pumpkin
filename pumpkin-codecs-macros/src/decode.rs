use crate::field::{FieldData, ParsedField};
use crate::{option_type, parse_enum_dispatch_attributes, parse_enum_dispatch_variant_attributes};
use proc_macro::TokenStream;
use proc_macro_error2::__export::proc_macro2;
use proc_macro_error2::__export::proc_macro2::Span;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, Ident, LitBool, LitStr,
};

pub fn derive_decode(codecs_crate: &Ident, input: &DeriveInput) -> Result<TokenStream, Error> {
    let name = input.ident.clone();

    match &input.data {
        Data::Struct(data) => Ok(derive_struct_decode(&name, codecs_crate, data)),
        Data::Enum(data) => derive_enum_decode(&name, codecs_crate, data, &input.attrs),
        Data::Union(_) => Err(Error::new_spanned(
            input,
            "Only structs and enums are supported",
        )),
    }
}

fn derive_struct_decode(
    name: &Ident,
    codecs_crate: &Ident,
    data: &DataStruct,
) -> TokenStream {
    // Add a special case for unit structs.
    if matches!(&data.fields, Fields::Unit) {
        return quote! {
            impl #codecs_crate::codec::Decode for #name {
                fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
                    DataResult::new_success((Self, input))
                }
            }
        }.into();
    }
    let variant_decode =
        derive_single_variant_decode(codecs_crate, name, &data.fields, &quote! { Self });
    quote! {
        impl #codecs_crate::codec::Decode for #name {
            fn decode<O: #codecs_crate::DynamicOps>(input: O::Value, ops: &'static O) -> #codecs_crate::DataResult<(Self, O::Value)> {
                ops.get_map(&input)
                    .with_lifecycle(#codecs_crate::Lifecycle::Stable)
                    .flat_map(|map| {
                        #variant_decode
                    })
            }
        }
    }.into()
}

fn derive_enum_decode(
    name: &Ident,
    codecs_crate: &Ident,
    data: &DataEnum,
    attrs: &[Attribute],
) -> Result<TokenStream, Error> {
    // Add a special case for all variants being unit variants.
    if data
        .variants
        .iter()
        .all(|v| matches!(v.fields, Fields::Unit))
    {
        let mut match_arms = Vec::new();
        for variant in &data.variants {
            let ident = &variant.ident;
            let ty = parse_enum_dispatch_variant_attributes(&variant.ident, &variant.attrs)?;
            let ty_lit = LitStr::new(&ty, Span::call_site());
            match_arms.push(quote! {
                #ty_lit => #codecs_crate::DataResult::new_success((Self::#ident, p))
            });
        }
        return Ok(
            quote! {
                        impl #codecs_crate::codec::Decode for Test {
                            fn decode<O: #codecs_crate::DynamicOps>(input: O::Value, ops: &'static O) -> #codecs_crate::DataResult<(Self, O::Value)> {
                                let string: DataResult<(String, O::Value)> = #codecs_crate::codec::Decode::decode(input, ops);
                                string.flat_map(|(s, p)| {
                                    match s.as_str() {
                                        #( #match_arms ),* ,
                                        _ => DataResult::new_error(format!("Invalid type '{s}'"))
                                    }
                                })
                            }
                        }
                    }.into()
        );
    }

    let dispatch_data = parse_enum_dispatch_attributes(name, attrs)?;
    let tag_key_lit = LitStr::new(&dispatch_data.tag_key, Span::call_site());
    let mut match_arms = Vec::new();
    for variant in &data.variants {
        // Try to get the variant's differentiator value first.
        let ty = parse_enum_dispatch_variant_attributes(&variant.ident, &variant.attrs)?;
        let ty_lit = LitStr::new(&ty, Span::call_site());
        let ident = &variant.ident;
        let qualified_variant_ident = quote! { Self::#ident };
        let variant_decode = if variant.fields.is_empty() {
            quote! { #codecs_crate::DataResult::new_success((#qualified_variant_ident, input.clone())) }
        } else {
            derive_single_variant_decode(
                codecs_crate,
                name,
                &variant.fields,
                &qualified_variant_ident,
            )
        };
        match_arms.push(quote! {
            #ty_lit => {
                #variant_decode
            }
        });
    }
    Ok(
        quote! {
            impl #codecs_crate::codec::Decode for #name {
                fn decode<O: #codecs_crate::DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
                    ops.get_map(&input)
                        .with_lifecycle(#codecs_crate::Lifecycle::Stable)
                        .flat_map(|map| {
                            let ty: #codecs_crate::DataResult<String> = #codecs_crate::codec::FieldDecode::decode_field::<O>(#tag_key_lit, &map, ops);
                            ty.flat_map(|ty| {
                                match ty.as_str() {
                                    #( #match_arms ),*
                                    _ => #codecs_crate::DataResult::new_error(format!("Invalid differentiator key {ty}"))
                                }
                            })
                        })
                }
            }
        }.into()
    )
}

/// Creates a single variant's decoding in tokens.
fn derive_single_variant_decode(
    codecs_crate: &Ident,
    variant_ident: &Ident,
    fields: &Fields,
    variant_tokens: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let mut builder_decodes = Vec::new();
    // The counted encoded values.
    let mut counter = 0;
    let mut field_inputs = Vec::new();
    let mut field_outputs = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let field = ParsedField::from_field(field, index);
        match decode_field_tokens(codecs_crate, field, &mut counter) {
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
            Err(e) => return e.to_compile_error(),
        }
    }
    if counter < 1 {
        // TODO
        return Error::new_spanned(variant_ident, "At least 1 field must be decoded")
            .to_compile_error();
    } else if counter > 16 {
        return Error::new_spanned(variant_ident, "No more than 16 fields may be decoded")
            .to_compile_error();
    }
    let constructor_tokens = match fields {
        Fields::Named(_) => quote! {
            |#( #field_inputs ),*| #variant_tokens {#( #field_outputs ),*}
        },
        Fields::Unnamed(_) => quote! {
            |#( #field_inputs ),*| #variant_tokens (#( #field_outputs ),*)
        },
        Fields::Unit => quote! {
            || #variant_tokens
        },
    };
    let apply_fn = if counter == 1 {
        format_ident!("map")
    } else {
        format_ident!("apply_{}", counter)
    };
    let other_apply_params = (1..counter).map(|i| format_ident!("a{i}"));
    quote! {
        #(#builder_decodes)*
        a0.#apply_fn(#constructor_tokens, #( #other_apply_params ),*)
            .map(|r| (r, input.clone()))
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

fn decode_field_tokens(
    codecs_crate: &Ident,
    field: ParsedField,
    counter: &mut usize,
) -> Result<DecodeFieldData, Error> {
    let ident = field.named_ident();
    match field.generate_field_data()? {
        FieldData::Present {
            name,
            lenient,
            default,
            implicit_default,
        } => {
            let encoded_name_lit = LitStr::new(&name, Span::call_site());
            let decoded_ident = format_ident!("a{counter}");
            let constructor_ident = ident.unwrap_or(&decoded_ident);
            *counter += 1;
            let builder_decode = {
                if let Some(ty) = option_type(field.ty()) {
                    // For an Option, it can be lenient.
                    let lenient_token = LitBool::new(lenient, Span::call_site());
                    quote! {
                        let #decoded_ident: DataResult<Option<#ty>> = #codecs_crate::codec::optional_field::OptionalFieldDecode::decode_optional_field::<O>(#encoded_name_lit, &map, ops, #lenient_token);
                    }
                } else if default.is_some() || implicit_default {
                    let lenient_token = LitBool::new(lenient, Span::call_site());
                    let default_tokens = default.unwrap_or_else(|| quote! {Default::default()});
                    let ty = field.ty();
                    quote! {
                        let #decoded_ident: DataResult<#ty> = #codecs_crate::codec::FieldDecode::decode_defaulted_field::<O>(#encoded_name_lit, &map, ops, #default_tokens, #lenient_token);
                    }
                } else {
                    if lenient {
                        return Err(Error::new_spanned(field.ty(), "Invalid use of `lenient`"));
                    }
                    quote! {
                        let #decoded_ident = #codecs_crate::codec::FieldDecode::decode_field::<O>(#encoded_name_lit, &map, ops);
                    }
                }
            };
            Ok(DecodeFieldData {
                builder_decode: Some(builder_decode),
                field_input: Some(constructor_ident.clone().into_token_stream()),
                field_output: constructor_ident.into_token_stream(),
            })
        }
        FieldData::Skipped { default } => {
            let default_tokens =
                ident.map_or_else(|| quote! { #default }, |ident| quote! { #ident: #default });
            Ok(DecodeFieldData {
                builder_decode: None,
                field_input: None,
                field_output: default_tokens,
            })
        }
    }
}
