use crate::field::ParsedField;
use crate::option_type;
use proc_macro::TokenStream;
use proc_macro_error2::__export::proc_macro2;
use proc_macro_error2::__export::proc_macro2::Span;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Data, DataStruct, DeriveInput, Error, Fields, Ident, LitBool, LitStr,
};

pub fn derive_decode(
    codecs_crate: &proc_macro2::TokenStream,
    input: &DeriveInput,
) -> Result<TokenStream, Error> {
    let name = input.ident.clone();

    match &input.data {
        Data::Struct(data) => Ok(derive_struct_decode(&name, codecs_crate, data)),
        Data::Enum(_) | Data::Union(_) => Err(Error::new_spanned(
            input,
            "Only structs are supported",
        )),
    }
}

/// Used to implement `Decode` for a type implementing `MapDecode`.
fn decode_delegate_impl(
    name: &Ident,
    codecs_crate: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        impl #codecs_crate::codec::Decode for #name {
            fn decode<O: #codecs_crate::DynamicOps>(input: O::Value, ops: &'static O) -> #codecs_crate::DataResult<(Self, O::Value)> {
                let map = #codecs_crate::DynamicOps::get_map(ops, &input);
                let single_result = #codecs_crate::DataResult::with_lifecycle(map, #codecs_crate::Lifecycle::Stable)
                    .flat_map(|map| {
                        #codecs_crate::codec::MapDecode::map_decode(map, ops)
                });
                #codecs_crate::DataResult::map(single_result, |s| (s, input))
            }
        }
    }
}

fn derive_struct_decode(
    name: &Ident,
    codecs_crate: &proc_macro2::TokenStream,
    data: &DataStruct,
) -> TokenStream {
    // Add a special case for unit structs.
    if matches!(&data.fields, Fields::Unit) {
        return quote! {
            impl #codecs_crate::codec::Decode for #name {
                fn decode<O: #codecs_crate::DynamicOps>(input: O::Value, ops: &'static O) -> #codecs_crate::DataResult<(Self, O::Value)> {
                    let map = #codecs_crate::DynamicOps::get_map(ops, &input);
                    let result = #codecs_crate::DataResult::map(map, |_| ());
                    #codecs_crate::DataResult::map(result, |()| (Self, input))
                }
            }
        }
        .into();
    }
    let variant_decode =
        derive_single_variant_decode(codecs_crate, name, &data.fields, &quote! { Self });

    let decode_impl = decode_delegate_impl(name, codecs_crate);
    quote! {
        impl #codecs_crate::codec::MapDecode for #name {
            fn map_decode<O: #codecs_crate::DynamicOps>(
                    map: impl #codecs_crate::MapLike<Value = O::Value>,
                    ops: &'static O,
                ) -> #codecs_crate::DataResult<Self> {
                #variant_decode
            }
        }

        #decode_impl
    }
    .into()
}

/// Creates a single variant's decoding in tokens.
fn derive_single_variant_decode(
    codecs_crate: &proc_macro2::TokenStream,
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
    codecs_crate: &proc_macro2::TokenStream,
    field: ParsedField,
    counter: &mut usize,
) -> Result<DecodeFieldData, Error> {
    if let Some(ident) = field.named_ident() {
        let encoded_name_lit = LitStr::new(&ident.to_string(), Span::call_site());
        let decoded_ident = format_ident!("a{counter}");
        let constructor_ident = ident;
        *counter += 1;
        let builder_decode = {
            if let Some(ty) = option_type(field.ty()) {
                let lenient_token = LitBool::new(false, Span::call_site());
                quote! {
                    let #decoded_ident: #codecs_crate::DataResult<Option<#ty>> = #codecs_crate::codec::optional_field::OptionalFieldDecode::decode_optional_field::<O>(#encoded_name_lit, &map, ops, #lenient_token);
                }
            } else {
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
    } else {
        Err(Error::new_spanned(
            field.ty(),
            "Tuple structs are not supported",
        ))
    }
}
