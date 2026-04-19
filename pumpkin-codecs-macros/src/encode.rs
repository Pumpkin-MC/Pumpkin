use crate::field::{FieldData, ParsedField};
use crate::option_type;
use proc_macro::TokenStream;
use proc_macro_error2::__export::proc_macro2;
use proc_macro_error2::__export::proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, LitStr, parse_macro_input};

pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    match &input.data {
        Data::Struct(data) => {
            // Add a special case for unit structs.
            if matches!(&data.fields, Fields::Unit) {
                return quote! {
                   impl crate::codec::Encode for #name {
                        fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
                            let mut builder = ops.map_builder();
                            builder.build(prefix)
                        }
                    }
               }.into();
            }

            let mut builder_encodes = Vec::new();
            for (index, field) in data.fields.iter().enumerate() {
                let field = ParsedField::from_field(field, index);
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
        Data::Enum(data) => {
            todo!()
        }
        Data::Union(_) => Error::new_spanned(&input, "Only structs and enums are supported")
            .to_compile_error()
            .into(),
    }
}

struct EncodeFieldData {
    builder_encode: Option<proc_macro2::TokenStream>,
}

fn encode_field_tokens(field: ParsedField) -> Result<EncodeFieldData, Error> {
    match field.generate_field_data()? {
        FieldData::Present {
            name,
            default,
            implicit_default,
            ..
        } => {
            let access = field.access();
            let encoded_name_lit = LitStr::new(&name, Span::call_site());
            let builder_encode = if option_type(field.ty()).is_some() {
                quote! {
                    builder = crate::codec::optional_field::OptionalFieldEncode::encode_optional_field(&self.#access, #encoded_name_lit, ops, builder);
                }
            } else if default.is_some() || implicit_default {
                let default_tokens = default.unwrap_or_else(|| quote! {Default::default()});
                quote! {
                    builder = crate::codec::FieldEncode::encode_defaulted_field(&self.#access, #encoded_name_lit, ops, builder, #default_tokens);
                }
            } else {
                quote! {
                    builder = crate::codec::FieldEncode::encode_field(&self.#access, #encoded_name_lit, ops, builder);
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
