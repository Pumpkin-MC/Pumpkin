use crate::field::{FieldData, ParsedField};
use crate::option_type;
use proc_macro::TokenStream;
use proc_macro_error2::__export::proc_macro2;
use proc_macro_error2::__export::proc_macro2::{Ident, Span};
use quote::quote;
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
                            fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
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
                        fn encode<O: #codecs_crate::DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
                            #variant_encode
                        }
                    }
                }.into()
            )
        }
        Data::Enum(data) => {
            todo!()
        }
        Data::Union(_) => Err(Error::new_spanned(input, "Only structs and enums are supported"))
    }
}

/// Creates a single variant's encoding in tokens.
fn derive_single_variant_encode(codecs_crate: &Ident, fields: &Fields) -> proc_macro2::TokenStream {
    let mut builder_encodes = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let field = ParsedField::from_field(field, index);
        match encode_field_tokens(codecs_crate, field) {
            Ok(EncodeFieldData { builder_encode }) => {
                builder_encodes.push(builder_encode);
            }
            Err(e) => return e.to_compile_error(),
        }
    }
    quote! {
        let mut builder = ops.map_builder();
        #(#builder_encodes)*
        builder.build(prefix)
    }
}

struct EncodeFieldData {
    builder_encode: Option<proc_macro2::TokenStream>,
}

fn encode_field_tokens(codecs_crate: &Ident, field: ParsedField) -> Result<EncodeFieldData, Error> {
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
                    builder = #codecs_crate::codec::optional_field::OptionalFieldEncode::encode_optional_field(&self.#access, #encoded_name_lit, ops, builder);
                }
            } else if default.is_some() || implicit_default {
                let default_tokens = default.unwrap_or_else(|| quote! {Default::default()});
                quote! {
                    builder = #codecs_crate::codec::FieldEncode::encode_defaulted_field(&self.#access, #encoded_name_lit, ops, builder, #default_tokens);
                }
            } else {
                quote! {
                    builder = #codecs_crate::codec::FieldEncode::encode_field(&self.#access, #encoded_name_lit, ops, builder);
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
