use crate::field::ParsedField;
use crate::option_type;
use proc_macro::TokenStream;
use proc_macro_error2::__export::proc_macro2;
use proc_macro_error2::__export::proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Error, Fields, LitStr};

pub fn derive_encode(
    codecs_crate: &proc_macro2::TokenStream,
    input: &DeriveInput,
) -> Result<TokenStream, Error> {
    let name = input.ident.clone();

    match &input.data {
        Data::Struct(data) => Ok(derive_struct_encode(&name, codecs_crate, data)),
        Data::Enum(_) | Data::Union(_) => {
            Err(Error::new_spanned(input, "Only structs are supported"))
        }
    }
}

/// Used to implement `Encode` for a type implementing `MapEncode`.
fn encode_delegate_impl(
    name: &Ident,
    codecs_crate: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        impl #codecs_crate::codec::Encode for #name {
            fn encode<O: #codecs_crate::DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> #codecs_crate::DataResult<O::Value> {
                let mut builder = #codecs_crate::DynamicOps::map_builder(ops);
                builder = #codecs_crate::codec::MapEncode::map_encode(self, ops, builder);
                #codecs_crate::struct_builder::StructBuilder::build(builder, prefix)
            }
        }
    }
}

fn derive_struct_encode(
    name: &Ident,
    codecs_crate: &proc_macro2::TokenStream,
    data: &DataStruct,
) -> TokenStream {
    // Add a special case for unit structs.
    if matches!(&data.fields, Fields::Unit) {
        return quote! {
            impl #codecs_crate::codec::Encode for #name {
                fn encode<O: #codecs_crate::DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> #codecs_crate::DataResult<O::Value> {
                    #codecs_crate::DynamicOps::merge_map_like_into_map(ops, prefix, #codecs_crate::EmptyMapLike::new())
                }
            }
        }.into();
    }
    let variant_encode = derive_single_variant_encode(codecs_crate, &data.fields);
    let encode_impl = encode_delegate_impl(name, codecs_crate);
    quote! {
            impl #codecs_crate::codec::MapEncode for #name {
                fn map_encode<O: #codecs_crate::DynamicOps, B: #codecs_crate::struct_builder::StructBuilder<Value=O::Value>>(&self, ops: &'static O, mut builder: B) -> B {
                    #variant_encode
                    builder
                }
            }

            #encode_impl
        }.into()
}

/// Creates a single variant's encoding in tokens.
fn derive_single_variant_encode(
    codecs_crate: &proc_macro2::TokenStream,
    fields: &Fields,
) -> proc_macro2::TokenStream {
    derive_single_variant_builder_encode(codecs_crate, fields, |f| {
        let access = f.access();
        quote! { &self. #access }
    })
}

/// Creates a single variant's encoding in tokens.
fn derive_single_variant_builder_encode(
    codecs_crate: &proc_macro2::TokenStream,
    fields: &Fields,
    access_fn: impl Fn(&ParsedField) -> proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
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

fn encode_field_tokens(
    codecs_crate: &proc_macro2::TokenStream,
    field: ParsedField,
    access_fn: impl Fn(&ParsedField) -> proc_macro2::TokenStream,
) -> Result<EncodeFieldData, Error> {
    let access = access_fn(&field);

    field.named_ident().map_or_else(
        || Err(Error::new_spanned(
            field.ty(),
            "Tuple structs are not supported",
        )),
        |ident| {
            let encoded_name_lit = LitStr::new(&ident.to_string(), Span::call_site());
            let builder_encode = if option_type(field.ty()).is_some() {
                quote! {
                builder = #codecs_crate::codec::optional_field::OptionalFieldEncode::encode_optional_field(#access, #encoded_name_lit, ops, builder);
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
    )
}
