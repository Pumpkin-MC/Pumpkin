use crate::field::{FieldData, ParsedField};
use crate::option_type;
use proc_macro::TokenStream;
use proc_macro_error2::__export::proc_macro2;
use proc_macro_error2::__export::proc_macro2::Span;
use quote::{ToTokens, format_ident, quote};
use syn::{Data, DeriveInput, Error, Fields, LitBool, LitStr, parse_macro_input};

pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    match &input.data {
        Data::Struct(data) => {
            // Add a special case for unit structs.
            if matches!(&data.fields, Fields::Unit) {
                return quote! {
                    impl crate::codec::Decode for #name {
                        fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
                            DataResult::new_success((Self, input))
                        }
                    }
                }.into();
            }

            let mut builder_decodes = Vec::new();
            // The counted encoded values.
            let mut counter = 0;
            let mut field_inputs = Vec::new();
            let mut field_outputs = Vec::new();
            for (index, field) in data.fields.iter().enumerate() {
                let field = ParsedField::from_field(field, index);
                match decode_field_tokens(field, &mut counter) {
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
            if counter < 1 {
                // TODO
                return Error::new_spanned(&input, "At least 1 field must be decoded")
                    .to_compile_error()
                    .into();
            } else if counter > 16 {
                return Error::new_spanned(&input, "No more than 16 fields may be decoded")
                    .to_compile_error()
                    .into();
            }
            let constructor_tokens = match &data.fields {
                Fields::Named(_) => quote! {
                    |#( #field_inputs ),*| Self {#( #field_outputs ),*}
                },
                Fields::Unnamed(_) => quote! {
                    |#( #field_inputs ),*| Self (#( #field_outputs ),*)
                },
                Fields::Unit => quote! {
                    || Self
                },
            };
            let apply_fn = if counter == 1 {
                format_ident!("map")
            } else {
                format_ident!("apply_{}", counter)
            };
            let other_apply_params = (1..counter).map(|i| format_ident!("a{i}"));
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
        Data::Enum(_) => {
            todo!();
        }
        Data::Union(_) => Error::new_spanned(&input, "Only structs are supported")
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

fn decode_field_tokens(field: ParsedField, counter: &mut usize) -> Result<DecodeFieldData, Error> {
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
                        let #decoded_ident: DataResult<Option<#ty>> = crate::codec::optional_field::OptionalFieldDecode::decode_optional_field::<O>(#encoded_name_lit, &map, ops, #lenient_token);
                    }
                } else if default.is_some() || implicit_default {
                    let lenient_token = LitBool::new(lenient, Span::call_site());
                    let default_tokens = default.unwrap_or_else(|| quote! {Default::default()});
                    let ty = field.ty();
                    quote! {
                        let #decoded_ident: DataResult<#ty> = crate::codec::FieldDecode::decode_defaulted_field::<O>(#encoded_name_lit, &map, ops, #default_tokens, #lenient_token);
                    }
                } else {
                    if lenient {
                        return Err(Error::new_spanned(field.ty(), "Invalid use of `lenient`"));
                    }
                    quote! {
                        let #decoded_ident = crate::codec::FieldDecode::decode_field::<O>(#encoded_name_lit, &map, ops);
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
            let default_tokens = ident.map_or_else(|| quote! { #default }, |ident| quote! { #ident: #default });
            Ok(DecodeFieldData {
                builder_decode: None,
                field_input: None,
                field_output: default_tokens,
            })
        }
    }
}
